//! REQUIREMENT: SMT-002 — Slot Assignment
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-002`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-002.md`.
//!
//! **Normative statement:** Each validator's slot in the SMT is computed as
//! `first_8_bytes_as_u64_be(sha256(pubkey)) mod 2^TREE_DEPTH`. The assignment
//! is deterministic: the same pubkey always yields the same slot. Different
//! pubkeys yield different slots with overwhelming probability.
//!
//! **How the tests prove this:**
//! - `slot_from_sha256_pubkey` manually computes the full algorithm and
//!   compares to compute_slot, confirming the implementation matches the spec.
//! - `first_8_bytes_big_endian` ensures big-endian (not little-endian)
//!   interpretation of the first 8 hash bytes.
//! - `result_mod_tree_capacity` checks the output is strictly less than 2^32.
//! - `same_pubkey_same_slot` invokes compute_slot three times on the same key
//!   to confirm determinism.
//! - `different_pubkeys_different_slots` generates 100 distinct keys and
//!   verifies zero collisions (expected given 2^32 slots).
//! - `known_test_vector` pins the output for pubkey=[0x00;48] to the exact
//!   hex value 0x87b081d5, providing a cross-implementation anchor.
//!
//! **Acceptance-criteria coverage (from spec):**
//! - [x] Slot computed from sha256(pubkey)
//! - [x] First 8 bytes interpreted as big-endian u64
//! - [x] Result reduced mod 2^TREE_DEPTH
//! - [x] Same pubkey -> same slot every time
//! - [x] Different pubkeys -> different slots (with high probability)
//! - [ ] Rust and Chialisp compute identical slots (cross-impl; Phase 3)

use chia_l2_consensus::testing::{compute_slot, TREE_DEPTH};
use sha2::{Digest, Sha256};

/// Verifies the slot algorithm end-to-end: sha256 -> first 8 bytes BE u64 -> mod 2^32.
/// Strategy: manual reimplementation of the algorithm compared to compute_slot.
/// Confidence: the library function matches the spec formula exactly.
#[test]
fn vv_req_smt_002_slot_from_sha256_pubkey() {
    // SMT-002: Slot is computed from sha256(pubkey)
    let pubkey = [0x42u8; 48]; // Test pubkey

    // Compute expected slot manually
    let mut hasher = Sha256::new();
    hasher.update(&pubkey);
    let hash: [u8; 32] = hasher.finalize().into();
    let n = u64::from_be_bytes(hash[0..8].try_into().unwrap());
    let expected_slot = n % (1u64 << TREE_DEPTH);

    let actual_slot = compute_slot(&pubkey);

    assert_eq!(
        actual_slot, expected_slot,
        "SMT-002: compute_slot must match manual sha256 computation"
    );
}

/// Verifies that the first 8 bytes of the SHA-256 hash are interpreted as
/// big-endian u64 (not little-endian). Strategy: compute both BE and LE
/// interpretations and confirm compute_slot matches the BE one. An
/// additional sanity check confirms BE != LE for the test hash.
/// Confidence: endianness bugs are the most common slot-computation error;
/// this test catches them directly.
#[test]
fn vv_req_smt_002_first_8_bytes_big_endian() {
    // SMT-002: First 8 bytes interpreted as big-endian u64
    let pubkey = [0x00u8; 48];

    let mut hasher = Sha256::new();
    hasher.update(&pubkey);
    let hash: [u8; 32] = hasher.finalize().into();

    // Verify we're using big-endian interpretation
    let be_value = u64::from_be_bytes(hash[0..8].try_into().unwrap());
    let le_value = u64::from_le_bytes(hash[0..8].try_into().unwrap());

    // These should be different unless hash[0..8] is symmetric
    // The important thing is compute_slot uses big-endian
    let slot = compute_slot(&pubkey);
    let expected = be_value % (1u64 << TREE_DEPTH);

    assert_eq!(
        slot, expected,
        "SMT-002: Slot must use big-endian interpretation of first 8 bytes"
    );

    // Sanity check: BE and LE should typically differ
    if hash[0] != hash[7] {
        assert_ne!(
            be_value, le_value,
            "SMT-002: BE and LE should differ for this hash"
        );
    }
}

/// Verifies the slot is reduced modulo 2^TREE_DEPTH.
/// Strategy: use a pubkey producing a large hash and assert the slot is
/// strictly less than the tree capacity.
/// Confidence: out-of-bounds slots would corrupt tree addressing.
#[test]
fn vv_req_smt_002_result_mod_tree_capacity() {
    // SMT-002: Result reduced mod 2^TREE_DEPTH
    let pubkey = [0xffu8; 48]; // Will produce large hash value

    let slot = compute_slot(&pubkey);

    // Slot must be less than 2^TREE_DEPTH
    let tree_capacity = 1u64 << TREE_DEPTH;
    assert!(
        slot < tree_capacity,
        "SMT-002: Slot {} must be < tree capacity {}",
        slot,
        tree_capacity
    );
}

/// Verifies determinism: the same pubkey always maps to the same slot.
/// Strategy: call compute_slot three times on the same input and compare.
/// Confidence: any non-determinism (e.g. random salt) would fail here.
#[test]
fn vv_req_smt_002_same_pubkey_same_slot() {
    // SMT-002: Same pubkey -> same slot every time
    let pubkey = [0x12u8; 48];

    let slot1 = compute_slot(&pubkey);
    let slot2 = compute_slot(&pubkey);
    let slot3 = compute_slot(&pubkey);

    assert_eq!(slot1, slot2, "SMT-002: Same pubkey must produce same slot");
    assert_eq!(slot2, slot3, "SMT-002: Same pubkey must produce same slot");
}

/// Verifies collision resistance: 100 distinct pubkeys produce 100 distinct
/// slots. Strategy: insert slots into a HashSet and check uniqueness.
/// With 100 keys in a 2^32 space, birthday-bound collision probability is
/// negligible (~1e-6), so this is effectively a deterministic assertion.
/// Confidence: the hash function provides uniform distribution across slots.
#[test]
fn vv_req_smt_002_different_pubkeys_different_slots() {
    // SMT-002: Different pubkeys -> different slots (with high probability)
    // Test with 100 distinct pubkeys - statistical expectation is no collisions

    let mut slots = std::collections::HashSet::new();

    for i in 0u8..100 {
        let mut pubkey = [0u8; 48];
        pubkey[0] = i;
        let slot = compute_slot(&pubkey);
        slots.insert(slot);
    }

    // With 100 pubkeys and 2^32 slots, collision probability is negligible
    assert_eq!(
        slots.len(),
        100,
        "SMT-002: 100 distinct pubkeys should produce 100 distinct slots"
    );
}

/// Pins the slot computation for pubkey=[0x00;48] to the exact value 0x87b081d5.
/// Strategy: hardcoded expected value derived from the spec's walk-through of
/// sha256([0x00;48]). This serves as a cross-implementation anchor: any Rue or
/// CLVM implementation must also produce this value.
/// Confidence: bit-exact regression test guards against silent algorithm changes.
#[test]
fn vv_req_smt_002_known_test_vector() {
    // SMT-002: Verify against a known test vector
    // pubkey = all zeros (48 bytes)
    let pubkey = [0x00u8; 48];

    // sha256([0x00; 48]) = 0x17b0761f87b081d5cf10757ccc89f12be355c70e2e29df288b65b30710dcbcd1
    // First 8 bytes: 0x17b0761f87b081d5
    // As BE u64: 1,706,752,145,447,198,165
    // mod 2^32: 2,276,262,357 (0x87b081d5)

    let expected_slot = 0x87b081d5u64;
    let actual_slot = compute_slot(&pubkey);

    assert_eq!(
        actual_slot, expected_slot,
        "SMT-002: compute_slot([0x00; 48]) must equal 0x87b081d5 ({})",
        expected_slot
    );
}
