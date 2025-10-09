//! # Signable Trait
//!
//! This module defines the `Signable` trait, which provides a common interface for
//! cryptographically signing data structures using private keys and hash algorithms.
//! The trait is designed to be implemented by any type that needs to support
//! digital signatures for authentication and integrity verification.
//!
//! ## Supported Hash Algorithms
//!
//! The trait supports various hash algorithms through the `atlas_c2pa_lib::cose::HashAlgorithm`
//! enum, including:
//! - SHA-256
//! - SHA-384
//! - SHA-512
//!
//! ## Examples
//!
//! ### Implementing Signable for a Custom Type
//!
//! ```
//! use atlas_cli::signing::signable::Signable;
//! use atlas_cli::error::Result;
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//! use std::path::PathBuf;
//!
//! struct MyDocument {
//!     content: String,
//!     signature: Option<Vec<u8>>,
//! }
//!
//! impl Signable for MyDocument {
//!     fn sign(&mut self, key_path: PathBuf, hash_alg: HashAlgorithm) -> Result<()> {
//!         // Implementation would:
//!         // 1. Load the private key from key_path
//!         // 2. Hash the document content using hash_alg
//!         // 3. Sign the hash with the private key
//!         // 4. Store the signature in self.signature
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ### Using a Signable Type
//!
//! ```no_run
//! use atlas_cli::in_toto::dsse::Envelope;
//! use atlas_cli::signing::signable::Signable;
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//! use std::path::PathBuf;
//!
//! let mut envelope = Envelope::new(&b"data".to_vec(), "text/plain".to_string());
//!
//! // Sign with a private key using SHA-384
//! envelope.sign(
//!     PathBuf::from("private_key.pem"),
//!     HashAlgorithm::Sha384
//! ).unwrap();
//! ```

use crate::error::Result;

use atlas_c2pa_lib::cose::HashAlgorithm;
use std::path::PathBuf;

/// A trait for types that can be cryptographically signed.
///
/// This module defines the `Signable` trait, which provides a common interface for
/// cryptographically signing data structures using private keys and hash algorithms.
/// The trait is designed to be implemented by any type that needs to support
/// digital signatures for authentication and integrity verification.
///
/// ## Implementation Recommendations
///
/// Implementations should:
/// 1. Load the private key from the provided path
/// 2. Prepare the data to be signed (serialization, canonicalization, etc.)
/// 3. Create a cryptographic signature using the specified hash algorithm
/// 4. Store or attach the signature to the data structure
/// 5. Return appropriate errors for any failure conditions
///
/// ## Error Handling
///
/// Implementations should return errors for common failure scenarios:
/// - Invalid or missing private key files
/// - Unsupported key formats or algorithms
/// - Cryptographic operation failures
/// - Data preparation or serialization errors
///
/// ## Examples
///
/// ### Basic Implementation Pattern
///
/// ```no_run
/// use atlas_cli::signing::signable::Signable;
/// use atlas_cli::error::Result;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
/// use std::path::PathBuf;
///
/// struct SignableDocument {
///     data: Vec<u8>,
///     signatures: Vec<Vec<u8>>,
/// }
///
/// impl Signable for SignableDocument {
///     fn sign(&mut self, key_path: PathBuf, hash_alg: HashAlgorithm) -> Result<()> {
///         // 1. Load private key (implementation-specific)
///         // let private_key = load_private_key(&key_path)?;
///         
///         // 2. Prepare data for signing
///         // let data_to_sign = prepare_signing_data(&self.data);
///         
///         // 3. Create signature
///         // let signature = create_signature(&data_to_sign, &private_key, &hash_alg)?;
///         
///         // 4. Store signature
///         // self.signatures.push(signature);
///         
///         Ok(())
///     }
/// }
/// ```
///
/// ### Usage with Different Hash Algorithms
///
/// ```no_run
/// use atlas_cli::signing::signable::Signable;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
/// use std::path::PathBuf;
/// # use atlas_cli::in_toto::dsse::Envelope;
///
/// let mut envelope = Envelope::new(&b"important data".to_vec(), "text/plain".to_string());
/// let key_path = PathBuf::from("signing_key.pem");
///
/// // Sign with SHA-256
/// envelope.sign(key_path.clone(), HashAlgorithm::Sha256).unwrap();
///
/// // Sign with SHA-384 for higher security
/// envelope.sign(key_path.clone(), HashAlgorithm::Sha384).unwrap();
/// ```
pub trait Signable {
    /// Signs the implementing type using a private key and hash algorithm.
    ///
    /// This method performs cryptographic signing of the data structure using
    /// the specified private key and hash algorithm. The exact signing process
    /// depends on the implementation.
    ///
    /// ## Arguments
    ///
    /// * `key_path` - Path to the private key file (PEM, PKCS#8, etc.)
    /// * `hash_alg` - Hash algorithm to use for signature generation
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` on successful signing, or an error if signing fails.
    ///
    /// ## Errors
    ///
    /// This method may return errors in the following situations:
    /// - **Key Loading**: Private key file doesn't exist, is corrupted, or has wrong format
    /// - **Unsupported Algorithm**: The hash algorithm is not supported by the key type
    /// - **Cryptographic Failure**: The signing operation fails due to invalid key or data
    /// - **Storage Failure**: Unable to attach the signature to the data structure
    ///
    /// ## Examples
    ///
    /// ### Signing a DSSE Envelope
    ///
    /// ```no_run
    /// use atlas_cli::in_toto::dsse::Envelope;
    /// use atlas_cli::signing::signable::Signable;
    /// use atlas_c2pa_lib::cose::HashAlgorithm;
    /// use std::path::PathBuf;
    ///
    /// let mut envelope = Envelope::new(&b"payload data".to_vec(), "application/json".to_string());
    ///
    /// // Sign with an RSA private key using SHA-384
    /// envelope.sign(
    ///     PathBuf::from("rsa_private_key.pem"),
    ///     HashAlgorithm::Sha384
    /// ).unwrap();
    ///
    /// // Verify the envelope now has a signature
    /// assert!(envelope.validate());
    /// assert!(!envelope.signatures().is_empty());
    /// ```
    ///
    /// ### Error Handling
    ///
    /// ```no_run
    /// use atlas_cli::in_toto::dsse::Envelope;
    /// use atlas_cli::signing::signable::Signable;
    /// use atlas_c2pa_lib::cose::HashAlgorithm;
    /// use std::path::PathBuf;
    ///
    /// let mut envelope = Envelope::new(&b"data".to_vec(), "text/plain".to_string());
    ///
    /// // Handle signing errors gracefully
    /// match envelope.sign(PathBuf::from("nonexistent_key.pem"), HashAlgorithm::Sha256) {
    ///     Ok(()) => println!("Signing successful"),
    ///     Err(e) => eprintln!("Signing failed: {}", e),
    /// }
    /// ```
    fn sign(&mut self, key_path: PathBuf, hash_alg: HashAlgorithm) -> Result<()>;
}
