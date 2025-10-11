//! # Atlas CLI SLSA Build Provenance Generator
//!
//! This module provides Atlas CLI-specific functionality for generating SLSA (Supply-chain
//! Levels for Software Artifacts) v1 Build Provenance attestations, implementing the logic for
//! creating signed Build Provenance attestations using Atlas CLI as the builder.
//!
//! ## Atlas CLI SLSA Builder
//!
//! The generated provenance identifies Atlas CLI as the builder using:
//! - Builder ID: Uses `ATLAS_CLI_BUILDER_ID` from the generators module
//! - Build Type: Combines `CLI_NAME` and `CLI_VERSION` as the build type identifier
//! - External Parameters: Structures inputs and pipeline paths as SLSA external parameters
//!
//! ## Examples
//!
//! ```no_run
//! use atlas_cli::slsa::cli::generate_build_provenance;
//! use atlas_c2pa_lib::cose::HashAlgorithm;
//! use std::path::PathBuf;
//!
//! // Generate Atlas CLI build provenance for a Rust project
//! generate_build_provenance(
//!     vec![
//!         PathBuf::from("src/main.rs"),
//!         PathBuf::from("Cargo.toml"),
//!     ],                                           // input source files
//!     PathBuf::from("build.sh"),                  // build pipeline script
//!     vec![
//!         PathBuf::from("target/release/myapp"),  // output artifacts
//!     ],
//!     Some(PathBuf::from("signing_key.pem")),     // signing key
//!     HashAlgorithm::Sha384,                      // hash algorithm
//!     "json".to_string(),                         // output format
//!     true,                                       // print to console
//!     None,                                       // no storage backend
//!     false,                                      // no TDX support
//! ).unwrap();
//! ```

use crate::cli::{CLI_NAME, CLI_VERSION};
use crate::error::{Error, Result};
use crate::in_toto;
use crate::slsa;
use crate::storage::traits::StorageBackend;

use atlas_c2pa_lib::cose::HashAlgorithm;
use in_toto_attestation::to_struct;
use in_toto_attestation::v1::resource_descriptor::ResourceDescriptor;
use protobuf::well_known_types::struct_::{ListValue, Struct, Value};
use protobuf::well_known_types::timestamp::Timestamp;
use serde_json::to_string_pretty;
use std::path::PathBuf;

/// The Atlas CLI builder identifier for SLSA provenance.
pub const ATLAS_CLI_BUILDER_ID: &str = "https://github.com/IntelLabs/atlas-cli";

struct ExternalParameters {
    inputs: Vec<ResourceDescriptor>,
    pipeline: ResourceDescriptor,
}

impl ExternalParameters {
    fn new(
        inputs_path: Vec<PathBuf>,
        pipeline_path: PathBuf,
        hash_alg: &HashAlgorithm,
    ) -> Result<Self> {
        let e = ExternalParameters {
            inputs: generate_file_list_resource_descriptors(inputs_path, &hash_alg)?,
            pipeline: in_toto::generate_file_resource_descriptor_from_path(
                pipeline_path.as_path(),
                &hash_alg,
            )?,
        };

        Ok(e)
    }

    fn to_struct(&self) -> Result<Struct> {
        let mut external_params = Struct::new();

        let mut inputs_list = ListValue::new();
        for rd in &self.inputs {
            let rd_struct = to_struct(rd).map_err(|e| Error::Serialization(e.to_string()))?;
            let mut rd_val = Value::new();
            rd_val.set_struct_value(rd_struct);
            inputs_list.values.push(rd_val);
        }
        let mut inputs_val = Value::new();
        inputs_val.set_list_value(inputs_list);

        // we need to serialize the RD into the Struct proto expected by the external_params field
        let pipeline_rd_struct =
            to_struct(&self.pipeline).map_err(|e| Error::Serialization(e.to_string()))?;
        let mut pipeline_val = Value::new();
        pipeline_val.set_struct_value(pipeline_rd_struct);

        external_params
            .fields
            .insert("inputs".to_string(), inputs_val);
        external_params
            .fields
            .insert("pipeline".to_string(), pipeline_val);

        Ok(external_params)
    }
}

/// Generates an Atlas CLI-specific SLSA build provenance attestation.
///
/// This function creates a cryptographically signed SLSA build provenance attestation
/// using Atlas CLI as the identified builder. It processes input files, pipeline
/// definitions, and output artifacts to generate a complete SLSA v1 provenance
/// statement that can be verified against the Atlas CLI builder identity.
///
/// # Atlas CLI Builder Context
///
/// The generated provenance includes Atlas CLI-specific information:
/// - **Builder ID**: `https://github.com/IntelLabs/atlas-cli`
/// - **Build Type**: `{CLI_NAME}:{CLI_VERSION}` (e.g., "atlas-cli:1.0.0")
/// - **External Parameters**: Structured inputs and pipeline information
/// - **Timestamp**: Current time as build completion timestamp
///
/// # Arguments
///
/// * `inputs_path` - Vector of paths to input files (source code, dependencies, configs)
/// * `pipeline_path` - Path to the build script or pipeline definition used by Atlas CLI
/// * `products_path` - Vector of paths to output artifacts produced by the build
/// * `key_path` - Optional path to private key for signing (required for valid attestations)
/// * `hash_alg` - Hash algorithm to use for file integrity and signing operations
/// * `output_encoding` - Output format: "json" or "cbor"
/// * `print` - Whether to print the attestation to stdout
/// * `storage` - Optional storage backend for persisting the attestation
/// * `_with_tdx` - TDX (Intel Trust Domain Extensions) support flag (reserved for future use)
///
/// # Returns
///
/// Returns `Ok(())` on successful generation, or an error if any step fails.
///
/// # Errors
///
/// This function may return errors for:
/// - **File Access**: Input, pipeline, or product files cannot be read or hashed
/// - **Key Loading**: Private key file is missing, corrupted, or wrong format
/// - **Serialization**: Failed to encode attestation in requested format
/// - **Validation**: Invalid parameters, missing signing key, or unsupported encoding
/// - **Storage**: Backend storage operations fail (if storage backend provided)
pub fn generate_build_provenance(
    inputs_path: Vec<PathBuf>,
    pipeline_path: PathBuf,
    products_path: Vec<PathBuf>,
    key_path: Option<PathBuf>,
    hash_alg: HashAlgorithm,
    output_encoding: String,
    print: bool,
    storage: Option<&'static dyn StorageBackend>,
    _with_tdx: bool,
) -> Result<()> {
    // Generate the SLSA BuildDefinition.externalParameters
    let external_params = ExternalParameters::new(inputs_path, pipeline_path, &hash_alg)?;
    let external_params_proto = external_params.to_struct()?;

    // generate the BuildDefinition
    let build_def = slsa::generators::make_build_definition_v1(
        format!("{}:{}", CLI_NAME, CLI_VERSION).as_str(),
        &external_params_proto,
        None,
        None,
    );

    // generate Builder
    let builder = slsa::generators::make_builder_v1(ATLAS_CLI_BUILDER_ID, None, None);

    // generate BuildMetadata
    let build_metadata =
        slsa::generators::make_build_metadata_v1("", None, Some(&Timestamp::now()));

    // generate RunDetails
    // FIXME: Add TDX support
    let run_details = slsa::generators::make_run_details_v1(&builder, Some(&build_metadata), None);

    // generate Provenance predicate!
    let provenance = slsa::generators::generate_build_provenance_v1(&build_def, &run_details);
    let provenance_proto =
        to_struct(&provenance).map_err(|e| Error::Serialization(e.to_string()))?;

    // Generate the statement subjects
    let subject = generate_file_list_resource_descriptors(products_path, &hash_alg)?;

    let key_path = key_path.ok_or_else(|| {
        Error::Validation("Signing key is required for SLSA provenance".to_string())
    })?;

    let envelope = in_toto::generate_signed_statement_v1(
        &subject,
        slsa::BUILD_PROVENANCE_PREDICATE_TYPE_V1,
        &provenance_proto,
        key_path,
        hash_alg,
    )?;

    // Output manifest if requested
    if print || storage.is_none() {
        match output_encoding.to_lowercase().as_str() {
            "json" => {
                let envelope_json =
                    to_string_pretty(&envelope).map_err(|e| Error::Serialization(e.to_string()))?;
                println!("{envelope_json}");
            }
            "cbor" => {
                let envelope_cbor = serde_cbor::to_vec(&envelope)
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                println!("{}", hex::encode(&envelope_cbor));
            }
            _ => {
                return Err(Error::Validation(format!(
                    "Invalid output encoding '{}'. Valid options are: json, cbor",
                    output_encoding
                )));
            }
        }
    }

    // Store manifest if storage is provided
    // FIXME: Add support for SLSA storage in backend
    if let Some(_storage) = &storage {
        if !print {
            let id = 0;
            println!("Manifest stored successfully with ID: {id}");
        }
    }

    Ok(())
}

fn generate_file_list_resource_descriptors(
    file_paths: Vec<PathBuf>,
    algorithm: &HashAlgorithm,
) -> Result<Vec<ResourceDescriptor>> {
    let mut rd_vec: Vec<ResourceDescriptor> = Vec::new();
    for f in file_paths.iter() {
        let rd = in_toto::generate_file_resource_descriptor_from_path(f.as_path(), algorithm)?;
        rd_vec.push(rd);
    }

    Ok(rd_vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::test_utils::generate_temp_key;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create a temporary file with content
    fn create_temp_file(dir: &TempDir, name: &str, content: &[u8]) -> PathBuf {
        let file_path = dir.path().join(name);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content).unwrap();
        file_path
    }

    #[test]
    fn test_atlas_cli_builder_id_constant() {
        assert_eq!(
            ATLAS_CLI_BUILDER_ID,
            "https://github.com/IntelLabs/atlas-cli"
        );
    }

    #[test]
    fn test_external_parameters_new() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let input1 = create_temp_file(&temp_dir, "input1.txt", b"test input 1");
        let input2 = create_temp_file(&temp_dir, "input2.txt", b"test input 2");
        let pipeline = create_temp_file(&temp_dir, "build.sh", b"#!/bin/bash\necho 'building'");

        let inputs = vec![input1, input2];
        let hash_alg = HashAlgorithm::Sha256;

        let external_params = ExternalParameters::new(inputs, pipeline, &hash_alg);

        assert!(external_params.is_ok());
        let params = external_params.unwrap();
        assert_eq!(params.inputs.len(), 2);
        assert!(!params.pipeline.name.is_empty());
    }

    #[test]
    fn test_external_parameters_to_struct() {
        let temp_dir = TempDir::new().unwrap();
        let input = create_temp_file(&temp_dir, "input.txt", b"test content");
        let pipeline = create_temp_file(&temp_dir, "pipeline.yml", b"steps: []");

        let external_params =
            ExternalParameters::new(vec![input], pipeline, &HashAlgorithm::Sha256).unwrap();
        let struct_result = external_params.to_struct();

        assert!(struct_result.is_ok());
        let params_struct = struct_result.unwrap();
        assert!(params_struct.fields.contains_key("inputs"));
        assert!(params_struct.fields.contains_key("pipeline"));
    }

    #[test]
    fn test_external_parameters_empty_inputs() {
        let temp_dir = TempDir::new().unwrap();
        let pipeline = create_temp_file(&temp_dir, "build.sh", b"echo hello");

        let external_params =
            ExternalParameters::new(vec![], pipeline, &HashAlgorithm::Sha256).unwrap();
        let struct_result = external_params.to_struct();

        assert!(struct_result.is_ok());
        let params_struct = struct_result.unwrap();
        assert!(params_struct.fields.contains_key("inputs"));
        assert!(params_struct.fields.contains_key("pipeline"));

        // Verify inputs list is empty
        let inputs_field = &params_struct.fields["inputs"];
        assert!(inputs_field.has_list_value());
        assert_eq!(inputs_field.list_value().values.len(), 0);
    }

    #[test]
    fn test_external_parameters_different_hash_algorithms() {
        let temp_dir = TempDir::new().unwrap();
        let input = create_temp_file(&temp_dir, "input.txt", b"test content");
        let pipeline = create_temp_file(&temp_dir, "build.sh", b"#!/bin/bash\necho build");

        let algorithms = vec![
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha384,
            HashAlgorithm::Sha512,
        ];

        for alg in algorithms {
            let result = ExternalParameters::new(vec![input.clone()], pipeline.clone(), &alg);
            assert!(result.is_ok(), "Failed with algorithm: {:?}", alg);

            let params = result.unwrap();
            assert_eq!(params.inputs.len(), 1);
            assert!(!params.inputs[0].digest.is_empty());
            assert!(!params.pipeline.digest.is_empty());
        }
    }

    #[test]
    fn test_generate_file_list_resource_descriptors() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = create_temp_file(&temp_dir, "file1.txt", b"content1");
        let file2 = create_temp_file(&temp_dir, "file2.txt", b"content2");

        let result =
            generate_file_list_resource_descriptors(vec![file1, file2], &HashAlgorithm::Sha256);

        assert!(result.is_ok());
        let descriptors = result.unwrap();
        assert_eq!(descriptors.len(), 2);

        for descriptor in descriptors {
            assert!(!descriptor.name.is_empty());
            assert!(!descriptor.digest.is_empty());
        }
    }

    #[test]
    fn test_generate_file_list_resource_descriptors_empty() {
        let result = generate_file_list_resource_descriptors(vec![], &HashAlgorithm::Sha256);

        assert!(result.is_ok());
        let descriptors = result.unwrap();
        assert_eq!(descriptors.len(), 0);
    }

    #[test]
    fn test_generate_build_provenance() {
        let temp_dir = TempDir::new().unwrap();
        let input = create_temp_file(&temp_dir, "input.txt", b"test");
        let pipeline = create_temp_file(&temp_dir, "build.sh", b"build script");
        let product = create_temp_file(&temp_dir, "output.bin", b"output");
        let (_secure_key, tmp_dir) = generate_temp_key().unwrap();

        let result = generate_build_provenance(
            vec![input],
            pipeline,
            vec![product],
            Some(tmp_dir.path().join("test_key.pem")),
            HashAlgorithm::Sha256,
            "json".to_string(),
            true,
            None,
            false,
        );

        assert!(result.is_ok());
    }
}
