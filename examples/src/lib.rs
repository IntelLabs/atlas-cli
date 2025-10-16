pub mod subcommands;
pub mod command;
pub mod config;
pub mod error;
pub mod framework;
pub mod recorder;
pub mod utils;

// Re-export main types
pub use command::AtlasCommand;
pub use config::{Environment, Step, WorkflowConfig};
pub use error::{AtlasError, Result};
pub use framework::AtlasTestFramework;
pub use recorder::CommandRecorder;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
