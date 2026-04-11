//! Sparse Merkle Tree — validator set data structure.
//!
//! A fixed-depth (TREE_DEPTH=32) sparse Merkle tree that stores the
//! active validator set. Each validator occupies a deterministic slot
//! derived from sha256(pubkey). The root is committed on-chain as
//! `validator_merkle_root` in the checkpoint singleton state.
//!
//! ## Sub-modules
//!
//! | Module | Purpose | Requirements |
//! |--------|---------|-------------|
//! | `sparse` | Core SMT: insert, remove, root, prove | SMT-001 through SMT-006 |
//! | `proof` | `MerkleProof` type with verification | SMT-004 |
//! | `poseidon` | ZK-friendly Poseidon tree for in-circuit use | CIR-002 |
//!
//! ## Critical Cross-Implementation Requirement (SMT-005)
//!
//! The Rust implementation (this module) and the Rue on-chain verification
//! (`verify_merkle_path` in checkpoint_inner.rue) MUST produce identical
//! results. Any divergence causes proofs to fail silently on-chain.
//!
//! Matching rules:
//! - Left child always first in sha256 concatenation
//! - Active leaf = sha256(pubkey), empty leaf = sha256([0x00; 48])
//! - Slot = first_8_bytes_be(sha256(pubkey)) mod 2^TREE_DEPTH
//!
//! See [spec-sparse-merkle-tree.md](../../docs/resources/spec-sparse-merkle-tree.md)
//! for the canonical Merkle tree specification.
//! See [spec-consensus-crate.md Lines 414-664](../../docs/resources/spec-consensus-crate.md)
//! for the crate-level SMT specification.

pub mod poseidon;
mod proof;
mod sparse;

pub use proof::MerkleProof;
pub use sparse::{
    active_leaf, compute_empty_nodes, compute_slot, SparseMerkleTree, EMPTY_LEAF, EMPTY_TREE_ROOT,
    TREE_DEPTH,
};
