"""Main framework for executing Atlas CLI commands"""

import subprocess
import json
import os
import sys
import yaml
import re
from pathlib import Path
from typing import Dict, List, Optional, Any, Union
from datetime import datetime
import logging

from .utils import AtlasCommand, extract_manifest_id, resolve_path, validate_config
from .recorder import CommandRecorder

logger = logging.getLogger("atlas_test")


class AtlasTestFramework:
    """Main framework for executing Atlas CLI commands"""

    def __init__(self, config_file: str):
        self.config_file = Path(config_file).resolve()
        self.config_dir = self.config_file.parent
        self.project_root = self._find_project_root()

        # Load configuration
        self.config = self._load_config(str(self.config_file))

        # Setup paths and environment
        self._setup_paths()
        self._check_atlas_cli()

        # Storage for manifest IDs
        self.manifests: Dict[str, str] = {}
        self.variables: Dict[str, str] = {}

        # Command recorder
        output_dir = self.config.get("environment", {}).get(
            "output_dir", "./test_output"
        )
        if not output_dir.startswith("/"):
            output_dir = str(self.config_dir / output_dir)
        self.command_recorder = CommandRecorder(output_dir)

        # Current step tracking
        self.current_step = None

    def _find_project_root(self) -> Path:
        """Find project root directory"""
        current = self.config_dir
        while current != current.parent:
            if (current / "pyproject.toml").exists():
                return current
            current = current.parent
        return self.config_dir.parent

    def _load_config(self, config_file: str) -> Dict:
        """Load YAML configuration"""
        with open(config_file, "r") as f:
            config = yaml.safe_load(f)

        # Validate configuration
        validate_config(config)

        return config

    def _setup_paths(self):
        """Setup path resolution"""
        self.shared_dir = self.project_root / "shared"
        self.example_name = (
            self.config_dir.name if "examples" in str(self.config_dir) else "test"
        )

        # Ensure output directory exists
        output_dir = self.config.get("environment", {}).get(
            "output_dir", "./test_output"
        )
        if not output_dir.startswith("/"):
            output_dir = self.config_dir / output_dir
        Path(output_dir).mkdir(parents=True, exist_ok=True)

    def _check_atlas_cli(self):
        """Check if atlas-cli is available"""
        try:
            result = subprocess.run(
                "atlas-cli --version", shell=True, capture_output=True, text=True
            )
            if result.returncode != 0:
                raise RuntimeError("atlas-cli not found in PATH")

            logger.info(f"✅ Atlas CLI found: {result.stdout.strip()}")
        except Exception as e:
            raise RuntimeError(f"Atlas CLI not available: {e}")

    def _resolve_path(self, path: str) -> str:
        """Resolve path with special prefixes"""
        return resolve_path(self.config_dir, path, self.shared_dir)

    def _resolve_variables(self, text: str) -> str:
        """Resolve ${VARIABLE} references"""
        if not isinstance(text, str):
            return text

        # Replace manifest IDs
        for name, manifest_id in self.manifests.items():
            text = text.replace(f"${{{name}}}", manifest_id)

        # Replace other variables
        for key, value in self.variables.items():
            text = text.replace(f"${{{key}}}", str(value))

        return text

    def _resolve_config_values(self, config: Any) -> Any:
        """Recursively resolve all configuration values"""
        if isinstance(config, dict):
            resolved = {}
            for k, v in config.items():
                if k in [
                    "paths",
                    "path",
                    "file",
                    "output_file",
                    "signing_key",
                    "verifying_key",
                    "storage_url",
                ]:
                    if isinstance(v, list):
                        resolved[k] = [self._resolve_path(p) for p in v]
                    else:
                        resolved[k] = (
                            self._resolve_path(v)
                            if k != "storage_url" or not v.startswith("http")
                            else v
                        )
                else:
                    resolved[k] = self._resolve_config_values(v)
            return resolved
        elif isinstance(config, list):
            return [self._resolve_config_values(item) for item in config]
        elif isinstance(config, str):
            return self._resolve_variables(config)
        else:
            return config

    def setup(self):
        """Setup test environment"""
        logger.info(f"🚀 Setting up Atlas test: {self.config.get('name', 'Unnamed')}")
        logger.info(f"   {self.config.get('description', '')}")
        logger.info("=" * 80)

        env = self.config.get("environment", {})

        # Setup signing keys if needed
        if env.get("generate_keys", False):
            self._setup_signing_keys(env)

        # Verify storage
        if not env.get("dry_run", False):
            self._verify_storage(env)

        logger.info("✅ Test environment ready\n")

    def _setup_signing_keys(self, env: Dict):
        """Generate signing keys if they don't exist"""
        key_path = self._resolve_path(env.get("signing_key", "private.pem"))
        pub_path = self._resolve_path(env.get("verifying_key", "public.pem"))

        if Path(key_path).exists() and Path(pub_path).exists():
            logger.info("🔑 Using existing signing keys")
            return

        logger.info("🔑 Generating signing keys...")

        # Ensure directory exists
        Path(key_path).parent.mkdir(parents=True, exist_ok=True)
        Path(pub_path).parent.mkdir(parents=True, exist_ok=True)

        # Generate private key
        cmd = f"openssl genpkey -algorithm RSA -out {key_path} -pkeyopt rsa_keygen_bits:4096"
        result = self._run_command(cmd, silent=True)

        # Extract public key
        cmd = f"openssl rsa -pubout -in {key_path} -out {pub_path}"
        result = self._run_command(cmd, silent=True)

        logger.info(f"   ✓ Generated keys: {key_path}, {pub_path}")

    def _verify_storage(self, env: Dict):
        """Verify storage backend is accessible"""
        storage_type = env.get("storage_type", "local-fs")
        storage_url = self._resolve_path(env.get("storage_url", "./storage"))

        # For local-fs, just check if directory exists or can be created
        if storage_type in ["local-fs", "filesystem"]:
            storage_path = Path(storage_url)
            try:
                storage_path.mkdir(parents=True, exist_ok=True)
                logger.info(f"✅ Storage backend ready: {storage_url}")
            except Exception as e:
                logger.warning(f"⚠️  Could not create storage directory: {e}")
        elif storage_type == "database":
            # For database, we could ping the endpoint, but skip for now
            logger.info(f"ℹ️  Using database storage: {storage_url}")
        else:
            logger.info(f"ℹ️  Using {storage_type} storage: {storage_url}")

    def execute(self):
        """Execute all workflow steps"""
        steps = self.config.get("steps", [])
        total_steps = len(steps)

        logger.info(f"📋 Executing {total_steps} steps")
        logger.info("=" * 80)

        for i, step in enumerate(steps, 1):
            self.current_step = step.get("name", f"Step {i}")

            logger.info(f"\n▶️  Step {i}/{total_steps}: {self.current_step}")
            if "description" in step:
                logger.info(f"   {step['description']}")

            try:
                # Resolve all variables in step configuration
                step = self._resolve_config_values(step)

                # Execute the step
                result = self._execute_step(step)

                # Store result if specified
                if "store_as" in step and result:
                    self.manifests[step["store_as"]] = result
                    self.variables[step["store_as"]] = result
                    logger.info(
                        f"   📌 Stored as: {step['store_as']} = {result[:12]}..."
                    )

                logger.info(f"   ✅ {self.current_step} completed")

                # Pause if requested
                if step.get("pause_after", False) and self.config.get(
                    "environment", {}
                ).get("interactive", False):
                    input("\n⏸️  Press Enter to continue...")

            except Exception as e:
                logger.error(f"   ❌ {self.current_step} failed: {e}")
                if not self.config.get("environment", {}).get(
                    "continue_on_error", False
                ):
                    raise

    def _execute_step(self, step: Dict) -> Optional[str]:
        """Execute a single workflow step"""
        action = step.get("action")

        # Dataset operations
        if action == "dataset:create":
            return self._create_dataset(step)
        elif action == "dataset:verify":
            return self._verify_dataset(step)
        elif action == "dataset:list":
            return self._list_datasets(step)

        # Model operations
        elif action == "model:create":
            return self._create_model(step)
        elif action == "model:verify":
            return self._verify_model(step)
        elif action == "model:list":
            return self._list_models(step)

        # Software operations
        elif action == "software:create":
            return self._create_software(step)
        elif action == "software:verify":
            return self._verify_software(step)

        # Evaluation operations
        elif action == "evaluation:create":
            return self._create_evaluation(step)
        elif action == "evaluation:verify":
            return self._verify_evaluation(step)

        # Manifest operations (cross-reference and linking)
        elif action == "manifest:link":
            return self._link_manifests(step)
        elif action == "manifest:validate":
            return self._validate_manifest(step)
        elif action == "manifest:show":
            return self._show_manifest(step)
        elif action == "manifest:export":
            return self._export_manifest(step)

        # Utility operations
        elif action == "shell:command":
            return self._run_shell_command(step)
        elif action == "file:tamper":
            return self._tamper_file(step)
        else:
            raise ValueError(f"Unknown action: {action}")

    def _list_datasets(self, step: Dict) -> str:
        """List all dataset manifests (for debugging)"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("dataset", "list")
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        # Log the output for debugging
        if result.stdout:
            logger.debug(f"Datasets found:\n{result.stdout}")

        return result.stdout

    def _list_models(self, step: Dict) -> str:
        """List all model manifests (for debugging)"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("model", "list")
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        # Log the output for debugging
        if result.stdout:
            logger.debug(f"Models found:\n{result.stdout}")

        return result.stdout

    def _verify_dataset(self, step: Dict) -> str:
        """Verify a dataset manifest integrity"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("dataset", "verify")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        # Check expected result
        if "expect" in step:
            expected = step["expect"]
            if expected == "success" and result.returncode != 0:
                raise AssertionError(f"Expected verification to succeed but it failed")
            elif expected == "failure" and result.returncode == 0:
                raise AssertionError(f"Expected verification to fail but it succeeded")

        return "valid" if result.returncode == 0 else "invalid"

    def _verify_model(self, step: Dict) -> str:
        """Verify a model manifest integrity"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("model", "verify")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        # Check expected result
        if "expect" in step:
            expected = step["expect"]
            if expected == "success" and result.returncode != 0:
                raise AssertionError(f"Expected verification to succeed but it failed")
            elif expected == "failure" and result.returncode == 0:
                raise AssertionError(f"Expected verification to fail but it succeeded")

        return "valid" if result.returncode == 0 else "invalid"

    def _verify_software(self, step: Dict) -> str:
        """Verify a software manifest integrity"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("software", "verify")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        # Check expected result
        if "expect" in step:
            expected = step["expect"]
            if expected == "success" and result.returncode != 0:
                raise AssertionError(f"Expected verification to succeed but it failed")
            elif expected == "failure" and result.returncode == 0:
                raise AssertionError(f"Expected verification to fail but it succeeded")

        return "valid" if result.returncode == 0 else "invalid"

    def _verify_evaluation(self, step: Dict) -> str:
        """Verify an evaluation manifest integrity"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("evaluation", "verify")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        # Check expected result
        if "expect" in step:
            expected = step["expect"]
            if expected == "success" and result.returncode != 0:
                raise AssertionError(f"Expected verification to succeed but it failed")
            elif expected == "failure" and result.returncode == 0:
                raise AssertionError(f"Expected verification to fail but it succeeded")

        return "valid" if result.returncode == 0 else "invalid"

    def _validate_manifest(self, step: Dict) -> str:
        """Validate manifest cross-references"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("manifest", "validate")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        return "valid" if result.returncode == 0 else "invalid"

    def _create_dataset(self, step: Dict) -> str:
        """Create a dataset manifest using Atlas CLI"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        # Build Atlas CLI command
        cmd = AtlasCommand("atlas-cli").add_subcommand("dataset", "create")

        # Add paths
        paths = params.get("paths", [])
        if paths:
            cmd.add_flag("paths", ",".join(paths))

        # Add required parameters
        cmd.add_flag("name", params.get("name", "Dataset"))

        # Add ingredient names
        ingredient_names = params.get("ingredient_names", [params.get("name")])
        if isinstance(ingredient_names, str):
            ingredient_names = [ingredient_names]
        cmd.add_flag("ingredient-names", ",".join(ingredient_names))

        # Add author info
        cmd.add_flag(
            "author-org", params.get("author_org", env.get("author_org", "Test Org"))
        )
        cmd.add_flag(
            "author-name",
            params.get("author_name", env.get("author_name", "Test User")),
        )

        # Add optional parameters
        if params.get("description"):
            cmd.add_flag("description", params["description"])

        # Add storage configuration
        cmd.add_flag("storage-type", env.get("storage_type", "local-fs"))
        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)
        cmd.add_flag("storage-url", storage_url)

        # Add signing key
        if env.get("signing_key"):
            cmd.add_flag("key", self._resolve_path(env["signing_key"]))

        # Add hash algorithm
        cmd.add_flag("hash-alg", env.get("hash_alg", "sha384"))

        # Add linked manifests
        if params.get("linked_manifests"):
            for manifest_id in params["linked_manifests"]:
                cmd.add_flag("linked-manifests", manifest_id)

        # Add TDX support if requested
        if params.get("with_tdx", False):
            cmd.add_flag("with-tdx", True)

        # Execute command
        command_str = cmd.build()
        result = self._run_command(command_str)

        # Extract manifest ID from output
        manifest_id = extract_manifest_id(result.stdout)
        if not manifest_id:
            raise ValueError("Could not extract manifest ID from output")
        return manifest_id

    def _create_model(self, step: Dict) -> str:
        """Create a model manifest using Atlas CLI"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        cmd = AtlasCommand("atlas-cli").add_subcommand("model", "create")

        # Add paths
        paths = params.get("paths", [])
        if paths:
            cmd.add_flag("paths", ",".join(paths))

        # Add required parameters
        cmd.add_flag("name", params.get("name", "Model"))

        # Add ingredient names
        ingredient_names = params.get("ingredient_names", [params.get("name")])
        if isinstance(ingredient_names, str):
            ingredient_names = [ingredient_names]
        cmd.add_flag("ingredient-names", ",".join(ingredient_names))

        # Add author info
        cmd.add_flag(
            "author-org", params.get("author_org", env.get("author_org", "Test Org"))
        )
        cmd.add_flag(
            "author-name",
            params.get("author_name", env.get("author_name", "Test User")),
        )

        # Add optional parameters
        if params.get("description"):
            cmd.add_flag("description", params["description"])

        # Add storage configuration
        cmd.add_flag("storage-type", env.get("storage_type", "local-fs"))
        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)
        cmd.add_flag("storage-url", storage_url)

        # Add signing key
        if env.get("signing_key"):
            cmd.add_flag("key", self._resolve_path(env["signing_key"]))

        # Add hash algorithm
        cmd.add_flag("hash-alg", env.get("hash_alg", "sha384"))

        # Add linked manifests
        if params.get("linked_manifests"):
            for manifest_id in params["linked_manifests"]:
                cmd.add_flag("linked-manifests", manifest_id)

        # Execute command
        command_str = cmd.build()
        result = self._run_command(command_str)

        # Extract manifest ID
        manifest_id = extract_manifest_id(result.stdout)
        if not manifest_id:
            raise ValueError("Could not extract manifest ID from output")
        return manifest_id

    def _create_software(self, step: Dict) -> str:
        """Create a software manifest using Atlas CLI"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        cmd = AtlasCommand("atlas-cli").add_subcommand("software", "create")

        # Add paths
        paths = params.get("paths", [])
        if paths:
            cmd.add_flag("paths", ",".join(paths))

        # Add required parameters
        cmd.add_flag("name", params.get("name", "Software"))
        cmd.add_flag("software-type", params.get("software_type", "script"))
        cmd.add_flag("version", params.get("version", "1.0.0"))

        # Add ingredient names
        ingredient_names = params.get("ingredient_names", [params.get("name")])
        if isinstance(ingredient_names, str):
            ingredient_names = [ingredient_names]
        cmd.add_flag("ingredient-names", ",".join(ingredient_names))

        # Add author info
        cmd.add_flag(
            "author-org", params.get("author_org", env.get("author_org", "Test Org"))
        )
        cmd.add_flag(
            "author-name",
            params.get("author_name", env.get("author_name", "Test User")),
        )

        # Add optional parameters
        if params.get("description"):
            cmd.add_flag("description", params["description"])

        # Add storage configuration
        cmd.add_flag("storage-type", env.get("storage_type", "local-fs"))
        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)
        cmd.add_flag("storage-url", storage_url)

        # Add signing key
        if env.get("signing_key"):
            cmd.add_flag("key", self._resolve_path(env["signing_key"]))

        # Add hash algorithm
        cmd.add_flag("hash-alg", env.get("hash_alg", "sha384"))

        # Add linked manifests
        if params.get("linked_manifests"):
            for manifest_id in params["linked_manifests"]:
                cmd.add_flag("linked-manifests", manifest_id)

        # Execute command
        command_str = cmd.build()
        result = self._run_command(command_str)

        # Extract manifest ID
        manifest_id = extract_manifest_id(result.stdout)
        if not manifest_id:
            raise ValueError("Could not extract manifest ID from output")
        return manifest_id

    def _create_evaluation(self, step: Dict) -> str:
        """Create an evaluation manifest using Atlas CLI"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        cmd = AtlasCommand("atlas-cli").add_subcommand("evaluation", "create")

        # Add required parameters
        cmd.add_flag("path", params.get("path"))
        cmd.add_flag("name", params.get("name", "Evaluation"))
        cmd.add_flag("model-id", params.get("model_id"))
        cmd.add_flag("dataset-id", params.get("dataset_id"))

        # Add author info
        cmd.add_flag(
            "author-org", params.get("author_org", env.get("author_org", "Test Org"))
        )
        cmd.add_flag(
            "author-name",
            params.get("author_name", env.get("author_name", "Test User")),
        )

        # Add optional parameters
        if params.get("description"):
            cmd.add_flag("description", params["description"])

        # Add metrics
        if params.get("metrics"):
            for key, value in params["metrics"].items():
                cmd.add_flag("metrics", f"{key}={value}")

        # Add storage configuration
        cmd.add_flag("storage-type", env.get("storage_type", "local-fs"))
        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)
        cmd.add_flag("storage-url", storage_url)

        # Add signing key
        if env.get("signing_key"):
            cmd.add_flag("key", self._resolve_path(env["signing_key"]))

        # Execute command
        command_str = cmd.build()
        result = self._run_command(command_str)

        # Extract manifest ID
        manifest_id = extract_manifest_id(result.stdout)
        if not manifest_id:
            raise ValueError("Could not extract manifest ID from output")
        return manifest_id

    def _link_manifests(self, step: Dict) -> str:
        """Link two manifests"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("manifest", "link")
            .add_flag("source", params.get("source"))
            .add_flag("target", params.get("target"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str)

        return "linked" if result.returncode == 0 else "failed"

    def _verify_manifest(self, step: Dict) -> str:
        """Verify a manifest"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("manifest", "verify")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        # Check expected result
        if "expect" in step:
            expected = step["expect"]
            if expected == "success" and result.returncode != 0:
                raise AssertionError(f"Expected verification to succeed but it failed")
            elif expected == "failure" and result.returncode == 0:
                raise AssertionError(f"Expected verification to fail but it succeeded")

        return "valid" if result.returncode == 0 else "invalid"

    def _validate_manifest(self, step: Dict) -> str:
        """Validate manifest cross-references"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("manifest", "validate")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str, check=False)

        return "valid" if result.returncode == 0 else "invalid"

    def _show_manifest(self, step: Dict) -> str:
        """Show manifest details"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("manifest", "show")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str)

        # Save output if requested
        if params.get("save_to"):
            output_file = Path(self._resolve_path(params["save_to"]))
            output_file.parent.mkdir(parents=True, exist_ok=True)
            output_file.write_text(result.stdout)
            logger.info(f"   💾 Saved to: {output_file}")

        return result.stdout

    def _export_manifest(self, step: Dict) -> str:
        """Export provenance graph"""
        params = step.get("parameters", {})
        env = self.config.get("environment", {})

        output_file = params.get(
            "output_file",
            f"./output/provenance_{params.get('manifest_id', 'unknown')[:8]}.json",
        )
        output_file = self._resolve_path(output_file)

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("manifest", "export")
            .add_flag("id", params.get("manifest_id"))
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
            .add_flag("format", params.get("format", "json"))
            .add_flag("output", output_file)
            .add_flag("max-depth", params.get("max_depth", 10))
        )

        command_str = cmd.build()
        self._run_command(command_str)

        logger.info(f"   📊 Exported to: {output_file}")
        return output_file

    def _list_manifests(self, step: Dict) -> str:
        """List all manifests"""
        env = self.config.get("environment", {})

        storage_url = env.get("storage_url", "./storage")
        if not storage_url.startswith("http"):
            storage_url = self._resolve_path(storage_url)

        cmd = (
            AtlasCommand("atlas-cli")
            .add_subcommand("manifest", "list")
            .add_flag("storage-type", env.get("storage_type", "local-fs"))
            .add_flag("storage-url", storage_url)
        )

        command_str = cmd.build()
        result = self._run_command(command_str)

        return result.stdout

    def _run_shell_command(self, step: Dict) -> str:
        """Run a custom shell command"""
        params = step.get("parameters", {})
        command = params.get("command", "")

        result = self._run_command(command, check=params.get("check", True))

        if params.get("capture_as"):
            self.variables[params["capture_as"]] = result.stdout.strip()
            logger.info(f"   📌 Captured as: {params['capture_as']}")

        return result.stdout.strip()

    def _tamper_file(self, step: Dict) -> str:
        """Tamper with a file for integrity testing"""
        params = step.get("parameters", {})
        file_path = Path(self._resolve_path(params.get("file", "")))

        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        logger.info(f"   ⚠️  Tampering with: {file_path}")

        # Read and modify content
        if file_path.suffix in [".txt", ".csv", ".json", ".yaml"]:
            content = file_path.read_text()
            content += "\n# TAMPERED DATA"
            file_path.write_text(content)
        else:
            content = file_path.read_bytes()
            tampered = bytearray(content)
            if len(tampered) > 0:
                tampered[0] ^= 0xFF  # Flip first byte
            file_path.write_bytes(bytes(tampered))

        logger.info(f"   ✓ File tampered")
        return "tampered"

    def _run_command(
        self, command: str, check: bool = True, silent: bool = False
    ) -> subprocess.CompletedProcess:
        """Execute a shell command"""
        env = self.config.get("environment", {})

        if not silent:
            # Log the command (the $ will be added by the formatter)
            logger.info(command, extra={"command": True})

        # Handle dry run
        if env.get("dry_run", False):
            self.command_recorder.record(command, None, True, self.current_step)
            return subprocess.CompletedProcess(
                args=command,
                returncode=0,
                stdout="[DRY RUN] Command would be executed",
                stderr="",
            )

        # Execute actual command
        result = subprocess.run(
            command, shell=True, capture_output=True, text=True, check=False
        )

        # Record command
        output_id = None
        if "create" in command and result.returncode == 0:
            try:
                output_id = extract_manifest_id(result.stdout)
            except:
                pass

        self.command_recorder.record(
            command, result, result.returncode == 0, self.current_step, output_id
        )

        # Check for errors
        if check and result.returncode != 0:
            error_msg = f"Command failed with code {result.returncode}"
            if result.stderr:
                error_msg += f"\n{result.stderr}"
            raise RuntimeError(error_msg)

        return result

    def teardown(self):
        """Cleanup and finalize test execution"""
        logger.info("\n🏁 Test execution complete")

        # Export reproducible script
        if hasattr(self, "command_recorder") and self.command_recorder:
            self.command_recorder.export_script(self.manifests)
            self.command_recorder.show_summary()

        # Show created manifests
        if self.manifests:
            logger.info("\n📦 Created Manifests:")
            for name, manifest_id in self.manifests.items():
                logger.info(f"   {name}: {manifest_id}")
