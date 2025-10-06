use ring::digest::{Context, SHA256, SHA384};

/// Hash data using SHA256
#[allow(dead_code)]
pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
    let mut context = Context::new(&SHA256);
    context.update(data);
    context.finish().as_ref().to_vec()
}

/// Hash data using SHA384 (default)
pub fn hash_sha384(data: &[u8]) -> Vec<u8> {
    let mut context = Context::new(&SHA384);
    context.update(data);
    context.finish().as_ref().to_vec()
}

/// Supported hash algorithms
#[allow(dead_code)]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
}

/// Hash data with specified algorithm
#[allow(dead_code)]
pub fn hash_with_algorithm(data: &[u8], algorithm: &HashAlgorithm) -> Vec<u8> {
    match algorithm {
        HashAlgorithm::Sha256 => hash_sha256(data),
        HashAlgorithm::Sha384 => hash_sha384(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = hash_sha256(data);
        assert_eq!(hash.len(), 32); // SHA256 produces 32 bytes
    }

    #[test]
    fn test_sha384() {
        let data = b"hello world";
        let hash = hash_sha384(data);
        assert_eq!(hash.len(), 48); // SHA384 produces 48 bytes
    }
}
