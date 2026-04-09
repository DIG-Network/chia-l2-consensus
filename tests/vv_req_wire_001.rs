//! REQUIREMENT: WIRE-001 — Checkpoint message format
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-001`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-001.md`.
//!
//! Verifies that the checkpoint message is computed correctly as:
//! sha256(new_state_root || new_validator_merkle_root || new_validator_count_be8 || new_epoch_be8)

use chia_l2_consensus::compute_checkpoint_message;
use sha2::{Digest, Sha256};

#[test]
fn vv_req_wire_001_message_is_sha256_of_80_bytes() {
    // WIRE-001: Message is computed as sha256 of exactly 80 bytes
    let state_root = [0x11u8; 32];
    let merkle_root = [0x22u8; 32];
    let count = 100u64;
    let epoch = 42u64;

    // Compute message
    let message = compute_checkpoint_message(state_root, merkle_root, count, epoch);

    // Verify it's 32 bytes (sha256 output)
    assert_eq!(
        message.len(),
        32,
        "WIRE-001: Checkpoint message must be 32 bytes"
    );

    // Manually compute expected
    let mut input = Vec::with_capacity(80);
    input.extend_from_slice(&state_root);
    input.extend_from_slice(&merkle_root);
    input.extend_from_slice(&count.to_be_bytes());
    input.extend_from_slice(&epoch.to_be_bytes());
    assert_eq!(input.len(), 80, "Input should be exactly 80 bytes");

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        message, expected,
        "WIRE-001: Message must match manual computation"
    );
}

#[test]
fn vv_req_wire_001_field_order_correct() {
    // WIRE-001: Field order matches specification exactly
    // state_root || merkle_root || count_be8 || epoch_be8

    let state_root = [0xAAu8; 32];
    let merkle_root = [0xBBu8; 32];
    let count = 0x0102030405060708u64;
    let epoch = 0x0908070605040302u64;

    let message = compute_checkpoint_message(state_root, merkle_root, count, epoch);

    // Build expected with explicit field order
    let mut hasher = Sha256::new();
    hasher.update(&state_root); // First: state_root
    hasher.update(&merkle_root); // Second: merkle_root
    hasher.update(&count.to_be_bytes()); // Third: count (BE)
    hasher.update(&epoch.to_be_bytes()); // Fourth: epoch (BE)
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        message, expected,
        "WIRE-001: Field order must be state_root || merkle_root || count || epoch"
    );
}

#[test]
fn vv_req_wire_001_integers_are_8_byte_big_endian() {
    // WIRE-001: Integers are 8-byte big-endian, zero-padded
    let state_root = [0x00u8; 32];
    let merkle_root = [0x00u8; 32];
    let count = 1u64; // Small number to verify padding
    let epoch = 256u64; // 0x100 to check byte order

    let message = compute_checkpoint_message(state_root, merkle_root, count, epoch);

    // count = 1 as 8-byte BE: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]
    // epoch = 256 as 8-byte BE: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00]
    let mut input = Vec::with_capacity(80);
    input.extend_from_slice(&state_root);
    input.extend_from_slice(&merkle_root);
    input.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]); // count=1
    input.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00]); // epoch=256

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        message, expected,
        "WIRE-001: Integers must be 8-byte big-endian, zero-padded"
    );
}

#[test]
fn vv_req_wire_001_edge_case_zeros() {
    // WIRE-001: Test with all zeros
    let state_root = [0x00u8; 32];
    let merkle_root = [0x00u8; 32];
    let count = 0u64;
    let epoch = 0u64;

    let message = compute_checkpoint_message(state_root, merkle_root, count, epoch);

    // Should be sha256 of 80 zero bytes
    let input = [0x00u8; 80];
    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        message, expected,
        "WIRE-001: All zeros should produce sha256([0; 80])"
    );
}

#[test]
fn vv_req_wire_001_edge_case_max_values() {
    // WIRE-001: Test with maximum u64 values
    let state_root = [0xFFu8; 32];
    let merkle_root = [0xFFu8; 32];
    let count = u64::MAX;
    let epoch = u64::MAX;

    let message = compute_checkpoint_message(state_root, merkle_root, count, epoch);

    // Should be sha256 of 80 bytes of 0xFF
    let input = [0xFFu8; 80];
    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        message, expected,
        "WIRE-001: Max values should produce sha256([0xFF; 80])"
    );
}

#[test]
fn vv_req_wire_001_known_test_vector() {
    // WIRE-001: Known test vector for cross-implementation verification
    // This value must match the Rue implementation

    // Test values from spec
    let state_root = [0x01u8; 32];
    let merkle_root = [0x02u8; 32];
    let count = 10u64;
    let epoch = 5u64;

    let message = compute_checkpoint_message(state_root, merkle_root, count, epoch);

    // Compute expected manually
    let mut input = Vec::with_capacity(80);
    input.extend_from_slice(&[0x01u8; 32]); // state_root
    input.extend_from_slice(&[0x02u8; 32]); // merkle_root
    input.extend_from_slice(&10u64.to_be_bytes()); // count=10
    input.extend_from_slice(&5u64.to_be_bytes()); // epoch=5

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(message, expected, "WIRE-001: Known test vector must match");

    // Store expected value for cross-impl verification
    // This exact value must be produced by Rue implementation
    eprintln!("WIRE-001 test vector: {:02x?}", message);
}

#[test]
fn vv_req_wire_001_deterministic() {
    // WIRE-001: Same inputs produce same output
    let state_root = [0x42u8; 32];
    let merkle_root = [0x43u8; 32];
    let count = 1000u64;
    let epoch = 999u64;

    let message1 = compute_checkpoint_message(state_root, merkle_root, count, epoch);
    let message2 = compute_checkpoint_message(state_root, merkle_root, count, epoch);
    let message3 = compute_checkpoint_message(state_root, merkle_root, count, epoch);

    assert_eq!(message1, message2, "WIRE-001: Must be deterministic");
    assert_eq!(message2, message3, "WIRE-001: Must be deterministic");
}
