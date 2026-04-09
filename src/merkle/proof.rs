//! Merkle proof types.
//!
//! See [spec-sparse-merkle-tree.md](../../docs/resources/spec-sparse-merkle-tree.md).

use chia_protocol::Bytes32;

/// A Merkle proof for membership or non-membership.
#[derive(Debug, Clone, Default)]
pub struct MerkleProof {
    /// The leaf value at the proven slot.
    pub leaf: Bytes32,

    /// The slot being proven.
    pub slot: u32,

    /// Sibling hashes from leaf to root (length = TREE_DEPTH).
    pub siblings: Vec<Bytes32>,
}

impl MerkleProof {
    /// Verify this proof against a root.
    pub fn verify(&self, _root: Bytes32) -> bool {
        // TODO: Implement verification
        false
    }
}
