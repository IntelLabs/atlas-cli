use crate::error::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub timestamp: DateTime<Local>,
    pub step: String,
    pub command: String,
    pub success: bool,
    pub output_id: Option<String>,
    pub return_code: i32,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Debug)]
pub struct CommandRecorder {
    pub commands: Vec<CommandEntry>,
    pub output_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
    pub script_file: Option<PathBuf>,
}

impl CommandRecorder {
    /// Create a new command recorder
    pub fn new(output_dir: Option<PathBuf>) -> Result<Self> {
        let (log_file, script_file) = if let Some(ref dir) = output_dir {
            // Ensure directory exists
            std::fs::create_dir_all(dir)?;

            let log = dir.join("commands.log");
            let script = dir.join("reproduce.sh");

            // Clear existing files
            if log.exists() {
                std::fs::remove_file(&log)?;
            }

            (Some(log), Some(script))
        } else {
            (None, None)
        };

        Ok(Self {
            commands: Vec::new(),
            output_dir,
            log_file,
            script_file,
        })
    }

    /// Record a command execution
    pub fn record(
        &mut self,
        command: &str,
        result: Option<&std::process::Output>,
        success: bool,
        step_name: Option<&str>,
        output_id: Option<String>,
    ) -> Result<()> {
        let entry = CommandEntry {
            timestamp: Local::now(),
            step: step_name.unwrap_or("Unknown").to_string(),
            command: command.to_string(),
            success,
            output_id,
            return_code: result.map(|r| r.status.code().unwrap_or(-1)).unwrap_or(0),
            stdout: result.map(|r| String::from_utf8_lossy(&r.stdout).to_string()),
            stderr: result.map(|r| String::from_utf8_lossy(&r.stderr).to_string()),
        };

        // Write to log file immediately
        if let Some(ref log_file) = self.log_file {
            self.write_to_log(&entry, log_file)?;
        }

        self.commands.push(entry);
        Ok(())
    }

    /// Write command entry to log file
    fn write_to_log(&self, entry: &CommandEntry, log_file: &Path) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        writeln!(file, "\n{}", "=".repeat(80))?;
        writeln!(
            file,
            "[{}] Step: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.step
        )?;
        writeln!(file, "$ {}", entry.command)?;
        writeln!(file, "Return Code: {}", entry.return_code)?;

        if let Some(ref stdout) = entry.stdout {
            if !stdout.trim().is_empty() {
                writeln!(file, "\nSTDOUT:\n{}", stdout)?;
            }
        }

        if let Some(ref stderr) = entry.stderr {
            if !stderr.trim().is_empty() {
                writeln!(file, "\nSTDERR:\n{}", stderr)?;
            }
        }

        if let Some(ref id) = entry.output_id {
            writeln!(file, "\nGenerated ID: {}", id)?;
        }

        Ok(())
    }

    /// Export all commands as an executable shell script
    pub fn export_script(&self, variables: &HashMap<String, String>) -> Result<()> {
        if let Some(ref script_file) = self.script_file {
            let mut file = File::create(script_file)?;

            // Write shebang and header
            writeln!(file, "#!/bin/bash")?;
            writeln!(file, "# Atlas CLI Test Reproduction Script")?;
            writeln!(
                file,
                "# Generated: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            )?;
            writeln!(
                file,
                "# This script reproduces all Atlas CLI commands from the test run\n"
            )?;

            // Error handling
            writeln!(file, "set -e  # Exit on error")?;
            writeln!(file, "set -u  # Exit on undefined variable\n")?;

            // Color output
            writeln!(file, "# Colors for output")?;
            writeln!(file, r#"RED="\033[0;31m""#)?;
            writeln!(file, r#"GREEN="\033[0;32m""#)?;
            writeln!(file, r#"YELLOW="\033[1;33m""#)?;
            writeln!(file, r#"BLUE="\033[0;34m""#)?;
            writeln!(file, r#"NC="\033[0m"\n"#)?;

            // Check Atlas CLI
            writeln!(file, "# Check Atlas CLI is available")?;
            writeln!(file, "if ! command -v atlas-cli &> /dev/null; then")?;
            writeln!(
                file,
                r#"    echo -e "${{RED}}Error: atlas-cli not found in PATH${{NC}}""#
            )?;
            writeln!(file, r#"    echo "Please install Atlas CLI first""#)?;
            writeln!(file, "    exit 1")?;
            writeln!(file, "fi\n")?;

            writeln!(
                file,
                r#"echo -e "${{GREEN}}Starting Atlas CLI test reproduction...${{NC}}"\n"#
            )?;

            // Add variables
            if !variables.is_empty() {
                writeln!(file, "# Manifest IDs from original test run")?;
                for (key, value) in variables {
                    writeln!(file, r#"# {}="{}""#, key, value)?;
                }
                writeln!(file)?;
            }

            // Helper function for manifest ID extraction
            writeln!(file, "# Helper function to extract manifest ID from output")?;
            writeln!(file, "extract_manifest_id() {{")?;
            writeln!(
                file,
                r#"    grep -oE "Manifest stored successfully with ID: [^ ]+" | cut -d" " -f6 || \"#
            )?;
            writeln!(file, r#"    grep -oE "ID: [^ ]+" | cut -d" " -f2 || \"#)?;
            writeln!(file, r#"    echo "unknown""#)?;
            writeln!(file, "}}\n")?;

            // Add commands
            let mut current_step = None;
            for (i, cmd) in self.commands.iter().enumerate() {
                if current_step.as_ref() != Some(&cmd.step) {
                    current_step = Some(cmd.step.clone());
                    writeln!(file, "\n# {}", "=".repeat(60))?;
                    writeln!(file, "# Step: {}", cmd.step)?;
                    writeln!(file, "# {}", "=".repeat(60))?;
                    writeln!(
                        file,
                        r#"echo -e "\n${{GREEN}}▶ Executing: {}${{NC}}""#,
                        cmd.step
                    )?;
                }

                writeln!(file, "\n# Command {}", i + 1)?;
                writeln!(file, r#"echo -e "${{BLUE}}$ {}${{NC}}""#, cmd.command)?;
                writeln!(file, "{}", cmd.command)?;

                // Error checking
                writeln!(file, "if [ $? -ne 0 ]; then")?;
                writeln!(
                    file,
                    r#"    echo -e "${{RED}}✗ Failed: {}${{NC}}""#,
                    cmd.step
                )?;
                writeln!(file, "    exit 1")?;
                writeln!(file, "else")?;
                writeln!(
                    file,
                    r#"    echo -e "${{GREEN}}✓ Completed: {}${{NC}}""#,
                    cmd.step
                )?;
                writeln!(file, "fi")?;
            }

            writeln!(
                file,
                r#"\necho -e "\n${{GREEN}}✅ All commands executed successfully!${{NC}}""#
            )?;

            // Make executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(script_file)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(script_file, perms)?;
            }

            println!("📝 Reproduction script saved to: {}", script_file.display());
        }

        Ok(())
    }

    /// Export commands to JSON
    pub fn export_json(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.commands)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Show execution summary
    pub fn show_summary(&self) {
        let total = self.commands.len();
        let successful = self.commands.iter().filter(|c| c.success).count();
        let failed = total - successful;

        println!("\n{}", "=".repeat(80));
        println!("📊 EXECUTION SUMMARY");
        println!("{}", "=".repeat(80));
        println!("Total Commands: {}", total);
        println!("✅ Successful: {}", successful);

        if failed > 0 {
            println!("❌ Failed: {}", failed);
            println!("\nFailed Commands:");
            for cmd in &self.commands {
                if !cmd.success {
                    println!(
                        "  - [{}] {}...",
                        cmd.step,
                        &cmd.command[..100.min(cmd.command.len())]
                    );
                }
            }
        }

        if let Some(ref log_file) = self.log_file {
            println!("\n📄 Full log: {}", log_file.display());
        }
        if let Some(ref script_file) = self.script_file {
            println!("📜 Reproduction script: {}", script_file.display());
        }

        println!("{}", "=".repeat(80));
    }

    /// Clear all recorded commands
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_command_recording() {
        let dir = tempdir().unwrap();
        let mut recorder = CommandRecorder::new(Some(dir.path().to_path_buf())).unwrap();

        recorder
            .record(
                "atlas-cli dataset create --name=Test",
                None,
                true,
                Some("Create Dataset"),
                Some("dataset_123".to_string()),
            )
            .unwrap();

        assert_eq!(recorder.commands.len(), 1);
        assert_eq!(recorder.commands[0].step, "Create Dataset");
        assert_eq!(
            recorder.commands[0].output_id,
            Some("dataset_123".to_string())
        );
    }

    #[test]
    fn test_script_export() {
        let dir = tempdir().unwrap();
        let mut recorder = CommandRecorder::new(Some(dir.path().to_path_buf())).unwrap();

        recorder
            .record(
                "atlas-cli dataset create --name=Test",
                None,
                true,
                Some("Create Dataset"),
                None,
            )
            .unwrap();

        recorder
            .record(
                "atlas-cli model create --name=Model",
                None,
                true,
                Some("Create Model"),
                None,
            )
            .unwrap();

        let mut vars = HashMap::new();
        vars.insert("DATASET_ID".to_string(), "dataset_123".to_string());

        recorder.export_script(&vars).unwrap();

        let script_path = dir.path().join("reproduce.sh");
        assert!(script_path.exists());

        let content = std::fs::read_to_string(script_path).unwrap();
        assert!(content.contains("#!/bin/bash"));
        assert!(content.contains("atlas-cli dataset create"));
        assert!(content.contains("atlas-cli model create"));
    }
}
