use colored::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use tokio::runtime::Runtime;
use tracing::info;

use crate::{
    subcommands as actions,
    command::AtlasCommand,
    config::{ConfigOverrides, WorkflowConfig},
    error::{AtlasError, Result},
    recorder::CommandRecorder,
    utils::*,
};

pub struct AtlasTestFramework {
    pub config_file: PathBuf,
    pub config_dir: PathBuf,
    pub project_root: PathBuf,
    pub config: WorkflowConfig,
    pub shared_dir: PathBuf,
    pub manifests: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub command_recorder: CommandRecorder,
    pub current_step: Option<String>,
    runtime: Runtime,
}

impl AtlasTestFramework {
    pub fn new(config_file: impl AsRef<Path>) -> Result<Self> {
        let config_file = config_file.as_ref().canonicalize()?;
        let config_dir = config_file
            .parent()
            .ok_or_else(|| AtlasError::PathError("Invalid config file path".to_string()))?
            .to_path_buf();

        let project_root = find_project_root(&config_dir);
        let shared_dir = project_root.join("shared");

        let config = WorkflowConfig::from_file(&config_file)?;

        let output_dir = if config.environment.output_dir.starts_with('/') {
            PathBuf::from(&config.environment.output_dir)
        } else {
            config_dir.join(&config.environment.output_dir)
        };

        ensure_dir_exists(&output_dir)?;

        let command_recorder = CommandRecorder::new(Some(output_dir))?;

        let version = check_atlas_cli()?;
        info!("✅ Atlas CLI found: {}", version);

        let runtime = Runtime::new()?;

        Ok(Self {
            config_file,
            config_dir,
            project_root,
            config,
            shared_dir,
            manifests: HashMap::new(),
            variables: HashMap::new(),
            command_recorder,
            current_step: None,
            runtime,
        })
    }

    pub fn apply_overrides(&mut self, overrides: ConfigOverrides) {
        let output_dir_clone = overrides.output_dir.clone();

        self.config.apply_overrides(overrides);

        if let Some(output_dir) = output_dir_clone {
            let path = if output_dir.starts_with('/') {
                PathBuf::from(output_dir)
            } else {
                self.config_dir.join(output_dir)
            };

            if let Ok(recorder) = CommandRecorder::new(Some(path)) {
                self.command_recorder = recorder;
            }
        }
    }

    pub fn setup(&mut self) -> Result<()> {
        info!("🚀 Setting up Atlas test: {}", self.config.name.bold());
        if let Some(ref desc) = self.config.description {
            info!("   {}", desc.dimmed());
        }
        println!("{}", "=".repeat(80));

        if self.config.environment.generate_keys {
            self.setup_signing_keys()?;
        }

        if !self.config.environment.dry_run {
            self.verify_storage()?;
        }

        info!("✅ Test environment ready\n");
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let total_steps = self.config.steps.len();
        info!("📋 Executing {} steps", total_steps);
        println!("{}", "=".repeat(80));

        for (i, mut step) in self.config.steps.clone().into_iter().enumerate() {
            let step_num = i + 1;
            self.current_step = Some(step.name.clone());

            print_step_header(
                step_num,
                total_steps,
                &step.name,
                step.description.as_deref(),
            );

            step = self.resolve_step_variables(step)?;

            match self.execute_step(&step) {
                Ok(result) => {
                    if let Some(ref store_as) = step.store_as {
                        if let Some(result) = result {
                            self.manifests.insert(store_as.clone(), result.clone());
                            self.variables.insert(store_as.clone(), result.clone());
                            info!(
                                "   📌 Stored as: {} = {}...",
                                store_as,
                                &result[..12.min(result.len())]
                            );
                        }
                    }

                    print_success(&format!("{} completed", step.name));

                    if step.pause_after.unwrap_or(false) && self.config.environment.interactive {
                        println!("\n⏸️  Press Enter to continue...");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                    }
                }
                Err(e) => {
                    print_error(&format!("{} failed: {}", step.name, e));

                    if !self.config.environment.continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    fn execute_step(&mut self, step: &crate::config::Step) -> Result<Option<String>> {
        let action_parts: Vec<&str> = step.action.split(':').collect();
        if action_parts.len() != 2 {
            return Err(AtlasError::InvalidParameter(format!(
                "Invalid action format: {}",
                step.action
            )));
        }

        let category = action_parts[0];
        let action = action_parts[1];

        match category {
            "dataset" => actions::dataset::execute_dataset_action(self, action, step),
            "model" => actions::model::execute_model_action(self, action, step),
            "software" => actions::software::execute_software_action(self, action, step),
            "evaluation" => actions::evaluation::execute_evaluation_action(self, action, step),
            "manifest" => actions::manifest::execute_manifest_action(self, action, step),
            "shell" => actions::utility::execute_shell_command(self, step),
            "file" => actions::utility::execute_file_action(self, action, step),
            _ => Err(AtlasError::UnknownAction(step.action.clone())),
        }
    }

    pub fn teardown(&mut self) -> Result<()> {
        info!("\n🏁 Test execution complete");

        self.command_recorder.export_script(&self.manifests)?;
        self.command_recorder.show_summary();

        if !self.manifests.is_empty() {
            println!("\n📦 Created Manifests:");
            for (name, id) in &self.manifests {
                println!("   {}: {}", name, id);
            }
        }

        Ok(())
    }

    pub fn run_command(&mut self, command: &str, check: bool) -> Result<Output> {
        if !command.trim().is_empty() {
            print_command(command);
        }

        if self.config.environment.dry_run {
            self.command_recorder.record(
                command,
                None,
                true,
                self.current_step.as_deref(),
                None,
            )?;

            return Ok(Output {
                status: ExitStatus::default(),
                stdout: b"[DRY RUN] Command would be executed".to_vec(),
                stderr: Vec::new(),
            });
        }

        let output = Command::new("sh").arg("-c").arg(command).output()?;

        let output_id = if output.status.success() && command.contains("create") {
            extract_manifest_id(&String::from_utf8_lossy(&output.stdout))
        } else {
            None
        };

        self.command_recorder.record(
            command,
            Some(&output),
            output.status.success(),
            self.current_step.as_deref(),
            output_id,
        )?;

        if check && !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AtlasError::CommandError(format!(
                "Command failed with code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            )));
        }

        Ok(output)
    }

    fn setup_signing_keys(&mut self) -> Result<()> {
        let key_path = self.resolve_path(&self.config.environment.signing_key);
        let pub_path = self.resolve_path(&self.config.environment.verifying_key);

        if key_path.exists() && pub_path.exists() {
            info!("🔑 Using existing signing keys");
            return Ok(());
        }

        info!("🔑 Generating signing keys...");

        self.runtime
            .block_on(async { generate_signing_keys(&key_path, &pub_path).await })?;

        info!(
            "   ✓ Generated keys: {}, {}",
            key_path.display(),
            pub_path.display()
        );
        Ok(())
    }

    fn verify_storage(&self) -> Result<()> {
        let storage_type = &self.config.environment.storage_type;
        let storage_url = self.resolve_path(&self.config.environment.storage_url);

        match storage_type.as_str() {
            "local-fs" | "filesystem" => {
                ensure_dir_exists(&storage_url)?;
                info!("✅ Storage backend ready: {}", storage_url.display());
            }
            "database" => {
                info!(
                    "ℹ️  Using database storage: {}",
                    self.config.environment.storage_url
                );
            }
            _ => {
                info!(
                    "ℹ️  Using {} storage: {}",
                    storage_type, self.config.environment.storage_url
                );
            }
        }

        Ok(())
    }

    pub fn resolve_path(&self, path: &str) -> PathBuf {
        if path.starts_with("http://") || path.starts_with("https://") {
            return PathBuf::from(path);
        }
        resolve_path(&self.config_dir, path, Some(&self.shared_dir))
    }

    fn resolve_step_variables(&self, mut step: crate::config::Step) -> Result<crate::config::Step> {
        for (_, value) in step.parameters.iter_mut() {
            *value = resolve_value(value.take(), &self.variables);
        }

        step.name = resolve_variables(&step.name, &self.variables);
        if let Some(desc) = step.description {
            step.description = Some(resolve_variables(&desc, &self.variables));
        }

        Ok(step)
    }

    pub fn build_common_flags(&self, cmd: &mut AtlasCommand) {
        self.build_common_flags_with_options(cmd, true);
    }

    pub fn build_common_flags_with_options(&self, cmd: &mut AtlasCommand, include_key: bool) {
        let env = &self.config.environment;

        cmd.add_flag("storage-type", Some(&env.storage_type));

        let storage_url = if env.storage_url.starts_with("http") {
            env.storage_url.clone()
        } else {
            self.resolve_path(&env.storage_url).display().to_string()
        };
        cmd.add_flag("storage-url", Some(storage_url));

        if include_key && !env.signing_key.is_empty() {
            let key_path = self.resolve_path(&env.signing_key);
            cmd.add_flag("key", Some(key_path.display().to_string()));
        }

        if include_key {
            cmd.add_flag("hash-alg", Some(env.hash_algorithm()));
        }
    }
}
