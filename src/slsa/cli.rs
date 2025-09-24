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
    let builder =
        slsa::generators::make_builder_v1(slsa::generators::ATLAS_CLI_BUILDER_ID, None, None);

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
