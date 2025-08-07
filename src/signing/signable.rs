use crate::error::Result;

use std::path::PathBuf;
use atlas_c2pa_lib::cose::HashAlgorithm;

pub trait Signable {
    fn sign(&mut self, key_path: PathBuf, hash_alg: HashAlgorithm) -> Result<()>;
}
