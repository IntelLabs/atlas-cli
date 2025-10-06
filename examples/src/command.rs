use std::fmt;

#[derive(Debug, Clone)]
pub struct AtlasCommand {
    parts: Vec<String>,
}

impl AtlasCommand {
    pub fn new(base_cmd: &str) -> Self {
        Self {
            parts: vec![base_cmd.to_string()],
        }
    }

    pub fn add_subcommand(&mut self, command: &str, action: &str) -> &mut Self {
        self.parts.push(command.to_string());
        self.parts.push(action.to_string());
        self
    }

    pub fn add_flag<T: ToString>(&mut self, flag: &str, value: Option<T>) -> &mut Self {
        if let Some(val) = value {
            let flag_str = if flag.len() == 1 {
                format!("-{}", flag)
            } else {
                format!("--{}", flag)
            };

            let value_str = val.to_string();

            if value_str == "true" {
                self.parts.push(flag_str);
            } else if value_str != "false" {
                if self.needs_quoting(&value_str) {
                    self.parts.push(format!(
                        "{}=\"{}\"",
                        flag_str,
                        value_str.replace('\"', "\\\"")
                    ));
                } else {
                    self.parts.push(format!("{}={}", flag_str, value_str));
                }
            }
        }
        self
    }

    pub fn add_multi_flag(&mut self, flag: &str, values: &[String]) -> &mut Self {
        for value in values {
            self.add_flag(flag, Some(value));
        }
        self
    }

    pub fn build(&self) -> String {
        self.parts.join(" ")
    }

    fn needs_quoting(&self, s: &str) -> bool {
        s.contains(|c: char| c.is_whitespace() || "\"'`$\\!*?<>|&;(){}[]#~".contains(c))
    }
}

impl fmt::Display for AtlasCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build())
    }
}

impl AtlasCommand {
    pub fn dataset(mut self, action: &str) -> Self {
        self.add_subcommand("dataset", action);
        self
    }

    pub fn model(mut self, action: &str) -> Self {
        self.add_subcommand("model", action);
        self
    }

    pub fn software(mut self, action: &str) -> Self {
        self.add_subcommand("software", action);
        self
    }

    pub fn evaluation(mut self, action: &str) -> Self {
        self.add_subcommand("evaluation", action);
        self
    }

    pub fn manifest(mut self, action: &str) -> Self {
        self.add_subcommand("manifest", action);
        self
    }

    pub fn with_paths(mut self, paths: &[String]) -> Self {
        if !paths.is_empty() {
            self.add_flag("paths", Some(paths.join(",")));
        }
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.add_flag("name", Some(name));
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.add_flag("description", Some(desc));
        self
    }

    pub fn with_storage(mut self, storage_type: &str, storage_url: &str) -> Self {
        self.add_flag("storage-type", Some(storage_type));
        self.add_flag("storage-url", Some(storage_url));
        self
    }

    pub fn with_signing_key(mut self, key_path: &str) -> Self {
        self.add_flag("key", Some(key_path));
        self
    }

    pub fn with_hash_alg(mut self, alg: &str) -> Self {
        self.add_flag("hash-alg", Some(alg));
        self
    }

    pub fn with_linked_manifests(mut self, manifests: &[String]) -> Self {
        for manifest in manifests {
            self.add_flag("linked-manifests", Some(manifest));
        }
        self
    }

    pub fn with_author(mut self, org: Option<&str>, name: Option<&str>) -> Self {
        if let Some(org) = org {
            self.add_flag("author-org", Some(org));
        }
        if let Some(name) = name {
            self.add_flag("author-name", Some(name));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_command() {
        let cmd = AtlasCommand::new("atlas-cli")
            .dataset("create")
            .with_name("Test Dataset")
            .build();

        assert_eq!(cmd, "atlas-cli dataset create --name=\"Test Dataset\"");
    }

    #[test]
    fn test_complex_command() {
        let cmd = AtlasCommand::new("atlas-cli")
            .model("create")
            .with_paths(&["model.pkl".to_string(), "config.json".to_string()])
            .with_name("Test Model")
            .with_description("A test model")
            .with_storage("database", "http://localhost:8080")
            .with_signing_key("/path/to/key.pem")
            .with_hash_alg("sha384")
            .build();

        assert!(cmd.contains("--paths=model.pkl,config.json"));
        assert!(cmd.contains("--name=\"Test Model\""));
        assert!(cmd.contains("--description=\"A test model\""));
    }

    #[test]
    fn test_boolean_flags() {
        let mut cmd = AtlasCommand::new("atlas-cli");
        cmd.add_flag("verbose", Some(true));
        cmd.add_flag("quiet", Some(false));
        let result = cmd.build();

        assert!(result.contains("--verbose"));
        assert!(!result.contains("--quiet"));
    }

    #[test]
    fn test_quoting_special_chars() {
        let mut cmd = AtlasCommand::new("atlas-cli");
        cmd.add_flag("name", Some("Test $pecial"));
        let result = cmd.build();

        assert!(result.contains("--name=\"Test $pecial\""));
    }
}
