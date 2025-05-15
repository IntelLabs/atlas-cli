mod types;

pub use types::{Error, Result};

/// Format error for display to the user
pub fn format_error(error: &Error) -> String {
    error.to_string()
}
