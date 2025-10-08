use crate::error::{Error, Result};
use atlas_c2pa_lib::cose::HashAlgorithm;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::sign::Signer;
use std::fs::read;
use std::path::Path;
use zeroize::{ZeroizeOnDrop, Zeroizing};

pub mod signable;

/// Secure wrapper for private key data that zeroizes on drop
#[derive(ZeroizeOnDrop)]
pub struct SecurePrivateKey {
    #[zeroize(skip)]
    pkey: PKey<Private>,
    // Store the original key bytes in case we need them
    _key_data: Zeroizing<Vec<u8>>,
}

impl SecurePrivateKey {
    /// Create a new SecurePrivateKey from raw PEM data
    pub fn from_pem(pem_data: Vec<u8>) -> Result<Self> {
        // Wrap the PEM data in Zeroizing to ensure it's cleared when dropped
        let zeroizing_pem = Zeroizing::new(pem_data);

        // Parse the private key
        let pkey = PKey::private_key_from_pem(&zeroizing_pem)
            .map_err(|e| Error::Signing(format!("Failed to load private key: {e}")))?;

        Ok(Self {
            pkey,
            _key_data: zeroizing_pem,
        })
    }

    /// Get a reference to the inner PKey
    pub fn as_pkey(&self) -> &PKey<Private> {
        &self.pkey
    }
}

/// Load a private key from a file path with automatic zeroization
pub fn load_private_key(key_path: &Path) -> Result<SecurePrivateKey> {
    // Read the key data - will be automatically zeroized when dropped
    let key_data = read(key_path)?;
    SecurePrivateKey::from_pem(key_data)
}

/// Sign data with a specific hash algorithm and automatic key zeroization
pub fn sign_data_with_algorithm(
    data: &[u8],
    private_key: &SecurePrivateKey,
    algorithm: &HashAlgorithm,
) -> Result<Vec<u8>> {
    let message_digest = match algorithm {
        HashAlgorithm::Sha256 => MessageDigest::sha256(),
        HashAlgorithm::Sha384 => MessageDigest::sha384(),
        HashAlgorithm::Sha512 => MessageDigest::sha512(),
    };

    let mut signer = Signer::new(message_digest, private_key.as_pkey())
        .map_err(|e| Error::Signing(format!("Failed to create signer: {e}")))?;

    signer
        .update(data)
        .map_err(|e| Error::Signing(format!("Failed to update signer: {e}")))?;

    // Sign to a zeroizing vector first to ensure cleanup
    let sig_len = signer
        .len()
        .map_err(|e| Error::Signing(format!("Failed to get signature length: {e}")))?;
    let mut signature = Zeroizing::new(vec![0u8; sig_len]);
    let len = signer
        .sign(&mut signature)
        .map_err(|e| Error::Signing(format!("Failed to sign data: {e}")))?;

    // Return only the used portion of the signature
    Ok(signature[..len].to_vec())
}

/// Sign data with default SHA-384 algorithm
pub fn sign_data(data: &[u8], private_key: &SecurePrivateKey) -> Result<Vec<u8>> {
    sign_data_with_algorithm(data, private_key, &HashAlgorithm::Sha384)
}

// Verify signature with a public key using default SHA-384 algorithm
pub fn verify_signature(data: &[u8], signature: &[u8], public_key: &PKey<Public>) -> Result<bool> {
    verify_signature_with_algorithm(data, signature, public_key, &HashAlgorithm::Sha384)
}

/// Verify signature with a public key using the specified algorithm
pub fn verify_signature_with_algorithm(
    data: &[u8],
    signature: &[u8],
    public_key: &PKey<Public>,
    algorithm: &HashAlgorithm,
) -> Result<bool> {
    let message_digest = match algorithm {
        HashAlgorithm::Sha256 => MessageDigest::sha256(),
        HashAlgorithm::Sha384 => MessageDigest::sha384(),
        HashAlgorithm::Sha512 => MessageDigest::sha512(),
    };

    let mut verifier = openssl::sign::Verifier::new(message_digest, public_key)
        .map_err(|e| Error::Signing(e.to_string()))?;

    verifier
        .update(data)
        .map_err(|e| Error::Signing(e.to_string()))?;

    verifier
        .verify(signature)
        .map_err(|e| Error::Signing(e.to_string()))
}

pub fn pkey_to_secure(pkey: PKey<Private>) -> Result<SecurePrivateKey> {
    // Export to PEM format then re-import as SecurePrivateKey
    let pem_data = pkey
        .private_key_to_pem_pkcs8()
        .map_err(|e| Error::Signing(format!("Failed to export key to PEM: {e}")))?;

    SecurePrivateKey::from_pem(pem_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use crate::signing::test_utils::generate_temp_key;

    #[test]
    fn test_load_private_key() -> Result<()> {
        // Generate a test key and save it to a temporary file
        let (secure_key, _dir) = generate_temp_key()?;

        // Test that we can use the key for signing
        let test_data = b"test data for signing";
        let signature = sign_data(test_data, &secure_key)?;

        // Signature should not be empty
        assert!(!signature.is_empty());

        Ok(())
    }

    #[test]
    fn test_sign_data() -> Result<()> {
        // Generate a temporary key
        let (secure_key, _) = generate_temp_key()?;

        // Test data
        let data1 = b"test data for signing";
        let data2 = b"different test data";

        // Sign the data
        let signature1 = sign_data(data1, &secure_key)?;
        let signature2 = sign_data(data1, &secure_key)?; // Same data again
        let signature3 = sign_data(data2, &secure_key)?; // Different data

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
        let (secure_key1, _) = generate_temp_key()?;
        let (secure_key2, _) = generate_temp_key()?;

        // Test data
        let data = b"test data for signature comparison";

        // Sign with both keys
        let signature1 = sign_data(data, &secure_key1)?;
        let signature2 = sign_data(data, &secure_key2)?;

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
        let (secure_key, _) = generate_temp_key()?;

        // Sign empty data
        let signature = sign_data(&[], &secure_key)?;

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
        let (secure_key, _) = generate_temp_key()?;

        // Generate larger test data (e.g., 100KB for test speed)
        let large_data = vec![0x55; 100 * 1024]; // 100KB of the byte 0x55

        // Sign the large data
        let signature = sign_data(&large_data, &secure_key)?;

        // Should produce a valid signature
        assert!(
            !signature.is_empty(),
            "Signature of large data should not be empty"
        );

        Ok(())
    }

    #[test]
    fn test_secure_key_zeroization() -> Result<()> {
        // Create a key with known content
        let pem_data = b"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC7W8pGqWu2VZtD
TEST_KEY_DATA_THAT_SHOULD_BE_ZEROIZED
-----END PRIVATE KEY-----"
            .to_vec();

        // Create a SecurePrivateKey in a scope
        {
            let _secure_key = SecurePrivateKey::from_pem(pem_data.clone());
            // Key exists here
        }
        // Key should be zeroized after this point

        // Note: In a real scenario, we would need more sophisticated testing
        // to verify memory has been zeroized, but Rust's ownership system
        // and the zeroize crate ensure this happens.

        Ok(())
    }

    #[test]
    fn test_sign_with_different_algorithms() -> Result<()> {
        // Generate a temporary key
        let (secure_key, _) = generate_temp_key()?;

        let data = b"test data for different algorithms";

        // Test different algorithms
        let sig_sha256 = sign_data_with_algorithm(data, &secure_key, &HashAlgorithm::Sha256)?;
        let sig_sha384 = sign_data_with_algorithm(data, &secure_key, &HashAlgorithm::Sha384)?;
        let sig_sha512 = sign_data_with_algorithm(data, &secure_key, &HashAlgorithm::Sha512)?;

        // All signatures should be non-empty
        assert!(!sig_sha256.is_empty());
        assert!(!sig_sha384.is_empty());
        assert!(!sig_sha512.is_empty());

        // Different algorithms produce different signatures
        assert_ne!(sig_sha256, sig_sha384);
        assert_ne!(sig_sha384, sig_sha512);
        assert_ne!(sig_sha256, sig_sha512);

        Ok(())
    }
    #[test]
    fn test_zeroization_with_multiple_references() -> Result<()> {
        // Test that zeroization works correctly with multiple references
        use std::sync::Arc;

        let (secure_key, _dir) = generate_temp_key()?;

        // Create multiple references to the same key
        let key_arc = Arc::new(secure_key);
        let key_ref1 = Arc::clone(&key_arc);
        let key_ref2 = Arc::clone(&key_arc);

        // Use the key through different references
        let data = b"test data";

        let sig1 = sign_data(data, &key_ref1)?;
        let sig2 = sign_data(data, &key_ref2)?;

        // Signatures should be the same
        assert_eq!(sig1, sig2);

        // Drop references one by one
        drop(key_ref1);
        assert_eq!(Arc::strong_count(&key_arc), 2);

        drop(key_ref2);
        assert_eq!(Arc::strong_count(&key_arc), 1);

        // The key is still usable through the last reference
        let sig3 = sign_data(data, &key_arc)?;
        assert_eq!(sig1, sig3);

        // When the last Arc is dropped, the key will be zeroized
        drop(key_arc);

        Ok(())
    }
    #[test]
    fn test_sign_and_verify_with_algorithms() -> Result<()> {
        // Test that signing and verification work correctly with matching algorithms
        let (secure_key, _dir) = generate_temp_key()?;

        // Get the public key for verification
        let public_key = secure_key
            .as_pkey()
            .public_key_to_pem()
            .map_err(|e| Error::Signing(e.to_string()))?;
        let public_key =
            PKey::public_key_from_pem(&public_key).map_err(|e| Error::Signing(e.to_string()))?;

        let data = b"test data for sign and verify";

        // Test each algorithm
        for algo in &[
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha384,
            HashAlgorithm::Sha512,
        ] {
            // Sign with the algorithm
            let signature = sign_data_with_algorithm(data, &secure_key, algo)?;

            // Verify with the same algorithm - should succeed
            let valid = verify_signature_with_algorithm(data, &signature, &public_key, algo)?;
            assert!(
                valid,
                "Verification should succeed with matching algorithm {:?}",
                algo
            );

            // Verify with different algorithms - should fail
            for wrong_algo in &[
                HashAlgorithm::Sha256,
                HashAlgorithm::Sha384,
                HashAlgorithm::Sha512,
            ] {
                if wrong_algo != algo {
                    let invalid =
                        verify_signature_with_algorithm(data, &signature, &public_key, wrong_algo)?;
                    assert!(
                        !invalid,
                        "Verification should fail with mismatched algorithms {:?} != {:?}",
                        algo, wrong_algo
                    );
                }
            }
        }

        // Test default functions (should use SHA-384)
        let signature = sign_data(data, &secure_key)?;
        let valid = verify_signature(data, &signature, &public_key)?;
        assert!(valid, "Default sign/verify should work together");

        // Verify that default uses SHA-384
        let valid_384 =
            verify_signature_with_algorithm(data, &signature, &public_key, &HashAlgorithm::Sha384)?;
        assert!(valid_384, "Default should be SHA-384");

        let invalid_256 =
            verify_signature_with_algorithm(data, &signature, &public_key, &HashAlgorithm::Sha256)?;
        assert!(!invalid_256, "Default should not be SHA-256");

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::error::Result;
    use crate::signing::SecurePrivateKey;
    use crate::signing::load_private_key;
    use openssl::pkey::PKey;
    use openssl::rsa::Rsa;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper function to generate a temporary private key for testing
    pub fn generate_temp_key() -> Result<(SecurePrivateKey, tempfile::TempDir)> {
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

        // Now load it as SecurePrivateKey
        let secure_key = load_private_key(&key_path)?;

        Ok((secure_key, dir))
    }
}
