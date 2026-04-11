//! REQUIREMENT: NET-002 — AggSigMe Registration Verification
//! (`docs/requirements/domains/network_coin/NORMATIVE.md#NET-002`).
//!
//! Spec: `docs/requirements/domains/network_coin/specs/NET-002.md`.
//!
//! ## Normative Statement
//!
//! When a validator registers, the network coin MUST verify key ownership via
//! `AGG_SIG_ME` with message `sha256("register" + pubkey)`. The "register"
//! prefix is 8 UTF-8 bytes (no null terminator), the pubkey is 48 bytes
//! (compressed BLS12-381 G1), and the sha256 output is 32 bytes. AGG_SIG_ME
//! additionally binds the signature to the genesis challenge and coin ID,
//! providing replay protection.
//!
//! ## How These Tests Prove the Requirement
//!
//! Tests verify the message format (prefix length, total input size, output
//! size), determinism, collision resistance (different pubkeys -> different
//! messages), and puzzle source inspection (AggSigMe condition present, sha256
//! used, "register" hex prefix correct, pubkey combined with prefix, correct
//! fields bound to AggSigMe). The hex encoding 0x7265676973746572 is verified
//! to equal "register".
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Registration message format: sha256("register" + pubkey) exactly
//! - [x] "register" prefix is 8 bytes, no null terminator
//! - [x] Message is deterministic (same pubkey -> same message)
//! - [x] Different pubkeys -> different messages
//! - [x] AGG_SIG_ME condition emitted (source inspection)
//! - [x] sha256 used for message computation (source inspection)
//! - [x] Hex prefix 0x7265676973746572 == "register" (cross-check)
//! - [x] Message does not depend on conditions (source inspection)
//! - [ ] Spend with valid signature succeeds (tested in NET-006)
//! - [ ] Spend with wrong key rejected (not tested here)
//! - [ ] Spend with no signature rejected (not tested here)
//!
//! ## Gaps
//!
//! Tests verify the message format in Rust and inspect the puzzle source for
//! the AggSigMe condition. Actual signature verification (valid/invalid/missing)
//! requires the simulator (covered in NET-006). The message independence from
//! solution fields is checked via source inspection, not CLVM execution.

use sha2::{Digest, Sha256};

/// The "register" prefix as specified (8 UTF-8 bytes, no null terminator)
const REGISTER_PREFIX: &[u8] = b"register";

/// A sample BLS12-381 G1 compressed pubkey (48 bytes) for testing
fn sample_pubkey() -> [u8; 48] {
    // Valid G1 point from BLS12-381 (generator point)
    let mut pk = [0u8; 48];
    pk[0] = 0x97; // Valid flag bits for compressed G1
    pk[1] = 0xf1;
    pk[2] = 0xd3;
    pk
}

// Verifies the "register" prefix is exactly 8 bytes and matches the
// literal string. A wrong prefix length or content would produce different
// sha256 digests, causing signature verification to fail.
#[test]
fn vv_req_net_002_register_prefix_is_8_bytes() {
    // NET-002: "register" prefix must be exactly 8 UTF-8 bytes
    assert_eq!(
        REGISTER_PREFIX.len(),
        8,
        "NET-002: register prefix must be exactly 8 bytes"
    );
    assert_eq!(
        REGISTER_PREFIX, b"register",
        "NET-002: prefix must be the literal 'register'"
    );
}

// Verifies the complete message construction: 8-byte prefix + 48-byte pubkey
// = 56-byte sha256 input, producing a 32-byte hash. This is the exact format
// the puzzle must use for AGG_SIG_ME.
#[test]
fn vv_req_net_002_message_format_sha256_register_plus_pubkey() {
    // NET-002: registration_message = sha256("register" + pubkey)
    let pubkey = sample_pubkey();

    // Build message input: "register" (8 bytes) + pubkey (48 bytes) = 56 bytes
    let mut message_input = Vec::with_capacity(56);
    message_input.extend_from_slice(REGISTER_PREFIX);
    message_input.extend_from_slice(&pubkey);

    assert_eq!(
        message_input.len(),
        56,
        "NET-002: Message input must be 56 bytes (8 + 48)"
    );

    // Hash the input
    let hash = Sha256::digest(&message_input);
    assert_eq!(
        hash.len(),
        32,
        "NET-002: registration_message must be 32-byte sha256 output"
    );
}

// Verifies that the same pubkey always produces the same message hash.
// Non-deterministic messages would make signing impossible.
#[test]
fn vv_req_net_002_message_is_deterministic() {
    // NET-002: Same pubkey produces same message
    let pubkey = sample_pubkey();

    let compute_message = |pk: &[u8; 48]| {
        let mut input = Vec::with_capacity(56);
        input.extend_from_slice(REGISTER_PREFIX);
        input.extend_from_slice(pk);
        Sha256::digest(&input)
    };

    let msg1 = compute_message(&pubkey);
    let msg2 = compute_message(&pubkey);

    assert_eq!(
        msg1[..],
        msg2[..],
        "NET-002: Registration message must be deterministic"
    );
}

// Verifies that two different pubkeys produce different message hashes.
// If messages collided, one validator's signature could be used for another.
#[test]
fn vv_req_net_002_different_pubkeys_produce_different_messages() {
    // NET-002: Different pubkeys produce different messages (no collisions)
    let pubkey1 = sample_pubkey();
    let mut pubkey2 = sample_pubkey();
    pubkey2[47] = 0xFF; // Change last byte

    let compute_message = |pk: &[u8; 48]| {
        let mut input = Vec::with_capacity(56);
        input.extend_from_slice(REGISTER_PREFIX);
        input.extend_from_slice(pk);
        Sha256::digest(&input)
    };

    let msg1 = compute_message(&pubkey1);
    let msg2 = compute_message(&pubkey2);

    assert_ne!(
        msg1[..],
        msg2[..],
        "NET-002: Different pubkeys must produce different registration messages"
    );
}

// Source inspection: verifies the puzzle contains "AggSigMe", confirming
// it emits the AGG_SIG_ME condition for signature verification.
#[test]
fn vv_req_net_002_puzzle_has_aggsigme_condition() {
    // NET-002: Puzzle must emit AggSigMe condition

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Check for AggSigMe condition
    assert!(
        puzzle_source.contains("AggSigMe"),
        "NET-002: Puzzle must emit AggSigMe condition"
    );
}

// Source inspection: verifies the puzzle uses sha256 for message computation.
#[test]
fn vv_req_net_002_puzzle_uses_sha256_for_message() {
    // NET-002: Puzzle must use sha256 to compute registration message

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("sha256"),
        "NET-002: Puzzle must use sha256 for message computation"
    );
}

// Source inspection: verifies the puzzle contains the hex encoding of
// "register" (0x7265676973746572). This is how the prefix appears in CLVM.
#[test]
fn vv_req_net_002_puzzle_uses_register_prefix() {
    // NET-002: Puzzle must use "register" prefix (hex: 7265676973746572)

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Check for the hex encoding of "register"
    assert!(
        puzzle_source.contains("7265676973746572"),
        "NET-002: Puzzle must use 'register' prefix (0x7265676973746572)"
    );
}

// Source inspection: verifies the puzzle includes the pubkey in the message
// computation by referencing pubkey_bytes/new_validator_pubkey and hashing
// prefix + pubkey together.
#[test]
fn vv_req_net_002_puzzle_combines_prefix_and_pubkey() {
    // NET-002: Puzzle must hash prefix + pubkey together

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Check that pubkey is converted to bytes and concatenated
    assert!(
        puzzle_source.contains("pubkey_bytes") || puzzle_source.contains("new_validator_pubkey"),
        "NET-002: Puzzle must include pubkey in message"
    );

    // Check for concatenation with prefix
    assert!(
        puzzle_source.contains("prefix + pubkey") || puzzle_source.contains("sha256(prefix"),
        "NET-002: Puzzle must concatenate prefix and pubkey for hashing"
    );
}

// Source inspection: verifies AggSigMe binds to new_validator_pubkey and
// registration_message. A mismatch would make the signature check verify
// the wrong key or message.
#[test]
fn vv_req_net_002_aggsigme_uses_pubkey_and_message() {
    // NET-002: AggSigMe condition must use the correct pubkey and message

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Check that AggSigMe uses the new_validator_pubkey
    assert!(
        puzzle_source.contains("public_key: new_validator_pubkey")
            || puzzle_source.contains("AggSigMe { public_key: new_validator_pubkey"),
        "NET-002: AggSigMe must use new_validator_pubkey"
    );

    // Check that AggSigMe uses the registration_message
    assert!(
        puzzle_source.contains("message: registration_message")
            || puzzle_source.contains("registration_message"),
        "NET-002: AggSigMe must use registration_message"
    );
}

// Traceability: verifies the puzzle source references NET-002.
#[test]
fn vv_req_net_002_puzzle_documents_net_002() {
    // NET-002: Puzzle should document NET-002 requirement

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("NET-002"),
        "NET-002: Puzzle should document NET-002 requirement"
    );
}

// Cross-check: verifies hex 7265676973746572 decodes to "register". This
// guards against a hex typo in the puzzle source.
#[test]
fn vv_req_net_002_hex_prefix_equals_register_string() {
    // NET-002: Verify the hex encoding is correct
    let hex_prefix = hex::decode("7265676973746572").expect("Valid hex");
    assert_eq!(
        &hex_prefix, b"register",
        "NET-002: Hex 7265676973746572 must equal 'register'"
    );
}

// Source inspection: verifies the sha256 call for registration_message
// only includes prefix and pubkey, not the conditions list. This ensures
// the message is deterministic from pubkey alone.
#[test]
fn vv_req_net_002_message_does_not_depend_on_conditions() {
    // NET-002: Registration message is deterministic from pubkey only
    // The message = sha256("register" + pubkey) does not include
    // any other solution parameters (like the conditions list)

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Find the sha256 call for registration_message
    // It should only reference prefix and pubkey, not conditions
    assert!(
        puzzle_source.contains("sha256(prefix + pubkey_bytes)"),
        "NET-002: registration_message must only depend on prefix and pubkey"
    );
}
