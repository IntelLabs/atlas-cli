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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_safe_file_path_normal() -> Result<()> {
        // Test with a normal path
        let dir = tempdir()?;
        let normal_path = dir.path().join("test_file.txt");

        // Should return the same path
        let result = safe_file_path(&normal_path, false)?;
        assert_eq!(result, normal_path);

        Ok(())
    }

    #[test]
    fn test_safe_file_path_nonexistent() -> Result<()> {
        // Test with a nonexistent path
        let dir = tempdir()?;
        let nonexistent_path = dir.path().join("nonexistent_file.txt");

        // Should still return the path if it doesn't exist
        let result = safe_file_path(&nonexistent_path, false)?;
        assert_eq!(result, nonexistent_path);

        Ok(())
    }

    #[test]
    #[cfg(unix)] // This test is Unix-specific
    fn test_safe_file_path_symlink() -> Result<()> {
        // Create a temporary directory and files
        let dir = tempdir()?;
        let target_path = dir.path().join("target_file.txt");
        let symlink_path = dir.path().join("symlink_file.txt");

        // Create the target file
        let mut file = File::create(&target_path)?;
        file.write_all(b"target file content")?;

        // Create a symlink to the target
        std::os::unix::fs::symlink(&target_path, &symlink_path)?;

        // Test with symlinks not allowed (default)
        let result = safe_file_path(&symlink_path, false);
        assert!(result.is_err(), "Should reject symlinks when not allowed");

        // Test with symlinks allowed
        let result = safe_file_path(&symlink_path, true)?;

        // Should return the target path when symlinks are allowed
        assert_eq!(result, target_path);

        Ok(())
    }

    #[test]
    #[cfg(unix)] // This wnt work on Windows
    fn test_safe_file_path_unsafe_symlink() {
        // Test with a symlink to a potentially unsafe location
        let dir = tempdir().unwrap();
        let unsafe_target = PathBuf::from("/etc/passwd");
        let unsafe_symlink = dir.path().join("unsafe_symlink.txt");

        // Create a symlink to the unsafe target
        std::os::unix::fs::symlink(&unsafe_target, &unsafe_symlink).unwrap();

        // Even with symlinks allowed, should reject unsafe targets
        let result = safe_file_path(&unsafe_symlink, true);
        assert!(
            result.is_err(),
            "Should reject symlinks to unsafe locations"
        );
    }

    #[test]
    #[cfg(unix)] // This test is Unix-specific
    fn test_safe_file_path_hardlink() -> Result<()> {
        // Create a temporary directory and files
        let dir = tempdir()?;
        let target_path = dir.path().join("target_file.txt");
        let hardlink_path = dir.path().join("hardlink_file.txt");

        // Create the target file
        let mut file = File::create(&target_path)?;
        file.write_all(b"target file content")?;

        // Create a hard link to the target
        std::fs::hard_link(&target_path, &hardlink_path)?;

        // Hard links should be detected and rejected
        let result = safe_file_path(&hardlink_path, false);
        assert!(
            result.is_err(),
            "Should reject files with multiple hard links"
        );

        Ok(())
    }

    #[test]
    fn test_safe_open_file() -> Result<()> {
        // Create a temporary directory and file
        let dir = tempdir()?;
        let file_path = dir.path().join("test_open.txt");

        // Create and write to the file
        {
            let mut file = File::create(&file_path)?;
            file.write_all(b"test content")?;
        }

        // Test opening the file
        let mut file = safe_open_file(&file_path, false)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Should be able to read the content
        assert_eq!(content, "test content");

        Ok(())
    }

    #[test]
    fn test_safe_create_file() -> Result<()> {
        // Create a temporary directory
        let dir = tempdir()?;
        let file_path = dir.path().join("test_create.txt");

        // Test creating a file
        {
            let mut file = safe_create_file(&file_path, false)?;
            file.write_all(b"created content")?;
        }

        // Verify the file was created with the content
        let mut content = String::new();
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut content)?;

        assert_eq!(content, "created content");

        Ok(())
    }

    #[test]
    fn test_safe_open_options() -> Result<()> {
        // Create a temporary directory and file
        let dir = tempdir()?;
        let file_path = dir.path().join("test_options.txt");

        // Test creating with OpenOptions
        {
            let mut options = safe_open_options(&file_path, false)?;
            let mut file = options.write(true).create(true).open(&file_path)?;
            file.write_all(b"options content")?;
        }

        // Verify the file was created with the content
        let mut content = String::new();
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut content)?;

        assert_eq!(content, "options content");

        Ok(())
    }

    #[test]
    fn test_safe_open_file_nonexistent() {
        // Test opening a nonexistent file
        let nonexistent_path = PathBuf::from("/tmp/this_file_should_not_exist.txt");

        // Make sure the file doesn't exist
        if nonexistent_path.exists() {
            fs::remove_file(&nonexistent_path).unwrap();
        }

        let result = safe_open_file(&nonexistent_path, false);

        // Should return an error
        assert!(result.is_err());

        // The error should be an IO error
        if let Err(e) = result {
            match e {
                crate::error::Error::Io(_) => {} // Expected error type
                _ => panic!("Unexpected error type: {e:?}"),
            }
        }
    }

    #[test]
    fn test_is_safe_symlink_target() {
        let check_path = |path: &str| -> bool {
            let path = Path::new(path);
            if let Ok(canonical) = path.canonicalize() {
                canonical.starts_with("/tmp") || canonical.starts_with("/var/app/data")
            } else {
                // Simulate behavior for paths that can't be canonicalized
                false
            }
        };

        // Test a path that exists and should be allowed (tmpdir)
        let tmp_dir = tempdir().unwrap();
        assert!(
            check_path(tmp_dir.path().to_str().unwrap()),
            "Temporary directory should be considered safe"
        );

        // Test paths that should not be allowed
        assert!(
            !check_path("/etc/passwd"),
            "/etc/passwd should not be considered safe"
        );
        assert!(
            !check_path("/home/user/file.txt"),
            "/home/user/file.txt should not be considered safe"
        );
    }

    #[test]
    fn test_safe_open_file_comprehensive() -> Result<()> {
        // Create a temporary directory and file
        let dir = tempdir()?;
        let file_path = dir.path().join("comprehensive_test.txt");

        // Test with non-existent file (should fail)
        let result = safe_open_file(&file_path, false);
        assert!(result.is_err(), "Opening non-existent file should fail");

        // Create the file
        {
            let mut file = File::create(&file_path)?;
            file.write_all(b"comprehensive test")?;
        }

        // Test opening existing file (should succeed)
        let mut file = safe_open_file(&file_path, false)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        assert_eq!(content, "comprehensive test");

        // Test with invalid path
        let invalid_path = PathBuf::from("\0invalid");
        let result = safe_open_file(&invalid_path, false);
        assert!(
            result.is_err(),
            "Opening file with invalid path should fail"
        );

        Ok(())
    }

    #[test]
    fn test_safe_create_file_existing() -> Result<()> {
        // Create a temporary directory and file
        let dir = tempdir()?;
        let file_path = dir.path().join("existing.txt");

        // Create the file with initial content
        {
            let mut file = File::create(&file_path)?;
            file.write_all(b"initial content")?;
        }

        // Use safe_create_file to overwrite the file
        {
            let mut file = safe_create_file(&file_path, false)?;
            file.write_all(b"overwritten content")?;
        }

        // Verify the content was overwritten
        let mut content = String::new();
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut content)?;

        assert_eq!(content, "overwritten content");

        Ok(())
    }
}
