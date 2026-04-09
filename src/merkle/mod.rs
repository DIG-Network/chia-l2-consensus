//! Sparse Merkle Tree implementation.
//!
//! See [spec-consensus-crate.md Lines 414-664](../docs/resources/spec-consensus-crate.md).

mod proof;
mod sparse;

pub use proof::MerkleProof;
pub use sparse::{compute_empty_nodes, SparseMerkleTree, EMPTY_LEAF, TREE_DEPTH};
