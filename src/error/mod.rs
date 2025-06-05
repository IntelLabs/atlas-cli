mod types;

pub use types::{Error, Result};

/// Format an error for display to the user
///
/// # Examples
///
/// ```
/// use atlas_cli::error::{Error, format_error};
///
/// let error = Error::Validation("Invalid input".to_string());
/// let formatted = format_error(&error);
/// assert_eq!(formatted, "Validation error: Invalid input");
///
/// let io_error = Error::Storage("Connection failed".to_string());
/// let formatted = format_error(&io_error);
/// assert_eq!(formatted, "Storage error: Connection failed");
/// ```
pub fn format_error(error: &Error) -> String {
    error.to_string()
}
