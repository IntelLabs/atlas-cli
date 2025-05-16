pub mod cc_attestation;
pub mod cli;
pub mod error;
pub mod hash;
pub mod manifest;
pub mod signing;
pub mod storage;
#[cfg(test)]
mod tests;
pub mod utils;

use std::path::PathBuf;
use storage::config::StorageConfig;

// Re-export error types
pub use error::{Error, Result};

/// CLI configuration options
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to private key for signing
    pub key_path: Option<PathBuf>,
    /// Storage backend configuration
    pub storage_config: StorageConfig,
    /// Whether to show progress bars
    pub show_progress: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_path: None,
            storage_config: StorageConfig::default(),
            show_progress: true,
        }
    }
}

/// Initialize logging for the CLI
pub fn init_logging() -> Result<()> {
    env_logger::try_init().map_err(|e| Error::InitializationError(e.to_string()))
}

// Re-export commonly used types and traits
pub use storage::traits::StorageBackend;
