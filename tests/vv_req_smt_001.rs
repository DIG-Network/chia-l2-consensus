//! REQUIREMENT: SMT-001 — Fixed depth tree structure
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-001`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-001.md`.
//!
//! Verifies that the sparse Merkle tree has the correct fixed-depth structure.

use chia_l2_consensus::merkle::{SparseMerkleTree, EMPTY_LEAF, TREE_DEPTH};
use sha2::{Digest, Sha256};

#[test]
fn vv_req_smt_001_tree_depth_is_32() {
    // SMT-001: TREE_DEPTH must be 32
    assert_eq!(TREE_DEPTH, 32, "SMT-001: TREE_DEPTH must be 32");
}

#[test]
fn vv_req_smt_001_tree_reports_correct_depth() {
    // SMT-001: Tree instance must report depth 32
    let tree = SparseMerkleTree::new();
    assert_eq!(tree.depth(), 32, "SMT-001: tree.depth() must return 32");
}

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

#[test]
fn vv_req_smt_001_empty_nodes_precomputed() {
    // SMT-001: Empty nodes at each level are precomputed
    use chia_l2_consensus::merkle::compute_empty_nodes;

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

#[test]
fn vv_req_smt_001_empty_tree_root_equals_top_empty_node() {
    // SMT-001: Empty tree root is empty_nodes[TREE_DEPTH]
    use chia_l2_consensus::merkle::compute_empty_nodes;

    let tree = SparseMerkleTree::new();
    let empty_nodes = compute_empty_nodes(TREE_DEPTH);

    assert_eq!(
        tree.root().as_ref(),
        &empty_nodes[TREE_DEPTH as usize],
        "SMT-001: Empty tree root must equal empty_nodes[TREE_DEPTH]"
    );
}
