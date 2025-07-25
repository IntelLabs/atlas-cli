//! # Atlas CLI
//!
//! Machine Learning (ML) Lifecycle & Transparency Manager
//!
//! A command-line interface tool for creating, managing, and verifying Content Provenance
//! and Authenticity (C2PA) manifests for machine learning models, datasets, and related artifacts.
//!
//! ## Installation
//!
//! ```bash
//! cargo install atlas-cli
//! ```
//!
//! ## Quick Start
//!
//! Create a model manifest:
//! ```bash
//! atlas-cli model create \
//!     --paths=model.onnx \
//!     --ingredient-names="Main Model" \
//!     --name="My Model" \
//!     --author-org="My Organization" \
//!     --author-name="My Name" \
//!     --print
//! ```
//!
//! For more examples and detailed documentation, see:
//! - [User Guide](https://github.com/IntelLabs/atlas-cli/blob/main/docs/USER_GUIDE.md)
//! - [Examples](https://github.com/IntelLabs/atlas-cli/blob/main/docs/EXAMPLES.md)
//! - [Development Guide](https://github.com/IntelLabs/atlas-cli/blob/main/docs/DEVELOPMENT.md)

#![doc(html_root_url = "https://docs.rs/atlas-cli/0.1.0")]

pub mod cc_attestation;
pub mod cli;
pub mod error;
pub mod hash;
pub mod in_toto;
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
///
/// # Examples
///
/// ```
/// use atlas_cli::init_logging;
///
/// // Initialize with default settings
/// let result = init_logging();
/// // Note: This might fail if already initialized
/// assert!(result.is_ok() || result.is_err());
/// ```
pub fn init_logging() -> Result<()> {
    env_logger::try_init().map_err(|e| Error::InitializationError(e.to_string()))
}

// Re-export commonly used types and traits
pub use storage::traits::StorageBackend;
