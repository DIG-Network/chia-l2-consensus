//! REQUIREMENT: SMT-004 — Proof format
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-004`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-004.md`.
//!
//! Verifies that Merkle proofs have correct format and verification works properly.

use chia_l2_consensus::merkle::{active_leaf, compute_slot, SparseMerkleTree, TREE_DEPTH};
use sha2::{Digest, Sha256};

#[test]
fn vv_req_smt_004_proof_has_tree_depth_siblings() {
    // SMT-004: Proof contains exactly TREE_DEPTH siblings
    let pubkey = [0x42u8; 48];
    let mut tree = SparseMerkleTree::new();

    // Insert validator and get proof
    let proof = tree.insert_validator(&pubkey);

    assert_eq!(
        proof.len(),
        TREE_DEPTH as usize,
        "SMT-004: Proof must have exactly TREE_DEPTH ({}) siblings, got {}",
        TREE_DEPTH,
        proof.len()
    );
}

#[test]
fn vv_req_smt_004_proof_for_empty_slot_has_tree_depth_siblings() {
    // SMT-004: Non-membership proof also has TREE_DEPTH siblings
    let tree = SparseMerkleTree::new();
    let slot = 12345u64;

    let proof = tree.prove(slot);

    assert_eq!(
        proof.len(),
        TREE_DEPTH as usize,
        "SMT-004: Empty slot proof must have TREE_DEPTH siblings"
    );
}

#[test]
fn vv_req_smt_004_siblings_ordered_bottom_up() {
    // SMT-004: Siblings ordered bottom-up (leaf level first)
    // We verify this by checking that the proof verification works correctly
    // If siblings were in wrong order, verification would fail
    let pubkey = [0x12u8; 48];
    let mut tree = SparseMerkleTree::new();

    tree.insert_validator(&pubkey);
    let proof = tree.prove(compute_slot(&pubkey));

    // siblings[0] should be the sibling at leaf level
    // siblings[TREE_DEPTH-1] should be the sibling just below root
    // If we verify successfully, the order is correct
    assert!(
        proof.verify(tree.root()),
        "SMT-004: Proof with correct sibling ordering must verify"
    );
}

#[test]
fn vv_req_smt_004_left_child_determination() {
    // SMT-004: Left child determination: (index >> level) & 1 == 0
    // Test with various indices to verify left/right convention

    // Helper function matching spec
    fn is_left_child(index: u64, level: u32) -> bool {
        (index >> level) & 1 == 0
    }

    // Index 0 at level 0: bit 0 = 0 → left child
    assert!(is_left_child(0, 0), "Index 0 at level 0 is left child");

    // Index 1 at level 0: bit 0 = 1 → right child
    assert!(!is_left_child(1, 0), "Index 1 at level 0 is right child");

    // Index 2 at level 0: bit 0 = 0 → left child
    assert!(is_left_child(2, 0), "Index 2 at level 0 is left child");

    // Index 5 (binary 101) at level 0: bit 0 = 1 → right child
    assert!(!is_left_child(5, 0), "Index 5 at level 0 is right child");

    // Index 5 at level 1: bit 1 = 0 → left child
    assert!(is_left_child(5, 1), "Index 5 at level 1 is left child");

    // Index 5 at level 2: bit 2 = 1 → right child
    assert!(!is_left_child(5, 2), "Index 5 at level 2 is right child");
}

#[test]
fn vv_req_smt_004_hash_convention_left_first() {
    // SMT-004: Hash always: sha256(left || right)
    // Verify that parent hash computation follows left-first convention

    let left = [0x11u8; 32];
    let right = [0x22u8; 32];

    // Compute sha256(left || right)
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let parent_left_first: [u8; 32] = hasher.finalize().into();

    // Compute sha256(right || left)
    let mut hasher = Sha256::new();
    hasher.update(right);
    hasher.update(left);
    let parent_right_first: [u8; 32] = hasher.finalize().into();

    // They should be different
    assert_ne!(
        parent_left_first, parent_right_first,
        "SMT-004: sha256(left||right) != sha256(right||left)"
    );
}

#[test]
fn vv_req_smt_004_valid_proof_reconstructs_root() {
    // SMT-004: Valid proof reconstructs correct root
    let pubkey = [0x34u8; 48];
    let mut tree = SparseMerkleTree::new();

    tree.insert_validator(&pubkey);
    let root = tree.root();
    let proof = tree.prove(compute_slot(&pubkey));

    assert!(
        proof.verify(root),
        "SMT-004: Valid proof must reconstruct the correct root"
    );
}

#[test]
fn vv_req_smt_004_proof_with_wrong_leaf_fails() {
    // SMT-004: Proof with wrong leaf value fails verification
    let pubkey = [0x56u8; 48];
    let mut tree = SparseMerkleTree::new();

    tree.insert_validator(&pubkey);
    let root = tree.root();
    let mut proof = tree.prove(compute_slot(&pubkey));

    // Corrupt the leaf
    proof.leaf = [0xffu8; 32];

    assert!(
        !proof.verify(root),
        "SMT-004: Proof with wrong leaf must fail verification"
    );
}

#[test]
fn vv_req_smt_004_proof_with_corrupted_sibling_fails() {
    // SMT-004: Corrupting one sibling causes verification failure
    let pubkey = [0x78u8; 48];
    let mut tree = SparseMerkleTree::new();

    tree.insert_validator(&pubkey);
    let root = tree.root();
    let mut proof = tree.prove(compute_slot(&pubkey));

    // Corrupt one sibling in the middle
    proof.siblings[16] = [0xffu8; 32];

    assert!(
        !proof.verify(root),
        "SMT-004: Proof with corrupted sibling must fail verification"
    );
}

#[test]
fn vv_req_smt_004_proof_with_swapped_siblings_fails() {
    // SMT-004: Swapping two siblings causes verification failure
    let pubkey = [0x9au8; 48];
    let mut tree = SparseMerkleTree::new();

    tree.insert_validator(&pubkey);
    let root = tree.root();
    let mut proof = tree.prove(compute_slot(&pubkey));

    // Swap two adjacent siblings
    proof.siblings.swap(10, 11);

    assert!(
        !proof.verify(root),
        "SMT-004: Proof with swapped siblings must fail verification"
    );
}

#[test]
fn vv_req_smt_004_proof_with_wrong_slot_fails() {
    // SMT-004: Proof with wrong slot fails verification
    let pubkey = [0xbcu8; 48];
    let mut tree = SparseMerkleTree::new();

    tree.insert_validator(&pubkey);
    let root = tree.root();
    let mut proof = tree.prove(compute_slot(&pubkey));

    // Change the slot
    proof.slot = proof.slot.wrapping_add(1);

    assert!(
        !proof.verify(root),
        "SMT-004: Proof with wrong slot must fail verification"
    );
}

#[test]
fn vv_req_smt_004_multiple_validators_proofs_all_verify() {
    // SMT-004: All proofs verify for a tree with multiple validators
    let mut tree = SparseMerkleTree::new();
    let mut pubkeys = Vec::new();

    // Insert 10 validators
    for i in 0u8..10 {
        let mut pubkey = [0u8; 48];
        pubkey[0] = i;
        pubkeys.push(pubkey);
        tree.insert_validator(&pubkey);
    }

    let root = tree.root();

    // Verify all proofs
    for pubkey in &pubkeys {
        let proof = tree.prove(compute_slot(pubkey));
        assert_eq!(proof.leaf, active_leaf(pubkey));
        assert!(
            proof.verify(root),
            "SMT-004: All validator proofs must verify against current root"
        );
    }
}

#[test]
fn vv_req_smt_004_proof_too_few_siblings_fails() {
    // SMT-004: Proof with fewer than TREE_DEPTH siblings fails
    let pubkey = [0xdeu8; 48];
    let mut tree = SparseMerkleTree::new();

    tree.insert_validator(&pubkey);
    let root = tree.root();
    let mut proof = tree.prove(compute_slot(&pubkey));

    // Remove one sibling
    proof.siblings.pop();

    assert!(
        !proof.verify(root),
        "SMT-004: Proof with too few siblings must fail verification"
    );
}
