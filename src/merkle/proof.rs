//! Merkle proof types.
//!
//! See [spec-sparse-merkle-tree.md](../../docs/resources/spec-sparse-merkle-tree.md).

use sha2::{Digest, Sha256};

use super::sparse::TREE_DEPTH;

/// A Merkle proof for membership or non-membership.
///
/// Contains exactly TREE_DEPTH sibling hashes, from leaf level to root level.
///
/// Source: spec-sparse-merkle-tree.md Lines 247-325
#[derive(Debug, Clone, Default)]
pub struct MerkleProof {
    /// The leaf value at the proven slot.
    pub leaf: [u8; 32],

    /// The slot being proven.
    pub slot: u64,

    /// Sibling hashes from leaf to root (length = TREE_DEPTH).
    pub siblings: Vec<[u8; 32]>,
}

impl MerkleProof {
    /// Verify this proof against a root.
    ///
    /// Returns true if the proof is valid for the given root.
    ///
    /// Source: spec-sparse-merkle-tree.md Lines 327-380
    pub fn verify(&self, root: [u8; 32]) -> bool {
        // Must have exactly TREE_DEPTH siblings
        if self.siblings.len() != TREE_DEPTH as usize {
            return false;
        }

        let mut current = self.leaf;
        let mut index = self.slot;

        for sibling in &self.siblings {
            let mut hasher = Sha256::new();

            // Left child first in concatenation
            // If index is even, current is left child; otherwise right child
            if index.is_multiple_of(2) {
                hasher.update(current);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(current);
            }

            current = hasher.finalize().into();
            index >>= 1;
        }

        current == root
    }

    /// Verify this proof for a specific pubkey against a root.
    ///
    /// This verifies:
    /// 1. The leaf equals sha256(pubkey) (active validator)
    /// 2. The Merkle path from leaf to root is valid
    ///
    /// Source: spec-sparse-merkle-tree.md Lines 327-380 (CIR-002)
    pub fn verify_for_pubkey(&self, pubkey: &[u8; 48], root: [u8; 32]) -> bool {
        // Compute expected leaf for this pubkey
        let expected_leaf = super::sparse::active_leaf(pubkey);

        // Verify the leaf matches
        if self.leaf != expected_leaf {
            return false;
        }

        // Verify the Merkle path
        self.verify(root)
    }

    /// Get the number of siblings (should be TREE_DEPTH).
    pub fn len(&self) -> usize {
        self.siblings.len()
    }

    /// Check if proof is empty (has no siblings).
    pub fn is_empty(&self) -> bool {
        self.siblings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_proof_is_empty() {
        let proof = MerkleProof::default();
        assert!(proof.is_empty());
        assert_eq!(proof.len(), 0);
    }
}
