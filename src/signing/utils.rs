use crate::error::{Error, Result};
use openssl::pkey::{PKey, Private};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn load_private_key(path: impl AsRef<Path>) -> Result<PKey<Private>> {
    let mut key_file = File::open(path)?;
    let mut key_content = Vec::new();
    key_file.read_to_end(&mut key_content)?;

    PKey::private_key_from_pem(&key_content).map_err(|e| Error::Signing(e.to_string()))
}

pub fn sign_manifest(manifest_json: &[u8], private_key: &PKey<Private>) -> Result<Vec<u8>> {
    let mut signer =
        openssl::sign::Signer::new(openssl::hash::MessageDigest::sha256(), private_key)
            .map_err(|e| Error::Signing(e.to_string()))?;

    signer
        .update(manifest_json)
        .map_err(|e| Error::Signing(e.to_string()))?;

    signer
        .sign_to_vec()
        .map_err(|e| Error::Signing(e.to_string()))
}
