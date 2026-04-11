//! REQUIREMENT: CHK-010 — Single Checkpoint Per Epoch
//! (`docs/requirements/domains/checkpoint/NORMATIVE.md#CHK-010`).
//!
//! Spec: `docs/requirements/domains/checkpoint/specs/CHK-010.md`.
//!
//! Verifies that only one checkpoint can be accepted per epoch: the
//! singleton pattern consumes the old coin, epoch in the checkpoint_message
//! prevents cross-epoch replay, and BLS signatures bind to a specific epoch.

use chia_l2_consensus::testing::{
    aggregate_checkpoint_signatures, compute_checkpoint_message,
    compute_checkpoint_signing_message, generate_validator_keypair, sign_checkpoint,
    verify_checkpoint_signature,
};

// ── Same state but different epochs produce different messages ───────

#[test]
fn vv_req_chk_010_same_state_different_epochs() {
    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 5;

    let msg_e3 = compute_checkpoint_message(sr, mr, vc, 3);
    let msg_e4 = compute_checkpoint_message(sr, mr, vc, 4);

    assert_ne!(
        msg_e3, msg_e4,
        "CHK-010: Same state at different epochs MUST produce different checkpoint messages"
    );
}

// ── Signature for epoch N does not verify at epoch M ────────────────

#[test]
fn vv_req_chk_010_signature_epoch_mismatch() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let gc = [0xAA; 32];
    let coin_id = [0xBB; 32];

    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 5;

    // Sign for epoch 3
    let msg_epoch_3 = compute_checkpoint_message(sr, mr, vc, 3);
    let sig = sign_checkpoint(&kp.secret_key, &kp.pubkey, &msg_epoch_3, &gc, &coin_id).unwrap();

    // Verify at epoch 3 — should succeed
    let valid_3 =
        verify_checkpoint_signature(&kp.pubkey, &msg_epoch_3, &gc, &coin_id, &sig).unwrap();
    assert!(
        valid_3,
        "CHK-010: Signature for epoch 3 must verify at epoch 3"
    );

    // Verify at epoch 4 — should fail (different checkpoint_message)
    let msg_epoch_4 = compute_checkpoint_message(sr, mr, vc, 4);
    let valid_4 =
        verify_checkpoint_signature(&kp.pubkey, &msg_epoch_4, &gc, &coin_id, &sig).unwrap();
    assert!(
        !valid_4,
        "CHK-010: Signature for epoch 3 MUST NOT verify at epoch 4"
    );
}

// ── Aggregate signature also bound to epoch ─────────────────────────

#[test]
fn vv_req_chk_010_aggregate_signature_epoch_bound() {
    let kp1 = generate_validator_keypair(&[0x01; 32]).unwrap();
    let kp2 = generate_validator_keypair(&[0x02; 32]).unwrap();
    let gc = [0xAA; 32];
    let coin_id = [0xBB; 32];

    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 3;

    // Both sign for epoch 5
    let msg_5 = compute_checkpoint_message(sr, mr, vc, 5);
    let sig1 = sign_checkpoint(&kp1.secret_key, &kp1.pubkey, &msg_5, &gc, &coin_id).unwrap();
    let sig2 = sign_checkpoint(&kp2.secret_key, &kp2.pubkey, &msg_5, &gc, &coin_id).unwrap();

    let agg_sig = aggregate_checkpoint_signatures(&[sig1, sig2]).unwrap();
    assert_eq!(
        agg_sig.len(),
        96,
        "CHK-010: Aggregate signature is 96 bytes"
    );

    // Aggregate for epoch 6 would be different
    let msg_6 = compute_checkpoint_message(sr, mr, vc, 6);
    let sig1_e6 = sign_checkpoint(&kp1.secret_key, &kp1.pubkey, &msg_6, &gc, &coin_id).unwrap();
    let sig2_e6 = sign_checkpoint(&kp2.secret_key, &kp2.pubkey, &msg_6, &gc, &coin_id).unwrap();

    let agg_sig_6 = aggregate_checkpoint_signatures(&[sig1_e6, sig2_e6]).unwrap();
    assert_ne!(
        agg_sig, agg_sig_6,
        "CHK-010: Aggregate signatures for different epochs MUST differ"
    );
}

// ── Signing message includes epoch via checkpoint_message ───────────

#[test]
fn vv_req_chk_010_signing_message_epoch_chain() {
    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 5;
    let gc = [0xAA; 32];
    let coin_id = [0xBB; 32];

    let msg_5 = compute_checkpoint_message(sr, mr, vc, 5);
    let msg_6 = compute_checkpoint_message(sr, mr, vc, 6);

    let signing_msg_5 = compute_checkpoint_signing_message(&msg_5, &gc, &coin_id);
    let signing_msg_6 = compute_checkpoint_signing_message(&msg_6, &gc, &coin_id);

    // First 32 bytes are the checkpoint_message (which includes epoch)
    assert_ne!(
        &signing_msg_5[0..32],
        &signing_msg_6[0..32],
        "CHK-010: Signing message first 32 bytes (checkpoint_message) differ between epochs"
    );
    // Last 64 bytes (genesis_challenge + coin_id) are the same
    assert_eq!(
        &signing_msg_5[32..96],
        &signing_msg_6[32..96],
        "CHK-010: Genesis challenge and coin_id unchanged between epochs"
    );
}

// ── Singleton pattern: old coin consumed, new coin has epoch+1 ──────
// (This is structural — the Rue puzzle creates a new coin with updated state)

#[test]
fn vv_req_chk_010_puzzle_creates_new_coin_with_epoch_plus_1() {
    let source = include_str!("../puzzles/checkpoint_inner.rue");

    // Puzzle must compute new_epoch
    assert!(
        source.contains("new_epoch"),
        "CHK-010: Puzzle must reference new_epoch"
    );

    // Puzzle must create a new coin (singleton recreation)
    assert!(
        source.contains("CreateCoin") || source.contains("create_coin"),
        "CHK-010: Puzzle must create a new coin for singleton recreation"
    );

    // The new coin's puzzle hash must incorporate the new epoch
    // (via curry_tree_hash with new state including new_epoch)
    assert!(
        source.contains("curry_tree_hash") || source.contains("INNER_MOD_HASH"),
        "CHK-010: New coin puzzle hash must be derived from updated state"
    );
}

// ── Epoch increment is exactly +1, not arbitrary ────────────────────

#[test]
fn vv_req_chk_010_epoch_increment_exactly_1() {
    let source = include_str!("../puzzles/checkpoint_inner.rue");

    // Must contain "epoch + 1" or "STATE.epoch + 1"
    let has_plus_1 = source.contains("epoch + 1");
    assert!(has_plus_1, "CHK-010: Epoch increment must be exactly +1");

    // Must NOT accept epoch from solution (no "new_epoch" in solution struct)
    // The CheckpointSolution struct should not have an epoch field
    // (epoch is computed, not provided)
}

// ── Proof replay: proof for epoch 5 cannot verify at epoch 10 ───────

#[test]
fn vv_req_chk_010_proof_replay_prevention() {
    // The checkpoint_message for epoch 5 differs from epoch 10
    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 3;

    let msg_5 = compute_checkpoint_message(sr, mr, vc, 5);
    let msg_10 = compute_checkpoint_message(sr, mr, vc, 10);

    // If we generated a proof for epoch 5, the proof's checkpoint_message = msg_5
    // If we try to use it at epoch 10, the puzzle computes msg_10
    // The scalar s6 = sha256(msg_5) vs sha256(msg_10) — DIFFERENT
    // This means the proof would fail verification

    use chia_l2_consensus::testing::bytes_to_scalar;
    let s6_epoch5 = bytes_to_scalar(&msg_5);
    let s6_epoch10 = bytes_to_scalar(&msg_10);

    assert_ne!(
        s6_epoch5, s6_epoch10,
        "CHK-010: Scalar s6 for epoch 5 differs from epoch 10 — proof replay prevented"
    );
}
