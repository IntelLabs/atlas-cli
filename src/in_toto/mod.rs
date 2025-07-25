use crate::error::Result;
use crate::signing;
use crate::signing::metadata_signer::MetadataSigner;

use in_toto_attestation::predicates::provenance::v1::provenance::Provenance;
use in_toto_attestation::v1::resource_descriptor::ResourceDescriptor;
use in_toto_attestation::v1::statement::Statement;

use std::path::PathBuf;
use atlas_c2pa_lib::cose::HashAlgorithm;

pub mod dsse;

pub struct DsseSigner {
    payload: Vec<u8>,
    payload_type: String,
    key_path: PathBuf,
    hash_alg: HashAlgorithm,
}

impl DsseSigner {
    pub fn new(payload: &Vec<u8>, payload_type: String, key_path: PathBuf, hash_alg: HashAlgorithm) -> Self {
	Self {
	    payload: payload.clone(),
	    payload_type: payload_type,
	    key_path: key_path,
	    hash_alg: hash_alg,
	}
    }
}

impl MetadataSigner for DsseSigner {
    fn sign(&self) -> Result<Vec<u8>> {
	let private_key = signing::load_private_key(&self.key_path)?;

	// DSSE requires that payload_type and payload be signed
	let mut data_to_sign: Vec<u8> = Vec::new();
	data_to_sign.extend_from_slice(&self.payload_type.clone().into_bytes());
	
        // DSSE requires payload to be JSON bytes
	data_to_sign.extend_from_slice(&self.payload);

        // Use the signing module with the specified algorithm
        signing::sign_data_with_algorithm(&data_to_sign, &private_key, &self.hash_alg)
    }
}
