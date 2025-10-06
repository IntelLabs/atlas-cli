use crate::error::{Error, Result};
use crate::signing;
use crate::signing::signable::Signable;
use atlas_c2pa_lib::cose::HashAlgorithm;
use atlas_c2pa_lib::manifest::Manifest;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::path::PathBuf;

impl Signable for Manifest {
    fn sign(&mut self, key_path: PathBuf, hash_alg: HashAlgorithm) -> Result<()> {
        let private_key = signing::load_private_key(&key_path)?;

        // Serialize claim to CBOR for signing
        let claim_cbor =
            serde_cbor::to_vec(&self.claim).map_err(|e| Error::Serialization(e.to_string()))?;

        // Use the signing module with the specified algorithm
        let signature = signing::sign_data_with_algorithm(&claim_cbor, &private_key, &hash_alg)?;

        // Add signature to claim
        self.claim.signature = Some(STANDARD.encode(&signature));

        Ok(())
    }
}
