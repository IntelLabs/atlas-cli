//! # Hash Module
//!
//! This module provides cryptographic hash functions for the Atlas CLI, supporting
//! SHA-256, SHA-384, and SHA-512 algorithms. It integrates with the `atlas-c2pa-lib`
//! to use consistent hash algorithm types throughout the codebase.
//!
//! ## Features
//!
//! - Calculate hashes of byte data with configurable algorithms
//! - Calculate file hashes efficiently using streaming
//! - Combine multiple hashes into a single hash
//! - Verify data integrity by comparing hashes
//! - Automatic algorithm detection based on hash length
//!
//! ## Algorithm Support
//!
//! The module supports the following hash algorithms:
//! - **SHA-256**: 256-bit hash (64 hex characters) - Default for backward compatibility
//! - **SHA-384**: 384-bit hash (96 hex characters) - Default for new manifests
//! - **SHA-512**: 512-bit hash (128 hex characters) - Maximum security
//!
//! ## Examples
//!
//! ### Basic hashing with default algorithm (SHA-384)
//! ```
//! use atlas_cli::hash::calculate_hash;
//!
//! let data = b"Hello, World!";
//! let hash = calculate_hash(data);
//! assert_eq!(hash.len(), 96); // SHA-384 produces 96 hex characters
//! ```
//!
//! ### Hashing with specific algorithm
//! ```
//! use atlas_cli::hash::calculate_hash_with_algorithm;
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//!
//! let data = b"Hello, World!";
//! let hash = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha512);
//! assert_eq!(hash.len(), 128); // SHA-512 produces 128 hex characters
//! ```
//!
//! ### File hashing
//! ```no_run
//! use atlas_cli::hash::calculate_file_hash_with_algorithm;
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//! use std::path::Path;
//!
//! let path = Path::new("large_file.bin");
//! let hash = calculate_file_hash_with_algorithm(path, &HashAlgorithm::Sha384).unwrap();
//! assert_eq!(hash.len(), 96); // SHA-384 produces 96 hex characters
//! ```

use crate::error::{Error, Result};
use crate::utils::safe_open_file;
use atlas_c2pa_lib::cose::HashAlgorithm;
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::io::Read;
use std::path::Path;
use subtle::ConstantTimeEq;

/// Calculate SHA-384 hash of the given data
///
/// This function uses SHA-384 by default. For other algorithms, use
/// [`calculate_hash_with_algorithm`].
///
/// # Arguments
///
/// * `data` - The byte slice to hash
///
/// # Returns
///
/// A hexadecimal string representation of the hash (96 characters for SHA-384)
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::calculate_hash;
///
/// let data = b"Hello, World!";
/// let hash = calculate_hash(data);
///
/// // SHA-384 produces 96 character hex string
/// assert_eq!(hash.len(), 96);
///
/// // Same data produces same hash
/// let hash2 = calculate_hash(data);
/// assert_eq!(hash, hash2);
///
/// // Different data produces different hash
/// let hash3 = calculate_hash(b"Different data");
/// assert_ne!(hash, hash3);
/// ```
pub fn calculate_hash(data: &[u8]) -> String {
    calculate_hash_with_algorithm(data, &HashAlgorithm::Sha384)
}

/// Calculate hash of data using the specified algorithm
///
/// # Arguments
///
/// * `data` - The byte slice to hash
/// * `algorithm` - The hash algorithm to use (SHA-256, SHA-384, or SHA-512)
///
/// # Returns
///
/// A hexadecimal string representation of the hash:
/// - SHA-256: 64 characters
/// - SHA-384: 96 characters
/// - SHA-512: 128 characters
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::calculate_hash_with_algorithm;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
///
/// let data = b"Hello, World!";
///
/// // SHA-256
/// let hash256 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha256);
/// assert_eq!(hash256.len(), 64);
///
/// // SHA-384
/// let hash384 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha384);
/// assert_eq!(hash384.len(), 96);
///
/// // SHA-512
/// let hash512 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha512);
/// assert_eq!(hash512.len(), 128);
///
/// // Different algorithms produce different hashes
/// assert_ne!(hash256, hash384);
/// assert_ne!(hash384, hash512);
/// ```
pub fn calculate_hash_with_algorithm(data: &[u8], algorithm: &HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::Sha256 => hex::encode(Sha256::digest(data)),
        HashAlgorithm::Sha384 => hex::encode(Sha384::digest(data)),
        HashAlgorithm::Sha512 => hex::encode(Sha512::digest(data)),
    }
}

/// Calculate SHA-256 hash of a file
///
/// This function uses SHA-256 by default. For other algorithms, use
/// [`calculate_file_hash_with_algorithm`].
///
/// # Arguments
///
/// * `path` - Path to the file to hash
///
/// # Returns
///
/// * `Ok(String)` - The hexadecimal hash string (64 characters for SHA-384)
/// * `Err(Error)` - If the file cannot be read
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::hash::calculate_file_hash;
/// use std::path::Path;
///
/// let path = Path::new("example.txt");
/// match calculate_file_hash(&path) {
///     Ok(hash) => {
///         assert_eq!(hash.len(), 96);
///         println!("File hash: {}", hash);
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn calculate_file_hash(path: impl AsRef<Path>) -> Result<String> {
    calculate_file_hash_with_algorithm(path, &HashAlgorithm::Sha384)
}

/// Calculate hash of a file using the specified algorithm
///
/// This function efficiently hashes files of any size by reading them in chunks,
/// avoiding loading the entire file into memory.
///
/// # Arguments
///
/// * `path` - Path to the file to hash
/// * `algorithm` - The hash algorithm to use
///
/// # Returns
///
/// * `Ok(String)` - The hexadecimal hash string
/// * `Err(Error)` - If the file cannot be read
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::hash::calculate_file_hash_with_algorithm;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
/// use std::path::Path;
///
/// let path = Path::new("large_model.onnx");
///
/// // Use SHA-512 for maximum security
/// let hash = calculate_file_hash_with_algorithm(&path, &HashAlgorithm::Sha512)?;
/// assert_eq!(hash.len(), 128);
///
/// # Ok::<(), atlas_cli::error::Error>(())
/// ```
pub fn calculate_file_hash_with_algorithm(
    path: impl AsRef<Path>,
    algorithm: &HashAlgorithm,
) -> Result<String> {
    let file = safe_open_file(path.as_ref(), false)?;

    match algorithm {
        HashAlgorithm::Sha256 => hash_reader::<Sha256, _>(file),
        HashAlgorithm::Sha512 => hash_reader::<Sha512, _>(file),
        _ => hash_reader::<Sha384, _>(file),
    }
}

///
/// This function concatenates the decoded bytes of multiple hashes and produces
/// a new SHA-384 hash. This is useful for creating a single hash that represents
/// multiple components.
///
/// # Arguments
///
/// * `hashes` - Array of hexadecimal hash strings to combine
///
/// # Returns
///
/// * `Ok(String)` - The combined hash (96 characters, SHA-384)
/// * `Err(Error)` - If any input hash is invalid hexadecimal
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::{calculate_hash, combine_hashes};
///
/// let hash1 = calculate_hash(b"data1");
/// let hash2 = calculate_hash(b"data2");
///
/// let combined = combine_hashes(&[&hash1, &hash2]).unwrap();
/// assert_eq!(combined.len(), 96);
///
/// // Order matters
/// let combined_reversed = combine_hashes(&[&hash2, &hash1]).unwrap();
/// assert_ne!(combined, combined_reversed);
/// ```
pub fn combine_hashes(hashes: &[&str]) -> Result<String> {
    let mut hasher = Sha384::new();
    for hash in hashes {
        let bytes = hex::decode(hash).map_err(Error::HexDecode)?;
        hasher.update(&bytes);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Verify that data matches the expected hash
///
/// This function automatically detects the hash algorithm based on the hash length
/// and verifies that the provided data produces the same hash.
///
/// # Arguments
///
/// * `data` - The data to verify
/// * `expected_hash` - The expected hash in hexadecimal format
///
/// # Returns
///
/// * `true` if the data matches the hash
/// * `false` if the data doesn't match or the hash format is invalid
///
/// # Algorithm Detection
///
/// - 64 characters: SHA-256
/// - 96 characters: SHA-384
/// - 128 characters: SHA-512
/// - Other lengths: Defaults to SHA-256
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::{calculate_hash, verify_hash};
///
/// let data = b"test data";
/// let hash = calculate_hash(data);
///
/// // Correct data verifies successfully
/// assert!(verify_hash(data, &hash));
///
/// // Wrong data fails verification
/// assert!(!verify_hash(b"wrong data", &hash));
///
/// // Invalid hash format returns false
/// assert!(!verify_hash(data, "invalid_hash"));
/// ```
pub fn verify_hash(data: &[u8], expected_hash: &str) -> bool {
    let algorithm = detect_hash_algorithm(expected_hash);
    let calculated_hash = calculate_hash_with_algorithm(data, &algorithm);

    // Convert both to bytes for constant-time comparison
    let calculated_bytes = calculated_hash.as_bytes();
    let expected_bytes = expected_hash.as_bytes();

    // Length must match first
    if calculated_bytes.len() != expected_bytes.len() {
        return false;
    }

    // Constant-time comparison
    calculated_bytes.ct_eq(expected_bytes).into()
}

/// Verify hash with an explicitly specified algorithm
///
/// Use this when you know which algorithm was used to create the hash.
///
/// # Arguments
///
/// * `data` - The data to verify
/// * `expected_hash` - The expected hash in hexadecimal format
/// * `algorithm` - The hash algorithm that was used
///
/// # Returns
///
/// * `true` if the data matches the hash
/// * `false` if the data doesn't match
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::{calculate_hash_with_algorithm, verify_hash_with_algorithm};
/// use atlas_c2pa_lib::cose::HashAlgorithm;
///
/// let data = b"test data";
/// let hash = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha384);
///
/// // Verification with correct algorithm succeeds
/// assert!(verify_hash_with_algorithm(data, &hash, &HashAlgorithm::Sha384));
///
/// // Verification with wrong algorithm fails
/// assert!(!verify_hash_with_algorithm(data, &hash, &HashAlgorithm::Sha256));
/// ```
pub fn verify_hash_with_algorithm(
    data: &[u8],
    expected_hash: &str,
    algorithm: &HashAlgorithm,
) -> bool {
    let calculated_hash = calculate_hash_with_algorithm(data, algorithm);

    let calculated_bytes = match hex::decode(calculated_hash) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let expected_bytes = match hex::decode(expected_hash) {
        Ok(b) => b,
        Err(_) => return false,
    };

    if calculated_bytes.len() != expected_bytes.len() {
        return false;
    }

    calculated_bytes.ct_eq(&expected_bytes).into()
}

/// Detect hash algorithm based on hash length
///
/// This function infers the hash algorithm from the hexadecimal hash string length.
///
/// # Arguments
///
/// * `hash` - Hexadecimal hash string
///
/// # Returns
///
/// The detected `HashAlgorithm`:
/// - 64 characters → SHA-256
/// - 96 characters → SHA-384
/// - 128 characters → SHA-512
/// - Other lengths → SHA-384 (default)
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::detect_hash_algorithm;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
///
/// let sha256_hash = "a".repeat(64);
/// let sha384_hash = "b".repeat(96);
/// let sha512_hash = "c".repeat(128);
///
/// assert!(matches!(detect_hash_algorithm(&sha256_hash), HashAlgorithm::Sha256));
/// assert!(matches!(detect_hash_algorithm(&sha384_hash), HashAlgorithm::Sha384));
/// assert!(matches!(detect_hash_algorithm(&sha512_hash), HashAlgorithm::Sha512));
/// ```
pub fn detect_hash_algorithm(hash: &str) -> HashAlgorithm {
    match hash.len() {
        64 => HashAlgorithm::Sha256,
        96 => HashAlgorithm::Sha384,
        128 => HashAlgorithm::Sha512,
        _ => HashAlgorithm::Sha384,
    }
}

/// Get the expected hash length for an algorithm
///
/// # Arguments
///
/// * `algorithm` - Algorithm name as a string (case-insensitive)
///
/// # Returns
///
/// The expected hexadecimal string length:
/// - "sha256" → 64
/// - "sha384" → 96
/// - "sha512" → 128
/// - Other → 96 (default)
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::get_hash_length;
///
/// assert_eq!(get_hash_length("sha256"), 64);
/// assert_eq!(get_hash_length("SHA384"), 96);
/// assert_eq!(get_hash_length("sha512"), 128);
/// assert_eq!(get_hash_length("unknown"), 96); // defaults to SHA-384
/// ```
pub fn get_hash_length(algorithm: &str) -> usize {
    match algorithm.to_lowercase().as_str() {
        "sha256" => 64,
        "sha384" => 96,
        "sha512" => 128,
        _ => 96,
    }
}

/// Get the algorithm name as used in manifests
///
/// Converts a `HashAlgorithm` to its string representation for storage in manifests.
///
/// # Arguments
///
/// * `algorithm` - The hash algorithm
///
/// # Returns
///
/// The algorithm name as a string:
/// - `HashAlgorithm::Sha256` → "sha256"
/// - `HashAlgorithm::Sha384` → "sha384"
/// - `HashAlgorithm::Sha512` → "sha512"
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::algorithm_to_string;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
///
/// assert_eq!(algorithm_to_string(&HashAlgorithm::Sha256), "sha256");
/// assert_eq!(algorithm_to_string(&HashAlgorithm::Sha384), "sha384");
/// assert_eq!(algorithm_to_string(&HashAlgorithm::Sha512), "sha512");
/// ```
pub fn algorithm_to_string(algorithm: &HashAlgorithm) -> &'static str {
    algorithm.as_str()
}

/// Parse algorithm from string
///
/// Converts a string algorithm name to a `HashAlgorithm` enum value.
///
/// # Arguments
///
/// * `s` - Algorithm name (case-sensitive: "sha256", "sha384", or "sha512")
///
/// # Returns
///
/// * `Ok(HashAlgorithm)` - The parsed algorithm
/// * `Err(Error)` - If the algorithm name is not recognized
///
/// # Examples
///
/// ```
/// use atlas_cli::hash::parse_algorithm;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
///
/// let algo = parse_algorithm("sha384").unwrap();
/// assert!(matches!(algo, HashAlgorithm::Sha384));
///
/// // Invalid algorithm names return an error
/// assert!(parse_algorithm("sha1").is_err());
/// assert!(parse_algorithm("SHA256").is_err()); // case sensitive
/// ```
pub fn parse_algorithm(s: &str) -> Result<HashAlgorithm> {
    use std::str::FromStr;
    HashAlgorithm::from_str(s).map_err(Error::Validation)
}

/// Internal helper to hash data from a reader using streaming
fn hash_reader<D: Digest, R: Read>(mut reader: R) -> Result<String> {
    let mut hasher = D::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
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
        assert_eq!(hash.len(), 96);
    }
    #[test]
    fn test_calculate_hash_with_algorithms() -> Result<()> {
        let data = b"test data";

        // Test different algorithms produce different length hashes
        let sha256 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha256);
        let sha384 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha384);
        let sha512 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha512);

        assert_eq!(sha256.len(), 64);
        assert_eq!(sha384.len(), 96);
        assert_eq!(sha512.len(), 128);

        // Different algorithms produce different hashes
        assert_ne!(sha256, sha384);
        assert_ne!(sha384, sha512);
        assert_ne!(sha256, sha512);

        Ok(())
    }

    #[test]
    fn test_calculate_file_hash() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");

        // Create a test file
        let mut file = safe_create_file(&file_path, false)?;
        file.write_all(b"test data")?;

        let hash = calculate_file_hash(&file_path)?;
        assert_eq!(hash.len(), 96); // Changed from 64 to 96

        // Verify hash changes with content
        let mut file = safe_create_file(&file_path, false)?;
        file.write_all(b"different data")?;

        let new_hash = calculate_file_hash(&file_path)?;
        assert_ne!(hash, new_hash);

        Ok(())
    }

    #[test]
    fn test_calculate_file_hash_with_algorithms() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test_algos.txt");

        // Create a test file
        let mut file = safe_create_file(&file_path, false)?;
        file.write_all(b"test data for algorithms")?;

        // Test different algorithms
        let sha256 = calculate_file_hash_with_algorithm(&file_path, &HashAlgorithm::Sha256)?;
        let sha384 = calculate_file_hash_with_algorithm(&file_path, &HashAlgorithm::Sha384)?;
        let sha512 = calculate_file_hash_with_algorithm(&file_path, &HashAlgorithm::Sha512)?;

        assert_eq!(sha256.len(), 64);
        assert_eq!(sha384.len(), 96);
        assert_eq!(sha512.len(), 128);

        // Different algorithms produce different hashes
        assert_ne!(sha256, sha384);
        assert_ne!(sha384, sha512);

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
    fn test_verify_hash_auto_detect() {
        let data = b"test data";

        // Create hashes with different algorithms
        let sha256 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha256);
        let sha384 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha384);
        let sha512 = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha512);

        // Verify should auto-detect the algorithm
        assert!(verify_hash(data, &sha256));
        assert!(verify_hash(data, &sha384));
        assert!(verify_hash(data, &sha512));
    }

    #[test]
    fn test_detect_hash_algorithm() {
        let sha256_hash = "a".repeat(64);
        let sha384_hash = "b".repeat(96);
        let sha512_hash = "c".repeat(128);

        assert!(matches!(
            detect_hash_algorithm(&sha256_hash),
            HashAlgorithm::Sha256
        ));
        assert!(matches!(
            detect_hash_algorithm(&sha384_hash),
            HashAlgorithm::Sha384
        ));
        assert!(matches!(
            detect_hash_algorithm(&sha512_hash),
            HashAlgorithm::Sha512
        ));

        // Unknown length defaults to SHA-384
        assert!(matches!(
            detect_hash_algorithm("short"),
            HashAlgorithm::Sha384
        ));
    }

    #[test]
    fn test_combine_hashes() -> Result<()> {
        let hash1 = calculate_hash(b"data1");
        let hash2 = calculate_hash(b"data2");

        let combined = combine_hashes(&[&hash1, &hash2])?;
        assert_eq!(combined.len(), 96);

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
        assert_eq!(hash.len(), 96);
        // Known SHA-384 hash of empty string
        assert_eq!(
            hash,
            "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b"
        );
    }

    #[test]
    fn test_hash_known_values() {
        // Test vectors for SHA-384
        let test_vectors: [(&[u8], &str); 2] = [
            (
                b"abc",
                "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7",
            ),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "3391fdddfc8dc7393707a65b1b4709397cf8b1d162af05abfe8f450de5f36bc6b0455a8520bc4e6f5fe95b1fe3c8452b",
            ),
        ];

        for (input, expected) in &test_vectors {
            let hash = calculate_hash(input);
            assert_eq!(&hash, expected);
        }
    }

    #[test]
    fn test_algorithm_to_string() {
        assert_eq!(algorithm_to_string(&HashAlgorithm::Sha256), "sha256");
        assert_eq!(algorithm_to_string(&HashAlgorithm::Sha384), "sha384");
        assert_eq!(algorithm_to_string(&HashAlgorithm::Sha512), "sha512");
    }

    #[test]
    fn test_parse_algorithm() {
        // Valid algorithms
        assert!(matches!(
            parse_algorithm("sha256").unwrap(),
            HashAlgorithm::Sha256
        ));
        assert!(matches!(
            parse_algorithm("sha384").unwrap(),
            HashAlgorithm::Sha384
        ));
        assert!(matches!(
            parse_algorithm("sha512").unwrap(),
            HashAlgorithm::Sha512
        ));

        // Invalid algorithms
        assert!(parse_algorithm("sha1").is_err());
        assert!(parse_algorithm("SHA256").is_err()); // case sensitive
        assert!(parse_algorithm("").is_err());
    }

    #[test]
    fn test_get_hash_length() {
        assert_eq!(get_hash_length("sha256"), 64);
        assert_eq!(get_hash_length("SHA256"), 64); // case insensitive
        assert_eq!(get_hash_length("sha384"), 96);
        assert_eq!(get_hash_length("sha512"), 128);
        assert_eq!(get_hash_length("unknown"), 96); // defaults to SHA-384
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
        assert_eq!(result.len(), 96); // Changed from 64 to 96

        // Test combining empty list of hashes
        match combine_hashes(&[]) {
            Ok(hash) => {
                // If it succeeds, verify it's a valid hash
                assert_eq!(hash.len(), 96); // Changed from 64 to 96
                // The hash of empty input should be the SHA-384 of empty data
                assert_eq!(
                    hash,
                    "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b"
                );
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

    #[test]
    fn test_large_file_hashing() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("large_file.bin");

        // Create a 10MB file
        {
            let mut file = safe_create_file(&file_path, false)?;
            let chunk = vec![0x42u8; 1024 * 1024]; // 1MB chunk
            for _ in 0..10 {
                file.write_all(&chunk)?;
            }
        }

        // Test that we can hash large files with different algorithms
        let sha256 = calculate_file_hash_with_algorithm(&file_path, &HashAlgorithm::Sha256)?;
        let sha384 = calculate_file_hash_with_algorithm(&file_path, &HashAlgorithm::Sha384)?;
        let sha512 = calculate_file_hash_with_algorithm(&file_path, &HashAlgorithm::Sha512)?;

        assert_eq!(sha256.len(), 64);
        assert_eq!(sha384.len(), 96);
        assert_eq!(sha512.len(), 128);

        // All should be different
        assert_ne!(sha256, sha384);
        assert_ne!(sha384, sha512);
        assert_ne!(sha256, sha512);

        Ok(())
    }

    #[test]
    fn test_cross_algorithm_verification() {
        // Test that verification fails when using wrong algorithm
        let data = b"cross algorithm test data";

        // Create hashes with each algorithm
        let sha256_hash = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha256);
        let sha384_hash = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha384);
        let sha512_hash = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha512);

        // Verify with correct algorithms should succeed
        assert!(verify_hash_with_algorithm(
            data,
            &sha256_hash,
            &HashAlgorithm::Sha256
        ));
        assert!(verify_hash_with_algorithm(
            data,
            &sha384_hash,
            &HashAlgorithm::Sha384
        ));
        assert!(verify_hash_with_algorithm(
            data,
            &sha512_hash,
            &HashAlgorithm::Sha512
        ));

        // Verify with wrong algorithms should fail
        assert!(!verify_hash_with_algorithm(
            data,
            &sha256_hash,
            &HashAlgorithm::Sha384
        ));
        assert!(!verify_hash_with_algorithm(
            data,
            &sha256_hash,
            &HashAlgorithm::Sha512
        ));
        assert!(!verify_hash_with_algorithm(
            data,
            &sha384_hash,
            &HashAlgorithm::Sha256
        ));
        assert!(!verify_hash_with_algorithm(
            data,
            &sha384_hash,
            &HashAlgorithm::Sha512
        ));
        assert!(!verify_hash_with_algorithm(
            data,
            &sha512_hash,
            &HashAlgorithm::Sha256
        ));
        assert!(!verify_hash_with_algorithm(
            data,
            &sha512_hash,
            &HashAlgorithm::Sha384
        ));
    }

    #[test]
    fn test_binary_data_hashing() {
        // Test with various binary patterns
        let test_cases = vec![
            vec![0x00; 100],                // All zeros
            vec![0xFF; 100],                // All ones
            vec![0xAA; 100],                // Alternating bits (10101010)
            vec![0x55; 100],                // Alternating bits (01010101)
            (0..=255).collect::<Vec<u8>>(), // All byte values
        ];

        for (i, data) in test_cases.iter().enumerate() {
            let hash = calculate_hash(data);
            assert_eq!(hash.len(), 96, "Test case {} failed", i);

            // Verify each produces unique hash
            for (j, other_data) in test_cases.iter().enumerate() {
                if i != j {
                    let other_hash = calculate_hash(other_data);
                    assert_ne!(
                        hash, other_hash,
                        "Test cases {} and {} produced same hash",
                        i, j
                    );
                }
            }
        }
    }

    #[test]
    fn test_utf8_string_hashing() {
        // Test with various UTF-8 strings
        let test_strings = vec![
            "Hello, World!",
            "Hello, World!",      // Same string should produce same hash
            "Hello, World! ",     // Extra space should produce different hash
            "Здравствуй, мир!",   // Russian
            "你好，世界！",       // Chinese
            "こんにちは、世界！", // Japanese
            "🌍🌎🌏",             // Emojis
            "𝓗𝓮𝓵𝓵𝓸",              // Mathematical alphanumeric symbols
            "",                   // Empty string
            " ",                  // Single space
            "\n\r\t",             // Whitespace characters
        ];

        let mut hashes = Vec::new();
        for s in &test_strings {
            let hash = calculate_hash(s.as_bytes());
            hashes.push(hash);
        }

        // First two should be equal (same string)
        assert_eq!(hashes[0], hashes[1]);

        // All others should be unique
        for i in 0..hashes.len() {
            for j in 0..hashes.len() {
                if i != j && !(i == 0 && j == 1) && !(i == 1 && j == 0) {
                    assert_ne!(
                        hashes[i], hashes[j],
                        "Strings '{}' and '{}' produced same hash",
                        test_strings[i], test_strings[j]
                    );
                }
            }
        }
    }

    #[test]
    fn test_incremental_data_hashing() -> Result<()> {
        // Test that hashing data incrementally produces consistent results
        let dir = tempdir()?;
        let file_path = dir.path().join("incremental.txt");

        // Create a file with incremental content
        let mut content = String::new();
        let mut hashes = Vec::new();

        for i in 0..10 {
            content.push_str(&format!("Line {}\n", i));

            let mut file = safe_create_file(&file_path, false)?;
            file.write_all(content.as_bytes())?;
            drop(file); // Ensure file is closed

            let hash = calculate_file_hash(&file_path)?;
            hashes.push(hash);
        }

        // Each hash should be different
        for i in 0..hashes.len() {
            for j in i + 1..hashes.len() {
                assert_ne!(
                    hashes[i], hashes[j],
                    "Incremental content at positions {} and {} produced same hash",
                    i, j
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_hash_consistency_across_algorithms() {
        // Test that the same data always produces the same hash for each algorithm
        let data = b"consistency test data";
        let iterations = 100;

        let mut sha256_hashes = Vec::new();
        let mut sha384_hashes = Vec::new();
        let mut sha512_hashes = Vec::new();

        for _ in 0..iterations {
            sha256_hashes.push(calculate_hash_with_algorithm(data, &HashAlgorithm::Sha256));
            sha384_hashes.push(calculate_hash_with_algorithm(data, &HashAlgorithm::Sha384));
            sha512_hashes.push(calculate_hash_with_algorithm(data, &HashAlgorithm::Sha512));
        }

        // All hashes for the same algorithm should be identical
        for i in 1..iterations {
            assert_eq!(
                sha256_hashes[0], sha256_hashes[i],
                "SHA-256 inconsistent at iteration {}",
                i
            );
            assert_eq!(
                sha384_hashes[0], sha384_hashes[i],
                "SHA-384 inconsistent at iteration {}",
                i
            );
            assert_eq!(
                sha512_hashes[0], sha512_hashes[i],
                "SHA-512 inconsistent at iteration {}",
                i
            );
        }
    }

    #[test]
    fn test_combine_hashes_edge_cases() -> Result<()> {
        // Test combining different numbers of hashes
        let hash1 = calculate_hash(b"data1");
        let hash2 = calculate_hash(b"data2");
        let hash3 = calculate_hash(b"data3");

        // Single hash
        let single = combine_hashes(&[&hash1])?;
        assert_eq!(single.len(), 96);

        // Two hashes
        let double = combine_hashes(&[&hash1, &hash2])?;
        assert_eq!(double.len(), 96);
        assert_ne!(single, double);

        // Three hashes
        let triple = combine_hashes(&[&hash1, &hash2, &hash3])?;
        assert_eq!(triple.len(), 96);
        assert_ne!(double, triple);

        // Test associativity - (A + B) + C should equal A + (B + C)
        let ab = combine_hashes(&[&hash1, &hash2])?;
        let ab_c = combine_hashes(&[&ab, &hash3])?;

        let bc = combine_hashes(&[&hash2, &hash3])?;
        let a_bc = combine_hashes(&[&hash1, &bc])?;

        // This is expected behavior.
        assert_ne!(ab_c, a_bc);

        Ok(())
    }

    #[test]
    fn test_file_not_found_error() {
        // Test proper error handling for non-existent files
        let result = calculate_file_hash("/this/path/should/not/exist/test.txt");
        assert!(result.is_err());

        match result {
            Err(Error::Io(_)) => (), // Expected
            Err(e) => panic!("Expected Io error, got: {:?}", e),
            Ok(_) => panic!("Expected error for non-existent file"),
        }
    }

    #[test]
    fn test_special_filenames() -> Result<()> {
        let dir = tempdir()?;

        // Test with various special filenames
        let filenames = vec![
            "file with spaces.txt",
            "file-with-dashes.txt",
            "file_with_underscores.txt",
            "file.multiple.dots.txt",
            "UPPERCASE.TXT",
            "🦀rust🦀.txt", // Emoji in filename
            ".hidden_file",
            "very_long_filename_that_exceeds_typical_lengths_but_should_still_work_fine.txt",
        ];

        for filename in filenames {
            let file_path = dir.path().join(filename);
            let mut file = safe_create_file(&file_path, false)?;
            file.write_all(b"test content")?;
            drop(file);

            // Should be able to hash regardless of filename
            let hash = calculate_file_hash(&file_path)?;
            assert_eq!(hash.len(), 96, "Failed for filename: {}", filename);
        }

        Ok(())
    }

    #[test]
    fn test_concurrent_hashing_safety() {
        use std::sync::Arc;
        use std::thread;

        // Test that hashing is thread-safe
        let data = Arc::new(b"concurrent test data".to_vec());
        let num_threads = 10;
        let iterations_per_thread = 100;

        let mut handles = vec![];

        for _ in 0..num_threads {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                let mut hashes = Vec::new();
                for _ in 0..iterations_per_thread {
                    let hash = calculate_hash(&data_clone);
                    hashes.push(hash);
                }
                hashes
            });
            handles.push(handle);
        }

        // Collect all results
        let mut all_hashes = Vec::new();
        for handle in handles {
            let hashes = handle.join().expect("Thread panicked");
            all_hashes.extend(hashes);
        }

        // All hashes should be identical
        let expected_hash = calculate_hash(&data);
        for (i, hash) in all_hashes.iter().enumerate() {
            assert_eq!(hash, &expected_hash, "Hash mismatch at index {}", i);
        }
    }

    #[test]
    fn test_combine_hashes_with_invalid_hex() -> Result<()> {
        let valid_hash = calculate_hash(b"valid");

        // Test with invalid hex string
        let result = combine_hashes(&[&valid_hash, "not_valid_hex"]);
        assert!(result.is_err());

        // Test with odd-length hex string
        let result = combine_hashes(&[&valid_hash, "abc"]);
        assert!(result.is_err());

        // Test with non-ASCII characters
        let result = combine_hashes(&[&valid_hash, "café"]);
        assert!(result.is_err());

        Ok(())
    }
}
