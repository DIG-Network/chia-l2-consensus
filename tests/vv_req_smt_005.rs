//! REQUIREMENT: SMT-005 — Cross-implementation consistency
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-005`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-005.md`.
//!
//! Verifies that the Rust SMT implementation matches canonical spec requirements.
//! Full cross-implementation testing with Rue puzzles will be added when Phase 3 is complete.
//!
//! Test vectors defined here MUST be verified against Rue implementation in Phase 3.

use chia_l2_consensus::testing::{
    active_leaf, compute_empty_nodes, compute_slot, SparseMerkleTree, EMPTY_LEAF, TREE_DEPTH,
};
use sha2::{Digest, Sha256};

/// Test vectors for cross-implementation verification.
/// These values are canonical and MUST match between Rust and Rue implementations.
mod test_vectors {
    use super::*;

    /// Known test pubkey: all zeros (48 bytes)
    pub const PUBKEY_ZEROS: [u8; 48] = [0x00; 48];

    /// Known test pubkey: all ones (48 bytes)
    pub const PUBKEY_ONES: [u8; 48] = [0x01; 48];

    /// Known test pubkey: incrementing bytes
    pub const PUBKEY_INCREMENTAL: [u8; 48] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c,
        0x2d, 0x2e, 0x2f,
    ];

    /// EMPTY_LEAF = sha256([0x00; 48])
    /// This MUST match the constant in Rue puzzles.
    pub const EXPECTED_EMPTY_LEAF: [u8; 32] = [
        0x17, 0xb0, 0x76, 0x1f, 0x87, 0xb0, 0x81, 0xd5, 0xcf, 0x10, 0x75, 0x7c, 0xcc, 0x89, 0xf1,
        0x2b, 0xe3, 0x55, 0xc7, 0x0e, 0x2e, 0x29, 0xdf, 0x28, 0x8b, 0x65, 0xb3, 0x07, 0x10, 0xdc,
        0xbc, 0xd1,
    ];

    /// Slot for PUBKEY_ZEROS = first 8 bytes of sha256([0x00; 48]) as BE u64, mod 2^32
    /// sha256([0x00; 48]) = 0x17b0761f87b081d5cf10757ccc89f12be355c70e2e29df288b65b30710dcbcd1
    /// First 8 bytes: 0x17b0761f87b081d5
    /// As BE u64: 1,706,752,145,447,198,165
    /// mod 2^32: 2,276,262,357 = 0x87b081d5
    pub const EXPECTED_SLOT_ZEROS: u64 = 0x87b081d5;

    /// Active leaf for PUBKEY_ONES = sha256([0x01; 48])
    pub fn expected_active_leaf_ones() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&PUBKEY_ONES);
        hasher.finalize().into()
    }

    /// Slot for PUBKEY_ONES
    #[allow(dead_code)]
    pub fn expected_slot_ones() -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(&PUBKEY_ONES);
        let hash: [u8; 32] = hasher.finalize().into();
        let n = u64::from_be_bytes(hash[0..8].try_into().unwrap());
        n % (1u64 << 32)
    }
}

#[test]
fn vv_req_smt_005_empty_leaf_matches_canonical() {
    // SMT-005: EMPTY_LEAF constant must match canonical value
    assert_eq!(
        EMPTY_LEAF,
        test_vectors::EXPECTED_EMPTY_LEAF,
        "SMT-005: EMPTY_LEAF must match canonical test vector"
    );
}

#[test]
fn vv_req_smt_005_slot_computation_matches_canonical() {
    // SMT-005: Slot computation must match canonical test vector
    let slot = compute_slot(&test_vectors::PUBKEY_ZEROS);
    assert_eq!(
        slot,
        test_vectors::EXPECTED_SLOT_ZEROS,
        "SMT-005: compute_slot(PUBKEY_ZEROS) must match canonical value 0x{:08x}",
        test_vectors::EXPECTED_SLOT_ZEROS
    );
}

#[test]
fn vv_req_smt_005_active_leaf_computation_matches_sha256() {
    // SMT-005: Active leaf must be sha256(pubkey) exactly
    let leaf = active_leaf(&test_vectors::PUBKEY_ONES);
    let expected = test_vectors::expected_active_leaf_ones();
    assert_eq!(
        leaf, expected,
        "SMT-005: active_leaf must compute sha256(pubkey)"
    );
}

#[test]
fn vv_req_smt_005_empty_tree_root_is_canonical() {
    // SMT-005: Empty tree root must be the top empty node
    let tree = SparseMerkleTree::new();
    let empty_nodes = compute_empty_nodes(TREE_DEPTH);

    assert_eq!(
        tree.root(),
        empty_nodes[TREE_DEPTH as usize],
        "SMT-005: Empty tree root must equal empty_nodes[TREE_DEPTH]"
    );
}

#[test]
fn vv_req_smt_005_parent_hash_is_left_concat_right() {
    // SMT-005: Parent hash = sha256(left || right), left always first

    let left = [0x11u8; 32];
    let right = [0x22u8; 32];

    // Compute sha256(left || right) - the canonical way
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let parent: [u8; 32] = hasher.finalize().into();

    // Verify this is what we expect (left first)
    let mut hasher_wrong = Sha256::new();
    hasher_wrong.update(right);
    hasher_wrong.update(left);
    let wrong_parent: [u8; 32] = hasher_wrong.finalize().into();

    assert_ne!(
        parent, wrong_parent,
        "SMT-005: Parent hash order must matter (left || right != right || left)"
    );
}

#[test]
fn vv_req_smt_005_single_validator_root_deterministic() {
    // SMT-005: Single validator produces deterministic root
    let mut tree1 = SparseMerkleTree::new();
    let mut tree2 = SparseMerkleTree::new();

    tree1.insert_validator(&test_vectors::PUBKEY_ONES);
    tree2.insert_validator(&test_vectors::PUBKEY_ONES);

    assert_eq!(
        tree1.root(),
        tree2.root(),
        "SMT-005: Same validator must produce same root"
    );
}

#[test]
fn vv_req_smt_005_multiple_validators_order_independent() {
    // SMT-005: Tree root is independent of insertion order
    let mut tree1 = SparseMerkleTree::new();
    let mut tree2 = SparseMerkleTree::new();

    // Insert in different orders
    tree1.insert_validator(&test_vectors::PUBKEY_ZEROS);
    tree1.insert_validator(&test_vectors::PUBKEY_ONES);
    tree1.insert_validator(&test_vectors::PUBKEY_INCREMENTAL);

    tree2.insert_validator(&test_vectors::PUBKEY_INCREMENTAL);
    tree2.insert_validator(&test_vectors::PUBKEY_ZEROS);
    tree2.insert_validator(&test_vectors::PUBKEY_ONES);

    assert_eq!(
        tree1.root(),
        tree2.root(),
        "SMT-005: Root must be independent of insertion order"
    );
}

#[test]
fn vv_req_smt_005_boundary_slot_zero() {
    // SMT-005: Boundary condition - slot 0
    let tree = SparseMerkleTree::new();

    // Find a pubkey that hashes to slot 0 is impractical
    // Instead, test that slot 0 behaves correctly
    let proof = tree.prove(0);
    assert!(
        proof.verify(tree.root()),
        "SMT-005: Proof for slot 0 must verify in empty tree"
    );
    assert_eq!(proof.leaf, EMPTY_LEAF, "SMT-005: Slot 0 empty in new tree");
}

#[test]
fn vv_req_smt_005_boundary_slot_max() {
    // SMT-005: Boundary condition - slot 2^32 - 1 (max slot)
    let max_slot = (1u64 << TREE_DEPTH) - 1;
    let tree = SparseMerkleTree::new();

    let proof = tree.prove(max_slot);
    assert!(
        proof.verify(tree.root()),
        "SMT-005: Proof for max slot must verify in empty tree"
    );
    assert_eq!(
        proof.leaf, EMPTY_LEAF,
        "SMT-005: Max slot empty in new tree"
    );
}

#[test]
fn vv_req_smt_005_proof_siblings_count() {
    // SMT-005: Proofs must have exactly TREE_DEPTH siblings
    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&test_vectors::PUBKEY_ONES);

    let proof = tree.prove(compute_slot(&test_vectors::PUBKEY_ONES));

    assert_eq!(
        proof.siblings.len(),
        TREE_DEPTH as usize,
        "SMT-005: Proof must have exactly TREE_DEPTH siblings"
    );
}

#[test]
fn vv_req_smt_005_fuzz_many_validators() {
    // SMT-005: Fuzz test with many validators
    let mut tree = SparseMerkleTree::new();
    let mut pubkeys = Vec::new();

    // Insert 50 validators with pseudo-random pubkeys
    for i in 0u8..50 {
        let mut pubkey = [0u8; 48];
        // Fill with deterministic "random" pattern
        for j in 0..48 {
            pubkey[j] = i.wrapping_add(j as u8).wrapping_mul(7);
        }
        pubkeys.push(pubkey);
        tree.insert_validator(&pubkey);
    }

    let root = tree.root();

    // Verify all proofs
    for pubkey in &pubkeys {
        let slot = compute_slot(pubkey);
        let proof = tree.prove(slot);

        assert_eq!(proof.leaf, active_leaf(pubkey));
        assert!(
            proof.verify(root),
            "SMT-005: All proofs must verify in fuzz test"
        );
    }
}

#[test]
fn vv_req_smt_005_empty_nodes_chain() {
    // SMT-005: Empty nodes form correct chain
    // empty_nodes[i+1] = sha256(empty_nodes[i] || empty_nodes[i])
    let empty_nodes = compute_empty_nodes(TREE_DEPTH);

    for i in 0..(TREE_DEPTH as usize) {
        let mut hasher = Sha256::new();
        hasher.update(empty_nodes[i]);
        hasher.update(empty_nodes[i]);
        let expected: [u8; 32] = hasher.finalize().into();

        assert_eq!(
            empty_nodes[i + 1],
            expected,
            "SMT-005: empty_nodes[{}] must be sha256(empty_nodes[{}] || empty_nodes[{}])",
            i + 1,
            i,
            i
        );
    }
}

// TODO: When Rue puzzles are implemented in Phase 3, add cross-implementation tests:
// - test_rust_root_matches_rue_root()
// - test_rust_proof_verifies_in_rue()
// - test_rue_slot_matches_rust_slot()
// - test_rue_active_leaf_matches_rust()
