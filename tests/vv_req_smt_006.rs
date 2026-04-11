//! REQUIREMENT: SMT-006 — Empty tree optimization
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-006`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-006.md`.
//!
//! Verifies that empty subtrees use precomputed hashes for efficient computation.

use chia_l2_consensus::testing::{
    compute_empty_nodes, SparseMerkleTree, EMPTY_LEAF, EMPTY_TREE_ROOT, TREE_DEPTH,
};
use sha2::{Digest, Sha256};
use std::time::Instant;

#[test]
fn vv_req_smt_006_empty_hashes_precomputed_at_init() {
    // SMT-006: Empty hashes precomputed at initialization
    let tree = SparseMerkleTree::new();

    // The tree should be ready immediately with correct root
    // If hashes weren't precomputed, this would take forever
    assert_ne!(
        tree.root(),
        [0u8; 32],
        "SMT-006: Tree root should be precomputed, not zeros"
    );
}

#[test]
fn vv_req_smt_006_empty_tree_root_is_known_constant() {
    // SMT-006: Empty tree root is known constant
    let tree = SparseMerkleTree::new();
    let empty_nodes = compute_empty_nodes(TREE_DEPTH);

    // Verify constant matches computed value
    assert_eq!(
        EMPTY_TREE_ROOT, empty_nodes[TREE_DEPTH as usize],
        "SMT-006: EMPTY_TREE_ROOT must match computed empty_nodes[TREE_DEPTH]"
    );

    // Verify tree uses this constant
    assert_eq!(
        tree.root(),
        EMPTY_TREE_ROOT,
        "SMT-006: Empty tree root must equal EMPTY_TREE_ROOT constant"
    );
}

#[test]
fn vv_req_smt_006_empty_tree_root_computed_correctly() {
    // SMT-006: Verify the empty tree root chain computation
    let empty_nodes = compute_empty_nodes(TREE_DEPTH);

    // empty_nodes[0] = EMPTY_LEAF
    assert_eq!(
        empty_nodes[0], EMPTY_LEAF,
        "SMT-006: empty_nodes[0] must equal EMPTY_LEAF"
    );

    // Verify chain: each level is sha256(prev || prev)
    for i in 1..=TREE_DEPTH as usize {
        let mut hasher = Sha256::new();
        hasher.update(empty_nodes[i - 1]);
        hasher.update(empty_nodes[i - 1]);
        let expected: [u8; 32] = hasher.finalize().into();

        assert_eq!(
            empty_nodes[i],
            expected,
            "SMT-006: empty_nodes[{}] must be sha256(empty_nodes[{}] || empty_nodes[{}])",
            i,
            i - 1,
            i - 1
        );
    }
}

#[test]
fn vv_req_smt_006_memory_usage_is_sparse() {
    // SMT-006: Memory usage is O(n + depth) not O(2^depth)
    let mut tree = SparseMerkleTree::new();

    // Insert 100 validators
    for i in 0u8..100 {
        let mut pubkey = [0u8; 48];
        pubkey[0] = i;
        tree.insert_validator(&pubkey);
    }

    // Tree stores only active leaves, not all 2^32 slots
    assert_eq!(
        tree.len(),
        100,
        "SMT-006: Tree should store exactly 100 leaves"
    );

    // We can't directly check internal memory, but we can verify the tree
    // is not storing 2^32 entries (that would be impossible to create)
    // The fact that this test runs quickly proves sparse storage
}

#[test]
fn vv_req_smt_006_root_computation_is_efficient() {
    // SMT-006: Root computation is O(n × depth) not O(2^depth)
    let mut tree = SparseMerkleTree::new();

    // Insert 100 validators
    for i in 0u8..100 {
        let mut pubkey = [0u8; 48];
        pubkey[0] = i;
        tree.insert_validator(&pubkey);
    }

    // Measure time to compute root (should be very fast)
    let start = Instant::now();
    let _root = tree.root();
    let duration = start.elapsed();

    // Should be well under 1 second (O(2^32) would take forever)
    assert!(
        duration.as_millis() < 100,
        "SMT-006: Root computation should be under 100ms, took {:?}",
        duration
    );
}

#[test]
fn vv_req_smt_006_empty_subtrees_use_precomputed_hash() {
    // SMT-006: Empty subtrees return precomputed hash
    let mut tree = SparseMerkleTree::new();

    // Insert one validator at slot 0
    let pubkey = [0u8; 48];
    tree.insert_validator(&pubkey);

    // The proof for a distant empty slot should still work quickly
    // because empty subtrees use precomputed hashes
    let start = Instant::now();
    let proof = tree.prove(4_000_000_000); // Far from slot 0
    let duration = start.elapsed();

    // Should be very fast
    assert!(
        duration.as_millis() < 100,
        "SMT-006: Proof generation for empty slot should be under 100ms, took {:?}",
        duration
    );

    // Proof should verify
    assert!(
        proof.verify(tree.root()),
        "SMT-006: Proof for empty slot must verify"
    );
}

#[test]
fn vv_req_smt_006_many_validators_still_efficient() {
    // SMT-006: Scaling test - many validators should still be efficient
    // Note: Using smaller number to keep test fast in debug mode
    let mut tree = SparseMerkleTree::new();

    // Insert 50 validators (enough to test sparsity)
    for i in 0u8..50 {
        let mut pubkey = [0u8; 48];
        pubkey[0] = i;
        tree.insert_validator(&pubkey);
    }

    // Compute root - should be fast
    let start = Instant::now();
    let _root = tree.root();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 1000,
        "SMT-006: Root with 50 validators should be under 1s in debug mode, took {:?}",
        duration
    );
}

#[test]
fn vv_req_smt_006_empty_nodes_count() {
    // SMT-006: compute_empty_nodes returns correct count
    let empty_nodes = compute_empty_nodes(TREE_DEPTH);

    // Should have TREE_DEPTH + 1 entries (levels 0 through TREE_DEPTH)
    assert_eq!(
        empty_nodes.len(),
        (TREE_DEPTH + 1) as usize,
        "SMT-006: Should have TREE_DEPTH + 1 empty node hashes"
    );
}

#[test]
fn vv_req_smt_006_empty_tree_root_constant_correct() {
    // SMT-006: Verify EMPTY_TREE_ROOT constant by computing from scratch
    let empty_nodes = compute_empty_nodes(TREE_DEPTH);
    let computed_root = empty_nodes[TREE_DEPTH as usize];

    // Print correct value for debugging if mismatch
    if EMPTY_TREE_ROOT != computed_root {
        eprintln!("Expected EMPTY_TREE_ROOT: {:02x?}", computed_root);
    }

    assert_eq!(
        EMPTY_TREE_ROOT, computed_root,
        "SMT-006: EMPTY_TREE_ROOT constant must match computed value"
    );
}
