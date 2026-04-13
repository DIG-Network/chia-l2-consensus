//! REQUIREMENT: SMT-003 — Leaf Values (Active / Empty)
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-003`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-003.md`.
//!
//! **Normative statement:** Active leaves contain sha256(pubkey); empty leaves
//! contain EMPTY_LEAF = sha256([0u8; 48]). Membership proofs carry the active
//! leaf; non-membership proofs carry EMPTY_LEAF. Removing a validator restores
//! the slot to EMPTY_LEAF.
//!
//! **How the tests prove this:**
//! - `active_leaf_is_sha256_pubkey` manually hashes a pubkey and compares.
//! - `empty_leaf_is_sha256_48_zeros` re-derives the constant from scratch.
//! - `empty_leaf_is_known_constant` pins EMPTY_LEAF to its exact hex bytes.
//! - `membership_proof_uses_active_leaf` inserts a validator and checks the
//!   proof's leaf field is sha256(pubkey), then verifies the proof.
//! - `nonmembership_proof_uses_empty_leaf` checks a proof for an empty slot
//!   carries EMPTY_LEAF and verifies against the tree root.
//! - `remove_validator_sets_leaf_to_empty` inserts then removes a validator and
//!   confirms both the proof leaf and get_leaf return EMPTY_LEAF.
//! - `active_leaf_differs_from_empty_leaf` confirms active != empty for 100 keys.
//! - `tree_stores_active_leaf_correctly` verifies get_leaf returns sha256(pubkey)
//!   for an inserted validator.
//!
//! **Acceptance-criteria coverage (from spec):**
//! - [x] Active leaf = sha256(pubkey)
//! - [x] Empty leaf = sha256(48 zero bytes)
//! - [x] EMPTY_LEAF_HASH is a known constant
//! - [x] Membership proof uses sha256(pubkey) as leaf
//! - [x] Non-membership proof uses EMPTY_LEAF_HASH
//! - [ ] Same EMPTY_LEAF_HASH in Rust and Chialisp (cross-impl; Phase 3)

use chia_l2_consensus::testing::{active_leaf, compute_slot, SparseMerkleTree, EMPTY_LEAF};
use sha2::{Digest, Sha256};

/// Verifies that active_leaf(pubkey) equals sha256(pubkey).
/// Strategy: independent Sha256 computation compared to the library function.
/// Confidence: the leaf derivation matches the canonical spec formula.
#[test]
fn vv_req_smt_003_active_leaf_is_sha256_pubkey() {
    // SMT-003: Active leaf = sha256(pubkey)
    let pubkey = [0x42u8; 48];

    let computed_leaf = active_leaf(&pubkey);

    // Verify manually
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        computed_leaf, expected,
        "SMT-003: active_leaf must equal sha256(pubkey)"
    );
}

/// Verifies EMPTY_LEAF = sha256([0x00; 48]).
/// Strategy: hash 48 zero bytes independently and compare to the constant.
/// Confidence: the sentinel is spec-derived, not an arbitrary magic number.
#[test]
fn vv_req_smt_003_empty_leaf_is_sha256_48_zeros() {
    // SMT-003: Empty leaf = sha256(48 zero bytes)
    let zero_pubkey = [0x00u8; 48];

    let mut hasher = Sha256::new();
    hasher.update(zero_pubkey);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        EMPTY_LEAF, expected,
        "SMT-003: EMPTY_LEAF must equal sha256([0x00; 48])"
    );
}

/// Pins EMPTY_LEAF to its exact hex bytes for cross-implementation anchoring.
/// Strategy: hardcoded byte array compared to the library constant.
/// Confidence: bit-exact regression test; any accidental change is caught.
#[test]
fn vv_req_smt_003_empty_leaf_is_known_constant() {
    // SMT-003: EMPTY_LEAF_HASH is a known constant
    // Value from spec: sha256([0u8; 48])
    let expected: [u8; 32] = [
        0x17, 0xb0, 0x76, 0x1f, 0x87, 0xb0, 0x81, 0xd5, 0xcf, 0x10, 0x75, 0x7c, 0xcc, 0x89, 0xf1,
        0x2b, 0xe3, 0x55, 0xc7, 0x0e, 0x2e, 0x29, 0xdf, 0x28, 0x8b, 0x65, 0xb3, 0x07, 0x10, 0xdc,
        0xbc, 0xd1,
    ];

    assert_eq!(
        EMPTY_LEAF, expected,
        "SMT-003: EMPTY_LEAF must be the known constant"
    );
}

/// Verifies that a membership proof carries sha256(pubkey) as its leaf and
/// verifies against the tree root.
/// Strategy: insert a validator, inspect the returned proof's leaf field,
/// then call verify() to confirm end-to-end correctness.
/// Confidence: the proof object is usable for on-chain membership claims.
#[test]
fn vv_req_smt_003_membership_proof_uses_active_leaf() {
    // SMT-003: Membership proof uses sha256(pubkey) as leaf
    let pubkey = [0x12u8; 48];
    let mut tree = SparseMerkleTree::new();

    // Insert validator
    let proof = tree.insert_validator(&pubkey);

    // Proof leaf should be sha256(pubkey)
    let expected_leaf = active_leaf(&pubkey);
    assert_eq!(
        proof.leaf, expected_leaf,
        "SMT-003: Membership proof leaf must be sha256(pubkey)"
    );

    // Verify proof is valid
    assert!(
        proof.verify(tree.root()),
        "SMT-003: Membership proof must verify against tree root"
    );
}

/// Verifies that a non-membership proof for an unoccupied slot carries
/// EMPTY_LEAF and verifies against the tree root.
/// Strategy: generate a proof for a slot that was never filled and inspect
/// both the leaf field and verification result.
/// Confidence: non-membership proofs are valid for de-registration claims.
#[test]
fn vv_req_smt_003_nonmembership_proof_uses_empty_leaf() {
    // SMT-003: Non-membership proof uses EMPTY_LEAF_HASH
    let pubkey = [0x34u8; 48];
    let tree = SparseMerkleTree::new();
    let slot = compute_slot(&pubkey);

    // Generate proof for empty slot
    let proof = tree.prove(slot);

    // Proof leaf should be EMPTY_LEAF
    assert_eq!(
        proof.leaf, EMPTY_LEAF,
        "SMT-003: Non-membership proof leaf must be EMPTY_LEAF"
    );

    // Verify proof is valid
    assert!(
        proof.verify(tree.root()),
        "SMT-003: Non-membership proof must verify against tree root"
    );
}

/// Verifies that removing a validator restores the slot to EMPTY_LEAF.
/// Strategy: insert then remove, check both the returned proof leaf and
/// the tree's get_leaf for that slot.
/// Confidence: slot reuse after de-registration works correctly.
#[test]
fn vv_req_smt_003_remove_validator_sets_leaf_to_empty() {
    // SMT-003: Removing validator sets leaf back to EMPTY_LEAF
    let pubkey = [0x56u8; 48];
    let mut tree = SparseMerkleTree::new();

    // Insert then remove
    tree.insert_validator(&pubkey);
    let proof = tree.remove_validator(&pubkey);

    // After removal, leaf should be EMPTY_LEAF
    assert_eq!(
        proof.leaf, EMPTY_LEAF,
        "SMT-003: After removal, leaf must be EMPTY_LEAF"
    );

    // Verify the slot is now empty
    let slot = compute_slot(&pubkey);
    let leaf = tree.get_leaf(slot);
    assert_eq!(
        leaf, EMPTY_LEAF,
        "SMT-003: get_leaf after removal must return EMPTY_LEAF"
    );
}

/// Verifies that active_leaf(pk) != EMPTY_LEAF for 100 non-zero pubkeys.
/// Strategy: sweep pubkeys with varying first byte and assert inequality.
/// Confidence: no realistic pubkey will collide with the empty sentinel.
#[test]
fn vv_req_smt_003_active_leaf_differs_from_empty_leaf() {
    // SMT-003: Active leaf != EMPTY_LEAF for any realistic pubkey
    // Test with various pubkeys
    for i in 1u8..=100 {
        let mut pubkey = [0u8; 48];
        pubkey[0] = i;

        let leaf = active_leaf(&pubkey);
        assert_ne!(
            leaf, EMPTY_LEAF,
            "SMT-003: active_leaf({}) must differ from EMPTY_LEAF",
            i
        );
    }
}

/// Verifies the tree stores sha256(pubkey) at the computed slot after insertion.
/// Strategy: insert a validator, then call get_leaf on its slot and compare
/// to active_leaf.
/// Confidence: the in-memory tree state is consistent with leaf derivation.
#[test]
fn vv_req_smt_003_tree_stores_active_leaf_correctly() {
    // SMT-003: Tree stores sha256(pubkey) for inserted validators
    let pubkey = [0x78u8; 48];
    let mut tree = SparseMerkleTree::new();

    // Insert validator
    tree.insert_validator(&pubkey);

    // Verify stored leaf
    let slot = compute_slot(&pubkey);
    let stored_leaf = tree.get_leaf(slot);
    let expected_leaf = active_leaf(&pubkey);

    assert_eq!(
        stored_leaf, expected_leaf,
        "SMT-003: Tree must store sha256(pubkey) for active validator"
    );
}
