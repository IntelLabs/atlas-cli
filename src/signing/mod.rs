use crate::error::{Error, Result};
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::sign::Signer;
use std::fs::read;
use std::path::Path;

pub mod utils;

pub fn load_private_key(key_path: &Path) -> Result<PKey<Private>> {
    let key_data = read(key_path)?;
    PKey::private_key_from_pem(&key_data)
        .map_err(|e| crate::error::Error::Signing(format!("Failed to load private key: {}", e)))
}

pub fn sign_data(data: &[u8], private_key: &PKey<Private>) -> Result<Vec<u8>> {
    let mut signer = Signer::new(MessageDigest::sha256(), private_key)
        .map_err(|e| crate::error::Error::Signing(format!("Failed to create signer: {}", e)))?;

    signer
        .update(data)
        .map_err(|e| crate::error::Error::Signing(format!("Failed to update signer: {}", e)))?;

    signer
        .sign_to_vec()
        .map_err(|e| crate::error::Error::Signing(format!("Failed to sign data: {}", e)))
}

pub fn verify_signature(data: &[u8], signature: &[u8], public_key: &PKey<Public>) -> Result<bool> {
    let mut verifier =
        openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), public_key)
            .map_err(|e| Error::Signing(e.to_string()))?;

    verifier
        .update(data)
        .map_err(|e| Error::Signing(e.to_string()))?;

    verifier
        .verify(signature)
        .map_err(|e| Error::Signing(e.to_string()))
}
