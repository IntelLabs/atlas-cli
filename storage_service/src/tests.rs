#[cfg(test)]
mod tests {
    use crate::hash::{hash_sha384, hash_with_algorithm, HashAlgorithm};
    use crate::sign_data;
    use crate::merkle_tree::{LogLeaf, MerkleTree};
    use ring::signature::Ed25519KeyPair;
    use base64::{Engine as _, engine::general_purpose};
    use chrono::Utc;

    // Helper function to hash a string
    fn hash_string(data: &str) -> String {
        let hash_bytes = hash_sha384(data.as_bytes());
        general_purpose::STANDARD.encode(&hash_bytes)
    }

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
        
        // Test that we're using SHA384 (48 bytes = 64 base64 chas)
        let raw_hash = hash_sha384(data.as_bytes());
        assert_eq!(raw_hash.len(), 48); // SHA384 produces 48 bytes
    }

    #[actix_web::test]
    async fn test_signing() {
        // Generate a test key pair
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate key");
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).expect("Failed to parse key");
        
        // Sign some data
        let data = "test data";
        let signature = sign_data(&key_pair, data.as_bytes());
        
        // Signature should not be empty
        assert!(!signature.is_empty());
        
        // Ed25519 signatures are 64 bytes, which is 88 chars in base64 (including padding!!)
        let decoded = general_purpose::STANDARD.decode(&signature).unwrap();
        assert_eq!(decoded.len(), 64);
    }

    #[actix_web::test]
    async fn test_merkle_proof_simple() {
        // Create a tree with just 2 leaves for clarity
        let mut tree = MerkleTree::new();
        let now = Utc::now();
        
        // Use LogLeaf::new constructor
        let leaf1 = LogLeaf::new(
            "content_hash_1".to_string(),
            "manifest_1".to_string(),
            1,
            now,
        );
        
        let leaf2 = LogLeaf::new(
            "content_hash_2".to_string(),
            "manifest_2".to_string(),
            2,
            now,
        );
        
        // Add leaves to the tree
        tree.add_leaf(leaf1.clone());
        tree.add_leaf(leaf2.clone());
        
        // Verify we have a root hash
        assert!(tree.root_hash().is_some());
        
        // Generate a proof for manifest_1
        let proof = tree.generate_inclusion_proof("manifest_1").unwrap();
        
        // Verify proof elements
        assert_eq!(proof.manifest_id, "manifest_1");
        assert_eq!(proof.leaf_index, 0);
        assert_eq!(proof.merkle_path.len(), 1); // Should have one sibling
        assert_eq!(proof.tree_size, 2);
        
        // Verify the proof is valid using the tree method
        assert!(tree.verify_inclusion_proof(&proof));
        
        // Test proof for second leaf
        let proof2 = tree.generate_inclusion_proof("manifest_2").unwrap();
        assert_eq!(proof2.manifest_id, "manifest_2");
        assert_eq!(proof2.leaf_index, 1);
        assert!(tree.verify_inclusion_proof(&proof2));
    }

    #[actix_web::test]
    async fn test_merkle_tree_multiple_leaves() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();
        
        // Add 5 leaves
        for i in 0..5 {
            let leaf = LogLeaf::new(
                format!("content_hash_{}", i),
                format!("manifest_{}", i),
                i as u64 + 1,
                now,
            );
            tree.add_leaf(leaf);
        }
        
        // Verify tree size
        assert_eq!(tree.size(), 5);
        
        // Print tree structure for debugging
        println!("Tree has {} leaves, root: {:?}", tree.size(), tree.root_hash());
        
        // Generate and verify proofs for all leaves
        for i in 0..5 {
            let manifest_id = format!("manifest_{}", i);
            let proof = tree.generate_inclusion_proof(&manifest_id).unwrap();
            
            // Debug output
            println!("Proof for manifest_{}: leaf_index={}, tree_size={}, path_len={}", 
                     i, proof.leaf_index, proof.tree_size, proof.merkle_path.len());
            println!("  Proof root: {}", &proof.root_hash[..20]); // First 20 chars
            println!("  Tree root:  {}", &tree.root_hash().unwrap()[..20]);
            
            // Check basic proof properties
            assert_eq!(proof.manifest_id, manifest_id);
            assert_eq!(proof.tree_size, 5);
            assert_eq!(proof.leaf_index, i);
            
            // Verify the proof
            let is_valid = tree.verify_inclusion_proof(&proof);
            if !is_valid {
                println!("  Verification FAILED!");
                println!("  Path: {:?}", proof.merkle_path.iter().map(|h| &h[..20]).collect::<Vec<_>>());
            }
            assert!(is_valid, "Proof verification failed for manifest_{}", i);
        }
    }

    #[actix_web::test]
    async fn test_consistency_proof() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();
        
        // Build tree incrementally
        let mut roots = Vec::new();
        
        for i in 0..8 {
            let leaf = LogLeaf::new(
                format!("content_hash_{}", i),
                format!("manifest_{}", i),
                i as u64 + 1,
                now,
            );
            tree.add_leaf(leaf);
            
            if let Some(root) = tree.root_hash() {
                roots.push(root.clone());
            }
        }
        
        // Test consistency between different sizes
        for old_size in 1..7 {
            for new_size in (old_size + 1)..=8 {
                let proof = tree.generate_consistency_proof(old_size, new_size).unwrap();
                
                // Verify the proof contains expected roots
                assert_eq!(proof.old_root, roots[old_size - 1]);
                assert_eq!(proof.new_root, roots[new_size - 1]);
                
                // Verify the proof is valid
                let is_valid = tree.verify_consistency_proof(
                    old_size,
                    new_size,
                    &proof.old_root,
                    &proof.new_root,
                    &proof.proof_hashes,
                );
                assert!(is_valid);
            }
        }
    }

    #[actix_web::test]
    async fn test_hash_algorithms() {
        let data = b"test data";
        
        // Test SHA256
        let sha256_hash = hash_with_algorithm(data, &HashAlgorithm::Sha256);
        assert_eq!(sha256_hash.len(), 32); // SHA256 produces 32 bytes
        
        // Test SHA384
        let sha384_hash = hash_with_algorithm(data, &HashAlgorithm::Sha384);
        assert_eq!(sha384_hash.len(), 48); // SHA384 produces 48 bytes
        
        // Verify they produce different hashes
        assert_ne!(sha256_hash, sha384_hash);
    }

    #[actix_web::test]
    async fn test_leaf_lookup_methods() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();
        
        // Add some leaves
        for i in 0..3 {
            let leaf = LogLeaf::new(
                format!("content_hash_{}", i),
                format!("manifest_{}", i),
                i as u64 + 10, // sequence numbers 10, 11, 12
                now,
            );
            tree.add_leaf(leaf);
        }
        
        // Test get_leaf_by_manifest_id
        let leaf = tree.get_leaf_by_manifest_id("manifest_1").unwrap();
        assert_eq!(leaf.metadata.manifest_id, "manifest_1");
        assert_eq!(leaf.metadata.sequence_number, 11);
        
        // Test get_leaf_by_sequence
        let leaf = tree.get_leaf_by_sequence(12).unwrap();
        assert_eq!(leaf.metadata.manifest_id, "manifest_2");
        assert_eq!(leaf.metadata.sequence_number, 12);
        
        // Test non-existent lookups
        assert!(tree.get_leaf_by_manifest_id("manifest_999").is_none());
        assert!(tree.get_leaf_by_sequence(999).is_none());
    }

    #[actix_web::test]
    async fn test_inclusion_proof_verification_methods() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();
        
        // Add a leaf
        let leaf = LogLeaf::new(
            "content_hash".to_string(),
            "manifest_id".to_string(),
            1,
            now,
        );
        tree.add_leaf(leaf);
        
        // Generate proof
        let proof = tree.generate_inclusion_proof("manifest_id").unwrap();
        
        // Test verify_against_root method
        assert!(proof.verify_against_root(&proof.root_hash));
        assert!(!proof.verify_against_root("wrong_root_hash"));
        
        // Test describe method
        let description = proof.describe();
        assert!(description.contains("manifest_id"));
        assert!(description.contains("index 0"));
        assert!(description.contains("tree of size 1"));
    }

    #[actix_web::test]
    async fn test_consistency_proof_methods() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();
        
        // Add leaves
        for i in 0..4 {
            tree.add_leaf(LogLeaf::new(
                format!("hash_{}", i),
                format!("id_{}", i),
                i as u64,
                now,
            ));
        }
        
        // Generate consistency proof
        let proof = tree.generate_consistency_proof(2, 4).unwrap();
        
        // Test verify method
        assert!(proof.verify(&proof.old_root, &proof.new_root));
        assert!(!proof.verify("wrong_old_root", &proof.new_root));
        assert!(!proof.verify(&proof.old_root, "wrong_new_root"));
        
        // Test describe method
        let description = proof.describe();
        assert!(description.contains("tree size 2 to 4"));
        assert!(description.contains("proof elements:"));
    }
}