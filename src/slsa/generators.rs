use in_toto_attestation::predicates::provenance::v1::provenance;
use in_toto_attestation::v1::resource_descriptor::ResourceDescriptor;
use protobuf::MessageField;
use protobuf::well_known_types::struct_::Struct;
use protobuf::well_known_types::timestamp::Timestamp;
use std::collections::HashMap;

pub const ATLAS_CLI_BUILDER_ID: &str = "https://github.com/IntelLabs/atlas-cli";

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

pub fn generate_build_provenance_v1(
    build_definition: &provenance::BuildDefinition,
    run_details: &provenance::RunDetails,
) -> provenance::Provenance {
    let mut provenance = provenance::Provenance::new();
    provenance.build_definition = MessageField::some(build_definition.clone());
    provenance.run_details = MessageField::some(run_details.clone());

    provenance
}
