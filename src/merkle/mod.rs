//! Sparse Merkle Tree implementation.
//!
//! See [spec-consensus-crate.md Lines 414-664](../docs/resources/spec-consensus-crate.md).

mod proof;
mod sparse;

pub use proof::MerkleProof;
pub use sparse::{
    active_leaf, compute_empty_nodes, compute_slot, SparseMerkleTree, EMPTY_LEAF, EMPTY_TREE_ROOT,
    TREE_DEPTH,
};
