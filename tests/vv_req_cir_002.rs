//! REQUIREMENT: CIR-002 — Merkle Membership Constraint
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-002`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-002.md`.
//!
//! Verifies that for each signing pubkey, the circuit verifies a Merkle
//! inclusion proof demonstrating the pubkey exists in the validator set.

use chia_l2_consensus::merkle::{active_leaf, compute_slot, SparseMerkleTree, TREE_DEPTH};
use sha2::{Digest, Sha256};

#[test]
fn vv_req_cir_002_leaf_is_sha256_of_pubkey() {
    // CIR-002: Leaf value is sha256(pubkey) for active validators
    let pubkey = [0x11u8; 48];

    let leaf = active_leaf(&pubkey);

    // Manual computation
    let mut hasher = Sha256::new();
    hasher.update(&pubkey);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(leaf, expected, "CIR-002: Leaf must be sha256(pubkey)");
}

#[test]
fn vv_req_cir_002_merkle_proof_verifies_membership() {
    // CIR-002: Valid Merkle proof verifies against root
    let pubkey = [0x22u8; 48];

    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey);

    let root = tree.root();
    let proof = tree.prove_validator(&pubkey);

    // Verify the proof using pubkey
    let verified = proof.verify_for_pubkey(&pubkey, root);
    assert!(verified, "CIR-002: Valid Merkle proof must verify");
}

#[test]
fn vv_req_cir_002_invalid_proof_fails() {
    // CIR-002: Invalid proof (wrong sibling) must fail
    let pubkey = [0x33u8; 48];

    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey);

    let root = tree.root();
    let mut proof = tree.prove_validator(&pubkey);

    // Corrupt the first sibling
    if !proof.siblings.is_empty() {
        proof.siblings[0] = [0xFFu8; 32];
    }

    let verified = proof.verify_for_pubkey(&pubkey, root);
    assert!(!verified, "CIR-002: Invalid proof must fail verification");
}

#[test]
fn vv_req_cir_002_wrong_root_fails() {
    // CIR-002: Proof against wrong root must fail
    let pubkey = [0x44u8; 48];

    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey);

    let proof = tree.prove_validator(&pubkey);

    // Use a different (wrong) root
    let wrong_root = [0xABu8; 32];

    let verified = proof.verify_for_pubkey(&pubkey, wrong_root);
    assert!(!verified, "CIR-002: Proof against wrong root must fail");
}

#[test]
fn vv_req_cir_002_wrong_pubkey_fails() {
    // CIR-002: Proof for wrong pubkey must fail
    let pubkey1 = [0x55u8; 48];
    let pubkey2 = [0x66u8; 48];

    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey1);

    let root = tree.root();
    let proof = tree.prove_validator(&pubkey1);

    // Try to verify with wrong pubkey
    let verified = proof.verify_for_pubkey(&pubkey2, root);
    assert!(!verified, "CIR-002: Proof for wrong pubkey must fail");
}

#[test]
fn vv_req_cir_002_multiple_validators_each_verifies() {
    // CIR-002: Each validator's proof verifies independently
    let pubkeys: Vec<[u8; 48]> = (0..5).map(|i| [i as u8 + 1; 48]).collect();

    let mut tree = SparseMerkleTree::new();
    for pk in &pubkeys {
        tree.insert_validator(pk);
    }

    let root = tree.root();

    // Each pubkey should have a valid proof
    for pk in &pubkeys {
        let proof = tree.prove_validator(pk);
        let verified = proof.verify_for_pubkey(pk, root);
        assert!(verified, "CIR-002: Each validator's proof must verify");
    }
}

#[test]
fn vv_req_cir_002_proof_depth_matches_tree_depth() {
    // CIR-002: Proof has TREE_DEPTH siblings
    let pubkey = [0x77u8; 48];

    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey);

    let proof = tree.prove_validator(&pubkey);

    assert_eq!(
        proof.siblings.len(),
        TREE_DEPTH as usize,
        "CIR-002: Proof must have TREE_DEPTH siblings"
    );
}

#[test]
fn vv_req_cir_002_sibling_ordering_left_first() {
    // CIR-002: Sibling ordering: left child first in hash computation
    // At each level, if index bit = 0: current is LEFT, sibling is RIGHT
    // At each level, if index bit = 1: current is RIGHT, sibling is LEFT
    // Hash is always sha256(left || right)

    let pubkey = [0x88u8; 48];
    let slot = compute_slot(&pubkey);

    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey);

    let proof = tree.prove_validator(&pubkey);

    // Manually verify path computation
    let leaf = active_leaf(&pubkey);
    let mut current = leaf;
    let mut index = slot;

    for sibling in &proof.siblings {
        // Compute parent hash with correct ordering
        let mut hasher = Sha256::new();

        // Left child first in concatenation
        // If index is even (bit=0), current is left child
        if index % 2 == 0 {
            hasher.update(current);
            hasher.update(sibling);
        } else {
            hasher.update(sibling);
            hasher.update(current);
        }

        current = hasher.finalize().into();
        index >>= 1;
    }

    assert_eq!(
        current,
        tree.root(),
        "CIR-002: Manual path computation must match root"
    );
}

#[test]
fn vv_req_cir_002_index_from_pubkey_deterministic() {
    // CIR-002: Index (slot) is derived deterministically from pubkey
    let pubkey = [0x99u8; 48];

    let slot1 = compute_slot(&pubkey);
    let slot2 = compute_slot(&pubkey);
    let slot3 = compute_slot(&pubkey);

    assert_eq!(
        slot1, slot2,
        "CIR-002: Slot computation must be deterministic"
    );
    assert_eq!(
        slot2, slot3,
        "CIR-002: Slot computation must be deterministic"
    );

    // Slot must fit in TREE_DEPTH bits
    let max_slot = (1u64 << TREE_DEPTH) - 1;
    assert!(
        slot1 <= max_slot,
        "CIR-002: Slot must fit in TREE_DEPTH bits"
    );
}

#[test]
fn vv_req_cir_002_all_proofs_verify_against_same_root() {
    // CIR-002: All k proofs must verify against the same root
    let pubkeys: Vec<[u8; 48]> = (0..10).map(|i| [(i as u8) * 17 + 1; 48]).collect();

    let mut tree = SparseMerkleTree::new();
    for pk in &pubkeys {
        tree.insert_validator(pk);
    }

    let root = tree.root();

    // All proofs must verify against THIS root
    for pk in &pubkeys {
        let proof = tree.prove_validator(pk);
        assert!(
            proof.verify_for_pubkey(pk, root),
            "CIR-002: All proofs must verify against same root"
        );
    }

    // Different root should fail for all
    let different_root = [0x00u8; 32];
    for pk in &pubkeys {
        let proof = tree.prove_validator(pk);
        assert!(
            !proof.verify_for_pubkey(pk, different_root),
            "CIR-002: Different root must fail for all proofs"
        );
    }
}
