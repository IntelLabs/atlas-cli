use ring::digest::{Context, SHA256, SHA384};
use base64::{Engine as _, engine::general_purpose};
use std::fmt::Debug;

/// Trait for hashing functionality
pub trait Hasher: Send + Sync + Debug {
    fn hash(&self, data: &[u8]) -> String;
}

/// Default SHA384 hasher implementation
#[derive(Clone, Debug)]
pub struct DefaultHasher;

impl Hasher for DefaultHasher {
    fn hash(&self, data: &[u8]) -> String {
        let mut context = Context::new(&SHA384);
        context.update(data);
        let digest = context.finish();
        general_purpose::STANDARD.encode(digest.as_ref())
    }
}

/// SHA256 hasher implementation (if needed for compatibility)
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Sha256Hasher;

#[allow(dead_code)]
impl Hasher for Sha256Hasher {
    fn hash(&self, data: &[u8]) -> String {
        let mut context = Context::new(&SHA256);
        context.update(data);
        let digest = context.finish();
        general_purpose::STANDARD.encode(digest.as_ref())
    }
}