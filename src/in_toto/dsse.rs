use crate::error::{Error, Result};

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
	    payload: payload.clone(),
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
