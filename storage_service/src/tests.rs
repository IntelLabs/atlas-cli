#[cfg(test)]
mod tests {
    use crate::LogLeaf;
    use crate::MerkleTree;
    use crate::hash_string;
    use crate::sign_data;
    use ring::signature::Ed25519KeyPair;

    #[actix_web::test]
    async fn test_hashing() {
        // Test hash consistency
        let data = "test data";
        let hash1 = hash_string(data);
        let hash2 = hash_string(data);

        // Same input should produce same hash
        assert_eq!(hash1, hash2);

        // Different inputs should produce different hashes
        let hash3 = hash_string("different data");
        assert_ne!(hash1, hash3);
    }

    #[actix_web::test]
    async fn test_signing() {
        // Generate a test key pair
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate key");
        let key_pair =
            Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).expect("Failed to parse key");

        // Sign some data
        let data = "test data";
        let signature = sign_data(&key_pair, &data.as_bytes());
        // Signature should not be empty
        assert!(!signature.is_empty());
    }

    #[actix_web::test]
    async fn test_merkle_proof_simple() {
        // Create a tree with just 2 leaves for clarity
        let mut tree = MerkleTree::new();

        // Use predictable data
        let leaf1 = LogLeaf {
            manifest_id: "leaf1".to_string(),
            hash: "hash1".to_string(), // Use simple strings for nww
            sequence_number: 1,
            timestamp: chrono::Utc::now(),
        };

        let leaf2 = LogLeaf {
            manifest_id: "leaf2".to_string(),
            hash: "hash2".to_string(),
            sequence_number: 2,
            timestamp: chrono::Utc::now(),
        };

        // Calculate what the expected root hash would be
        let expected_root_hash = hash_string(&format!("hash1hash2"));

        // Add leaves to the tree
        tree.add_leaf(leaf1);
        tree.add_leaf(leaf2);

        // Verify the root hash matches our expectation
        assert_eq!(tree.root_hash.as_ref().unwrap(), &expected_root_hash);

        // Generate a proof for leaf1
        let proof = tree.generate_inclusion_proof("leaf1").unwrap();

        // Verify proof elements
        assert_eq!(proof.manifest_id, "leaf1");
        assert_eq!(proof.leaf_hash, "hash1");
        assert_eq!(proof.merkle_path.len(), 1);
        assert_eq!(proof.merkle_path[0], "hash2"); // The sibling hash
        assert_eq!(proof.root_hash, expected_root_hash);

        // Verify the proof is valid using the tree method
        assert!(tree.verify_inclusion_proof(&proof));

        // Manual verification steps
        let manual_hash = hash_string(&format!("hash1hash2"));
        assert_eq!(manual_hash, proof.root_hash);
    }
}
