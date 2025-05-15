use crate::error::Error;
use crate::utils::safe_open_file;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut file = safe_open_file(path.as_ref(), false)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).map_err(Error::Io)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}
