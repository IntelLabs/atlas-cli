//! # SLSA Build Provenance Structure Generators
//!
//! This module provides functions for generating the core data structures used in
//! SLSA (Supply-chain Levels for Software Artifacts) build provenance attestations.
//! These generators create the standardized components required by the SLSA v1
//! specification for documenting build processes and their outputs.
//!
//! ## Structure Overview
//!
//! SLSA build provenance consists of several key components:
//! - **BuildDefinition**: Describes what was built and how
//! - **Builder**: Identifies the build service or system
//! - **BuildMetadata**: Contains timing and execution metadata  
//! - **RunDetails**: Combines builder info with execution metadata
//! - **Provenance**: The complete provenance document
//!
//! ## Usage Patterns
//!
//! These generators are typically used in sequence to build up a complete
//! provenance attestation:
//!
//! ```no_run
//! use atlas_cli::slsa::cli::ATLAS_CLI_BUILDER_ID;
//! use atlas_cli::slsa::generators::*;
//! use protobuf::well_known_types::struct_::Struct;
//! use protobuf::well_known_types::timestamp::Timestamp;
//!
//! // 1. Create build definition
//! let external_params = Struct::new();
//! let build_def = make_build_definition_v1("tool:v1", &external_params, None, None);
//!
//! // 2. Create builder identity
//! let builder = make_builder_v1(ATLAS_CLI_BUILDER_ID, None, None);
//!
//! // 3. Create build metadata
//! let metadata = make_build_metadata_v1("build-123", None, Some(&Timestamp::now()));
//!
//! // 4. Create run details
//! let run_details = make_run_details_v1(&builder, Some(&metadata), None);
//!
//! // 5. Generate final provenance
//! let provenance = generate_build_provenance_v1(&build_def, &run_details);
//! ```

use in_toto_attestation::predicates::provenance::v1::provenance;
use in_toto_attestation::v1::resource_descriptor::ResourceDescriptor;
use protobuf::MessageField;
use protobuf::well_known_types::struct_::Struct;
use protobuf::well_known_types::timestamp::Timestamp;
use std::collections::HashMap;

/// Creates a SLSA v1 BuildDefinition structure.
///
/// The BuildDefinition describes what was built and how it was built, forming the core
/// of SLSA build provenance. It includes the build type, external parameters that
/// influenced the build, and optionally internal parameters and resolved dependencies.
///
/// # Arguments
///
/// * `build_type` - A URI or identifier describing the type of build (e.g., "tool:version")
/// * `external_parameters` - Build parameters visible to external observers (inputs, configs)
/// * `internal_parameters` - Optional internal build parameters (may be None for Build L1/L2)
/// * `resolved_dependencies` - Optional dependency list (required for Build L3)
///
/// # Returns
///
/// A `provenance::BuildDefinition` structure containing the build specification.
///
/// # SLSA Requirements
///
/// - **Build L1**: Requires `build_type` and `external_parameters`
/// - **Build L2**: Same as L1, may include `internal_parameters`
/// - **Build L3**: Additionally requires `resolved_dependencies`
///
/// # Examples
///
/// ## Basic Build Definition (Build L1)
///
/// ```
/// use atlas_cli::slsa::generators::make_build_definition_v1;
/// use protobuf::well_known_types::struct_::Struct;
///
/// let mut external_params = Struct::new();
/// // ... populate external_params with build inputs
///
/// let build_def = make_build_definition_v1(
///     "atlas-cli:1.0.0",
///     &external_params,
///     None,  // no internal parameters for L1
///     None,  // no resolved dependencies for L1
/// );
///
/// assert_eq!(build_def.build_type, "atlas-cli:1.0.0");
/// ```
pub fn make_build_definition_v1(
    build_type: &str,
    external_parameters: &Struct,
    internal_parameters: Option<&Struct>,
    resolved_dependencies: Option<&[ResourceDescriptor]>,
) -> provenance::BuildDefinition {
    let mut build_def = provenance::BuildDefinition::new();
    build_def.build_type = build_type.to_string();
    build_def.external_parameters = MessageField::some(external_parameters.clone());

    // this field is optional for all SLSA Build levels
    if internal_parameters.is_some() {
        build_def.internal_parameters = MessageField::some(internal_parameters.unwrap().clone());
    }

    // this field is only required for SLSA Build L3
    if resolved_dependencies.is_some() {
        build_def.resolved_dependencies = resolved_dependencies.unwrap().to_vec();
    }

    build_def
}

/// Creates a SLSA v1 Builder structure.
///
/// The Builder identifies the build service or system that performed the build.
/// This provides accountability and enables verification of the build environment's
/// trustworthiness. The builder identity is crucial for SLSA security guarantees.
///
/// # Arguments
///
/// * `id` - Unique identifier for the builder (typically a URI)
/// * `builder_dependencies` - Optional list of dependencies used by the builder itself
/// * `version` - Optional version information about the builder
///
/// # Returns
///
/// A `provenance::Builder` structure identifying the build system.
///
/// # Builder Identity
///
/// The `id` should be a globally unique identifier, typically:
/// - A URI pointing to the builder service (e.g., GitHub Actions workflow)
/// - A tool identifier with version (e.g., "atlas-cli:1.0.0")
/// - A hosted service endpoint
///
/// # Examples
///
/// ## Basic Builder (Minimal Configuration)
///
/// ```
/// use atlas_cli::slsa::cli::ATLAS_CLI_BUILDER_ID;
/// use atlas_cli::slsa::generators::make_builder_v1;
///
/// let builder = make_builder_v1(ATLAS_CLI_BUILDER_ID, None, None);
/// assert_eq!(builder.id, "https://github.com/IntelLabs/atlas-cli");
/// assert!(builder.builder_dependencies.is_empty());
/// assert!(builder.version.is_empty());
/// ```
pub fn make_builder_v1(
    id: &str,
    builder_dependencies: Option<&[ResourceDescriptor]>,
    version: Option<&HashMap<String, String>>,
) -> provenance::Builder {
    let mut builder = provenance::Builder::new();
    builder.id = id.to_string();

    // this field is always optional
    if builder_dependencies.is_some() {
        builder.builder_dependencies = builder_dependencies.unwrap().to_vec();
    }

    // this field is always optional
    if version.is_some() {
        builder.version = version.unwrap().clone();
    }

    builder
}

/// Creates a SLSA v1 BuildMetadata structure.
///
/// BuildMetadata contains information about the execution of the build process,
/// including timing information and unique identifiers. This metadata helps
/// with debugging, auditing, and correlating build events.
///
/// # Arguments
///
/// * `invocation_id` - Unique identifier for this specific build invocation
/// * `started_on` - Optional timestamp when the build started
/// * `finished_on` - Optional timestamp when the build completed
///
/// # Returns
///
/// A `provenance::BuildMetadata` structure containing execution metadata.
///
/// # Examples
///
/// ## Basic Metadata (Minimal Information)
///
/// ```
/// use atlas_cli::slsa::generators::make_build_metadata_v1;
///
/// let metadata = make_build_metadata_v1("build-12345", None, None);
/// assert_eq!(metadata.invocation_id, "build-12345");
/// assert!(metadata.started_on.is_none());
/// assert!(metadata.finished_on.is_none());
/// ```
///
/// ## Complete Metadata with Timestamps
///
/// ```
/// use atlas_cli::slsa::generators::make_build_metadata_v1;
/// use protobuf::well_known_types::timestamp::Timestamp;
///
/// let start_time = Timestamp::now();
/// let end_time = Timestamp::now();
///
/// let metadata = make_build_metadata_v1(
///     "uuid-abc-123-def",
///     Some(&start_time),
///     Some(&end_time),
/// );
///
/// assert_eq!(metadata.invocation_id, "uuid-abc-123-def");
/// assert!(metadata.started_on.is_some());
/// assert!(metadata.finished_on.is_some());
/// ```
pub fn make_build_metadata_v1(
    invocation_id: &str,
    started_on: Option<&Timestamp>,
    finished_on: Option<&Timestamp>,
) -> provenance::BuildMetadata {
    let mut build_metadata = provenance::BuildMetadata::new();
    build_metadata.invocation_id = invocation_id.to_string();

    if started_on.is_some() {
        build_metadata.started_on = MessageField::some(started_on.unwrap().clone());
    }

    if finished_on.is_some() {
        build_metadata.finished_on = MessageField::some(finished_on.unwrap().clone());
    }

    build_metadata
}

/// Creates a SLSA v1 RunDetails structure.
///
/// RunDetails combines builder identification with build execution metadata and
/// any byproducts produced during the build. This structure provides the runtime
/// context for the build provenance and links the builder identity to the specific
/// build execution.
///
/// # Arguments
///
/// * `builder` - The Builder that performed this build
/// * `metadata` - Optional BuildMetadata about the execution
/// * `byproducts` - Optional additional artifacts produced during the build
///
/// # Returns
///
/// A `provenance::RunDetails` structure containing execution context.
///
/// # Examples
///
/// ## Basic RunDetails (Required Fields Only)
///
/// ```
/// use atlas_cli::slsa::cli::ATLAS_CLI_BUILDER_ID;
/// use atlas_cli::slsa::generators::{make_builder_v1, make_run_details_v1};
///
/// let builder = make_builder_v1(ATLAS_CLI_BUILDER_ID, None, None);
/// let run_details = make_run_details_v1(&builder, None, None);
///
/// assert_eq!(run_details.builder.as_ref().unwrap().id, ATLAS_CLI_BUILDER_ID);
/// assert!(run_details.metadata.is_none());
/// assert!(run_details.byproducts.is_empty());
/// ```
///
/// ## Complete RunDetails with Metadata
///
/// ```
/// use atlas_cli::slsa::cli::ATLAS_CLI_BUILDER_ID;
/// use atlas_cli::slsa::generators::{
///     make_builder_v1, make_build_metadata_v1, make_run_details_v1
/// };
/// use protobuf::well_known_types::timestamp::Timestamp;
///
/// let builder = make_builder_v1(ATLAS_CLI_BUILDER_ID, None, None);
/// let metadata = make_build_metadata_v1("build-456", Some(&Timestamp::now()), None);
/// let run_details = make_run_details_v1(&builder, Some(&metadata), None);
///
/// assert!(run_details.metadata.is_some());
/// assert_eq!(
///     run_details.metadata.as_ref().unwrap().invocation_id,
///     "build-456"
/// );
/// ```
pub fn make_run_details_v1(
    builder: &provenance::Builder,
    metadata: Option<&provenance::BuildMetadata>,
    byproducts: Option<&[ResourceDescriptor]>,
) -> provenance::RunDetails {
    let mut run_details = provenance::RunDetails::new();
    run_details.builder = MessageField::some(builder.clone());

    // this field is optional for all SLSA Build levels
    if metadata.is_some() {
        run_details.metadata = MessageField::some(metadata.unwrap().clone());
    }

    // this field is optional for all SLSA Build levels
    if byproducts.is_some() {
        run_details.byproducts = byproducts.unwrap().to_vec();
    }

    run_details
}

/// Generates a complete SLSA v1 build provenance structure.
///
/// This function creates the final SLSA build provenance by combining a BuildDefinition
/// (describing what was built) with RunDetails (describing how it was built). The
/// resulting Provenance structure contains all the information needed to verify
/// the integrity and authenticity of the build process.
///
/// # Arguments
///
/// * `build_definition` - BuildDefinition describing the build specification
/// * `run_details` - RunDetails describing the build execution context
///
/// # Returns
///
/// A complete `in_toto_attestation::provenance::Provenance` structure ready for attestation.
///
/// # SLSA Provenance Structure
///
/// The generated provenance follows the SLSA v1 specification and contains:
/// - **BuildDefinition**: What was built (inputs, build type, parameters)
/// - **RunDetails**: How it was built (builder, metadata, timing)
///
/// This provenance can then be embedded in an in-toto attestation statement
/// and cryptographically signed to create a verifiable build attestation.
///
/// # Examples
///
/// ## Complete Provenance Generation Workflow
///
/// ```
/// use atlas_cli::slsa::cli::ATLAS_CLI_BUILDER_ID;
/// use atlas_cli::slsa::generators::*;
/// use protobuf::well_known_types::struct_::Struct;
/// use protobuf::well_known_types::timestamp::Timestamp;
///
/// // 1. Create build definition
/// let external_params = Struct::new();
/// let build_def = make_build_definition_v1(
///     "atlas-cli:1.0.0",
///     &external_params,
///     None,
///     None,
/// );
///
/// // 2. Create builder and metadata
/// let builder = make_builder_v1(ATLAS_CLI_BUILDER_ID, None, None);
/// let metadata = make_build_metadata_v1("build-789", Some(&Timestamp::now()), None);
/// let run_details = make_run_details_v1(&builder, Some(&metadata), None);
///
/// // 3. Generate final provenance
/// let provenance = generate_build_provenance_v1(&build_def, &run_details);
///
/// // Verify the provenance structure
/// assert!(provenance.build_definition.is_some());
/// assert!(provenance.run_details.is_some());
/// assert_eq!(
///     provenance.build_definition.as_ref().unwrap().build_type,
///     "atlas-cli:1.0.0"
/// );
/// ```
///
/// ## Integration with Attestation Generation
///
/// ```no_run
/// use atlas_cli::slsa::cli::ATLAS_CLI_BUILDER_ID;
/// use atlas_cli::slsa::generators::*;
/// use atlas_cli::in_toto;
/// use protobuf::well_known_types::struct_::Struct;
/// use in_toto_attestation::to_struct;
/// # use atlas_cli::error::Result;
/// # use atlas_c2pa_lib::cose::HashAlgorithm;
/// # use std::path::PathBuf;
/// # use in_toto_attestation::v1::resource_descriptor::ResourceDescriptor;
///
/// # fn example() -> Result<()> {
/// // Generate provenance
/// let build_def = make_build_definition_v1("tool:1.0", &Struct::new(), None, None);
/// let builder = make_builder_v1(ATLAS_CLI_BUILDER_ID, None, None);
/// let run_details = make_run_details_v1(&builder, None, None);
/// let provenance = generate_build_provenance_v1(&build_def, &run_details);
///
/// // Convert to protobuf Struct for attestation
/// let provenance_struct = to_struct(&provenance)
///     .map_err(|e| atlas_cli::error::Error::Serialization(e.to_string()))?;
///
/// // Generate signed attestation
/// let subjects = vec![ResourceDescriptor::new()];
/// let envelope = in_toto::generate_signed_statement_v1(
///     &subjects,
///     atlas_cli::slsa::BUILD_PROVENANCE_PREDICATE_TYPE_V1,
///     &provenance_struct,
///     PathBuf::from("key.pem"),
///     HashAlgorithm::Sha384,
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// # Specification Compliance
///
/// The generated provenance structure complies with:
/// - [SLSA v1 Build Provenance specification](https://slsa.dev/spec/v1/provenance)
/// - [in-toto Attestation Framework](https://github.com/in-toto/attestation)
pub fn generate_build_provenance_v1(
    build_definition: &provenance::BuildDefinition,
    run_details: &provenance::RunDetails,
) -> provenance::Provenance {
    let mut provenance = provenance::Provenance::new();
    provenance.build_definition = MessageField::some(build_definition.clone());
    provenance.run_details = MessageField::some(run_details.clone());

    provenance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slsa::cli::ATLAS_CLI_BUILDER_ID;

    #[test]
    fn test_make_build_definition_v1_minimal() {
        let external_params = Struct::new();
        let build_def =
            make_build_definition_v1("test-builder:1.0.0", &external_params, None, None);

        assert_eq!(build_def.build_type, "test-builder:1.0.0");
        assert!(build_def.external_parameters.is_some());
        assert!(build_def.internal_parameters.is_none());
        assert!(build_def.resolved_dependencies.is_empty());
    }

    #[test]
    fn test_make_build_definition_v1_complete() {
        let external_params = Struct::new();
        let internal_params = Struct::new();
        let dependencies = vec![ResourceDescriptor::new(), ResourceDescriptor::new()];

        let build_def = make_build_definition_v1(
            "test-builder:1.0.0",
            &external_params,
            Some(&internal_params),
            Some(&dependencies),
        );

        assert_eq!(build_def.build_type, "test-builder:1.0.0");
        assert!(build_def.external_parameters.is_some());
        assert!(build_def.internal_parameters.is_some());
        assert_eq!(build_def.resolved_dependencies.len(), 2);
    }

    #[test]
    fn test_make_builder_v1_minimal() {
        let builder = make_builder_v1("test-builder-id", None, None);

        assert_eq!(builder.id, "test-builder-id");
        assert!(builder.builder_dependencies.is_empty());
        assert!(builder.version.is_empty());
    }

    #[test]
    fn test_make_builder_v1_complete() {
        let dependencies = vec![ResourceDescriptor::new()];
        let mut version_info = HashMap::new();
        version_info.insert("version".to_string(), "2.0.0".to_string());

        let builder = make_builder_v1("complete-builder", Some(&dependencies), Some(&version_info));

        assert_eq!(builder.id, "complete-builder");
        assert_eq!(builder.builder_dependencies.len(), 1);
        assert_eq!(builder.version.len(), 1);
        assert_eq!(builder.version.get("version"), Some(&"2.0.0".to_string()));
    }

    #[test]
    fn test_make_build_metadata_v1_minimal() {
        let metadata = make_build_metadata_v1("test-invocation-123", None, None);

        assert_eq!(metadata.invocation_id, "test-invocation-123");
        assert!(metadata.started_on.is_none());
        assert!(metadata.finished_on.is_none());
    }

    #[test]
    fn test_make_build_metadata_v1_complete() {
        let start_time = Timestamp::now();
        let end_time = Timestamp::now();
        let metadata = make_build_metadata_v1(
            "complete-build-uuid-456",
            Some(&start_time),
            Some(&end_time),
        );

        assert_eq!(metadata.invocation_id, "complete-build-uuid-456");
        assert!(metadata.started_on.is_some());
        assert!(metadata.finished_on.is_some());
    }

    #[test]
    fn test_make_run_details_v1_minimal() {
        let builder = make_builder_v1("test-builder", None, None);
        let run_details = make_run_details_v1(&builder, None, None);

        assert!(run_details.builder.is_some());
        assert_eq!(run_details.builder.as_ref().unwrap().id, "test-builder");
        assert!(run_details.metadata.is_none());
        assert!(run_details.byproducts.is_empty());
    }

    #[test]
    fn test_make_run_details_v1_complete() {
        let builder = make_builder_v1("complete-builder", None, None);
        let metadata = make_build_metadata_v1("complete-invocation", None, None);
        let byproducts = vec![ResourceDescriptor::new()];

        let run_details = make_run_details_v1(&builder, Some(&metadata), Some(&byproducts));

        assert!(run_details.builder.is_some());
        assert_eq!(run_details.builder.as_ref().unwrap().id, "complete-builder");
        assert!(run_details.metadata.is_some());
        assert_eq!(
            run_details.metadata.as_ref().unwrap().invocation_id,
            "complete-invocation"
        );
        assert_eq!(run_details.byproducts.len(), 1);
    }

    #[test]
    fn test_generate_build_provenance_v1_minimal() {
        let external_params = Struct::new();
        let build_def = make_build_definition_v1("test:1.0", &external_params, None, None);
        let builder = make_builder_v1("test-builder", None, None);
        let run_details = make_run_details_v1(&builder, None, None);

        let provenance = generate_build_provenance_v1(&build_def, &run_details);

        assert!(provenance.build_definition.is_some());
        assert!(provenance.run_details.is_some());
        assert_eq!(
            provenance.build_definition.as_ref().unwrap().build_type,
            "test:1.0"
        );
        assert_eq!(
            provenance
                .run_details
                .as_ref()
                .unwrap()
                .builder
                .as_ref()
                .unwrap()
                .id,
            "test-builder"
        );
    }

    #[test]
    fn test_generate_build_provenance_v1_complete() {
        // Create a complete build definition
        let external_params = Struct::new();
        let internal_params = Struct::new();
        let dependencies = vec![ResourceDescriptor::new()];
        let build_def = make_build_definition_v1(
            "atlas-cli:1.0.0",
            &external_params,
            Some(&internal_params),
            Some(&dependencies),
        );

        // Create a complete builder
        let mut version_info = HashMap::new();
        version_info.insert("version".to_string(), "1.0.0".to_string());
        let builder_deps = vec![ResourceDescriptor::new()];
        let builder = make_builder_v1(
            ATLAS_CLI_BUILDER_ID,
            Some(&builder_deps),
            Some(&version_info),
        );

        // Create metadata with timestamps
        let start_time = Timestamp::now();
        let end_time = Timestamp::now();
        let metadata = make_build_metadata_v1("complete-build", Some(&start_time), Some(&end_time));

        // Create run details with byproducts
        let byproducts = vec![ResourceDescriptor::new(), ResourceDescriptor::new()];
        let run_details = make_run_details_v1(&builder, Some(&metadata), Some(&byproducts));

        // Generate provenance
        let provenance = generate_build_provenance_v1(&build_def, &run_details);

        // Verify provenance structure
        assert!(provenance.build_definition.is_some());
        assert!(provenance.run_details.is_some());

        // Verify build definition
        let build_def_ref = provenance.build_definition.as_ref().unwrap();
        assert_eq!(build_def_ref.build_type, "atlas-cli:1.0.0");
        assert!(build_def_ref.external_parameters.is_some());
        assert!(build_def_ref.internal_parameters.is_some());
        assert_eq!(build_def_ref.resolved_dependencies.len(), 1);

        // Verify run details
        let run_details_ref = provenance.run_details.as_ref().unwrap();
        assert!(run_details_ref.builder.is_some());
        assert!(run_details_ref.metadata.is_some());
        assert_eq!(run_details_ref.byproducts.len(), 2);

        // Verify builder details
        let builder_ref = run_details_ref.builder.as_ref().unwrap();
        assert_eq!(builder_ref.id, ATLAS_CLI_BUILDER_ID);
        assert_eq!(builder_ref.builder_dependencies.len(), 1);
        assert_eq!(builder_ref.version.len(), 1);

        // Verify metadata details
        let metadata_ref = run_details_ref.metadata.as_ref().unwrap();
        assert_eq!(metadata_ref.invocation_id, "complete-build");
        assert!(metadata_ref.started_on.is_some());
        assert!(metadata_ref.finished_on.is_some());
    }
}
