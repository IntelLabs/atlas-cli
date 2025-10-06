use thiserror::Error;

pub type Result<T> = std::result::Result<T, AtlasError>;

#[derive(Error, Debug)]
pub enum AtlasError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Command execution failed: {0}")]
    CommandError(String),

    #[error("Path resolution error: {0}")]
    PathError(String),

    #[error("Variable resolution error: {0}")]
    VariableError(String),

    #[error("Atlas CLI not found in PATH")]
    AtlasCliNotFound,

    #[error("Manifest ID extraction failed: {0}")]
    ManifestIdError(String),

    #[error("Action '{0}' not implemented")]
    UnknownAction(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Pattern matching error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("{0}")]
    Custom(String),
}

impl From<String> for AtlasError {
    fn from(s: String) -> Self {
        AtlasError::Custom(s)
    }
}

impl From<&str> for AtlasError {
    fn from(s: &str) -> Self {
        AtlasError::Custom(s.to_string())
    }
}
