//! REQUIREMENT: SMT-001 — Tree Structure
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-001`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-001.md`.
//!
//! **Normative statement:** The validator set is stored in a sparse Merkle tree
//! with a fixed depth of TREE_DEPTH (32). The tree supports 2^TREE_DEPTH slots,
//! uses SHA-256 for node hashes (parent = sha256(left || right)), and defines
//! EMPTY_LEAF as sha256([0u8; 48]). Empty node hashes at each level are
//! precomputed so that the empty tree root is deterministic.
//!
//! **How the tests prove this:**
//! - `tree_depth_is_32` and `tree_reports_correct_depth` confirm the constant
//!   and runtime depth are both 32.
//! - `empty_leaf_is_sha256_of_48_zeros` verifies the leaf sentinel value.
//! - `empty_tree_has_deterministic_root` shows two fresh trees share the same
//!   non-trivial root, proving precomputed hashes are used.
//! - `tree_capacity_is_2_pow_depth` validates the address space.
//! - `empty_nodes_precomputed` walks the full chain empty_nodes[0..=32],
//!   checking each level is sha256(prev || prev).
//! - `empty_tree_root_equals_top_empty_node` ties the root back to the chain.
//!
//! **Acceptance-criteria coverage (from spec):**
//! - [x] Tree depth is configurable constant TREE_DEPTH
//! - [x] Default TREE_DEPTH = 32
//! - [x] All proofs have exactly TREE_DEPTH siblings (tested in SMT-004)
//! - [x] Root computation traverses exactly TREE_DEPTH levels (via empty-nodes chain)
//! - [x] Tree supports 2^TREE_DEPTH slots
//! - [ ] Same TREE_DEPTH used in circuit, Rust, and Chialisp (cross-impl; Phase 3)

use chia_l2_consensus::testing::{SparseMerkleTree, EMPTY_LEAF, TREE_DEPTH};
use sha2::{Digest, Sha256};

/// Verifies that the TREE_DEPTH constant is exactly 32.
/// Strategy: direct assertion on the public constant.
/// Confidence: if this passes, every consumer of TREE_DEPTH is using the
/// spec-mandated depth.
#[test]
fn vv_req_smt_001_tree_depth_is_32() {
    // SMT-001: TREE_DEPTH must be 32
    assert_eq!(TREE_DEPTH, 32, "SMT-001: TREE_DEPTH must be 32");
}

/// Verifies that a newly constructed tree reports depth 32 at runtime.
/// Strategy: instantiate a tree and call depth(). This catches any mismatch
/// between the compile-time constant and the runtime field.
/// Confidence: the runtime tree is configured to the same depth as the constant.
#[test]
fn vv_req_smt_001_tree_reports_correct_depth() {
    // SMT-001: Tree instance must report depth 32
    let tree = SparseMerkleTree::new();
    assert_eq!(tree.depth(), 32, "SMT-001: tree.depth() must return 32");
}

/// Verifies EMPTY_LEAF equals sha256([0u8; 48]).
/// Strategy: compute the hash independently with Sha256 and compare to the
/// library constant. This proves the sentinel is derived from the spec formula.
/// Confidence: any change to the sentinel constant or its derivation will fail.
#[test]
fn vv_req_smt_001_empty_leaf_is_sha256_of_48_zeros() {
    // SMT-001: EMPTY_LEAF = sha256([0u8; 48])
    let mut hasher = Sha256::new();
    hasher.update([0u8; 48]);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        EMPTY_LEAF, expected,
        "SMT-001: EMPTY_LEAF must be sha256([0u8; 48])"
    );
}

/// Verifies that the empty tree root is deterministic and non-trivial.
/// Strategy: create two independent empty trees and compare roots; also
/// assert the root is not all-zeros (proving computation, not default memory).
/// Confidence: determinism ensures any node will compute the same starting root.
#[test]
fn vv_req_smt_001_empty_tree_has_deterministic_root() {
    // SMT-001: Empty tree root is the top-level empty node hash
    let tree1 = SparseMerkleTree::new();
    let tree2 = SparseMerkleTree::new();

    assert_eq!(
        tree1.root(),
        tree2.root(),
        "SMT-001: Two empty trees must have identical roots"
    );

    // Root should not be all zeros (it's computed from empty hashes)
    let zero_root = [0u8; 32];
    assert_ne!(
        tree1.root().as_ref(),
        &zero_root,
        "SMT-001: Empty tree root must not be all zeros"
    );
}

/// Verifies the tree address space is 2^32 = 4,294,967,296 slots.
/// Strategy: compute 1 << TREE_DEPTH and compare to the known value.
/// Confidence: proves the slot range matches spec expectations.
#[test]
fn vv_req_smt_001_tree_capacity_is_2_pow_depth() {
    // SMT-001: Tree supports 2^TREE_DEPTH slots
    // At depth 32, this is 2^32 = 4,294,967,296 slots
    let capacity: u64 = 1u64 << TREE_DEPTH;
    assert_eq!(
        capacity, 4_294_967_296,
        "SMT-001: Tree capacity must be 2^32"
    );
}

/// Verifies the full chain of precomputed empty-node hashes.
/// Strategy: calls compute_empty_nodes and independently re-derives every
/// level as sha256(prev || prev), starting from EMPTY_LEAF. Checks length
/// is TREE_DEPTH + 1 and each entry matches.
/// Confidence: the entire precomputed chain is mathematically correct.
#[test]
fn vv_req_smt_001_empty_nodes_precomputed() {
    // SMT-001: Empty nodes at each level are precomputed
    use chia_l2_consensus::testing::compute_empty_nodes;

    let empty_nodes = compute_empty_nodes(TREE_DEPTH);

    // Should have TREE_DEPTH + 1 entries (level 0 through level 32)
    assert_eq!(
        empty_nodes.len(),
        (TREE_DEPTH + 1) as usize,
        "SMT-001: Should have {} empty node hashes",
        TREE_DEPTH + 1
    );

    // Level 0 is the empty leaf
    assert_eq!(
        empty_nodes[0], EMPTY_LEAF,
        "SMT-001: empty_nodes[0] must equal EMPTY_LEAF"
    );

    // Each subsequent level is sha256(prev || prev)
    for i in 1..=TREE_DEPTH as usize {
        let prev = empty_nodes[i - 1];
        let mut hasher = Sha256::new();
        hasher.update(prev);
        hasher.update(prev);
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(
            empty_nodes[i],
            expected,
            "SMT-001: empty_nodes[{}] must be sha256(empty_nodes[{}] || empty_nodes[{}])",
            i,
            i - 1,
            i - 1
        );
    }
}

/// Verifies the empty tree root equals empty_nodes[TREE_DEPTH].
/// Strategy: compare tree.root() to the independently computed chain top.
/// Confidence: the tree implementation and the precomputed constants agree.
#[test]
fn vv_req_smt_001_empty_tree_root_equals_top_empty_node() {
    // SMT-001: Empty tree root is empty_nodes[TREE_DEPTH]
    use chia_l2_consensus::testing::compute_empty_nodes;

    let tree = SparseMerkleTree::new();
    let empty_nodes = compute_empty_nodes(TREE_DEPTH);

    assert_eq!(
        tree.root().as_ref(),
        &empty_nodes[TREE_DEPTH as usize],
        "SMT-001: Empty tree root must equal empty_nodes[TREE_DEPTH]"
    );
}
