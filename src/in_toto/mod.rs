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

pub fn json_to_struct_proto(json_str: &str) -> Result<Struct> {
    let msg_struct = parse_from_str::<Struct>(&json_str).map_err(|e| {
        Error::Serialization(format!("Failed to serialize in-toto statement: {}", e))
    })?;

    Ok(msg_struct)
}

pub fn make_minimal_resource_descriptor(name: &str, alg: &str, digest: &str) -> ResourceDescriptor {
    let digest_set = HashMap::from([(alg.to_string(), digest.to_string())]);

    let mut rd = ResourceDescriptor::new();
    rd.name = name.to_string();
    rd.digest = digest_set;

    rd
}

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
