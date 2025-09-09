use atlas_common::hash::{calculate_hash, calculate_hash_with_algorithm, HashAlgorithm};
use std::fmt::Debug;

/// Trait for hashing functionality
pub trait Hasher: Send + Sync + Debug {
    fn hash(&self, data: &[u8]) -> String;
}

/// Default SHA384 hasher implementation using atlas-common
#[derive(Clone, Debug)]
pub struct DefaultHasher;

impl Hasher for DefaultHasher {
    fn hash(&self, data: &[u8]) -> String {
        calculate_hash(data) // Uses atlas-common's SHA384 default
    }
}

/// SHA256 hasher implementation using atlas-common
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Sha256Hasher;

#[allow(dead_code)]
impl Hasher for Sha256Hasher {
    fn hash(&self, data: &[u8]) -> String {
        calculate_hash_with_algorithm(data, &HashAlgorithm::Sha256)
    }
}
