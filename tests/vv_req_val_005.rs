//! REQUIREMENT: VAL-005 — Forced Exit
//! (`docs/requirements/domains/validator/NORMATIVE.md#VAL-005`).
//!
//! Spec: `docs/requirements/domains/validator/specs/VAL-005.md`.
//!
//! Implementation: `src/validator/exit.rs`.
//!
//! Verifies that forced exit uses the same recovery mechanism as voluntary
//! exit, that collateral can be directed to a governance/slash address,
//! and that multiple validators can be force-exited in one checkpoint.

use chia_l2_consensus::testing::{
    compute_exit_announcement, generate_validator_keypair, is_validator_excluded,
    prepare_collateral_recovery, prepare_forced_exit, ForcedExitReason,
};
use chia_l2_consensus::testing::{SparseMerkleTree, EMPTY_LEAF};

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

// ── Forced exit uses same mechanism as voluntary ────────────────────

#[test]
fn vv_req_val_005_same_mechanism_as_voluntary() {
    let pks = test_pubkeys(5);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    // Force-exit validator 2
    tree.remove_validator(&pks[2]);

    let epoch: u64 = 10;
    let checkpoint_coin_id = [0xCC; 32];
    let governance_addr = [0xEE; 32];
    let collateral = 10_000_000_000_000u64;

    // Voluntary recovery (VAL-004)
    let voluntary = prepare_collateral_recovery(
        &tree,
        &pks[2],
        epoch,
        &checkpoint_coin_id,
        &governance_addr,
        collateral,
    )
    .unwrap();

    // Forced exit (VAL-005)
    let forced = prepare_forced_exit(
        &tree,
        &pks[2],
        epoch,
        &checkpoint_coin_id,
        &governance_addr,
        collateral,
        ForcedExitReason::GovernanceDecision,
    )
    .unwrap();

    // Same merkle proof
    assert_eq!(
        voluntary.merkle_proof.leaf, forced.params.merkle_proof.leaf,
        "VAL-005: Forced exit must use same proof as voluntary"
    );
    assert_eq!(
        voluntary.announcement_hash, forced.params.announcement_hash,
        "VAL-005: Same announcement hash"
    );
}

// ── Collateral can go to governance slash address ───────────────────

#[test]
fn vv_req_val_005_slash_to_governance_address() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    tree.remove_validator(&pks[0]);

    let governance_addr = [0xFF; 32]; // Slash address, not validator's own
    let forced = prepare_forced_exit(
        &tree,
        &pks[0],
        5,
        &[0xCC; 32],
        &governance_addr,
        10_000_000_000_000,
        ForcedExitReason::KeyCompromise,
    )
    .unwrap();

    assert_eq!(
        forced.params.destination, governance_addr,
        "VAL-005: Forced exit must direct collateral to governance address"
    );
}

// ── Multiple validators force-exited in one checkpoint ──────────────

#[test]
fn vv_req_val_005_multiple_forced_exits() {
    let pks = test_pubkeys(5);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    // Force-exit validators 1, 3, 4
    tree.remove_validator(&pks[1]);
    tree.remove_validator(&pks[3]);
    tree.remove_validator(&pks[4]);

    let epoch = 7u64;
    let cid = [0xCC; 32];
    let slash_addr = [0xDD; 32];

    for &idx in &[1, 3, 4] {
        assert!(
            is_validator_excluded(&tree, &pks[idx]),
            "VAL-005: Validator {} must be excluded",
            idx
        );

        let result = prepare_forced_exit(
            &tree,
            &pks[idx],
            epoch,
            &cid,
            &slash_addr,
            10_000_000_000_000,
            ForcedExitReason::GovernanceDecision,
        );
        assert!(
            result.is_ok(),
            "VAL-005: Forced exit must succeed for validator {}",
            idx
        );
    }

    // Remaining validators still active
    assert!(!is_validator_excluded(&tree, &pks[0]));
    assert!(!is_validator_excluded(&tree, &pks[2]));
}

// ── Active validator cannot be force-exited ─────────────────────────

#[test]
fn vv_req_val_005_active_cannot_be_forced() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    let result = prepare_forced_exit(
        &tree,
        &pks[0],
        5,
        &[0xCC; 32],
        &[0xDD; 32],
        10_000_000_000_000,
        ForcedExitReason::Misbehavior,
    );

    assert!(
        result.is_err(),
        "VAL-005: Cannot force-exit active validator (must be excluded in checkpoint first)"
    );
}

// ── Forced exit reason is recorded ──────────────────────────────────

#[test]
fn vv_req_val_005_reason_recorded() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    tree.remove_validator(&pks[0]);

    let forced = prepare_forced_exit(
        &tree,
        &pks[0],
        5,
        &[0xCC; 32],
        &[0xDD; 32],
        10_000_000_000_000,
        ForcedExitReason::KeyCompromise,
    )
    .unwrap();

    assert!(
        matches!(forced.reason, ForcedExitReason::KeyCompromise),
        "VAL-005: Reason must be recorded"
    );
}

// ── All forced exit reasons are constructible ───────────────────────

#[test]
fn vv_req_val_005_all_reasons() {
    let reasons = vec![
        ForcedExitReason::KeyCompromise,
        ForcedExitReason::ValidatorOffline,
        ForcedExitReason::Misbehavior,
        ForcedExitReason::GovernanceDecision,
    ];

    for reason in reasons {
        let label = format!("{:?}", reason);
        assert!(
            !label.is_empty(),
            "VAL-005: Reason {:?} must be Debug-printable",
            reason
        );
    }
}

// ── Exit announcement is same regardless of voluntary/forced ────────

#[test]
fn vv_req_val_005_announcement_independent_of_reason() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    tree.remove_validator(&pks[0]);

    let epoch = 5u64;
    let cid = [0xCC; 32];

    // Same announcement regardless of how exit was initiated
    let announcement = compute_exit_announcement(epoch, &pks[0], &cid);

    let forced = prepare_forced_exit(
        &tree,
        &pks[0],
        epoch,
        &cid,
        &[0xDD; 32],
        10_000_000_000_000,
        ForcedExitReason::Misbehavior,
    )
    .unwrap();

    assert_eq!(
        forced.params.announcement_hash, announcement,
        "VAL-005: Announcement must be same regardless of voluntary/forced"
    );
}

// ── Validator slot shows EMPTY_LEAF after exclusion ─────────────────

#[test]
fn vv_req_val_005_empty_leaf_after_exclusion() {
    let pks = test_pubkeys(3);
    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    tree.remove_validator(&pks[1]);

    let proof = tree.prove_validator(&pks[1]);
    assert_eq!(
        proof.leaf, EMPTY_LEAF,
        "VAL-005: Excluded validator slot must contain EMPTY_LEAF"
    );
}
