//! REQUIREMENT: SEC-005 — Lineage Enforcement
//! (`docs/requirements/domains/security/NORMATIVE.md#SEC-005`).
//!
//! Spec: `docs/requirements/domains/security/specs/SEC-005.md`.
//!
//! Implementation: `src/indexer/validator_set.rs`.
//!
//! Security verification: confirms that registration coin lineage is
//! enforced by the indexer, that coins without valid lineage are excluded,
//! and that puzzle hash alone is insufficient for inclusion.

use chia_l2_consensus::testing::{
    registration_coin_puzzle_hash, try_parse_registration_coin, LineageChecker,
};
use chia_protocol::{Bytes32, Coin};

/// Test pubkey (48 bytes).
fn test_pk() -> [u8; 48] {
    let mut pk = [0u8; 48];
    pk[0] = 0x80; // Valid BLS compression flag
    pk[1] = 0x42;
    pk
}

/// Test mod hash, checkpoint ID, collateral.
fn test_params() -> (Bytes32, Bytes32, u64) {
    let mod_hash = Bytes32::from([0xAA; 32]);
    let ckpt_id = Bytes32::from([0xBB; 32]);
    let collateral = 10_000_000_000_000u64;
    (mod_hash, ckpt_id, collateral)
}

// ── Valid parent accepted ───────────────────────────────────────────

#[test]
fn vv_req_sec_005_valid_parent_accepted() {
    let mut checker = LineageChecker::new();
    let network_coin_id = Bytes32::from([0x11; 32]);
    checker.record_network_coin_spend(network_coin_id);

    let pk = test_pk();
    let (mod_hash, ckpt_id, collateral) = test_params();
    let expected_ph = registration_coin_puzzle_hash(mod_hash, &pk, ckpt_id);

    let coin = Coin::new(network_coin_id, expected_ph, collateral);

    let result = try_parse_registration_coin(&checker, &coin, &pk, mod_hash, ckpt_id, collateral);
    assert!(
        result.is_some(),
        "SEC-005: Coin with valid network coin parent must be accepted"
    );
}

// ── Invalid parent rejected (core security property) ────────────────

#[test]
fn vv_req_sec_005_invalid_parent_rejected() {
    let mut checker = LineageChecker::new();
    let network_coin_id = Bytes32::from([0x11; 32]);
    checker.record_network_coin_spend(network_coin_id);

    let pk = test_pk();
    let (mod_hash, ckpt_id, collateral) = test_params();
    let expected_ph = registration_coin_puzzle_hash(mod_hash, &pk, ckpt_id);

    // Coin with WRONG parent (not a network coin spend)
    let fake_parent = Bytes32::from([0xFF; 32]);
    let fake_coin = Coin::new(fake_parent, expected_ph, collateral);

    let result =
        try_parse_registration_coin(&checker, &fake_coin, &pk, mod_hash, ckpt_id, collateral);
    assert!(
        result.is_none(),
        "SEC-005: Coin with invalid parent MUST be rejected (lineage enforcement)"
    );
}

// ── Correct puzzle hash alone is insufficient ───────────────────────

#[test]
fn vv_req_sec_005_puzzle_hash_alone_insufficient() {
    // Empty checker — no network coin spends recorded
    let checker = LineageChecker::new();

    let pk = test_pk();
    let (mod_hash, ckpt_id, collateral) = test_params();
    let expected_ph = registration_coin_puzzle_hash(mod_hash, &pk, ckpt_id);

    // Coin with correct puzzle hash but no valid parent
    let random_parent = Bytes32::from([0x99; 32]);
    let coin = Coin::new(random_parent, expected_ph, collateral);

    let result = try_parse_registration_coin(&checker, &coin, &pk, mod_hash, ckpt_id, collateral);
    assert!(
        result.is_none(),
        "SEC-005: Correct puzzle hash alone MUST NOT be sufficient — lineage is required"
    );
}

// ── Wrong puzzle hash rejected even with valid parent ───────────────

#[test]
fn vv_req_sec_005_wrong_puzzle_hash_rejected() {
    let mut checker = LineageChecker::new();
    let network_coin_id = Bytes32::from([0x11; 32]);
    checker.record_network_coin_spend(network_coin_id);

    let pk = test_pk();
    let (mod_hash, ckpt_id, collateral) = test_params();

    // Coin with valid parent but WRONG puzzle hash
    let wrong_ph = Bytes32::from([0xDE; 32]);
    let coin = Coin::new(network_coin_id, wrong_ph, collateral);

    let result = try_parse_registration_coin(&checker, &coin, &pk, mod_hash, ckpt_id, collateral);
    assert!(
        result.is_none(),
        "SEC-005: Wrong puzzle hash must be rejected even with valid parent"
    );
}

// ── Wrong collateral amount rejected ────────────────────────────────

#[test]
fn vv_req_sec_005_wrong_collateral_rejected() {
    let mut checker = LineageChecker::new();
    let network_coin_id = Bytes32::from([0x11; 32]);
    checker.record_network_coin_spend(network_coin_id);

    let pk = test_pk();
    let (mod_hash, ckpt_id, collateral) = test_params();
    let expected_ph = registration_coin_puzzle_hash(mod_hash, &pk, ckpt_id);

    // Coin with valid parent and puzzle hash but WRONG amount
    let wrong_amount = collateral - 1;
    let coin = Coin::new(network_coin_id, expected_ph, wrong_amount);

    let result = try_parse_registration_coin(&checker, &coin, &pk, mod_hash, ckpt_id, collateral);
    assert!(
        result.is_none(),
        "SEC-005: Wrong collateral amount must be rejected"
    );
}

// ── Empty checker rejects everything ────────────────────────────────

#[test]
fn vv_req_sec_005_empty_checker_rejects_all() {
    let checker = LineageChecker::new();

    assert_eq!(
        checker.network_coin_spend_count(),
        0,
        "SEC-005: Empty checker has no spends"
    );

    // Any parent is unknown
    assert!(
        !checker.is_network_coin_spend(&Bytes32::from([0x00; 32])),
        "SEC-005: Empty checker must reject all parents"
    );
    assert!(
        !checker.is_network_coin_spend(&Bytes32::from([0xFF; 32])),
        "SEC-005: Empty checker must reject all parents"
    );
}

// ── Multiple network coin spends tracked independently ──────────────

#[test]
fn vv_req_sec_005_multiple_spends_tracked() {
    let mut checker = LineageChecker::new();

    let id1 = Bytes32::from([0x01; 32]);
    let id2 = Bytes32::from([0x02; 32]);
    let id3 = Bytes32::from([0x03; 32]);

    checker.record_network_coin_spend(id1);
    checker.record_network_coin_spend(id2);
    checker.record_network_coin_spend(id3);

    assert_eq!(checker.network_coin_spend_count(), 3);

    assert!(checker.is_network_coin_spend(&id1));
    assert!(checker.is_network_coin_spend(&id2));
    assert!(checker.is_network_coin_spend(&id3));

    // Unknown IDs still rejected
    assert!(!checker.is_network_coin_spend(&Bytes32::from([0xFF; 32])));
}

// ── Duplicate spend recording is idempotent ─────────────────────────

#[test]
fn vv_req_sec_005_duplicate_idempotent() {
    let mut checker = LineageChecker::new();
    let id = Bytes32::from([0x42; 32]);

    checker.record_network_coin_spend(id);
    checker.record_network_coin_spend(id); // duplicate

    assert_eq!(
        checker.network_coin_spend_count(),
        1,
        "SEC-005: Duplicate recording must be idempotent"
    );
    assert!(checker.is_network_coin_spend(&id));
}

// ── Lineage verification is O(1) (HashSet-based) ────────────────────

#[test]
fn vv_req_sec_005_lineage_check_is_hashset() {
    // Verify the implementation uses HashSet by checking source
    let source = include_str!("../src/indexer/validator_set.rs");

    assert!(
        source.contains("HashSet<Bytes32>"),
        "SEC-005: LineageChecker must use HashSet for O(1) lineage lookup"
    );
}

// ── Registration coin puzzle hash is deterministic ──────────────────

#[test]
fn vv_req_sec_005_puzzle_hash_deterministic() {
    let pk = test_pk();
    let (mod_hash, ckpt_id, _) = test_params();

    let ph1 = registration_coin_puzzle_hash(mod_hash, &pk, ckpt_id);
    let ph2 = registration_coin_puzzle_hash(mod_hash, &pk, ckpt_id);

    assert_eq!(ph1, ph2, "SEC-005: Puzzle hash must be deterministic");

    // Different pubkey → different hash
    let mut pk2 = pk;
    pk2[2] = 0xFF;
    let ph3 = registration_coin_puzzle_hash(mod_hash, &pk2, ckpt_id);
    assert_ne!(
        ph1, ph3,
        "SEC-005: Different pubkey must produce different hash"
    );
}
