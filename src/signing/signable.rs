use crate::error::Result;

use atlas_c2pa_lib::cose::HashAlgorithm;
use std::path::PathBuf;

pub trait Signable {
    fn sign(&mut self, key_path: PathBuf, hash_alg: HashAlgorithm) -> Result<()>;
}
