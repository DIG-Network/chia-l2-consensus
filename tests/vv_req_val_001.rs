//! REQUIREMENT: VAL-001 — Key Generation
//! (`docs/requirements/domains/validator/NORMATIVE.md#VAL-001`).
//!
//! Spec: `docs/requirements/domains/validator/specs/VAL-001.md`.
//!
//! Implementation: `src/validator/keygen.rs`.
//!
//! Verifies that validators can generate BLS12-381 keypairs with 48-byte
//! compressed G1 public keys, sign and verify messages, and that different
//! entropy produces different keys.

use chia_l2_consensus::testing::{
    generate_validator_keypair, pubkey_from_secret, sign_message, verify_signature,
};

// ── Keypair generation succeeds ─────────────────────────────────────

#[test]
fn vv_req_val_001_keygen_succeeds() {
    let kp = generate_validator_keypair(&[0x42; 32]);
    assert!(kp.is_ok(), "VAL-001: Keypair generation must succeed");
}

// ── Public key is 48 bytes (compressed G1) ──────────────────────────

#[test]
fn vv_req_val_001_pubkey_is_48_bytes() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();

    assert_eq!(
        kp.pubkey.len(),
        48,
        "VAL-001: Public key must be 48 bytes (compressed BLS12-381 G1)"
    );
}

// ── Secret key is 32 bytes ──────────────────────────────────────────

#[test]
fn vv_req_val_001_secret_key_is_32_bytes() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();

    assert_eq!(
        kp.secret_key.len(),
        32,
        "VAL-001: Secret key must be 32 bytes"
    );
}

// ── Public key is non-zero ──────────────────────────────────────────

#[test]
fn vv_req_val_001_pubkey_nonzero() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();

    assert!(
        !kp.pubkey.iter().all(|&b| b == 0),
        "VAL-001: Public key must not be all zeros"
    );
}

// ── Public key has valid compression flag ────────────────────────────

#[test]
fn vv_req_val_001_pubkey_compression_flag() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();

    // BLS12-381 compressed G1: bit 7 of first byte is compression flag (must be 1)
    let first_byte = kp.pubkey[0];
    assert!(
        first_byte & 0x80 != 0,
        "VAL-001: Compressed G1 point must have bit 7 set (compression flag). Got 0x{:02x}",
        first_byte
    );
}

// ── Different entropy produces different keys ───────────────────────

#[test]
fn vv_req_val_001_different_entropy_different_keys() {
    let kp1 = generate_validator_keypair(&[0x01; 32]).unwrap();
    let kp2 = generate_validator_keypair(&[0x02; 32]).unwrap();

    assert_ne!(
        kp1.pubkey, kp2.pubkey,
        "VAL-001: Different entropy must produce different public keys"
    );
    assert_ne!(
        kp1.secret_key, kp2.secret_key,
        "VAL-001: Different entropy must produce different secret keys"
    );
}

// ── Same entropy produces same keys (deterministic) ─────────────────

#[test]
fn vv_req_val_001_deterministic() {
    let kp1 = generate_validator_keypair(&[0x42; 32]).unwrap();
    let kp2 = generate_validator_keypair(&[0x42; 32]).unwrap();

    assert_eq!(
        kp1.pubkey, kp2.pubkey,
        "VAL-001: Same entropy must produce same public key"
    );
    assert_eq!(
        kp1.secret_key, kp2.secret_key,
        "VAL-001: Same entropy must produce same secret key"
    );
}

// ── Public key derived from secret key ──────────────────────────────

#[test]
fn vv_req_val_001_pubkey_from_secret() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();

    let derived_pk = pubkey_from_secret(&kp.secret_key).unwrap();

    assert_eq!(
        kp.pubkey, derived_pk,
        "VAL-001: Public key must be derivable from secret key"
    );
}

// ── Sign and verify roundtrip ───────────────────────────────────────

#[test]
fn vv_req_val_001_sign_verify_roundtrip() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let message = b"test checkpoint message";

    let sig = sign_message(&kp.secret_key, message).unwrap();

    assert_eq!(
        sig.len(),
        96,
        "VAL-001: BLS signature must be 96 bytes (G2)"
    );

    let valid = verify_signature(&kp.pubkey, message, &sig).unwrap();
    assert!(valid, "VAL-001: Signature must verify with correct key");
}

// ── Signature fails with wrong key ──────────────────────────────────

#[test]
fn vv_req_val_001_wrong_key_fails() {
    let kp1 = generate_validator_keypair(&[0x01; 32]).unwrap();
    let kp2 = generate_validator_keypair(&[0x02; 32]).unwrap();
    let message = b"test message";

    let sig = sign_message(&kp1.secret_key, message).unwrap();

    let valid = verify_signature(&kp2.pubkey, message, &sig).unwrap();
    assert!(!valid, "VAL-001: Signature must NOT verify with wrong key");
}

// ── Signature fails with wrong message ──────────────────────────────

#[test]
fn vv_req_val_001_wrong_message_fails() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();

    let sig = sign_message(&kp.secret_key, b"message A").unwrap();

    let valid = verify_signature(&kp.pubkey, b"message B", &sig).unwrap();
    assert!(
        !valid,
        "VAL-001: Signature must NOT verify with wrong message"
    );
}

// ── Minimum entropy length check ────────────────────────────────────

#[test]
fn vv_req_val_001_entropy_too_short_fails() {
    let result = generate_validator_keypair(&[0x42; 16]); // Only 16 bytes
    assert!(
        result.is_err(),
        "VAL-001: Entropy shorter than 32 bytes must fail"
    );
}
