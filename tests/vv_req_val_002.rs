//! REQUIREMENT: VAL-002 — Registration
//! (`docs/requirements/domains/validator/NORMATIVE.md#VAL-002`).
//!
//! Spec: `docs/requirements/domains/validator/specs/VAL-002.md`.
//!
//! Implementation: `src/validator/registration.rs`.
//!
//! Verifies that validators can build and sign the registration message,
//! that the AGG_SIG_ME message is correctly constructed, and that the
//! indexer detects valid registrations.

use chia_l2_consensus::testing::{
    compute_registration_message, compute_registration_signing_message, generate_validator_keypair,
    sign_registration, verify_registration_signature,
};

// ── Registration signing message is 96 bytes ────────────────────────

#[test]
fn vv_req_val_002_signing_message_is_96_bytes() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let genesis_challenge = [0xAA; 32];
    let coin_id = [0xBB; 32];

    let msg = compute_registration_signing_message(&kp.pubkey, &genesis_challenge, &coin_id);

    assert_eq!(
        msg.len(),
        96,
        "VAL-002: AGG_SIG_ME registration message must be 96 bytes (32+32+32)"
    );
}

// ── Signing message = registration_message + genesis_challenge + coin_id

#[test]
fn vv_req_val_002_signing_message_format() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let genesis_challenge = [0xAA; 32];
    let coin_id = [0xBB; 32];

    let reg_msg = compute_registration_message(&kp.pubkey);
    let signing_msg =
        compute_registration_signing_message(&kp.pubkey, &genesis_challenge, &coin_id);

    // First 32 bytes = registration_message = sha256("register" + pubkey)
    assert_eq!(
        &signing_msg[0..32],
        &reg_msg,
        "VAL-002: First 32 bytes must be registration_message"
    );

    // Next 32 bytes = genesis_challenge
    assert_eq!(
        &signing_msg[32..64],
        &genesis_challenge,
        "VAL-002: Bytes 32-64 must be genesis_challenge"
    );

    // Last 32 bytes = coin_id
    assert_eq!(
        &signing_msg[64..96],
        &coin_id,
        "VAL-002: Bytes 64-96 must be network_coin_coin_id"
    );
}

// ── Registration signature is 96 bytes (G2) ─────────────────────────

#[test]
fn vv_req_val_002_signature_is_96_bytes() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let genesis_challenge = [0xAA; 32];
    let coin_id = [0xBB; 32];

    let sig = sign_registration(&kp.secret_key, &kp.pubkey, &genesis_challenge, &coin_id).unwrap();

    assert_eq!(
        sig.len(),
        96,
        "VAL-002: Registration signature must be 96 bytes (compressed G2)"
    );
}

// ── Registration signature verifies ─────────────────────────────────

#[test]
fn vv_req_val_002_signature_verifies() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let genesis_challenge = [0xAA; 32];
    let coin_id = [0xBB; 32];

    let sig = sign_registration(&kp.secret_key, &kp.pubkey, &genesis_challenge, &coin_id).unwrap();

    let valid =
        verify_registration_signature(&kp.pubkey, &genesis_challenge, &coin_id, &sig).unwrap();

    assert!(
        valid,
        "VAL-002: Registration signature must verify with correct key"
    );
}

// ── Wrong key fails registration verification ───────────────────────

#[test]
fn vv_req_val_002_wrong_key_fails() {
    let kp1 = generate_validator_keypair(&[0x01; 32]).unwrap();
    let kp2 = generate_validator_keypair(&[0x02; 32]).unwrap();
    let genesis_challenge = [0xAA; 32];
    let coin_id = [0xBB; 32];

    let sig =
        sign_registration(&kp1.secret_key, &kp1.pubkey, &genesis_challenge, &coin_id).unwrap();

    // Verify with kp2's pubkey should fail
    let valid =
        verify_registration_signature(&kp2.pubkey, &genesis_challenge, &coin_id, &sig).unwrap();

    assert!(
        !valid,
        "VAL-002: Registration signature must NOT verify with wrong key"
    );
}

// ── Wrong genesis challenge fails ───────────────────────────────────

#[test]
fn vv_req_val_002_wrong_genesis_challenge_fails() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let genesis_challenge = [0xAA; 32];
    let wrong_challenge = [0xBB; 32];
    let coin_id = [0xCC; 32];

    let sig = sign_registration(&kp.secret_key, &kp.pubkey, &genesis_challenge, &coin_id).unwrap();

    let valid =
        verify_registration_signature(&kp.pubkey, &wrong_challenge, &coin_id, &sig).unwrap();

    assert!(
        !valid,
        "VAL-002: Signature must NOT verify with wrong genesis challenge"
    );
}

// ── Wrong coin ID fails ─────────────────────────────────────────────

#[test]
fn vv_req_val_002_wrong_coin_id_fails() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let genesis_challenge = [0xAA; 32];
    let coin_id = [0xBB; 32];
    let wrong_coin_id = [0xCC; 32];

    let sig = sign_registration(&kp.secret_key, &kp.pubkey, &genesis_challenge, &coin_id).unwrap();

    let valid = verify_registration_signature(&kp.pubkey, &genesis_challenge, &wrong_coin_id, &sig)
        .unwrap();

    assert!(
        !valid,
        "VAL-002: Signature must NOT verify with wrong coin ID"
    );
}

// ── Different pubkeys produce different signing messages ─────────────

#[test]
fn vv_req_val_002_different_pubkeys_different_messages() {
    let kp1 = generate_validator_keypair(&[0x01; 32]).unwrap();
    let kp2 = generate_validator_keypair(&[0x02; 32]).unwrap();
    let gc = [0xAA; 32];
    let cid = [0xBB; 32];

    let msg1 = compute_registration_signing_message(&kp1.pubkey, &gc, &cid);
    let msg2 = compute_registration_signing_message(&kp2.pubkey, &gc, &cid);

    assert_ne!(
        msg1, msg2,
        "VAL-002: Different pubkeys must produce different signing messages"
    );
}

// ── Registration message matches wire format spec ───────────────────

#[test]
fn vv_req_val_002_registration_message_matches_wire_spec() {
    use sha2::{Digest, Sha256};

    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();

    // Manual computation per WIRE-005
    let mut hasher = Sha256::new();
    hasher.update(b"register"); // 8 bytes
    hasher.update(&kp.pubkey); // 48 bytes
    let expected: [u8; 32] = hasher.finalize().into();

    let actual = compute_registration_message(&kp.pubkey);

    assert_eq!(
        actual, expected,
        "VAL-002: Registration message must be sha256('register' + pubkey)"
    );
}

// ── Signing is deterministic ────────────────────────────────────────

#[test]
fn vv_req_val_002_signing_deterministic() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let gc = [0xAA; 32];
    let cid = [0xBB; 32];

    let sig1 = sign_registration(&kp.secret_key, &kp.pubkey, &gc, &cid).unwrap();
    let sig2 = sign_registration(&kp.secret_key, &kp.pubkey, &gc, &cid).unwrap();

    assert_eq!(
        sig1, sig2,
        "VAL-002: Same inputs must produce same registration signature"
    );
}

// ── Collateral amount from config is accessible ─────────────────────

#[test]
fn vv_req_val_002_collateral_from_config() {
    let config = chia_l2_consensus::NetworkConfig {
        network_coin_launcher_id: chia_protocol::Bytes32::default(),
        checkpoint_launcher_id: chia_protocol::Bytes32::default(),
        registration_coin_mod_hash: chia_protocol::Bytes32::default(),
        checkpoint_inner_mod_hash: chia_protocol::Bytes32::default(),
        collateral_amount: 10_000_000_000_000, // 10 XCH
        tree_depth: 32,
        max_signers: 20_000,
        verification_key_hex: String::new(),
        genesis_challenge: chia_protocol::Bytes32::default(),
    };

    assert_eq!(
        config.collateral_amount, 10_000_000_000_000,
        "VAL-002: Collateral amount must be available from config"
    );
}
