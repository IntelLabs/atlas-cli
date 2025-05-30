use crate::error::{Error, Result};
use atlas_c2pa_lib::cose::HashAlgorithm;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::sign::Signer;
use std::fs::read;
use std::path::Path;

pub mod utils;

pub fn load_private_key(key_path: &Path) -> Result<PKey<Private>> {
    let key_data = read(key_path)?;
    PKey::private_key_from_pem(&key_data)
        .map_err(|e| crate::error::Error::Signing(format!("Failed to load private key: {e}")))
}

pub fn sign_data_with_algorithm(
    data: &[u8],
    private_key: &PKey<Private>,
    algorithm: &HashAlgorithm,
) -> Result<Vec<u8>> {
    let message_digest = match algorithm {
        HashAlgorithm::Sha256 => MessageDigest::sha256(),
        HashAlgorithm::Sha384 => MessageDigest::sha384(),
        HashAlgorithm::Sha512 => MessageDigest::sha512(),
    };

    let mut signer = Signer::new(message_digest, private_key)
        .map_err(|e| crate::error::Error::Signing(format!("Failed to create signer: {e}")))?;

    signer
        .update(data)
        .map_err(|e| crate::error::Error::Signing(format!("Failed to update signer: {e}")))?;

    signer
        .sign_to_vec()
        .map_err(|e| crate::error::Error::Signing(format!("Failed to sign data: {e}")))
}

pub fn sign_data(data: &[u8], private_key: &PKey<Private>) -> Result<Vec<u8>> {
    sign_data_with_algorithm(data, private_key, &HashAlgorithm::Sha384)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use openssl::pkey::{PKey, Private};
    use openssl::rsa::Rsa;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper function to generate a temporary private key for testing
    fn generate_temp_key() -> Result<(PKey<Private>, tempfile::TempDir)> {
        // Create a temporary directory
        let dir = tempdir()?;
        let key_path = dir.path().join("test_key.pem");

        // Generate a new RSA key pair (using 2048 bits for speed in tests)
        let rsa = Rsa::generate(2048).map_err(|e| crate::error::Error::Signing(e.to_string()))?;

        // Convert to PKey
        let private_key =
            PKey::from_rsa(rsa).map_err(|e| crate::error::Error::Signing(e.to_string()))?;

        // Write private key to file
        let pem = private_key
            .private_key_to_pem_pkcs8()
            .map_err(|e| crate::error::Error::Signing(e.to_string()))?;

        let mut key_file = File::create(&key_path)?;
        key_file.write_all(&pem)?;

        Ok((private_key, dir))
    }

    #[test]
    fn test_load_private_key() -> Result<()> {
        // Generate a test key and save it to a temporary file
        let (original_key, dir) = generate_temp_key()?;
        let key_path = dir.path().join("test_key.pem");

        // Load the key using the module function
        let loaded_key = load_private_key(&key_path)?;

        // Verify the loaded key by signing the same data with both keys
        let test_data = b"test data for signing";

        let signature1 = sign_data(test_data, &original_key)?;
        let signature2 = sign_data(test_data, &loaded_key)?;

        // The signatures should match if the keys are the same
        assert_eq!(
            signature1, signature2,
            "Signatures from original and loaded keys should match"
        );

        Ok(())
    }

    #[test]
    fn test_sign_data() -> Result<()> {
        // Generate a temporary key
        let (private_key, _) = generate_temp_key()?;

        // Test data
        let data1 = b"test data for signing";
        let data2 = b"different test data";

        // Sign the data
        let signature1 = sign_data(data1, &private_key)?;
        let signature2 = sign_data(data1, &private_key)?; // Same data again
        let signature3 = sign_data(data2, &private_key)?; // Different data

        // Verify signatures have expected properties
        assert!(!signature1.is_empty(), "Signature should not be empty");

        // Same data should produce the same signature with the same key
        assert_eq!(
            signature1, signature2,
            "Signatures for the same data should match"
        );

        // Different data should produce different signatures
        assert_ne!(
            signature1, signature3,
            "Signatures for different data should not match"
        );

        Ok(())
    }

    #[test]
    fn test_signature_different_keys() -> Result<()> {
        // Generate two different keys
        let (private_key1, _) = generate_temp_key()?;
        let (private_key2, _) = generate_temp_key()?;

        // Test data
        let data = b"test data for signature comparison";

        // Sign with both keys
        let signature1 = sign_data(data, &private_key1)?;
        let signature2 = sign_data(data, &private_key2)?;

        // Different keys should produce different signatures for the same data
        assert_ne!(
            signature1, signature2,
            "Signatures from different keys should not match"
        );

        Ok(())
    }

    #[test]
    fn test_load_private_key_error() {
        // Attempt to load a non-existent key file
        let result = load_private_key(std::path::Path::new("/nonexistent/path/to/key.pem"));

        // Should return an error
        assert!(result.is_err(), "Loading non-existent key should fail");

        // The error should be an IO error
        if let Err(e) = result {
            match e {
                crate::error::Error::Io(_) => {} // Expected error type
                _ => panic!("Unexpected error type: {e:?}"),
            }
        }
    }

    #[test]
    fn test_sign_data_with_empty_data() -> Result<()> {
        // Generate a temporary key
        let (private_key, _) = generate_temp_key()?;

        // Sign empty data
        let signature = sign_data(&[], &private_key)?;

        // Even empty data should produce a valid signature
        assert!(
            !signature.is_empty(),
            "Signature of empty data should not be empty"
        );

        Ok(())
    }

    #[test]
    fn test_sign_large_data() -> Result<()> {
        // Generate a temporary key
        let (private_key, _) = generate_temp_key()?;

        // Generate larger test data (e.g., 100KB for test speed)
        let large_data = vec![0x55; 100 * 1024]; // 100KB of the byte 0x55

        // Sign the large data
        let signature = sign_data(&large_data, &private_key)?;

        // Should produce a valid signature
        assert!(
            !signature.is_empty(),
            "Signature of large data should not be empty"
        );

        Ok(())
    }

    #[test]
    fn test_load_key_with_restrictive_permissions() -> Result<()> {
        // Generate a test key and save it to a temporary file
        let (_, dir) = generate_temp_key()?;
        let key_path = dir.path().join("perm_test_key.pem");

        // Generate a new RSA key pair
        let rsa = Rsa::generate(2048).map_err(|e| crate::error::Error::Signing(e.to_string()))?;

        // Convert to PKey
        let private_key =
            PKey::from_rsa(rsa).map_err(|e| crate::error::Error::Signing(e.to_string()))?;

        // Write private key to file
        let pem = private_key
            .private_key_to_pem_pkcs8()
            .map_err(|e| crate::error::Error::Signing(e.to_string()))?;

        let mut key_file = File::create(&key_path)?;
        key_file.write_all(&pem)?;

        // Set more restrictive permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&key_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600); // Owner read/write only
            std::fs::set_permissions(&key_path, permissions)?;
        }

        // Try to load the key
        let result = load_private_key(&key_path);

        // The load should succeed regardless of permissions
        assert!(
            result.is_ok(),
            "Loading key with restricted permissions should succeed"
        );

        Ok(())
    }
}
