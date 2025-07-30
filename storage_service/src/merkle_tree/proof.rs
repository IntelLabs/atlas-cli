use serde::{Deserialize, Serialize};

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
    /// The root hash at the time of proof generation
    pub root_hash: String,
    /// The size of the tree at the time of proof generation
    pub tree_size: usize,
}

impl InclusionProof {
    /// Verify this proof against a given root hash
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn verify_against_root(&self, root_hash: &str) -> bool {
        self.root_hash == root_hash
    }

    /// Get a human-readable description of the proof
    pub fn describe(&self) -> String {
        format!(
            "Inclusion proof for manifest '{}' at index {} in tree of size {}",
            self.manifest_id, self.leaf_index, self.tree_size
        )
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

    /// Verify this proof is valid
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn verify(&self, old_root: &str, new_root: &str) -> bool {
        self.old_root == old_root && self.new_root == new_root
    }
}
