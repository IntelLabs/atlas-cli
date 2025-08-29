use crate::error::{Error, Result};
use crate::signing;
use crate::signing::signable::Signable;

use atlas_c2pa_lib::cose::HashAlgorithm;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Signature {
    #[serde_as(as = "serde_with::base64::Base64")]
    sig: Vec<u8>,
    keyid: String,
}

impl Signature {
    fn new(sig: Vec<u8>, keyid: String) -> Self {
        Self {
            sig: sig,
            keyid: keyid,
        }
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Envelope {
    #[serde_as(as = "serde_with::base64::Base64")]
    payload: Vec<u8>,
    payload_type: String,
    signatures: Vec<Signature>,
}

impl Envelope {
    pub fn new(payload: &Vec<u8>, payload_type: String) -> Self {
        Self {
            payload: payload,
            payload_type: payload_type,
            signatures: vec![],
        }
    }

    pub fn add_signature(&mut self, sig: Vec<u8>, keyid: String) -> Result<()> {
        if sig.is_empty() {
            return Err(Error::Signing("DSSE signature cannot be empty".to_string()));
        }

        let sig_struct = Signature::new(sig, keyid);
        self.signatures.push(sig_struct);

        Ok(())
    }
}

impl Signable for Envelope {
    fn sign(&mut self, key_path: PathBuf, hash_alg: HashAlgorithm) -> Result<()> {
        let private_key = signing::load_private_key(&key_path)?;

        // DSSE requires that payload_type and payload be signed
        let mut data_to_sign: Vec<u8> = Vec::new();
        data_to_sign.extend_from_slice(&self.payload_type.clone().into_bytes());

        // DSSE requires payload to be JSON bytes
        data_to_sign.extend_from_slice(&self.payload);

        // Use the signing module with the specified algorithm
        let signature = signing::sign_data_with_algorithm(&data_to_sign, &private_key, &hash_alg)?;

        self.add_signature(signature, "".to_string()) // keyid is optional
    }
}
