use crate::error::Result;
use crate::hash;
use crate::utils::safe_create_file;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_calculate_hash() -> Result<()> {
    let data = b"test data";
    let hash = hash::calculate_hash(data);
    assert_eq!(hash.len(), 96); // SHA-384 hash is 96 hex characters
    Ok(())
}

#[test]
fn test_calculate_file_hash() -> Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("test.txt");

    // Create a test file
    let mut file = safe_create_file(&file_path, false)?;
    file.write_all(b"test data")?;

    let hash = hash::calculate_file_hash(&file_path)?;
    assert_eq!(hash.len(), 96);

    // Verify hash changes with content
    let mut file = safe_create_file(&file_path, false)?;
    file.write_all(b"different data")?;

    let new_hash = hash::calculate_file_hash(&file_path)?;
    assert_ne!(hash, new_hash);

    Ok(())
}

#[test]
fn test_verify_hash() -> Result<()> {
    let data = b"test data";
    let hash = hash::calculate_hash(data);

    assert!(hash::verify_hash(data, &hash));
    assert!(!hash::verify_hash(b"different data", &hash));

    Ok(())
}

#[test]
fn test_combine_hashes() -> Result<()> {
    let hash1 = hash::calculate_hash(b"data1");
    let hash2 = hash::calculate_hash(b"data2");

    let combined = hash::combine_hashes(&[&hash1, &hash2])?;
    assert_eq!(combined.len(), 96);

    // Test order matters
    let combined2 = hash::combine_hashes(&[&hash2, &hash1])?;
    assert_ne!(combined, combined2);

    Ok(())
}
