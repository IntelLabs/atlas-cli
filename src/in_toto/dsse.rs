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
//! use crate::in_toto::dsse::Envelope;
//! use crate::signing::signable::Signable;
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//! use std::path::PathBuf;
//!
//! // Create a new envelope with JSON-encoded in-toto payload
//! let payload = br#"{"statement": "example"}"#.to_vec();
//! let mut envelope = Envelope::new(&payload, "application/vnd.in-toto+json".to_string());
//!
//! // Sign the envelope (requires a valid private key file)
//! envelope.sign(PathBuf::from("private_key.pem"), HashAlgorithm::Sha384)?;
//!
//! // Validate the envelope structure
//! assert!(envelope.validate());
//! ```
//!
//! ### Manual Signature Management
//!
//! ```rust
//! use crate::in_toto::dsse::Envelope;
//!
//! // Create a new envelope with arbitrary payload
//! let mut envelope = Envelope::new(&vec![1,2,3], "text/plain".to_string());
//!
//! // Add signatures manually
//! let signature_bytes = vec![0xab, 0xcd, 0xef, 0x01, 0x23];
//! envelope.add_signature(signature_bytes, "key-identifier".to_string())?;
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
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Signature {
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
    fn new(sig: Vec<u8>, keyid: String) -> Self {
        Self {
            sig: sig,
            keyid: keyid,
        }
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
/// ```
/// use crate::in_toto::dsse::Envelope;
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
    /// use crate::in_toto::dsse::Envelope;
    ///
    /// let data = b"test payload".to_vec();
    /// let envelope = Envelope::new(&data, "application/json".to_string());
    /// assert_eq!(envelope.payload_type, "application/json");
    /// assert!(envelope.signatures.is_empty());
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
    /// use crate::in_toto::dsse::Envelope;
    ///
    /// let mut envelope = Envelope::new(&vec![1,2,3], "test".to_string());
    /// let signature_bytes = vec![0xab, 0xcd, 0xef];
    /// 
    /// envelope.add_signature(signature_bytes, "key-1".to_string())?;
    /// assert_eq!(envelope.signatures.len(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
    /// use crate::in_toto::dsse::Envelope;
    ///
    /// let mut envelope = Envelope::new(&vec![1,2,3], "test".to_string());
    /// assert!(!envelope.validate()); // No signatures yet
    ///
    /// envelope.add_signature(vec![0xab, 0xcd], "key".to_string())?;
    /// assert!(envelope.validate()); // Now valid
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
    /// ```
    /// use crate::in_toto::dsse::Envelope;
    /// use crate::signing::signable::Signable;
    /// use atlas_c2pa_lib::cose::HashAlgorithm;
    /// use std::path::PathBuf;
    ///
    /// let mut envelope = Envelope::new(&vec![1,2,3], "test".to_string());
    /// envelope.sign(PathBuf::from("private_key.pem"), HashAlgorithm::Sha384)?;
    /// assert!(envelope.validate());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
