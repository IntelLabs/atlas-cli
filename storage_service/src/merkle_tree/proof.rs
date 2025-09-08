use serde::{Deserialize, Serialize};

/// Common trait for all Merkle tree proofs
pub trait MerkleProof {
    /// Get a human-readable description of the proof
    fn describe(&self) -> String;

    /// Verify the internal consistency of this proof
    /// This checks that the proof structure is valid, but doesn't verify
    /// it against a specific tree state (that's done by the tree now )
    fn verify_structure(&self) -> bool;
}

/// Proof of inclusion for a leaf in the Merkle tree
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InclusionProof {
    /// The manifest ID this proof is for
    pub manifest_id: String,
    /// The index of the leaf in the tree
    pub leaf_index: usize,
    /// The hash of the leaf
    pub leaf_hash: String,
    /// The Merkle path from leaf to root
    pub merkle_path: Vec<String>,
    /// The size of the tree at the time of proof generation
    pub tree_size: usize,
    /// The root hash this proof leads to
    pub root_hash: String,
}

impl InclusionProof {
    /// Get a human-readable description of the proof
    pub fn describe(&self) -> String {
        format!(
            "Inclusion proof for manifest '{}' at index {} in tree of size {}",
            self.manifest_id, self.leaf_index, self.tree_size
        )
    }

    /// Verify that the merkle path correctly hashes from leaf to root
    pub fn verify_path(&self, hasher: &dyn crate::merkle_tree::hasher::Hasher) -> bool {
        if self.tree_size == 0 {
            return false;
        }

        if self.leaf_index >= self.tree_size {
            return false;
        }

        // Start with the leaf hash
        let mut current_hash = self.leaf_hash.clone();
        let mut level_pos = self.leaf_index;
        let mut level_size = self.tree_size;
        let mut path_index = 0;

        // Traverse up the tree using the Merkle path
        while level_size > 1 {
            // Check if this node has a sibling
            let has_sibling = if level_pos % 2 == 0 {
                level_pos + 1 < level_size
            } else {
                true // Left nodes always have a right sibling
            };

            if has_sibling && path_index < self.merkle_path.len() {
                let sibling_hash = &self.merkle_path[path_index];
                let is_left = level_pos % 2 == 0;

                current_hash = if is_left {
                    let combined = format!("node:{}:{}", current_hash, sibling_hash);
                    hasher.hash(combined.as_bytes())
                } else {
                    let combined = format!("node:{}:{}", sibling_hash, current_hash);
                    hasher.hash(combined.as_bytes())
                };

                path_index += 1;
            }

            // Move to parent level
            level_pos /= 2;
            level_size = (level_size + 1) / 2; // Ceiling division
        }

        // Verify we used all path elements and the final hash matches the root
        path_index == self.merkle_path.len() && current_hash == self.root_hash
    }
}

impl MerkleProof for InclusionProof {
    fn describe(&self) -> String {
        self.describe()
    }

    fn verify_structure(&self) -> bool {
        // Basic structural checks
        if self.manifest_id.is_empty() || self.tree_size == 0 {
            return false;
        }

        if self.leaf_index >= self.tree_size {
            return false;
        }

        if self.leaf_hash.is_empty() || self.root_hash.is_empty() {
            return false;
        }

        // For a single-node tree, there should be no merkle path
        if self.tree_size == 1 {
            return self.merkle_path.is_empty() && self.leaf_index == 0;
        }

        // For larger trees, verify the path length makes sense
        // The path length should be at most log2(tree_size)
        let max_path_length = (self.tree_size as f64).log2().ceil() as usize;
        self.merkle_path.len() <= max_path_length
    }
}

/// Proof of consistency between two tree sizes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsistencyProof {
    /// The old tree size
    pub old_size: usize,
    /// The new tree size
    pub new_size: usize,
    /// The old root hash
    pub old_root: String,
    /// The new root hash
    pub new_root: String,
    /// The consistency proof hashes
    pub proof_hashes: Vec<String>,
}

impl ConsistencyProof {
    /// Get a human-readable description of the proof
    pub fn describe(&self) -> String {
        format!(
            "Consistency proof from tree size {} to {} (proof elements: {})",
            self.old_size,
            self.new_size,
            self.proof_hashes.len()
        )
    }

    /// Verify this proof against expected root values
    pub fn verify(&self, expected_old_root: &str, expected_new_root: &str) -> bool {
        self.old_root == expected_old_root && self.new_root == expected_new_root
    }
}

impl MerkleProof for ConsistencyProof {
    fn describe(&self) -> String {
        self.describe()
    }

    fn verify_structure(&self) -> bool {
        // Basic structural checks
        if self.old_size == 0 || self.new_size == 0 {
            return false;
        }

        if self.old_size > self.new_size {
            return false;
        }

        if self.old_root.is_empty() || self.new_root.is_empty() {
            return false;
        }

        // For same size, should have empty proof and same roots
        if self.old_size == self.new_size {
            return self.proof_hashes.is_empty() && self.old_root == self.new_root;
        }

        true
    }
}
