//! REQUIREMENT: VAL-004 — Voluntary Exit
//! (`docs/requirements/domains/validator/NORMATIVE.md#VAL-004`).
//!
//! Spec: `docs/requirements/domains/validator/specs/VAL-004.md`.
//!
//! Implementation: `src/validator/exit.rs`.
//!
//! Verifies that a validator can exit voluntarily: generate non-membership
//! proof after exclusion, compute the exit announcement, and prepare the
//! collateral recovery parameters.

use chia_l2_consensus::testing::{active_leaf, SparseMerkleTree, EMPTY_LEAF};
use chia_l2_consensus::testing::{
    compute_exit_announcement, compute_membership_announcement_message, generate_validator_keypair,
    is_validator_excluded, prepare_collateral_recovery,
};

/// Helper: generate deterministic test pubkeys.
fn test_pubkeys(n: usize) -> Vec<[u8; 48]> {
    (0..n)
        .map(|i| {
            let mut entropy = [0u8; 32];
            entropy[0] = i as u8;
            entropy[1] = (i >> 8) as u8;
            generate_validator_keypair(&entropy).unwrap().pubkey
        })
        .collect()
}

// ── Active validator is NOT excluded ────────────────────────────────

#[test]
fn vv_req_val_004_active_validator_not_excluded() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    assert!(
        !is_validator_excluded(&tree, &pks[0]),
        "VAL-004: Active validator must NOT be excluded"
    );
}

// ── Removed validator IS excluded ───────────────────────────────────

#[test]
fn vv_req_val_004_removed_validator_is_excluded() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    // Remove validator 1
    tree.remove_validator(&pks[1]);

    assert!(
        is_validator_excluded(&tree, &pks[1]),
        "VAL-004: Removed validator must be excluded"
    );
    // Others still active
    assert!(
        !is_validator_excluded(&tree, &pks[0]),
        "VAL-004: Other validators must remain active"
    );
}

// ── Non-membership proof valid after exclusion ──────────────────────

#[test]
fn vv_req_val_004_non_membership_proof_valid() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    tree.remove_validator(&pks[1]);
    let root = tree.root();
    let proof = tree.prove_validator(&pks[1]);

    // Proof leaf must be EMPTY_LEAF (non-membership)
    assert_eq!(
        proof.leaf, EMPTY_LEAF,
        "VAL-004: Non-membership proof leaf must be EMPTY_LEAF"
    );

    // Proof must verify against current root
    assert!(
        proof.verify(root),
        "VAL-004: Non-membership proof must verify against tree root"
    );
}

// ── Membership proof invalid after exclusion ────────────────────────

#[test]
fn vv_req_val_004_membership_proof_invalid_after_exclusion() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    tree.remove_validator(&pks[1]);

    // Try to construct a membership proof for excluded validator
    // The tree returns EMPTY_LEAF, not active_leaf
    let proof = tree.prove_validator(&pks[1]);
    assert_ne!(
        proof.leaf,
        active_leaf(&pks[1]),
        "VAL-004: After exclusion, leaf must NOT be active_leaf"
    );
}

// ── Exit announcement matches WIRE-004 ──────────────────────────────

#[test]
fn vv_req_val_004_exit_announcement_matches_wire() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let epoch: u64 = 5;
    let checkpoint_coin_id = [0xCC; 32];

    let announcement = compute_exit_announcement(epoch, &kp.pubkey, &checkpoint_coin_id);

    // Inner hash per WIRE-004 (non-membership)
    let inner = compute_membership_announcement_message(epoch, &kp.pubkey, false);

    // Full announcement = sha256(checkpoint_coin_id + inner)
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(checkpoint_coin_id);
    hasher.update(inner);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        announcement, expected,
        "VAL-004: Exit announcement must match sha256(coin_id + inner_announcement)"
    );
}

// ── Exit announcement is 32 bytes ───────────────────────────────────

#[test]
fn vv_req_val_004_exit_announcement_is_32_bytes() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();

    let announcement = compute_exit_announcement(5, &kp.pubkey, &[0xCC; 32]);

    assert_eq!(
        announcement.len(),
        32,
        "VAL-004: Exit announcement must be 32 bytes"
    );
}

// ── Different epochs produce different announcements ────────────────

#[test]
fn vv_req_val_004_different_epochs() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let cid = [0xCC; 32];

    let a1 = compute_exit_announcement(5, &kp.pubkey, &cid);
    let a2 = compute_exit_announcement(6, &kp.pubkey, &cid);

    assert_ne!(
        a1, a2,
        "VAL-004: Different epochs must produce different announcements"
    );
}

// ── Prepare collateral recovery returns valid params ────────────────

#[test]
fn vv_req_val_004_prepare_recovery() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    tree.remove_validator(&pks[1]);

    let epoch: u64 = 5;
    let checkpoint_coin_id = [0xCC; 32];
    let destination = [0xDD; 32];
    let collateral_amount: u64 = 10_000_000_000_000;

    let params = prepare_collateral_recovery(
        &tree,
        &pks[1],
        epoch,
        &checkpoint_coin_id,
        &destination,
        collateral_amount,
    )
    .unwrap();

    assert_eq!(params.pubkey, pks[1], "VAL-004: Params pubkey must match");
    assert_eq!(params.epoch, epoch, "VAL-004: Params epoch must match");
    assert_eq!(
        params.checkpoint_coin_id, checkpoint_coin_id,
        "VAL-004: Params coin ID must match"
    );
    assert_eq!(
        params.destination, destination,
        "VAL-004: Params destination must match"
    );
    assert_eq!(
        params.collateral_amount, collateral_amount,
        "VAL-004: Params collateral must match"
    );
    assert_eq!(
        params.merkle_proof.leaf, EMPTY_LEAF,
        "VAL-004: Proof must be non-membership"
    );
    assert!(
        params.merkle_proof.verify(tree.root()),
        "VAL-004: Proof must verify against tree root"
    );
}

// ── Prepare recovery fails for active validator ─────────────────────

#[test]
fn vv_req_val_004_recovery_fails_for_active() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    let result = prepare_collateral_recovery(
        &tree,
        &pks[0], // Still active
        5,
        &[0xCC; 32],
        &[0xDD; 32],
        10_000_000_000_000,
    );

    assert!(
        result.is_err(),
        "VAL-004: Cannot prepare recovery for active validator"
    );
}

// ── Prepare recovery fails for never-registered pubkey ──────────────

#[test]
fn vv_req_val_004_recovery_works_for_never_registered() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks[0..2] {
        tree.insert_validator(pk);
    }

    // pks[2] was never registered — slot is empty
    let result = prepare_collateral_recovery(
        &tree,
        &pks[2],
        5,
        &[0xCC; 32],
        &[0xDD; 32],
        10_000_000_000_000,
    );

    // Non-membership proof should work (slot is empty)
    // but in practice the registration coin doesn't exist
    // so the bundle would fail on-chain. The prepare function
    // only checks tree state, not on-chain state.
    assert!(
        result.is_ok(),
        "VAL-004: Prepare succeeds for empty slot (on-chain check is separate)"
    );
}
