use crate::error::{Error, Result};
use crate::signing::SecurePrivateKey;
use crate::signing::Zeroizing;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn load_private_key(path: impl AsRef<Path>) -> Result<SecurePrivateKey> {
    let mut key_file = File::open(path)?;
    let mut key_content = Vec::new();
    key_file.read_to_end(&mut key_content)?;

    // Wrap the key content in Zeroizing to ensure it's cleared
    let zeroizing_content = Zeroizing::new(key_content);

    // Create SecurePrivateKey from the zeroizing content
    SecurePrivateKey::from_pem(zeroizing_content.to_vec())
}

pub fn sign_manifest(manifest_json: &[u8], private_key: &SecurePrivateKey) -> Result<Vec<u8>> {
    let mut signer = openssl::sign::Signer::new(
        openssl::hash::MessageDigest::sha256(),
        private_key.as_pkey(),
    )
    .map_err(|e| Error::Signing(e.to_string()))?;

    signer
        .update(manifest_json)
        .map_err(|e| Error::Signing(e.to_string()))?;

    // Use a zeroizing buffer for the signature
    let sig_len = signer
        .len()
        .map_err(|e| Error::Signing(format!("Failed to get signature length: {}", e)))?;
    let mut signature = Zeroizing::new(vec![0u8; sig_len]);
    let len = signer
        .sign(&mut signature)
        .map_err(|e| Error::Signing(format!("Failed to sign: {}", e)))?;

    // Return only the used portion of the signature
    Ok(signature[..len].to_vec())
}
