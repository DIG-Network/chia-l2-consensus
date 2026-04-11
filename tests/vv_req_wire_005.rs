//! REQUIREMENT: WIRE-005 — Registration Message Format
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-005`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-005.md`.
//!
//! Verifies that the registration message for AGG_SIG_ME is:
//! sha256("register" || pubkey) where "register" is 8-byte UTF-8.

use chia_l2_consensus::testing::compute_registration_message;
use sha2::{Digest, Sha256};

/// WIRE-005: "register" prefix is exactly 8 bytes
const REGISTER_PREFIX: &[u8] = b"register";
const REGISTER_PREFIX_LEN: usize = 8;

/// WIRE-005: Total input size is 56 bytes
/// "register" (8) + pubkey (48)
const REGISTRATION_INPUT_SIZE: usize = REGISTER_PREFIX_LEN + 48;

#[test]
fn vv_req_wire_005_prefix_is_8_bytes() {
    // WIRE-005: "register" prefix is exactly 8 bytes UTF-8
    assert_eq!(
        REGISTER_PREFIX.len(),
        REGISTER_PREFIX_LEN,
        "WIRE-005: 'register' must be exactly 8 bytes"
    );
    assert_eq!(
        b"register".len(),
        8,
        "WIRE-005: 'register' must be exactly 8 bytes UTF-8"
    );
}

#[test]
fn vv_req_wire_005_input_size_is_56_bytes() {
    // WIRE-005: Total input to sha256 is 56 bytes
    assert_eq!(
        REGISTRATION_INPUT_SIZE, 56,
        "WIRE-005: Registration message input must be 56 bytes"
    );
}

#[test]
fn vv_req_wire_005_message_is_32_bytes() {
    // WIRE-005: Message is sha256 output (32 bytes)
    let pubkey = [0x11u8; 48];

    let message = compute_registration_message(&pubkey);

    assert_eq!(
        message.len(),
        32,
        "WIRE-005: Registration message must be 32 bytes"
    );
}

#[test]
fn vv_req_wire_005_format_is_correct() {
    // WIRE-005: Format is sha256("register" || pubkey)
    let pubkey = [0xAAu8; 48];

    let message = compute_registration_message(&pubkey);

    // Manual computation
    let mut input = Vec::with_capacity(REGISTRATION_INPUT_SIZE);
    input.extend_from_slice(b"register"); // 8 bytes
    input.extend_from_slice(&pubkey); // 48 bytes

    assert_eq!(input.len(), 56, "Input should be 56 bytes");

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        message, expected,
        "WIRE-005: Message must match manual computation"
    );
}

#[test]
fn vv_req_wire_005_different_pubkeys_differ() {
    // WIRE-005: Different pubkeys produce different messages
    let pubkey1 = [0x11u8; 48];
    let pubkey2 = [0x22u8; 48];

    let msg1 = compute_registration_message(&pubkey1);
    let msg2 = compute_registration_message(&pubkey2);

    assert_ne!(
        msg1, msg2,
        "WIRE-005: Different pubkeys must produce different messages"
    );
}

#[test]
fn vv_req_wire_005_deterministic() {
    // WIRE-005: Same inputs always produce same output
    let pubkey = [0x77u8; 48];

    let msg1 = compute_registration_message(&pubkey);
    let msg2 = compute_registration_message(&pubkey);
    let msg3 = compute_registration_message(&pubkey);

    assert_eq!(msg1, msg2, "WIRE-005: Must be deterministic");
    assert_eq!(msg2, msg3, "WIRE-005: Must be deterministic");
}

#[test]
fn vv_req_wire_005_known_test_vector() {
    // WIRE-005: Known test vector for cross-implementation verification
    let pubkey = [0x01u8; 48];

    let message = compute_registration_message(&pubkey);

    // Compute expected
    let mut input = Vec::with_capacity(56);
    input.extend_from_slice(b"register");
    input.extend_from_slice(&[0x01u8; 48]);

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(message, expected, "WIRE-005: Known test vector must match");

    // Print for cross-impl verification
    eprintln!("WIRE-005 test vector: {:02x?}", message);
}

#[test]
fn vv_req_wire_005_all_zeros_pubkey() {
    // WIRE-005: Test with all zeros pubkey
    let pubkey = [0x00u8; 48];

    let message = compute_registration_message(&pubkey);

    // sha256("register" || [0; 48])
    let mut input = Vec::with_capacity(56);
    input.extend_from_slice(b"register");
    input.extend_from_slice(&[0x00u8; 48]);

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(message, expected, "WIRE-005: Zero pubkey must work");
}

#[test]
fn vv_req_wire_005_all_ff_pubkey() {
    // WIRE-005: Test with all 0xFF pubkey
    let pubkey = [0xFFu8; 48];

    let message = compute_registration_message(&pubkey);

    // sha256("register" || [0xFF; 48])
    let mut input = Vec::with_capacity(56);
    input.extend_from_slice(b"register");
    input.extend_from_slice(&[0xFFu8; 48]);

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(message, expected, "WIRE-005: 0xFF pubkey must work");
}
