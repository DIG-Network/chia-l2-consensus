//! REQUIREMENT: IDX-002 — Lineage Verification
//! (`docs/requirements/domains/indexer/NORMATIVE.md#IDX-002`).
//!
//! Spec: `docs/requirements/domains/indexer/specs/IDX-002.md`.
//!
//! Implementation: `src/indexer/validator_set.rs`.
//!
//! Verifies each registration coin's lineage: parent must be a known network
//! coin spend, pubkey extracted from memo, puzzle hash matches, collateral correct.

use chia_protocol::{Bytes32, Coin};

// ── IDX-002: LineageChecker tracks network coin spends ────────────────

#[test]
fn vv_req_idx_002_lineage_checker_empty() {
    use chia_l2_consensus::testing::LineageChecker;

    let checker = LineageChecker::new();
    let fake_id = Bytes32::from([0xAA; 32]);
    assert!(
        !checker.verify_registration_coin_lineage(&fake_id),
        "IDX-002: Empty checker must reject all parents"
    );
}

#[test]
fn vv_req_idx_002_lineage_checker_records_and_verifies() {
    use chia_l2_consensus::testing::LineageChecker;

    let mut checker = LineageChecker::new();
    let spend_id = Bytes32::from([0xBB; 32]);
    let other_id = Bytes32::from([0xCC; 32]);

    checker.record_network_coin_spend(spend_id);

    assert!(
        checker.verify_registration_coin_lineage(&spend_id),
        "IDX-002: Recorded spend ID must pass lineage check"
    );
    assert!(
        !checker.verify_registration_coin_lineage(&other_id),
        "IDX-002: Unknown spend ID must fail lineage check"
    );
    assert_eq!(checker.network_coin_spend_count(), 1);
}

// ── IDX-002: Registration coin puzzle hash computation ────────────────

#[test]
fn vv_req_idx_002_registration_puzzle_hash_computation() {
    // The indexer must compute the expected registration coin puzzle hash
    // for a given pubkey and checkpoint singleton ID, and compare it
    // to the coin's actual puzzle hash.
    //
    // This uses curry_tree_hash with the registration coin mod hash.
    use chia_l2_consensus::testing::registration_coin_puzzle_hash;

    let reg_mod_hash = Bytes32::from([0x11; 32]);
    let pubkey = [0xAA; 48];
    let checkpoint_id = Bytes32::from([0xBB; 32]);

    let hash = registration_coin_puzzle_hash(reg_mod_hash, &pubkey, checkpoint_id);

    // Must be deterministic
    let hash2 = registration_coin_puzzle_hash(reg_mod_hash, &pubkey, checkpoint_id);
    assert_eq!(hash, hash2, "IDX-002: Puzzle hash must be deterministic");

    // Different pubkey → different hash
    let hash3 = registration_coin_puzzle_hash(reg_mod_hash, &[0xBB; 48], checkpoint_id);
    assert_ne!(
        hash, hash3,
        "IDX-002: Different pubkey must give different hash"
    );
}

// ── IDX-002: Full registration coin validation ────────────────────────

#[test]
fn vv_req_idx_002_try_parse_valid_registration() {
    // A registration coin with valid lineage, correct puzzle hash,
    // and correct collateral should be accepted.
    use chia_l2_consensus::testing::{
        registration_coin_puzzle_hash, try_parse_registration_coin, LineageChecker,
    };
    use chia_l2_consensus::NetworkConfig;

    let pubkey = [0xAA; 48];
    let reg_mod_hash = Bytes32::from([0x11; 32]);
    let checkpoint_id = Bytes32::from([0xBB; 32]);
    let collateral = 1_000_000u64;

    let expected_ph = registration_coin_puzzle_hash(reg_mod_hash, &pubkey, checkpoint_id);

    // Simulate: network coin was spent (its coin ID recorded)
    let network_coin_id = Bytes32::from([0xCC; 32]);
    let mut checker = LineageChecker::new();
    checker.record_network_coin_spend(network_coin_id);

    // The registration coin: parent = network coin, puzzle_hash = expected, amount = collateral
    let reg_coin = Coin::new(network_coin_id, expected_ph, collateral);

    let result = try_parse_registration_coin(
        &checker,
        &reg_coin,
        &pubkey,
        reg_mod_hash,
        checkpoint_id,
        collateral,
    );
    assert!(
        result.is_some(),
        "IDX-002: Valid registration coin must be parsed"
    );
    let record = result.unwrap();
    assert_eq!(record.pubkey, pubkey.to_vec());
    assert_eq!(record.coin, reg_coin);
}

#[test]
fn vv_req_idx_002_reject_wrong_parent() {
    // A coin with a parent that is NOT a network coin spend must be rejected.
    use chia_l2_consensus::testing::{
        registration_coin_puzzle_hash, try_parse_registration_coin, LineageChecker,
    };

    let pubkey = [0xAA; 48];
    let reg_mod_hash = Bytes32::from([0x11; 32]);
    let checkpoint_id = Bytes32::from([0xBB; 32]);
    let collateral = 1_000_000u64;
    let expected_ph = registration_coin_puzzle_hash(reg_mod_hash, &pubkey, checkpoint_id);

    let checker = LineageChecker::new(); // empty — no network coin spends recorded

    let fake_parent = Bytes32::from([0xFF; 32]);
    let reg_coin = Coin::new(fake_parent, expected_ph, collateral);

    let result = try_parse_registration_coin(
        &checker,
        &reg_coin,
        &pubkey,
        reg_mod_hash,
        checkpoint_id,
        collateral,
    );
    assert!(
        result.is_none(),
        "IDX-002: Coin with wrong parent must be silently rejected"
    );
}

#[test]
fn vv_req_idx_002_reject_wrong_puzzle_hash() {
    // A coin with valid parent but wrong puzzle hash must be rejected.
    use chia_l2_consensus::testing::{try_parse_registration_coin, LineageChecker};

    let pubkey = [0xAA; 48];
    let reg_mod_hash = Bytes32::from([0x11; 32]);
    let checkpoint_id = Bytes32::from([0xBB; 32]);
    let collateral = 1_000_000u64;

    let network_coin_id = Bytes32::from([0xCC; 32]);
    let mut checker = LineageChecker::new();
    checker.record_network_coin_spend(network_coin_id);

    let wrong_ph = Bytes32::from([0xFF; 32]); // wrong puzzle hash
    let reg_coin = Coin::new(network_coin_id, wrong_ph, collateral);

    let result = try_parse_registration_coin(
        &checker,
        &reg_coin,
        &pubkey,
        reg_mod_hash,
        checkpoint_id,
        collateral,
    );
    assert!(
        result.is_none(),
        "IDX-002: Coin with wrong puzzle hash must be rejected"
    );
}

#[test]
fn vv_req_idx_002_reject_wrong_collateral() {
    // A coin with valid parent and puzzle hash but wrong amount must be rejected.
    use chia_l2_consensus::testing::{
        registration_coin_puzzle_hash, try_parse_registration_coin, LineageChecker,
    };

    let pubkey = [0xAA; 48];
    let reg_mod_hash = Bytes32::from([0x11; 32]);
    let checkpoint_id = Bytes32::from([0xBB; 32]);
    let collateral = 1_000_000u64;
    let expected_ph = registration_coin_puzzle_hash(reg_mod_hash, &pubkey, checkpoint_id);

    let network_coin_id = Bytes32::from([0xCC; 32]);
    let mut checker = LineageChecker::new();
    checker.record_network_coin_spend(network_coin_id);

    let wrong_amount = 999_999u64; // wrong collateral
    let reg_coin = Coin::new(network_coin_id, expected_ph, wrong_amount);

    let result = try_parse_registration_coin(
        &checker,
        &reg_coin,
        &pubkey,
        reg_mod_hash,
        checkpoint_id,
        collateral,
    );
    assert!(
        result.is_none(),
        "IDX-002: Coin with wrong collateral must be rejected"
    );
}

// ── Spec ───────────────────────────────────────────────────────────

#[test]
fn vv_req_idx_002_spec_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/indexer/specs/IDX-002.md").exists());
}
