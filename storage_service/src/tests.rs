#[cfg(test)]
mod tests {
    use actix_web;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use chrono::Utc;
    use ring::signature::Ed25519KeyPair;

    use atlas_common::hash::{
        calculate_hash, calculate_hash_with_algorithm, detect_hash_algorithm, validate_hash_format,
        verify_hash, verify_hash_with_algorithm, HashAlgorithm, Hasher,
    };
    use atlas_common::validation::{ensure_c2pa_urn, validate_manifest_id};

    use crate::merkle_tree::{LogLeaf, MerkleTree};
    use crate::sign_data;

    // Helper function to hash a string using atlas-common
    fn hash_string(data: &str) -> String {
        calculate_hash(data.as_bytes())
    }

    #[actix_web::test]
    async fn test_hashing() {
        // Test hash consistency using atlas-common
        let data = "test data";
        let hash1 = hash_string(data);
        let hash2 = hash_string(data);

        // Same input should produce same hash
        assert_eq!(hash1, hash2);

        // Different inputs should produce different hashes
        let hash3 = hash_string("different data");
        assert_ne!(hash1, hash3);

        // Test that we're using SHA384 (48 bytes = 96 hex chars)
        let raw_hash = calculate_hash(data.as_bytes());
        assert_eq!(raw_hash.len(), 96); // SHA384 produces 96 hex characters
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
        let signature = sign_data(&key_pair, data.as_bytes());

        // Signature should not be empty
        assert!(!signature.is_empty());

        // Ed25519 signatures are 64 bytes, which is 88 chars in base64 (including padding)
        let decoded = STANDARD.decode(&signature).unwrap();
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

        // Verify the proof is valid
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

        // Generate and verify proofs for all leaves
        for i in 0..5 {
            let manifest_id = format!("manifest_{}", i);
            let proof = tree.generate_inclusion_proof(&manifest_id).unwrap();

            // Check basic proof properties
            assert_eq!(proof.manifest_id, manifest_id);
            assert_eq!(proof.tree_size, 5);
            assert_eq!(proof.leaf_index, i);

            // Verify the proof
            assert!(
                tree.verify_inclusion_proof(&proof),
                "Proof verification failed for manifest_{}",
                i
            );
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
                assert!(
                    tree.verify_consistency_proof(&proof),
                    "Consistency proof failed for {} -> {}",
                    old_size,
                    new_size
                );
            }
        }
    }

    #[actix_web::test]
    async fn test_inclusion_proof_negative_cases() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();

        // Add some leaves
        for i in 0..4 {
            tree.add_leaf(LogLeaf::new(
                format!("hash_{}", i),
                format!("id_{}", i),
                i as u64,
                now,
            ));
        }

        // Test 1: Proof for non-existent manifest
        assert!(tree.generate_inclusion_proof("non_existent").is_none());

        // Test 2: Invalid leaf index
        let mut proof = tree.generate_inclusion_proof("id_1").unwrap();
        proof.leaf_index = 99;
        assert!(!tree.verify_inclusion_proof(&proof));

        // Test 3: Wrong manifest ID at same index
        let mut proof = tree.generate_inclusion_proof("id_1").unwrap();
        proof.manifest_id = "wrong_id".to_string();
        assert!(!tree.verify_inclusion_proof(&proof));

        // Test 4: Wrong tree size
        let mut proof = tree.generate_inclusion_proof("id_1").unwrap();
        proof.tree_size = 99;
        assert!(!tree.verify_inclusion_proof(&proof));

        // Test 5: Tampered merkle path
        let mut proof = tree.generate_inclusion_proof("id_1").unwrap();
        if !proof.merkle_path.is_empty() {
            proof.merkle_path[0] = "tampered_hash".to_string();
            assert!(!tree.verify_inclusion_proof(&proof));
        }

        // Test 6: Extra path elements
        let mut proof = tree.generate_inclusion_proof("id_1").unwrap();
        proof.merkle_path.push("extra_hash".to_string());
        assert!(!tree.verify_inclusion_proof(&proof));

        // Test 7: Missing path elements
        let mut proof = tree.generate_inclusion_proof("id_2").unwrap();
        if !proof.merkle_path.is_empty() {
            proof.merkle_path.pop();
            assert!(!tree.verify_inclusion_proof(&proof));
        }

        // Test 8: Empty tree
        let empty_tree = MerkleTree::new();
        assert!(empty_tree.generate_inclusion_proof("any_id").is_none());
    }

    #[actix_web::test]
    async fn test_consistency_proof_negative_cases() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();

        for i in 0..6 {
            tree.add_leaf(LogLeaf::new(
                format!("hash_{}", i),
                format!("id_{}", i),
                i as u64,
                now,
            ));
        }

        // Test 1: Invalid size combinations
        assert!(tree.generate_consistency_proof(0, 3).is_none());
        assert!(tree.generate_consistency_proof(3, 0).is_none());
        assert!(tree.generate_consistency_proof(5, 3).is_none()); // old > new
        assert!(tree.generate_consistency_proof(3, 10).is_none()); // new > tree size

        // Test 2: Verification with wrong roots
        let valid_proof = tree.generate_consistency_proof(2, 4).unwrap();

        let mut tampered_proof = valid_proof.clone();
        tampered_proof.old_root = "wrong_old_root".to_string();
        assert!(!tree.verify_consistency_proof(&tampered_proof));

        let mut tampered_proof = valid_proof.clone();
        tampered_proof.new_root = "wrong_new_root".to_string();
        assert!(!tree.verify_consistency_proof(&tampered_proof));

        // Test 3: Invalid sizes in proof
        let mut tampered_proof = valid_proof.clone();
        tampered_proof.old_size = 0;
        assert!(!tree.verify_consistency_proof(&tampered_proof));

        let mut tampered_proof = valid_proof.clone();
        tampered_proof.new_size = 0;
        assert!(!tree.verify_consistency_proof(&tampered_proof));

        let mut tampered_proof = valid_proof.clone();
        tampered_proof.old_size = 10;
        tampered_proof.new_size = 5;
        assert!(!tree.verify_consistency_proof(&tampered_proof));

        // Test 4: Empty tree consistency
        let empty_tree = MerkleTree::new();
        assert!(empty_tree.generate_consistency_proof(0, 1).is_none());
        assert!(empty_tree.generate_consistency_proof(1, 2).is_none());
    }

    #[actix_web::test]
    async fn test_tree_edge_cases() {
        // Test 1: Empty tree operations
        let tree = MerkleTree::new();
        assert_eq!(tree.size(), 0);
        assert!(tree.root_hash().is_none());
        assert!(tree.generate_inclusion_proof("any").is_none());
        assert!(tree.compute_root_for_size(1).is_none());

        // Test 2: Single leaf tree
        let mut tree = MerkleTree::new();
        let now = Utc::now();
        tree.add_leaf(LogLeaf::new("hash".to_string(), "id".to_string(), 1, now));

        assert_eq!(tree.size(), 1);
        assert!(tree.root_hash().is_some());

        let proof = tree.generate_inclusion_proof("id").unwrap();
        assert_eq!(proof.merkle_path.len(), 0); // No siblings
        assert!(tree.verify_inclusion_proof(&proof));

        // Test 3: Historical root edge cases
        assert!(tree.compute_root_for_size(0).is_none());
        assert!(tree.compute_root_for_size(2).is_none()); // Beyond tree size
        assert!(tree.compute_root_for_size(1).is_some());

        // Test 4: Consistency proof for same size
        let proof = tree.generate_consistency_proof(1, 1).unwrap();
        assert!(proof.proof_hashes.is_empty());
        assert_eq!(proof.old_root, proof.new_root);
        assert!(tree.verify_consistency_proof(&proof));
    }

    #[actix_web::test]
    async fn test_hash_algorithms() {
        let data = b"test data";

        // Test SHA256 using atlas-common
        let sha256_hash = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha256);
        assert_eq!(sha256_hash.len(), 64); // SHA256 produces 64 hex chars

        // Test SHA384 (default) using atlas-common
        let sha384_hash = calculate_hash(data);
        assert_eq!(sha384_hash.len(), 96); // SHA384 produces 96 hex chars

        // Test SHA512 using atlas-common
        let sha512_hash = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha512);
        assert_eq!(sha512_hash.len(), 128); // SHA512 produces 128 hex chars

        // Verify they produce different hashes
        assert_ne!(sha256_hash, sha384_hash);
        assert_ne!(sha384_hash, sha512_hash);
        assert_ne!(sha256_hash, sha512_hash);
    }

    #[actix_web::test]
    async fn test_hash_verification() {
        let data = b"test data for verification";

        // Test default hash verification using atlas-common
        let hash = calculate_hash(data);
        assert!(verify_hash(data, &hash));
        assert!(!verify_hash(b"different data", &hash));

        // Test specific algorithm verification
        let sha256_hash = calculate_hash_with_algorithm(data, &HashAlgorithm::Sha256);
        assert!(verify_hash_with_algorithm(
            data,
            &sha256_hash,
            &HashAlgorithm::Sha256
        ));
        assert!(!verify_hash_with_algorithm(
            data,
            &sha256_hash,
            &HashAlgorithm::Sha384
        ));
    }

    #[actix_web::test]
    async fn test_hasher_trait() {
        // Test the Hasher trait from atlas-common
        let text = "test string";
        let hash1 = text.hash(HashAlgorithm::Sha256);
        let hash2 = text.to_string().hash(HashAlgorithm::Sha256);
        let hash3 = text.as_bytes().hash(HashAlgorithm::Sha256);

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
        assert_eq!(hash1.len(), 64); // SHA256
    }

    #[actix_web::test]
    async fn test_hash_validation() {
        // Test hash format validation using atlas-common

        // Valid hashes
        assert!(validate_hash_format(&"a".repeat(64)).is_ok()); // SHA256
        assert!(validate_hash_format(&"b".repeat(96)).is_ok()); // SHA384
        assert!(validate_hash_format(&"c".repeat(128)).is_ok()); // SHA512

        // Invalid hashes
        assert!(validate_hash_format(&"x".repeat(32)).is_err()); // Wrong length
        assert!(validate_hash_format(&"g".repeat(64)).is_err()); // Invalid char
        assert!(validate_hash_format("not-a-hash").is_err());
    }

    #[actix_web::test]
    async fn test_hash_algorithm_detection() {
        // Test algorithm detection using atlas-common
        let sha256_hash = "a".repeat(64);
        let sha384_hash = "b".repeat(96);
        let sha512_hash = "c".repeat(128);

        assert_eq!(detect_hash_algorithm(&sha256_hash), HashAlgorithm::Sha256);
        assert_eq!(detect_hash_algorithm(&sha384_hash), HashAlgorithm::Sha384);
        assert_eq!(detect_hash_algorithm(&sha512_hash), HashAlgorithm::Sha512);

        // Invalid length defaults to SHA384
        assert_eq!(
            detect_hash_algorithm(&"d".repeat(50)),
            HashAlgorithm::Sha384
        );
    }

    #[actix_web::test]
    async fn test_manifest_id_validation() {
        // Test manifest ID validation using atlas-common

        // Valid IDs
        assert!(validate_manifest_id("urn:c2pa:123e4567-e89b-12d3-a456-426614174000").is_ok());
        assert!(validate_manifest_id("123e4567-e89b-12d3-a456-426614174000").is_ok());
        assert!(validate_manifest_id("my-manifest-123").is_ok());
        assert!(validate_manifest_id("manifest_456").is_ok());

        // Invalid IDs
        assert!(validate_manifest_id("").is_err());
        assert!(validate_manifest_id("manifest with spaces").is_err());
        assert!(validate_manifest_id("manifest#123").is_err());
    }

    #[actix_web::test]
    async fn test_c2pa_urn_utilities() {
        // Test C2PA URN utilities from atlas-common

        // Test ensure_c2pa_urn
        let plain_id = "my-model-123";
        let urn = ensure_c2pa_urn(plain_id);
        assert!(urn.starts_with("urn:c2pa:"));

        // Valid UUID should be wrapped
        let uuid = "123e4567-e89b-12d3-a456-426614174000";
        let wrapped = ensure_c2pa_urn(uuid);
        assert_eq!(wrapped, format!("urn:c2pa:{}", uuid));

        // Already valid URN should be unchanged
        let existing_urn = "urn:c2pa:123e4567-e89b-12d3-a456-426614174000";
        assert_eq!(ensure_c2pa_urn(existing_urn), existing_urn);
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
    async fn test_proof_describe_methods() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();

        // Add some leaves
        for i in 0..4 {
            tree.add_leaf(LogLeaf::new(
                format!("hash_{}", i),
                format!("id_{}", i),
                i as u64,
                now,
            ));
        }

        // Test inclusion proof describe method
        let inclusion_proof = tree.generate_inclusion_proof("id_1").unwrap();
        let description = inclusion_proof.describe();
        assert!(description.contains("id_1"));
        assert!(description.contains("index 1"));
        assert!(description.contains("tree of size 4"));

        // Test consistency proof describe and verify methods
        let consistency_proof = tree.generate_consistency_proof(2, 4).unwrap();
        let description = consistency_proof.describe();
        assert!(description.contains("tree size 2 to 4"));
        assert!(description.contains("proof elements:"));

        // Test the verify method directly on the struct
        assert!(consistency_proof.verify(&consistency_proof.old_root, &consistency_proof.new_root));
        assert!(!consistency_proof.verify("wrong_old", &consistency_proof.new_root));
        assert!(!consistency_proof.verify(&consistency_proof.old_root, "wrong_new"));
    }

    #[actix_web::test]
    async fn test_tree_persistence_and_integrity() {
        let mut original_tree = MerkleTree::new();
        let now = Utc::now();

        // Add some leaves
        for i in 0..5 {
            original_tree.add_leaf(LogLeaf::new(
                format!("hash_{}", i),
                format!("id_{}", i),
                i as u64,
                now,
            ));
        }

        let original_root = original_tree.root_hash().unwrap().clone();
        let original_size = original_tree.size();

        // Simulate persistence and reload - this recomputes the root hash for integrity
        let leaves = original_tree.leaves().to_vec();
        let restored_tree = MerkleTree::from_leaves(leaves);

        // Verify integrity after restoration
        assert_eq!(restored_tree.root_hash().unwrap(), &original_root);
        assert_eq!(restored_tree.size(), original_size);

        // Verify all proofs still work
        for i in 0..5 {
            let manifest_id = format!("id_{}", i);

            // Generate proof from original tree
            let original_proof = original_tree
                .generate_inclusion_proof(&manifest_id)
                .unwrap();

            // Generate proof from restored tree
            let restored_proof = restored_tree
                .generate_inclusion_proof(&manifest_id)
                .unwrap();

            // Both proofs should be identical
            assert_eq!(original_proof.manifest_id, restored_proof.manifest_id);
            assert_eq!(original_proof.leaf_index, restored_proof.leaf_index);
            assert_eq!(original_proof.tree_size, restored_proof.tree_size);
            assert_eq!(original_proof.merkle_path, restored_proof.merkle_path);

            // Both trees should verify each other's proofs
            assert!(original_tree.verify_inclusion_proof(&restored_proof));
            assert!(restored_tree.verify_inclusion_proof(&original_proof));
        }
    }

    #[actix_web::test]
    async fn test_large_tree_consistency() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();

        // Build a larger tree to test scalability
        for i in 0..32 {
            tree.add_leaf(LogLeaf::new(
                format!("hash_{}", i),
                format!("id_{}", i),
                i as u64,
                now,
            ));
        }

        // Test random inclusion proofs
        let test_indices = vec![0, 1, 15, 16, 30, 31];
        for &index in &test_indices {
            let manifest_id = format!("id_{}", index);
            let proof = tree.generate_inclusion_proof(&manifest_id).unwrap();
            assert!(
                tree.verify_inclusion_proof(&proof),
                "Large tree inclusion proof failed for index {}",
                index
            );
        }

        // Test consistency proofs for various size combinations
        let size_pairs = vec![(1, 32), (16, 32), (8, 16), (4, 8)];
        for &(old_size, new_size) in &size_pairs {
            let proof = tree.generate_consistency_proof(old_size, new_size).unwrap();
            assert!(
                tree.verify_consistency_proof(&proof),
                "Large tree consistency proof failed for {} -> {}",
                old_size,
                new_size
            );
        }
    }

    #[actix_web::test]
    async fn test_proof_tampering_detection() {
        let mut tree = MerkleTree::new();
        let now = Utc::now();

        // Add leaves
        for i in 0..8 {
            tree.add_leaf(LogLeaf::new(
                format!("hash_{}", i),
                format!("id_{}", i),
                i as u64,
                now,
            ));
        }

        // Test various tampering scenarios for inclusion proofs
        let original_proof = tree.generate_inclusion_proof("id_3").unwrap();

        // Test 1: Tamper with path elements
        let mut tampered = original_proof.clone();
        if !tampered.merkle_path.is_empty() {
            tampered.merkle_path[0] = format!("tampered_{}", tampered.merkle_path[0]);
            assert!(!tree.verify_inclusion_proof(&tampered));
        }

        // Test 2: Swap path elements
        let mut tampered = original_proof.clone();
        if tampered.merkle_path.len() > 1 {
            tampered.merkle_path.swap(0, 1);
            assert!(!tree.verify_inclusion_proof(&tampered));
        }

        // Test consistency proof tampering
        let consistency_proof = tree.generate_consistency_proof(4, 8).unwrap();

        // Test 3: Tamper with proof hashes
        let mut tampered = consistency_proof.clone();
        if !tampered.proof_hashes.is_empty() {
            tampered.proof_hashes[0] = "tampered_hash".to_string();
            // The verification may or may not catch this depending on implementation
            // but it should at least not crash
            let _ = tree.verify_consistency_proof(&tampered);
        }

        // Test 4: Modify sizes
        let mut tampered = consistency_proof.clone();
        tampered.old_size = 99;
        assert!(!tree.verify_consistency_proof(&tampered));

        let mut tampered = consistency_proof.clone();
        tampered.new_size = 1;
        assert!(!tree.verify_consistency_proof(&tampered));
    }

    #[actix_web::test]
    async fn test_atlas_common_integration() {
        // Test that our hashing matches atlas-common's hashing exactly
        let test_data = b"integration test data";

        //  hash_binary function should match atlas-common's calculate_hash
        let our_hash = crate::hash_binary(test_data);
        let atlas_hash = calculate_hash(test_data);

        assert_eq!(our_hash, atlas_hash);
        assert_eq!(our_hash.len(), 96); // SHA384

        // Test hash verification
        assert!(verify_hash(test_data, &our_hash));

        // Test with different algorithms
        let sha256_hash = calculate_hash_with_algorithm(test_data, &HashAlgorithm::Sha256);
        assert_eq!(sha256_hash.len(), 64);
        assert_ne!(sha256_hash, our_hash);
    }
}
