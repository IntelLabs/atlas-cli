"""Utility functions for Atlas Test Framework"""

import re
import logging
from pathlib import Path
from typing import Optional, Dict, Any, List, Union

logger = logging.getLogger(__name__)


class AtlasCommand:
    """Builder for Atlas CLI commands with proper quoting"""

    def __init__(self, base_cmd: str = "atlas-cli"):
        self.base_cmd = base_cmd
        self.command_parts = [base_cmd]

    def add_subcommand(self, *parts: str) -> "AtlasCommand":
        """Add subcommand parts (e.g., 'model', 'create')"""
        self.command_parts.extend(parts)
        return self

    def add_flag(self, flag: str, value: Any = None) -> "AtlasCommand":
        """Add a flag with optional value, properly handling spaces and special characters"""
        if value is None or value is False:
            return self

        # Clean up flag name
        flag = flag.lstrip("-")

        # Determine flag format
        if len(flag) == 1:
            flag_str = f"-{flag}"
        else:
            flag_str = f"--{flag}"

        # Add flag with or without value
        if value is True:
            # Boolean flag
            self.command_parts.append(flag_str)
        else:
            # Flag with value
            value_str = self._format_value(value)

            # Check if value needs quoting (contains spaces or special chars)
            if self._needs_quoting(value_str):
                # Use double quotes for the value
                value_str = value_str.replace('"', '\\"')  # Escape any existing quotes
                self.command_parts.append(f'{flag_str}="{value_str}"')
            else:
                self.command_parts.append(f"{flag_str}={value_str}")

        return self

    def _format_value(self, value: Any) -> str:
        """Format a value for command line"""
        if isinstance(value, (list, tuple)):
            # Join list items with commas
            return ",".join(str(item) for item in value)
        else:
            return str(value)

    def _needs_quoting(self, s: str) -> bool:
        """Check if a string needs quoting"""
        # Check for spaces, quotes, or shell special characters
        return bool(re.search(r'[\s\'"`$\\!*?<>|&;(){}[\]#~]', s))

    def build(self) -> str:
        """Build the final command string"""
        return " ".join(str(part) for part in self.command_parts)


def extract_manifest_id(output: str) -> Optional[str]:
    """Extract manifest ID from Atlas CLI output"""

    # Common patterns Atlas CLI might use
    patterns = [
        r"Manifest stored successfully with ID: ([^\s]+)",
        r"Manifest ID: ([^\s]+)",
        r"ID: ([^\s]+)",
        r"Created manifest: ([^\s]+)",
        r"stored with id: ([^\s]+)",
        r"Updated manifest ID: ([^\s]+)",
    ]

    for pattern in patterns:
        match = re.search(pattern, output, re.IGNORECASE | re.MULTILINE)
        if match:
            manifest_id = match.group(1).strip()
            logger.debug(f"Extracted manifest ID: {manifest_id}")
            return manifest_id

    # Log for debugging
    logger.debug(f"Could not extract ID from output:\n{output[:500]}")
    return None


def resolve_path(base_dir: Path, path: str, shared_dir: Optional[Path] = None) -> str:
    """Resolve path with special prefixes

    Supports:
    - @shared/ - Reference to shared directory
    - @example/ - Reference to example directory
    - ./ - Relative to base directory
    - / - Absolute path
    """
    if not isinstance(path, str):
        return str(path)

    # Handle special prefixes
    if path.startswith("@shared/") and shared_dir:
        return str(shared_dir / path[8:])
    elif path.startswith("@example/"):
        return str(base_dir / path[9:])
    elif path.startswith("./"):
        return str((base_dir / path).resolve())
    elif path.startswith("/"):
        return path
    else:
        # Default to relative to base directory
        return str(base_dir / path)


def setup_logging(verbose: bool = False):
    """Setup colored logging"""

    class ColoredFormatter(logging.Formatter):
        COLORS = {
            "DEBUG": "\033[36m",  # Cyan
            "INFO": "\033[32m",  # Green
            "WARNING": "\033[33m",  # Yellow
            "ERROR": "\033[31m",  # Red
            "RESET": "\033[0m",
            "BOLD": "\033[1m",
            "COMMAND": "\033[94m",  # Blue
        }

        def format(self, record):
            if hasattr(record, "command") and record.command:
                # Add $ prefix and color for commands
                record.msg = (
                    f"{self.COLORS['COMMAND']}$ {record.msg}{self.COLORS['RESET']}"
                )
            else:
                color = self.COLORS.get(record.levelname, self.COLORS["RESET"])
                record.levelname = f"{color}{record.levelname}{self.COLORS['RESET']}"
            return super().format(record)

    # Configure root logger
    logger = logging.getLogger("atlas_test")
    logger.setLevel(logging.DEBUG if verbose else logging.INFO)

    # Remove existing handlers
    logger.handlers.clear()

    # Add console handler with colored output
    handler = logging.StreamHandler()
    handler.setFormatter(ColoredFormatter("%(asctime)s - %(levelname)s - %(message)s"))
    logger.addHandler(handler)

    return logger


def validate_config(config: Dict[str, Any]) -> None:
    """Validate configuration structure"""

    # Check required fields
    if "steps" not in config:
        raise ValueError("Configuration must contain 'steps' field")

    if not isinstance(config["steps"], list):
        raise ValueError("'steps' must be a list")

    if len(config["steps"]) == 0:
        raise ValueError("'steps' cannot be empty")

    # Validate each step
    for i, step in enumerate(config["steps"]):
        if "action" not in step:
            raise ValueError(f"Step {i+1} missing required 'action' field")

        # Check action format
        action = step["action"]
        if ":" not in action:
            raise ValueError(
                f"Step {i+1} action '{action}' must be in format 'category:action'"
            )

    # Validate environment if present
    if "environment" in config:
        env = config["environment"]

        # Check storage configuration
        if "storage_type" in env:
            valid_storage = ["database", "local-fs", "filesystem", "rekor"]
            if env["storage_type"] not in valid_storage:
                raise ValueError(f"Invalid storage_type: {env['storage_type']}")
