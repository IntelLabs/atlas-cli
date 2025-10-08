//! # Dead Simple Signing Envelope (DSSE) Implementation
//!
//! This module provides a Rust implementation of the [Dead Simple Signing Envelope (DSSE)
//! specification](https://github.com/secure-systems-lab/dsse/blob/master/envelope.md),
//! which is a standard format for signing arbitrary payloads. DSSE is commonly
//! used in software supply chain security frameworks, including in-toto and Sigstore.
//!
//! ## Overview
//!
//! DSSE defines a simple envelope format that contains:
//! - A payload (the actual data being signed)
//! - A payload type (describing the format of the payload)
//! - One or more signatures over the payload
//!
//! The signing process follows a specific algorithm where the signature is computed over
//! the concatenation of the payload type and the payload itself.
//!
//! ## Key Components
//!
//! - [`Envelope`] - The main DSSE container structure
//! - [`Signature`] - Individual cryptographic signatures with optional key identifiers
//!
//! ## Examples
//!
//! ### Creating and Signing a DSSE Envelope with in-toto payload
//!
//! ```no_run
//! use atlas_cli::in_toto::dsse::Envelope;
//! use atlas_cli::signing::signable::Signable;
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//! use std::path::PathBuf;
//!
//! // Create a new envelope with JSON-encoded in-toto payload
//! let payload = br#"{"statement": "example"}"#.to_vec();
//! let mut envelope = Envelope::new(&payload, "application/vnd.in-toto+json".to_string());
//!
//! // Sign the envelope (requires a valid private key file)
//! envelope.sign(PathBuf::from("private_key.pem"), HashAlgorithm::Sha384).unwrap();
//!
//! // Validate the envelope structure
//! assert!(envelope.validate());
//! ```
//!
//! ### Manual Signature Management
//!
//! ```rust
//! use atlas_cli::in_toto::dsse::Envelope;
//!
//! // Create a new envelope with arbitrary payload
//! let mut envelope = Envelope::new(&vec![1,2,3], "text/plain".to_string());
//!
//! // Add signatures manually
//! let signature_bytes = vec![0xab, 0xcd, 0xef, 0x01, 0x23];
//! envelope.add_signature(signature_bytes, "key-identifier".to_string()).unwrap();
//!
//! assert!(envelope.validate());
//! ```
//!
//! ## DSSE Specification Compliance
//!
//! This implementation follows the DSSE specification as defined at:
//! <https://github.com/secure-systems-lab/dsse>
//!
//! The key aspects of DSSE compliance include:
//! - Proper payload and payload type concatenation for signing
//! - Base64 encoding of binary data in JSON serialization
//! - Support for multiple signatures per envelope
//! - Validation of required fields and signature integrity

use crate::error::{Error, Result};
use crate::signing;
use crate::signing::signable::Signable;

use atlas_c2pa_lib::cose::HashAlgorithm;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// A cryptographic signature with optional key identifier for DSSE envelopes.
///
/// This struct represents a single signature within a DSSE (Dead Simple Signing Envelope).
/// It contains the base64-encoded signature bytes and an optional key identifier that
/// can be used to identify which key was used for signing.
///
/// # Fields
///
/// * `sig` - The cryptographic signature bytes (base64-encoded in JSON)
/// * `keyid` - Optional identifier for the signing key (can be empty)
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::in_toto::dsse::Signature;
///
/// // Create a new signature with some example bytes and a key identifier
/// let signature_bytes = vec![0xde, 0xad, 0xbe, 0xef, 0x13, 0xe7, 0x1e, 0x37];
/// let key_id = "signing-key-2024".to_string();
/// let signature = Signature::new(signature_bytes.clone(), key_id.clone());
///
/// // Access the signature data
/// assert_eq!(signature.sig(), &signature_bytes);
/// assert_eq!(signature.keyid(), &key_id);
///
/// // Create a signature without a key identifier
/// let anonymous_sig = Signature::new(vec![0x12, 0x34, 0x56], "".to_string());
/// assert_eq!(anonymous_sig.keyid(), "");
/// assert!(!anonymous_sig.sig().is_empty());
/// ```
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature {
    #[serde_as(as = "serde_with::base64::Base64")]
    sig: Vec<u8>,
    keyid: String,
}

impl Signature {
    /// Creates a new signature with the provided signature bytes and key identifier.
    ///
    /// # Arguments
    ///
    /// * `sig` - The cryptographic signature as a byte vector
    /// * `keyid` - String identifier for the key used to create the signature
    ///
    /// # Returns
    ///
    /// A new `Signature` instance.
    pub fn new(sig: Vec<u8>, keyid: String) -> Self {
        Self {
            sig: sig,
            keyid: keyid,
        }
    }

    /// Returns a reference to the signature bytes.
    ///
    /// # Returns
    ///
    /// A slice containing the cryptographic signature bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_cli::in_toto::dsse::Signature;
    ///
    /// let sig_bytes = vec![0xab, 0xcd, 0xef];
    /// let signature = Signature::new(sig_bytes.clone(), "key1".to_string());
    /// assert_eq!(signature.sig(), &sig_bytes);
    /// ```
    pub fn sig(&self) -> &[u8] {
        &self.sig
    }

    /// Returns a reference to the key identifier.
    ///
    /// # Returns
    ///
    /// A string slice containing the key identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_cli::in_toto::dsse::Signature;
    ///
    /// let signature = Signature::new(vec![0xab, 0xcd], "test-key".to_string());
    /// assert_eq!(signature.keyid(), "test-key");
    /// ```
    pub fn keyid(&self) -> &str {
        &self.keyid
    }
}

/// A DSSE (Dead Simple Signing Envelope) structure for signed payloads.
///
/// The Envelope represents a complete DSSE structure containing a payload, its type,
/// and one or more cryptographic signatures. This structure follows the DSSE specification
/// for creating tamper-evident, authenticated containers for arbitrary payloads.
///
/// # Fields
///
/// * `payload` - The actual data being signed (base64-encoded in JSON)
/// * `payload_type` - MIME type or identifier describing the payload format
/// * `signatures` - Vector of cryptographic signatures over the payload
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::in_toto::dsse::Envelope;
///
/// let payload = b"Hello, world!".to_vec();
/// let mut envelope = Envelope::new(&payload, "text/plain".to_string());
///
/// // Add signatures using the sign() method from Signable trait
/// // envelope.sign(key_path, hash_algorithm)?;
///
/// assert!(envelope.validate());
/// ```
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Envelope {
    #[serde_as(as = "serde_with::base64::Base64")]
    payload: Vec<u8>,
    payload_type: String,
    signatures: Vec<Signature>,
}

impl Envelope {
    /// Creates a new DSSE envelope with the specified payload and type.
    ///
    /// The envelope is created without any signatures. Signatures must be added
    /// separately using the `add_signature` method or the `sign` method from
    /// the `Signable` trait.
    ///
    /// # Arguments
    ///
    /// * `payload` - The data to be contained in the envelope
    /// * `payload_type` - String describing the payload format (e.g., MIME type)
    ///
    /// # Returns
    ///
    /// A new `Envelope` instance with an empty signatures vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_cli::in_toto::dsse::Envelope;
    ///
    /// let data = b"test payload".to_vec();
    /// let envelope = Envelope::new(&data, "application/json".to_string());
    /// assert_eq!(envelope.payload_type(), "application/json");
    /// assert!(envelope.signatures().is_empty());
    /// ```
    pub fn new(payload: &Vec<u8>, payload_type: String) -> Self {
        Self {
            payload: payload.to_vec(),
            payload_type: payload_type,
            signatures: vec![],
        }
    }

    /// Adds a signature to the envelope.
    ///
    /// This method appends a new signature to the envelope's signature list.
    /// Each signature includes the signature bytes and an optional key identifier.
    ///
    /// # Arguments
    ///
    /// * `sig` - The cryptographic signature as a byte vector
    /// * `keyid` - String identifier for the signing key (can be empty)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the signature is invalid.
    ///
    /// # Errors
    ///
    /// Returns a `Signing` error if the signature bytes are empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_cli::in_toto::dsse::Envelope;
    ///
    /// let mut envelope = Envelope::new(&vec![1,2,3], "test".to_string());
    /// let signature_bytes = vec![0xab, 0xcd, 0xef];
    ///
    /// envelope.add_signature(signature_bytes, "key-1".to_string()).unwrap();
    /// assert_eq!(envelope.signatures().len(), 1);
    /// ```
    pub fn add_signature(&mut self, sig: Vec<u8>, keyid: String) -> Result<()> {
        if sig.is_empty() {
            return Err(Error::Signing("DSSE signature cannot be empty".to_string()));
        }

        let sig_struct = Signature::new(sig, keyid);
        self.signatures.push(sig_struct);

        Ok(())
    }

    /// Validates the envelope structure and contents.
    ///
    /// This method performs basic validation to ensure the envelope contains
    /// all required fields and that signatures are properly formatted. It checks:
    /// - Payload is not empty
    /// - Payload type is specified
    /// - At least one signature is present
    /// - All signatures contain non-empty signature bytes
    ///
    /// # Returns
    ///
    /// `true` if the envelope is valid, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_cli::in_toto::dsse::Envelope;
    ///
    /// let mut envelope = Envelope::new(&vec![1,2,3], "test".to_string());
    /// assert!(!envelope.validate()); // No signatures yet
    ///
    /// envelope.add_signature(vec![0xab, 0xcd], "key".to_string()).unwrap();
    /// assert!(envelope.validate()); // Now valid
    /// ```
    pub fn validate(&self) -> bool {
        // check for required envelope fields
        if self.payload.is_empty() || self.payload_type.is_empty() || self.signatures.is_empty() {
            return false;
        }

        // check required signature fields
        for signature in &self.signatures {
            if signature.sig.is_empty() {
                return false;
            }
        }

        true
    }

    /// Returns a reference to the payload bytes.
    ///
    /// # Returns
    ///
    /// A slice containing the raw payload data.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_cli::in_toto::dsse::Envelope;
    ///
    /// let payload = b"test data".to_vec();
    /// let envelope = Envelope::new(&payload, "text/plain".to_string());
    /// assert_eq!(envelope.payload(), &payload);
    /// ```
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns a reference to the payload type.
    ///
    /// # Returns
    ///
    /// A string slice containing the payload type/MIME type.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_cli::in_toto::dsse::Envelope;
    ///
    /// let envelope = Envelope::new(&vec![1,2,3], "application/json".to_string());
    /// assert_eq!(envelope.payload_type(), "application/json");
    /// ```
    pub fn payload_type(&self) -> &str {
        &self.payload_type
    }

    /// Returns a reference to the signatures vector.
    ///
    /// # Returns
    ///
    /// A slice containing all signatures in the envelope.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_cli::in_toto::dsse::Envelope;
    ///
    /// let mut envelope = Envelope::new(&vec![1,2,3], "test".to_string());
    /// assert_eq!(envelope.signatures().len(), 0);
    ///
    /// envelope.add_signature(vec![0xab, 0xcd], "key1".to_string()).unwrap();
    /// assert_eq!(envelope.signatures().len(), 1);
    /// assert_eq!(envelope.signatures()[0].keyid(), "key1");
    /// ```
    pub fn signatures(&self) -> &[Signature] {
        &self.signatures
    }
}

/// Implementation of the `Signable` trait for DSSE envelopes.
///
/// This implementation allows envelopes to be signed using private keys and
/// specified hash algorithms. The signing process follows the DSSE specification,
/// which requires signing the concatenation of the payload type and payload.
impl Signable for Envelope {
    /// Signs the envelope using the provided private key and hash algorithm.
    ///
    /// This method implements the DSSE signing specification by:
    /// 1. Loading the private key from the specified path
    /// 2. Concatenating the payload type and payload bytes
    /// 3. Creating a cryptographic signature over the concatenated data
    /// 4. Adding the signature to the envelope
    ///
    /// # Arguments
    ///
    /// * `key_path` - Path to the private key file
    /// * `hash_alg` - Hash algorithm to use for signing
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful signing, or an error if signing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Private key cannot be loaded
    /// - Signing operation fails
    /// - Signature cannot be added to the envelope
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use atlas_cli::in_toto::dsse::Envelope;
    /// use atlas_cli::signing::signable::Signable;
    /// use atlas_c2pa_lib::cose::HashAlgorithm;
    /// use std::path::PathBuf;
    ///
    /// let mut envelope = Envelope::new(&vec![1,2,3], "test".to_string());
    /// envelope.sign(PathBuf::from("private_key.pem"), HashAlgorithm::Sha384).unwrap();
    /// assert!(envelope.validate());
    /// ```
    fn sign(&mut self, key_path: PathBuf, hash_alg: HashAlgorithm) -> Result<()> {
        let private_key = signing::load_private_key(&key_path)?;

        // DSSE requires that payload_type and payload be signed
        // We assume the payload is public
        let mut data_to_sign: Vec<u8> = Vec::new();
        data_to_sign.extend_from_slice(&self.payload_type.clone().into_bytes());

        // DSSE requires payload to be JSON bytes
        data_to_sign.extend_from_slice(&self.payload);

        // Use the signing module with the specified algorithm
        let signature = signing::sign_data_with_algorithm(&data_to_sign, &private_key, &hash_alg)?;

        self.add_signature(signature, "".to_string()) // keyid is optional
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use serde_json::{from_slice, from_str, json, to_string, to_vec};

    #[test]
    fn test_signature_new() {
        let sig_bytes = vec![0xde, 0xad, 0xbe, 0xef];
        let keyid = "test-key".to_string();

        let signature = Signature::new(sig_bytes.clone(), keyid.clone());

        assert_eq!(signature.sig(), &sig_bytes);
        assert_eq!(signature.keyid(), &keyid);
    }

    #[test]
    fn test_signature_empty_keyid() {
        let sig_bytes = vec![0x12, 0x34];
        let signature = Signature::new(sig_bytes.clone(), "".to_string());

        assert_eq!(signature.sig(), &sig_bytes);
        assert_eq!(signature.keyid(), "");
    }

    #[test]
    fn test_signature_getters() {
        let sig_bytes = vec![0xab, 0xcd, 0xef, 0x01, 0x23];
        let keyid = "signing-key-2024".to_string();
        let signature = Signature::new(sig_bytes.clone(), keyid.clone());

        assert_eq!(signature.sig().len(), 5);
        assert_eq!(signature.sig()[0], 0xab);
        assert_eq!(signature.keyid().len(), 16);
        assert!(signature.keyid().contains("2024"));
    }

    #[test]
    fn test_envelope_new() {
        let payload = b"test payload".to_vec();
        let payload_type = "text/plain".to_string();

        let envelope = Envelope::new(&payload, payload_type.clone());

        assert_eq!(envelope.payload(), &payload);
        assert_eq!(envelope.payload_type(), &payload_type);
        assert!(envelope.signatures().is_empty());
    }

    #[test]
    fn test_envelope_add_signature_success() {
        let mut envelope = Envelope::new(&vec![1, 2, 3], "test".to_string());
        let sig_bytes = vec![0xab, 0xcd, 0xef];
        let keyid = "key-1".to_string();

        let result = envelope.add_signature(sig_bytes.clone(), keyid.clone());

        assert!(result.is_ok());
        assert_eq!(envelope.signatures().len(), 1);
        assert_eq!(envelope.signatures()[0].sig(), &sig_bytes);
        assert_eq!(envelope.signatures()[0].keyid(), &keyid);
    }

    #[test]
    fn test_envelope_add_signature_empty_fails() {
        let mut envelope = Envelope::new(&vec![1, 2, 3], "test".to_string());

        let result = envelope.add_signature(vec![], "key-1".to_string());

        assert!(result.is_err());
        if let Err(Error::Signing(msg)) = result {
            assert_eq!(msg, "DSSE signature cannot be empty");
        }
        assert_eq!(envelope.signatures().len(), 0);
    }

    #[test]
    fn test_envelope_add_multiple_signatures() {
        let mut envelope = Envelope::new(&vec![1, 2, 3], "test".to_string());

        envelope
            .add_signature(vec![0x01, 0x02], "key-1".to_string())
            .unwrap();
        envelope
            .add_signature(vec![0x03, 0x04], "key-2".to_string())
            .unwrap();
        envelope
            .add_signature(vec![0x05, 0x06], "".to_string())
            .unwrap();

        assert_eq!(envelope.signatures().len(), 3);
        assert_eq!(envelope.signatures()[0].keyid(), "key-1");
        assert_eq!(envelope.signatures()[1].keyid(), "key-2");
        assert_eq!(envelope.signatures()[2].keyid(), "");
    }

    #[test]
    fn test_envelope_validate_valid() {
        let mut envelope = Envelope::new(&vec![1, 2, 3], "test".to_string());
        envelope
            .add_signature(vec![0xab, 0xcd], "key".to_string())
            .unwrap();

        assert!(envelope.validate());
    }

    #[test]
    fn test_envelope_validate_no_signatures() {
        let envelope = Envelope::new(&vec![1, 2, 3], "test".to_string());

        assert!(!envelope.validate());
    }

    #[test]
    fn test_envelope_validate_empty_payload() {
        let mut envelope = Envelope::new(&vec![], "test".to_string());
        envelope
            .add_signature(vec![0xab, 0xcd], "key".to_string())
            .unwrap();

        assert!(!envelope.validate());
    }

    #[test]
    fn test_envelope_validate_empty_payload_type() {
        let mut envelope = Envelope::new(&vec![1, 2, 3], "".to_string());
        envelope
            .add_signature(vec![0xab, 0xcd], "key".to_string())
            .unwrap();

        assert!(!envelope.validate());
    }

    #[test]
    fn test_envelope_validate_empty_signature_bytes() {
        let mut envelope = Envelope::new(&vec![1, 2, 3], "test".to_string());
        // Force add a signature with empty bytes by creating it directly
        let empty_sig = Signature::new(vec![], "key".to_string());
        envelope.signatures.push(empty_sig);

        assert!(!envelope.validate());
    }

    #[test]
    fn test_envelope_validate_mixed_signatures() {
        let mut envelope = Envelope::new(&vec![1, 2, 3], "test".to_string());
        envelope
            .add_signature(vec![0xab, 0xcd], "key1".to_string())
            .unwrap();
        // Force add an empty signature
        let empty_sig = Signature::new(vec![], "key2".to_string());
        envelope.signatures.push(empty_sig);

        assert!(!envelope.validate()); // Should fail due to empty signature
    }

    #[test]
    fn test_envelope_getters() {
        let payload = b"Hello, DSSE world!".to_vec();
        let payload_type = "text/plain".to_string();
        let mut envelope = Envelope::new(&payload, payload_type.clone());

        // Test initial state
        assert_eq!(envelope.payload(), &payload);
        assert_eq!(envelope.payload_type(), &payload_type);
        assert_eq!(envelope.signatures().len(), 0);

        // Add signatures and test
        envelope
            .add_signature(vec![0x01, 0x02, 0x03], "key1".to_string())
            .unwrap();
        envelope
            .add_signature(vec![0x04, 0x05, 0x06], "key2".to_string())
            .unwrap();

        assert_eq!(envelope.signatures().len(), 2);
        assert_eq!(envelope.signatures()[0].keyid(), "key1");
        assert_eq!(envelope.signatures()[1].keyid(), "key2");
    }

    #[test]
    fn test_envelope_json_payload_type() {
        let json_payload = json!({"field1": "hello", "field2": "world"});
        let envelope = Envelope::new(
            &to_vec(&json_payload).unwrap(),
            "application/json".to_string(),
        );

        let deserialized_payload: serde_json::Value = from_slice(envelope.payload()).unwrap();

        assert_eq!(envelope.payload_type(), "application/json");
        assert_eq!(deserialized_payload["field1"], "hello");
    }

    #[test]
    fn test_signature_serialization_fields() {
        // Test that the signature has the expected structure for serialization
        let sig_bytes = vec![0xde, 0xad, 0xbe, 0xef];
        let keyid = "test-key-id".to_string();
        let signature = Signature::new(sig_bytes.clone(), keyid.clone());

        // Verify the signature maintains its data correctly
        assert_eq!(signature.sig().len(), 4);
        assert_eq!(signature.keyid().len(), 11);
        assert_eq!(signature.sig(), &sig_bytes);
        assert_eq!(signature.keyid(), &keyid);
    }

    #[test]
    fn test_envelope_large_payload() {
        // Test with a larger payload to ensure there are no size-related issues
        let large_payload = vec![0x42; 10000]; // 10KB of 0x42 bytes
        let envelope = Envelope::new(&large_payload, "application/test".to_string());

        assert_eq!(envelope.payload().len(), 10000);
        assert_eq!(envelope.payload()[0], 0x42);
        assert_eq!(envelope.payload()[9999], 0x42);
        assert_eq!(envelope.payload_type(), "application/test");
    }

    #[test]
    fn test_envelope_json_serialization() {
        let payload = json!({"field1": "hello", "field2": "world"});
        let mut envelope =
            Envelope::new(&to_vec(&payload).unwrap(), "application/json".to_string());
        envelope
            .add_signature(vec![0x01, 0x02, 0x03], "key1".to_string())
            .unwrap();

        // Serialize to JSON
        let json_str = to_string(&envelope).unwrap();

        // Check that the payload is valid base64 in the JSON
        BASE64_STANDARD
            .decode(
                &from_str::<serde_json::Value>(&json_str).unwrap()["payload"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();

        // Deserialize back
        let deserialized: Envelope = serde_json::from_str(&json_str).unwrap();
        let deserialized_payload: serde_json::Value = from_slice(deserialized.payload()).unwrap();

        // Verify fields match
        assert_eq!(deserialized_payload["field1"], "hello");
        assert_eq!(deserialized.payload_type(), "application/json");
        assert_eq!(deserialized.signatures().len(), 1);
        assert_eq!(deserialized.signatures()[0].keyid(), "key1");
        assert_eq!(deserialized.signatures()[0].sig(), &[0x01, 0x02, 0x03]);
    }
}
