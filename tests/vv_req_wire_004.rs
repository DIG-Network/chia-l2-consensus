//! REQUIREMENT: WIRE-004 — Membership Announcement Format
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-004`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-004.md`.
//!
//! Verifies that membership announcements are formatted as:
//! sha256("membership" || epoch_be8 || pubkey || is_member_byte)

use chia_l2_consensus::testing::compute_membership_announcement_message;
use sha2::{Digest, Sha256};

/// WIRE-004: "membership" prefix is exactly 10 bytes
const MEMBERSHIP_PREFIX: &[u8] = b"membership";
const MEMBERSHIP_PREFIX_LEN: usize = 10;

/// WIRE-004: Total input size is 67 bytes
/// "membership" (10) + epoch (8) + pubkey (48) + is_member (1)
const MEMBERSHIP_INPUT_SIZE: usize = MEMBERSHIP_PREFIX_LEN + 8 + 48 + 1;

#[test]
fn vv_req_wire_004_prefix_is_10_bytes() {
    // WIRE-004: "membership" prefix is exactly 10 bytes UTF-8
    assert_eq!(
        MEMBERSHIP_PREFIX.len(),
        MEMBERSHIP_PREFIX_LEN,
        "WIRE-004: 'membership' must be exactly 10 bytes"
    );
    assert_eq!(
        b"membership".len(),
        10,
        "WIRE-004: 'membership' must be exactly 10 bytes UTF-8"
    );
}

#[test]
fn vv_req_wire_004_input_size_is_67_bytes() {
    // WIRE-004: Total input to sha256 is 67 bytes
    assert_eq!(
        MEMBERSHIP_INPUT_SIZE, 67,
        "WIRE-004: Membership announcement input must be 67 bytes"
    );
}

#[test]
fn vv_req_wire_004_message_is_32_bytes() {
    // WIRE-004: Message is sha256 output (32 bytes)
    let pubkey = [0x11u8; 48];
    let epoch = 100u64;
    let is_member = true;

    let message = compute_membership_announcement_message(epoch, &pubkey, is_member);

    assert_eq!(
        message.len(),
        32,
        "WIRE-004: Membership announcement message must be 32 bytes"
    );
}

#[test]
fn vv_req_wire_004_format_is_correct() {
    // WIRE-004: Format is sha256("membership" || epoch_be8 || pubkey || is_member)
    let pubkey = [0xAAu8; 48];
    let epoch = 42u64;
    let is_member = true;

    let message = compute_membership_announcement_message(epoch, &pubkey, is_member);

    // Manual computation
    let mut input = Vec::with_capacity(MEMBERSHIP_INPUT_SIZE);
    input.extend_from_slice(b"membership"); // 10 bytes
    input.extend_from_slice(&epoch.to_be_bytes()); // 8 bytes
    input.extend_from_slice(&pubkey); // 48 bytes
    input.push(0x01); // is_member = true

    assert_eq!(input.len(), 67, "Input should be 67 bytes");

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        message, expected,
        "WIRE-004: Message must match manual computation"
    );
}

#[test]
fn vv_req_wire_004_is_member_true() {
    // WIRE-004: is_member = true encoded as 0x01
    let pubkey = [0x33u8; 48];
    let epoch = 5u64;

    let message = compute_membership_announcement_message(epoch, &pubkey, true);

    // Manual computation with is_member = 0x01
    let mut input = Vec::with_capacity(67);
    input.extend_from_slice(b"membership");
    input.extend_from_slice(&epoch.to_be_bytes());
    input.extend_from_slice(&pubkey);
    input.push(0x01);

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(message, expected, "WIRE-004: is_member=true must use 0x01");
}

#[test]
fn vv_req_wire_004_is_member_false() {
    // WIRE-004: is_member = false encoded as 0x00
    let pubkey = [0x33u8; 48];
    let epoch = 5u64;

    let message = compute_membership_announcement_message(epoch, &pubkey, false);

    // Manual computation with is_member = 0x00
    let mut input = Vec::with_capacity(67);
    input.extend_from_slice(b"membership");
    input.extend_from_slice(&epoch.to_be_bytes());
    input.extend_from_slice(&pubkey);
    input.push(0x00);

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(message, expected, "WIRE-004: is_member=false must use 0x00");
}

#[test]
fn vv_req_wire_004_different_membership_values_differ() {
    // WIRE-004: is_member=true and is_member=false produce different messages
    let pubkey = [0x55u8; 48];
    let epoch = 10u64;

    let msg_true = compute_membership_announcement_message(epoch, &pubkey, true);
    let msg_false = compute_membership_announcement_message(epoch, &pubkey, false);

    assert_ne!(
        msg_true, msg_false,
        "WIRE-004: Member and non-member messages must differ"
    );
}

#[test]
fn vv_req_wire_004_epoch_is_big_endian() {
    // WIRE-004: Epoch is 8-byte big-endian
    let pubkey = [0x00u8; 48];
    let epoch = 256u64; // 0x100, verifies byte order
    let is_member = true;

    let message = compute_membership_announcement_message(epoch, &pubkey, is_member);

    // Manual with explicit big-endian
    let mut input = Vec::with_capacity(67);
    input.extend_from_slice(b"membership");
    input.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00]); // 256 BE
    input.extend_from_slice(&pubkey);
    input.push(0x01);

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(message, expected, "WIRE-004: Epoch must be big-endian");
}

#[test]
fn vv_req_wire_004_different_epochs_differ() {
    // WIRE-004: Different epochs produce different messages (replay protection)
    let pubkey = [0xAAu8; 48];

    let msg_epoch1 = compute_membership_announcement_message(1, &pubkey, true);
    let msg_epoch2 = compute_membership_announcement_message(2, &pubkey, true);

    assert_ne!(
        msg_epoch1, msg_epoch2,
        "WIRE-004: Different epochs must produce different messages"
    );
}

#[test]
fn vv_req_wire_004_different_pubkeys_differ() {
    // WIRE-004: Different pubkeys produce different messages
    let pubkey1 = [0x11u8; 48];
    let pubkey2 = [0x22u8; 48];
    let epoch = 5u64;

    let msg1 = compute_membership_announcement_message(epoch, &pubkey1, true);
    let msg2 = compute_membership_announcement_message(epoch, &pubkey2, true);

    assert_ne!(
        msg1, msg2,
        "WIRE-004: Different pubkeys must produce different messages"
    );
}

#[test]
fn vv_req_wire_004_deterministic() {
    // WIRE-004: Same inputs always produce same output
    let pubkey = [0x77u8; 48];
    let epoch = 99u64;

    let msg1 = compute_membership_announcement_message(epoch, &pubkey, true);
    let msg2 = compute_membership_announcement_message(epoch, &pubkey, true);
    let msg3 = compute_membership_announcement_message(epoch, &pubkey, true);

    assert_eq!(msg1, msg2, "WIRE-004: Must be deterministic");
    assert_eq!(msg2, msg3, "WIRE-004: Must be deterministic");
}

#[test]
fn vv_req_wire_004_known_test_vector() {
    // WIRE-004: Known test vector for cross-implementation verification
    let pubkey = [0x01u8; 48];
    let epoch = 10u64;
    let is_member = true;

    let message = compute_membership_announcement_message(epoch, &pubkey, is_member);

    // Compute expected
    let mut input = Vec::with_capacity(67);
    input.extend_from_slice(b"membership");
    input.extend_from_slice(&10u64.to_be_bytes());
    input.extend_from_slice(&[0x01u8; 48]);
    input.push(0x01);

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(message, expected, "WIRE-004: Known test vector must match");

    // Print for cross-impl verification
    eprintln!("WIRE-004 test vector (member): {:02x?}", message);
}
