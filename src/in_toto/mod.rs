//! # in-toto Attestation Generation Library
//!
//! This module provides functionality for creating and signing in-toto attestations using
//! Dead Simple Signing Envelope (DSSE) format. It includes utilities for generating
//! resource descriptors, creating signed statements, and working with protobuf structures.
//!
//! ## Key Components
//!
//! - **Resource Descriptors**: Structures that describe artifacts with their digests
//! - **Statement Generation**: Creation of in-toto attestation statements
//! - **DSSE Envelopes**: Containers for signed payloads with cryptographic signatures
//! - **JSON/Protobuf Conversion**: Utilities for converting between formats
//!
//! ## Examples
//!
//! ```no_run
//! use atlas_cli::in_toto::{make_minimal_resource_descriptor, generate_signed_statement_v1};
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//! use protobuf::well_known_types::struct_::Struct;
//! use std::path::PathBuf;
//!
//! // Create a resource descriptor for an artifact
//! let resource = make_minimal_resource_descriptor(
//!     "model.onnx",
//!     "sha384",
//!     "abc123..."
//! );
//!
//! // Generate a signed statement (requires valid key and predicate)
//! let predicate_struct = Struct::new();
//! let envelope = generate_signed_statement_v1(
//!     &[resource],
//!     "https://in-toto.io/Statement/v0.1",
//!     &predicate_struct,
//!     PathBuf::from("private_key.pem"),
//!     HashAlgorithm::Sha384,
//! ).unwrap();
//! ```

use crate::error::{Error, Result};
use crate::hash;
use crate::signing::signable::Signable;

use atlas_c2pa_lib::cose::HashAlgorithm;
use in_toto_attestation::generate_statement_v1;
use in_toto_attestation::v1::resource_descriptor::ResourceDescriptor;
use protobuf::well_known_types::struct_::Struct;
use protobuf_json_mapping::{parse_from_str, print_to_string};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod dsse;

use dsse::Envelope;

const DSSE_PAYLOAD_TYPE: &str = "application/vnd.in-toto+json";

/// Converts a JSON string to a protobuf Struct.
///
/// This function parses a JSON string and converts it into a protobuf `Struct` type,
/// which is commonly used in in-toto attestations as predicate data. The conversion
/// handles nested JSON objects and arrays appropriately.
///
/// # Arguments
///
/// * `json_str` - A JSON string to be converted to protobuf format
///
/// # Returns
///
/// A protobuf `Struct` on success, or a serialization error if parsing fails.
///
/// # Errors
///
/// Returns a `Serialization` error if:
/// - The input string is not valid JSON
/// - The JSON structure cannot be converted to protobuf format
///
/// # Examples
///
/// ```
/// use atlas_cli::in_toto::json_to_struct_proto;
///
/// let json_data = r#"{"name": "test", "version": "1.0"}"#;
/// let struct_proto = json_to_struct_proto(json_data).unwrap();
///
/// // The resulting struct can be used in in-toto predicates
/// assert!(!struct_proto.fields.is_empty());
/// ```
pub fn json_to_struct_proto(json_str: &str) -> Result<Struct> {
    let msg_struct = parse_from_str::<Struct>(&json_str).map_err(|e| {
        Error::Serialization(format!("Failed to serialize in-toto statement: {}", e))
    })?;

    Ok(msg_struct)
}

/// Creates a minimal resource descriptor with name and digest information.
///
/// This function constructs a basic in-toto resource descriptor containing only
/// the essential fields: name and digest. The digest is provided as a single
/// algorithm-value pair, making this suitable for simple attestation scenarios.
///
/// # Arguments
///
/// * `name` - The name or identifier of the resource (e.g., filename, artifact name)
/// * `alg` - The hash algorithm used (e.g., "sha256", "sha384", "sha512")
/// * `digest` - The hex-encoded hash value of the resource
///
/// # Returns
///
/// A `ResourceDescriptor` with the specified name and digest information.
///
/// # Examples
///
/// ```
/// use atlas_cli::in_toto::make_minimal_resource_descriptor;
///
/// let descriptor = make_minimal_resource_descriptor(
///     "model.onnx",
///     "sha384",
///     "a1b2c3d4e5f6..."
/// );
///
/// assert_eq!(descriptor.name, "model.onnx");
/// assert!(descriptor.digest.contains_key("sha384"));
/// ```
pub fn make_minimal_resource_descriptor(name: &str, alg: &str, digest: &str) -> ResourceDescriptor {
    let digest_set = HashMap::from([(alg.to_string(), digest.to_string())]);

    let mut rd = ResourceDescriptor::new();
    rd.name = name.to_string();
    rd.digest = digest_set;

    rd
}

/// Generates a resource descriptor from a file path by computing its hash.
///
/// This function creates a complete resource descriptor for a file by reading the file,
/// computing its hash using the specified algorithm, and constructing the descriptor
/// with the file path as the name and the computed hash as the digest.
///
/// # Arguments
///
/// * `path` - The file system path to the file to be described
/// * `algorithm` - The hash algorithm to use for computing the file content digest
///
/// # Returns
///
/// A `ResourceDescriptor` with the file path and computed hash, or an error if
/// the file cannot be read or hashed.
///
/// # Errors
///
/// Returns an error if:
/// - The file at the specified path cannot be read
/// - Hash computation fails
/// - The path cannot be converted to a string
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::in_toto::generate_file_resource_descriptor_from_path;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
/// use std::path::Path;
///
/// let path = Path::new("test_file.txt");
/// let rd = generate_file_resource_descriptor_from_path(
///     &path,
///     &HashAlgorithm::Sha384
/// ).unwrap();
///
/// assert_eq!(rd.name, "test_file.txt");
/// assert!(rd.digest.contains_key("sha384"));
/// ```
pub fn generate_file_resource_descriptor_from_path(
    path: &Path,
    algorithm: &HashAlgorithm,
) -> Result<ResourceDescriptor> {
    let file_hash = hash::calculate_file_hash_with_algorithm(path, algorithm)?;

    let digest_set = HashMap::from([(algorithm.as_str().to_string(), file_hash.to_string())]);

    let mut rd = ResourceDescriptor::new();
    rd.name = String::from(path.to_string_lossy());
    rd.digest = digest_set;

    Ok(rd)
}

/// Generates a signed in-toto Statement v1 wrapped in a DSSE envelope.
///
/// This function creates a complete in-toto attestation by generating a v1 Statement with
/// the provided subjects and predicate, then signing it using DSSE (Dead Simple Signing
/// Envelope) format. The resulting envelope contains the signed statement and can be
/// used for attestation verification.
///
/// # Arguments
///
/// * `subject` - Array of resource descriptors representing the artifacts being attested
/// * `predicate_type` - URI identifying the type of predicate (e.g., SLSA provenance type)
/// * `predicate` - Protobuf struct containing the predicate data specific to the statement type
/// * `key_path` - Path to the private key file used for signing
/// * `hash_alg` - Hash algorithm to use for signing operations
///
/// # Returns
///
/// A signed DSSE `Envelope` containing the in-toto statement, or an error if generation fails.
///
/// # Errors
///
/// Returns an error if:
/// - Statement generation fails
/// - Statement serialization fails
/// - Signing operation fails
/// - Private key cannot be loaded
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::in_toto::{generate_signed_statement_v1, make_minimal_resource_descriptor};
/// use atlas_c2pa_lib::cose::HashAlgorithm;
/// use protobuf::well_known_types::struct_::Struct;
/// use std::path::PathBuf;
///
/// let subjects = vec![make_minimal_resource_descriptor(
///     "artifact.bin",
///     "sha384",
///     "abc123..."
/// )];
///
/// let predicate = Struct::new(); // Needs to be populated with actual predicate data
///
/// let envelope = generate_signed_statement_v1(
///     &subjects,
///     "https://slsa.dev/provenance/v1",
///     &predicate,
///     PathBuf::from("private_key.pem"),
///     HashAlgorithm::Sha384,
/// ).unwrap();
///
/// assert!(envelope.validate());
/// ```
pub fn generate_signed_statement_v1(
    subject: &[ResourceDescriptor],
    predicate_type: &str,
    predicate: &Struct,
    key_path: PathBuf,
    hash_alg: HashAlgorithm,
) -> Result<Envelope> {
    let statement = generate_statement_v1(subject, predicate_type, predicate)
        .map_err(|e| Error::Signing(e.to_string()))?;

    let serialized_statement =
        print_to_string(&statement).map_err(|e| Error::Serialization(e.to_string()))?;

    let mut envelope = Envelope::new(
        &serialized_statement.into_bytes(),
        DSSE_PAYLOAD_TYPE.to_string(),
    );
    envelope.sign(key_path, hash_alg)?;

    Ok(envelope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::test_utils::generate_temp_key;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_json_to_struct_proto_simple() {
        let json_str = r#"{"name": "test", "version": "1.0"}"#;
        let result = json_to_struct_proto(json_str);

        assert!(result.is_ok());
        let struct_proto = result.unwrap();
        assert_eq!(struct_proto.fields.len(), 2);
        assert!(struct_proto.fields.contains_key("name"));
        assert!(struct_proto.fields.contains_key("version"));
    }

    #[test]
    fn test_json_to_struct_proto_empty() {
        let json_str = "{}";
        let result = json_to_struct_proto(json_str);

        assert!(result.is_ok());
        let struct_proto = result.unwrap();
        assert!(struct_proto.fields.is_empty());
    }

    #[test]
    fn test_json_to_struct_proto_invalid_json() {
        let invalid_json = r#"{"invalid": json"#;
        let result = json_to_struct_proto(invalid_json);

        assert!(result.is_err());
    }

    #[test]
    fn test_make_minimal_resource_descriptor() {
        let name = "test_file.txt";
        let alg = "sha256";
        let digest = "abc123def456";

        let rd = make_minimal_resource_descriptor(name, alg, digest);

        assert_eq!(rd.name, name);
        assert_eq!(rd.digest.len(), 1);
        assert_eq!(rd.digest.get(alg), Some(&digest.to_string()));
    }

    #[test]
    fn test_make_minimal_resource_descriptor_empty_values() {
        let rd = make_minimal_resource_descriptor("", "", "");

        assert_eq!(rd.name, "");
        assert_eq!(rd.digest.len(), 1);
        assert_eq!(rd.digest.get(""), Some(&"".to_string()));
    }

    #[test]
    fn test_generate_file_resource_descriptor_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let content = b"test content for hashing";

        // Create test file
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content).unwrap();

        let result =
            generate_file_resource_descriptor_from_path(&file_path, &HashAlgorithm::Sha256);

        assert!(result.is_ok());
        let rd = result.unwrap();
        assert_eq!(rd.name, file_path.to_string_lossy());
        assert!(rd.digest.contains_key("sha256"));
        assert!(!rd.digest["sha256"].is_empty());
    }

    #[test]
    fn test_generate_file_resource_descriptor_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let missing_file = temp_dir.path().join("nonexistent.txt");

        let result =
            generate_file_resource_descriptor_from_path(&missing_file, &HashAlgorithm::Sha256);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_file_resource_descriptor_different_algorithms() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test data").unwrap();

        let algorithms = vec![
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha384,
            HashAlgorithm::Sha512,
        ];

        for alg in algorithms {
            let result = generate_file_resource_descriptor_from_path(&file_path, &alg);
            assert!(result.is_ok());

            let rd = result.unwrap();
            assert!(rd.digest.contains_key(alg.as_str()));
            assert!(!rd.digest[alg.as_str()].is_empty());
        }
    }

    #[test]
    fn test_dsse_payload_type_constant() {
        assert_eq!(DSSE_PAYLOAD_TYPE, "application/vnd.in-toto+json");
    }

    #[test]
    fn test_generate_signed_statement_v1() {
        let (_secure_key, tmp_dir) = generate_temp_key().unwrap();

        let subjects = vec![make_minimal_resource_descriptor("test", "sha256", "abc123")];
        let predicate = Struct::new();

        let env = generate_signed_statement_v1(
            &subjects,
            "https://example.com/predicate",
            &predicate,
            tmp_dir.path().join("test_key.pem"),
            HashAlgorithm::Sha256,
        )
        .unwrap();

        let result = env.validate();
        assert!(result == true);
    }
}
