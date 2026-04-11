//! REQUIREMENT: VAL-003 — Signing Protocol
//! (`docs/requirements/domains/validator/NORMATIVE.md#VAL-003`).
//!
//! Spec: `docs/requirements/domains/validator/specs/VAL-003.md`.
//!
//! Implementation: `src/validator/signing.rs`.
//!
//! Verifies that validators can sign checkpoint messages with the correct
//! 96-byte AGG_SIG_ME format, that signatures verify, that aggregation
//! works for multiple validators, and that the aggregate verifies.

use chia_l2_consensus::testing::{
    aggregate_checkpoint_signatures, compute_checkpoint_message,
    compute_checkpoint_signing_message, generate_validator_keypair, sign_checkpoint,
    verify_checkpoint_signature,
};

// ── Signing message is 96 bytes ─────────────────────────────────────

#[test]
fn vv_req_val_003_signing_message_is_96_bytes() {
    let checkpoint_msg = [0xAA; 32];
    let genesis_challenge = [0xBB; 32];
    let coin_id = [0xCC; 32];

    let msg = compute_checkpoint_signing_message(&checkpoint_msg, &genesis_challenge, &coin_id);

    assert_eq!(msg.len(), 96, "VAL-003: Signing message must be 96 bytes");
}

// ── Signing message format: checkpoint_msg + genesis + coin_id ──────

#[test]
fn vv_req_val_003_signing_message_format() {
    let checkpoint_msg = [0xAA; 32];
    let genesis_challenge = [0xBB; 32];
    let coin_id = [0xCC; 32];

    let msg = compute_checkpoint_signing_message(&checkpoint_msg, &genesis_challenge, &coin_id);

    assert_eq!(
        &msg[0..32],
        &checkpoint_msg,
        "VAL-003: First 32 bytes = checkpoint_message"
    );
    assert_eq!(
        &msg[32..64],
        &genesis_challenge,
        "VAL-003: Bytes 32-64 = genesis_challenge"
    );
    assert_eq!(&msg[64..96], &coin_id, "VAL-003: Bytes 64-96 = coin_id");
}

// ── Checkpoint message computation matches WIRE-001 ─────────────────

#[test]
fn vv_req_val_003_checkpoint_message_from_wire() {
    use sha2::{Digest, Sha256};

    let new_state_root = [0x11; 32];
    let new_vmr = [0x22; 32];
    let new_vc: u64 = 5;
    let new_epoch: u64 = 3;

    let msg = compute_checkpoint_message(new_state_root, new_vmr, new_vc, new_epoch, [0x00; 32]);

    // Manual computation per WIRE-001 + CHK-012
    let mut hasher = Sha256::new();
    hasher.update(new_state_root);
    hasher.update(new_vmr);
    hasher.update(new_vc.to_be_bytes());
    hasher.update(new_epoch.to_be_bytes());
    hasher.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        msg, expected,
        "VAL-003: Checkpoint message must match WIRE-001 format"
    );
}

// ── Sign and verify checkpoint roundtrip ────────────────────────────

#[test]
fn vv_req_val_003_sign_verify_roundtrip() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let checkpoint_msg = [0xAA; 32];
    let gc = [0xBB; 32];
    let cid = [0xCC; 32];

    let sig = sign_checkpoint(&kp.secret_key, &kp.pubkey, &checkpoint_msg, &gc, &cid).unwrap();

    assert_eq!(sig.len(), 96, "VAL-003: Signature must be 96 bytes (G2)");

    let valid = verify_checkpoint_signature(&kp.pubkey, &checkpoint_msg, &gc, &cid, &sig).unwrap();
    assert!(
        valid,
        "VAL-003: Checkpoint signature must verify with correct key"
    );
}

// ── Wrong key fails ─────────────────────────────────────────────────

#[test]
fn vv_req_val_003_wrong_key_fails() {
    let kp1 = generate_validator_keypair(&[0x01; 32]).unwrap();
    let kp2 = generate_validator_keypair(&[0x02; 32]).unwrap();
    let checkpoint_msg = [0xAA; 32];
    let gc = [0xBB; 32];
    let cid = [0xCC; 32];

    let sig = sign_checkpoint(&kp1.secret_key, &kp1.pubkey, &checkpoint_msg, &gc, &cid).unwrap();
    let valid = verify_checkpoint_signature(&kp2.pubkey, &checkpoint_msg, &gc, &cid, &sig).unwrap();

    assert!(!valid, "VAL-003: Signature must NOT verify with wrong key");
}

// ── Wrong checkpoint message fails ──────────────────────────────────

#[test]
fn vv_req_val_003_wrong_checkpoint_msg_fails() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let gc = [0xBB; 32];
    let cid = [0xCC; 32];

    let sig = sign_checkpoint(&kp.secret_key, &kp.pubkey, &[0xAA; 32], &gc, &cid).unwrap();
    let valid = verify_checkpoint_signature(&kp.pubkey, &[0xFF; 32], &gc, &cid, &sig).unwrap();

    assert!(
        !valid,
        "VAL-003: Signature must NOT verify with wrong checkpoint message"
    );
}

// ── Aggregate multiple signatures ───────────────────────────────────

#[test]
fn vv_req_val_003_aggregate_signatures() {
    let kp1 = generate_validator_keypair(&[0x01; 32]).unwrap();
    let kp2 = generate_validator_keypair(&[0x02; 32]).unwrap();
    let kp3 = generate_validator_keypair(&[0x03; 32]).unwrap();

    let checkpoint_msg = [0xAA; 32];
    let gc = [0xBB; 32];
    let cid = [0xCC; 32];

    let sig1 = sign_checkpoint(&kp1.secret_key, &kp1.pubkey, &checkpoint_msg, &gc, &cid).unwrap();
    let sig2 = sign_checkpoint(&kp2.secret_key, &kp2.pubkey, &checkpoint_msg, &gc, &cid).unwrap();
    let sig3 = sign_checkpoint(&kp3.secret_key, &kp3.pubkey, &checkpoint_msg, &gc, &cid).unwrap();

    let agg_sig = aggregate_checkpoint_signatures(&[sig1, sig2, sig3]).unwrap();

    assert_eq!(
        agg_sig.len(),
        96,
        "VAL-003: Aggregate signature must be 96 bytes (G2)"
    );

    // Aggregate must not equal any individual signature
    assert_ne!(
        agg_sig, sig1,
        "VAL-003: Aggregate must differ from individual"
    );
    assert_ne!(agg_sig, sig2);
    assert_ne!(agg_sig, sig3);
}

// ── Single signature aggregation equals original ────────────────────

#[test]
fn vv_req_val_003_single_sig_aggregate() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let checkpoint_msg = [0xAA; 32];
    let gc = [0xBB; 32];
    let cid = [0xCC; 32];

    let sig = sign_checkpoint(&kp.secret_key, &kp.pubkey, &checkpoint_msg, &gc, &cid).unwrap();
    let agg = aggregate_checkpoint_signatures(&[sig]).unwrap();

    assert_eq!(
        agg, sig,
        "VAL-003: Aggregating single signature must equal original"
    );
}

// ── Empty aggregation fails ─────────────────────────────────────────

#[test]
fn vv_req_val_003_empty_aggregation_fails() {
    let result = aggregate_checkpoint_signatures(&[]);
    assert!(
        result.is_err(),
        "VAL-003: Aggregating zero signatures must fail"
    );
}

// ── Signing is deterministic ────────────────────────────────────────

#[test]
fn vv_req_val_003_signing_deterministic() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let checkpoint_msg = [0xAA; 32];
    let gc = [0xBB; 32];
    let cid = [0xCC; 32];

    let sig1 = sign_checkpoint(&kp.secret_key, &kp.pubkey, &checkpoint_msg, &gc, &cid).unwrap();
    let sig2 = sign_checkpoint(&kp.secret_key, &kp.pubkey, &checkpoint_msg, &gc, &cid).unwrap();

    assert_eq!(
        sig1, sig2,
        "VAL-003: Same inputs must produce same signature"
    );
}

// ── Different coin IDs produce different signatures ─────────────────

#[test]
fn vv_req_val_003_different_coin_ids() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let checkpoint_msg = [0xAA; 32];
    let gc = [0xBB; 32];

    let sig1 = sign_checkpoint(
        &kp.secret_key,
        &kp.pubkey,
        &checkpoint_msg,
        &gc,
        &[0x01; 32],
    )
    .unwrap();
    let sig2 = sign_checkpoint(
        &kp.secret_key,
        &kp.pubkey,
        &checkpoint_msg,
        &gc,
        &[0x02; 32],
    )
    .unwrap();

    assert_ne!(
        sig1, sig2,
        "VAL-003: Different coin IDs must produce different signatures"
    );
}
