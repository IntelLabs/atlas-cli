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
