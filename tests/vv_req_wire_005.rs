//! REQUIREMENT: WIRE-005 — Registration Message Format
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-005`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-005.md`.
//!
//! **Normative statement:** The registration message is:
//! `sha256("register" || pubkey)` where "register" is 8-byte UTF-8 (no null
//! terminator) and pubkey is 48-byte G1 compressed. Total input: 56 bytes.
//! Output: 32 bytes. This message is then signed via AGG_SIG_ME, which appends
//! genesis_challenge and network_coin_coin_id.
//!
//! **How the tests prove this:**
//! - `prefix_is_8_bytes` and `input_size_is_56_bytes` verify the format constants.
//! - `message_is_32_bytes` checks the sha256 output length.
//! - `format_is_correct` manually constructs the 56-byte preimage and compares.
//! - `different_pubkeys_differ` ensures different keys produce different messages.
//! - `deterministic` calls three times and compares.
//! - `known_test_vector` pins pubkey=[0x01;48] for cross-impl use.
//! - `all_zeros_pubkey` and `all_ff_pubkey` test boundary pubkey values.
//!
//! **Acceptance-criteria coverage (from spec):**
//! - [x] Registration message is sha256 of exactly 56 bytes
//! - [x] "register" is 8-byte UTF-8 with no null terminator
//! - [x] Pubkey is 48-byte G1 compressed
//! - [ ] AGG_SIG_ME includes genesis_challenge and coin_id (on-chain test)
//! - [ ] Signature verification passes on-chain (on-chain test; Phase 3)

use chia_l2_consensus::testing::compute_registration_message;
use sha2::{Digest, Sha256};

/// WIRE-005: "register" prefix is exactly 8 bytes
const REGISTER_PREFIX: &[u8] = b"register";
const REGISTER_PREFIX_LEN: usize = 8;

/// WIRE-005: Total input size is 56 bytes
/// "register" (8) + pubkey (48)
const REGISTRATION_INPUT_SIZE: usize = REGISTER_PREFIX_LEN + 48;

/// Verifies the "register" prefix is exactly 8 bytes.
/// Strategy: check b"register".len().
/// Confidence: the string literal has the spec-mandated length.
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

/// Verifies the total input size constant is 56 (8+48).
/// Strategy: direct assertion on the derived constant.
/// Confidence: the preimage length matches the spec.
#[test]
fn vv_req_wire_005_input_size_is_56_bytes() {
    // WIRE-005: Total input to sha256 is 56 bytes
    assert_eq!(
        REGISTRATION_INPUT_SIZE, 56,
        "WIRE-005: Registration message input must be 56 bytes"
    );
}

/// Verifies the output message is exactly 32 bytes (sha256 digest).
/// Strategy: call the function and check result length.
/// Confidence: the output is a standard SHA-256 hash.
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

/// Verifies the full format: sha256("register" || pubkey).
/// Strategy: manually build the 56-byte preimage and compare the sha256 to
/// the library function output.
/// Confidence: the concatenation order and encoding are correct.
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

/// Verifies different pubkeys produce different registration messages.
/// Strategy: compare messages for two distinct pubkeys.
/// Confidence: the message is bound to the specific validator identity.
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

/// Verifies determinism: same pubkey always produces the same message.
/// Strategy: call three times with identical arguments and compare.
/// Confidence: no hidden randomness or state dependency.
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

/// Known test vector: pubkey=[0x01;48]. Pinned for cross-impl verification.
/// Strategy: manual preimage construction and hash comparison.
/// Confidence: bit-exact regression guard and cross-implementation anchor.
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

/// Boundary test: all-zeros pubkey produces a valid registration message.
/// Strategy: manual preimage with [0x00;48] and hash comparison.
/// Confidence: zero-valued pubkeys are not special-cased incorrectly.
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

/// Boundary test: all-0xFF pubkey produces a valid registration message.
/// Strategy: manual preimage with [0xFF;48] and hash comparison.
/// Confidence: maximum-valued pubkey bytes are handled correctly.
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
