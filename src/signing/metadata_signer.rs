use crate::error::Result;

pub trait MetadataSigner {
    fn sign(&self) -> Result<Vec<u8>>;
}
