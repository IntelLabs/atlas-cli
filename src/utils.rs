use crate::error::{Error, Result};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

/// Ensures the path is not a symlink or hard link unless explicitly allowed
pub fn safe_file_path(path: &Path, allow_symlinks: bool) -> Result<PathBuf> {
    // Check if the file exists
    if path.exists() {
        // Check if it's a symlink
        if path.is_symlink() {
            if !allow_symlinks {
                return Err(Error::Validation(format!(
                    "Security error: Path {} is a symlink, which is not allowed",
                    path.display()
                )));
            }

            // If symlinks are allowed, check the target is valid
            let target = fs::read_link(path)?;

            // Validate the target path (customize this logic based on your requirements)
            // For example, ensure it's within a specific directory
            if !is_safe_symlink_target(&target) {
                return Err(Error::Validation(format!(
                    "Security error: Symlink target {} is not in an allowed location",
                    target.display()
                )));
            }

            return Ok(target);
        }

        // Check for hard links (files with multiple links)
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(path)?;
            if metadata.nlink() > 1 {
                return Err(Error::Validation(format!(
                    "Security error: Path {} has multiple hard links ({})",
                    path.display(),
                    metadata.nlink()
                )));
            }
        }
    }

    // Path is safe or doesn't exist yet !
    Ok(path.to_path_buf())
}

/// Checks if a symlink target is in an allowed location
fn is_safe_symlink_target(target: &Path) -> bool {
    if let Ok(canonical) = target.canonicalize() {
        // Only allow /tmp or /var/app/data for now
        canonical.starts_with("/tmp") || canonical.starts_with("/var/app/data")
    } else {
        false
    }
}

/// Safely opens a file for reading
pub fn safe_open_file(path: &Path, allow_symlinks: bool) -> Result<File> {
    let safe_path = safe_file_path(path, allow_symlinks)?;
    File::open(&safe_path).map_err(Error::from)
}

/// Safely creates a file for writing
pub fn safe_create_file(path: &Path, allow_symlinks: bool) -> Result<File> {
    let safe_path = safe_file_path(path, allow_symlinks)?;
    File::create(&safe_path).map_err(Error::from)
}

/// Safely opens a file with custom options
pub fn safe_open_options(path: &Path, allow_symlinks: bool) -> Result<OpenOptions> {
    let _safe_path = safe_file_path(path, allow_symlinks)?;
    Ok(OpenOptions::new())
}
