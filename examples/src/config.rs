use crate::error::{AtlasError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub environment: Environment,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Environment {
    pub storage_type: String,
    pub storage_url: String,
    pub signing_key: String,
    pub verifying_key: String,
    #[serde(default)]
    pub generate_keys: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_org: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    pub output_dir: String,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub interactive: bool,
    #[serde(default)]
    pub continue_on_error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_alg: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Step {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub action: String,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_as: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_after: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expect: Option<String>,
}

impl WorkflowConfig {
    /// Load configuration from YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: WorkflowConfig = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration structure
    pub fn validate(&self) -> Result<()> {
        // Check required fields
        if self.steps.is_empty() {
            return Err(AtlasError::ValidationError(
                "Configuration must contain at least one step".to_string(),
            ));
        }

        // Validate each step
        for (i, step) in self.steps.iter().enumerate() {
            if step.action.is_empty() {
                return Err(AtlasError::ValidationError(format!(
                    "Step {} missing required 'action' field",
                    i + 1
                )));
            }

            // Check action format (should contain ':')
            if !step.action.contains(':') {
                return Err(AtlasError::ValidationError(format!(
                    "Step {} action '{}' must be in format 'category:action'",
                    i + 1,
                    step.action
                )));
            }
        }

        // Validate environment
        self.environment.validate()?;

        Ok(())
    }

    /// Override settings from command line arguments
    pub fn apply_overrides(&mut self, overrides: ConfigOverrides) {
        if let Some(dry_run) = overrides.dry_run {
            self.environment.dry_run = dry_run;
        }
        if let Some(interactive) = overrides.interactive {
            self.environment.interactive = interactive;
        }
        if let Some(continue_on_error) = overrides.continue_on_error {
            self.environment.continue_on_error = continue_on_error;
        }
        if let Some(output_dir) = overrides.output_dir {
            self.environment.output_dir = output_dir;
        }
    }
}

impl Environment {
    /// Validate environment settings
    pub fn validate(&self) -> Result<()> {
        // Check storage type
        let valid_storage = vec!["database", "local-fs", "filesystem", "rekor"];
        if !valid_storage.contains(&self.storage_type.as_str()) {
            return Err(AtlasError::ValidationError(format!(
                "Invalid storage_type: {}",
                self.storage_type
            )));
        }

        Ok(())
    }

    /// Get hash algorithm with default
    pub fn hash_algorithm(&self) -> &str {
        self.hash_alg.as_deref().unwrap_or("sha384")
    }
}

/// Command line override options
#[derive(Debug, Default, Clone)]
pub struct ConfigOverrides {
    pub dry_run: Option<bool>,
    pub interactive: Option<bool>,
    pub continue_on_error: Option<bool>,
    pub output_dir: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let yaml = r#"
name: "Test Workflow"
description: "Test description"
environment:
  storage_type: database
  storage_url: http://localhost:8080
  signing_key: key.pem
  verifying_key: pub.pem
  output_dir: ./output
steps:
  - name: "Test Step"
    action: "dataset:create"
    parameters:
      name: "Test Dataset"
"#;

        let config: WorkflowConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_storage_type() {
        let yaml = r#"
name: "Test Workflow"
environment:
  storage_type: invalid
  storage_url: http://localhost:8080
  signing_key: key.pem
  verifying_key: pub.pem
  output_dir: ./output
steps:
  - name: "Test Step"
    action: "dataset:create"
"#;

        let config: WorkflowConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_action_format() {
        let yaml = r#"
name: "Test Workflow"
environment:
  storage_type: database
  storage_url: http://localhost:8080
  signing_key: key.pem
  verifying_key: pub.pem
  output_dir: ./output
steps:
  - name: "Test Step"
    action: "invalid_format"
"#;

        let config: WorkflowConfig = serde_yaml::from_str(yaml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("category:action"));
    }
}
