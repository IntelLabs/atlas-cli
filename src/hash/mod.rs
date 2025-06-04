use crate::error::{Error, Result};
use sha2::{Digest, Sha256};
use std::path::Path;
pub mod utils;
pub fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn calculate_file_hash(path: impl AsRef<Path>) -> Result<String> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(calculate_hash(&buffer))
}

pub fn combine_hashes(hashes: &[&str]) -> Result<String> {
    let mut hasher = Sha256::new();
    for hash in hashes {
        let bytes = hex::decode(hash).map_err(Error::HexDecode)?;
        hasher.update(&bytes);
    }
    Ok(hex::encode(hasher.finalize()))
}
// Additional hash-related functionality
pub fn verify_hash(data: &[u8], expected_hash: &str) -> bool {
    let calculated_hash = hex::encode(Sha256::digest(data));
    calculated_hash == expected_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use crate::utils::safe_create_file;
    use std::fs::OpenOptions;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_hash() {
        let data = b"test data";
        let hash = calculate_hash(data);
        assert_eq!(hash.len(), 64); // SHA-256 hash is 64 hex characters
    }

    #[test]
    fn test_calculate_file_hash() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");

        // Create a test file
        let mut file = safe_create_file(&file_path, false)?;
        file.write_all(b"test data")?;

        let hash = calculate_file_hash(&file_path)?;
        assert_eq!(hash.len(), 64);

        // Verify hash changes with content
        let mut file = safe_create_file(&file_path, false)?;
        file.write_all(b"different data")?;

        let new_hash = calculate_file_hash(&file_path)?;
        assert_ne!(hash, new_hash);

        Ok(())
    }

    #[test]
    fn test_verify_hash() {
        let data = b"test data";
        let hash = calculate_hash(data);

        assert!(verify_hash(data, &hash));
        assert!(!verify_hash(b"different data", &hash));

        // Additional verification tests
        let test_data = b"test verification data";
        let test_hash = calculate_hash(test_data);

        // Verification should succeed with correct hash
        assert!(verify_hash(test_data, &test_hash));

        // Verification should fail with incorrect hash
        assert!(!verify_hash(test_data, "incorrect_hash"));

        // Verification should fail with empty hash
        assert!(!verify_hash(test_data, ""));

        // Verify empty data
        let empty_hash = calculate_hash(b"");
        assert!(verify_hash(b"", &empty_hash));

        // Verification should fail with hash of wrong length
        assert!(!verify_hash(test_data, "short"));

        // Verification should fail with non-hex characters
        assert!(!verify_hash(test_data, &("Z".repeat(64))));
    }

    #[test]
    fn test_combine_hashes() -> Result<()> {
        let hash1 = calculate_hash(b"data1");
        let hash2 = calculate_hash(b"data2");

        let combined = combine_hashes(&[&hash1, &hash2])?;
        assert_eq!(combined.len(), 64);

        // Test order matters
        let combined2 = combine_hashes(&[&hash2, &hash1])?;
        assert_ne!(combined, combined2);

        Ok(())
    }

    #[test]
    fn test_hash_idempotence() {
        let data = b"hello world";
        let hash1 = calculate_hash(data);
        let hash2 = calculate_hash(data);

        // The same data should produce the same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_uniqueness() {
        let data1 = b"hello world";
        let data2 = b"Hello World"; // Capitalization should produce different hash

        let hash1 = calculate_hash(data1);
        let hash2 = calculate_hash(data2);

        // Different data should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_empty_data_hash() {
        let data = b"";
        let hash = calculate_hash(data);

        // Empty string should produce a valid hash with expected length
        assert_eq!(hash.len(), 64);
        // Known SHA-256 hash of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_known_values() {
        // Test vectors for SHA-256 with explicit type annotation
        let test_vectors: [(&[u8], &str); 2] = [
            (
                b"abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ];

        for (input, expected) in &test_vectors {
            let hash = calculate_hash(input);
            assert_eq!(&hash, expected);
        }
    }

    #[test]
    fn test_combine_hashes_determinism() -> Result<()> {
        let hash1 = calculate_hash(b"data1");
        let hash2 = calculate_hash(b"data2");

        let combined1 = combine_hashes(&[&hash1, &hash2])?;
        let combined2 = combine_hashes(&[&hash1, &hash2])?;

        // The same input hashes should produce the same combined hash
        assert_eq!(combined1, combined2);

        Ok(())
    }

    #[test]
    fn test_combine_hashes_empty() -> Result<()> {
        // Create a single hash
        let hash1 = calculate_hash(b"data1");

        // Test combining single hash
        let result = combine_hashes(&[&hash1])?;
        assert_eq!(result.len(), 64);

        // Test combining empty list of hashes
        match combine_hashes(&[]) {
            Ok(hash) => {
                // If it succeeds, verify it's a valid hash
                assert_eq!(hash.len(), 64);
                // The hash of empty input should generally match a default value
                // or be derived predictably from the empty input
            }
            Err(e) => {
                // If it errors, the error should indicate empty input
                assert!(
                    e.to_string().contains("empty")
                        || e.to_string().contains("no hashes")
                        || e.to_string().contains("invalid input"),
                    "Expected error about empty input, got: {e}"
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_file_hash_changes() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test_changes.txt");

        // Test with initial content
        {
            let mut file = safe_create_file(&file_path, false)?;
            file.write_all(b"initial content")?;
        }
        let hash1 = calculate_file_hash(&file_path)?;

        // Test after appending content
        {
            let mut file = OpenOptions::new().append(true).open(&file_path)?;
            file.write_all(b" with more data")?;
        }
        let hash2 = calculate_file_hash(&file_path)?;

        // Hashes should be different
        assert_ne!(hash1, hash2);

        // Test after overwriting with same content as initial
        {
            let mut file = safe_create_file(&file_path, false)?;
            file.write_all(b"initial content")?;
        }
        let hash3 = calculate_file_hash(&file_path)?;

        // Hash should be the same as the first hash
        assert_eq!(hash1, hash3);

        Ok(())
    }
}
