pub mod commands;
pub mod handlers;
use crate::error::Error;

// Re-export commonly used items
pub use commands::{
    CCAttestationCommands, DatasetCommands, ManifestCommands, ModelCommands, PipelineCommands,
    SoftwareCommands,
};
pub use handlers::{
    handle_cc_attestation_command, handle_dataset_command, handle_manifest_command,
    handle_model_command, handle_pipeline_command, handle_software_command,
};

// Optional: Add any CLI-specific constants or shared utilities
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CLI_NAME: &str = "atlas-cli";

pub fn format_error(error: &Error) -> String {
    match error {
        Error::Io(err) => format!("IO error: {err}"),
        Error::Storage(msg) => format!("Storage error: {msg}"),
        Error::Validation(msg) => format!("Validation error: {msg}"),
        Error::Manifest(msg) => format!("Manifest error: {msg}"),
        Error::Signing(msg) => format!("Signing error: {msg}"),
        Error::Serialization(msg) => format!("Serialization error: {msg}"),
        Error::InitializationError(msg) => format!("Initialization error: {msg}"),
        Error::HexDecode(err) => format!("Hex decode error: {err}"),
        Error::CCAttestationError(msg) => format!("CC attestation error: {msg}"),
        Error::Json(err) => format!("JSON error: {err}"),
    }
}

/// Helper function to print validation warnings to the user
pub fn print_validation_warning(message: &str) {
    eprintln!("Warning: {message}");
}

/// Helper function to confirm actions with the user
pub fn confirm_action(prompt: &str) -> bool {
    use std::io::{self, Write};

    print!("{prompt} [y/N]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        input.trim().to_lowercase() == "y"
    } else {
        false
    }
}

/// Function to initialize any CLI-specific requirements
pub fn initialize() -> Result<(), crate::error::Error> {
    // Set up logging if needed
    env_logger::init();

    // Check for required environment variables
    if std::env::var("REKOR_URL").is_err() {
        print_validation_warning("REKOR_URL not set, using default");
    }

    Ok(())
}

// Shared functionality for progress indication
pub mod progress {
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn create_progress_bar(len: u64) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("=>-"),
        );
        pb
    }
}
