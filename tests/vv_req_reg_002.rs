//! REQUIREMENT: REG-002 — Lineage Verification
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-002`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-002.md`.
//!
//! Implementation: `src/indexer/validator_set.rs`.
//!
//! Verifies that registration coins are only accepted when their parent coin
//! is a valid network coin spend. Coins without valid lineage are rejected.

use chia_protocol::Bytes32;

// ── LineageChecker type and API ─────────────────────────────────────

#[test]
fn vv_req_reg_002_lineage_checker_exists() {
    // REG-002: The crate must expose a LineageChecker type for verifying
    // registration coin parentage.
    let src = std::fs::read_to_string("src/indexer/validator_set.rs")
        .expect("Failed to read validator_set.rs");

    assert!(
        src.contains("LineageChecker")
            || src.contains("lineage_checker")
            || src.contains("verify_lineage"),
        "REG-002: src/indexer/validator_set.rs must have lineage verification logic"
    );
}

#[test]
fn vv_req_reg_002_tracks_network_coin_spends() {
    // REG-002: The lineage checker must maintain a set of valid network
    // coin spend IDs for O(1) lookup.
    let src = std::fs::read_to_string("src/indexer/validator_set.rs")
        .expect("Failed to read validator_set.rs");

    assert!(
        src.contains("network_coin_spend_ids") || src.contains("network_coin_spends"),
        "REG-002: Must maintain a set of network coin spend IDs"
    );
}

#[test]
fn vv_req_reg_002_can_record_network_coin_spend() {
    // REG-002: Must be able to record a network coin spend ID.
    use chia_l2_consensus::testing::LineageChecker;

    let mut checker = LineageChecker::new();
    let spend_id = Bytes32::default();
    checker.record_network_coin_spend(spend_id);

    assert!(
        checker.is_network_coin_spend(&spend_id),
        "REG-002: Recorded network coin spend must be queryable"
    );
}

#[test]
fn vv_req_reg_002_unknown_parent_rejected() {
    // REG-002: A coin whose parent is NOT a known network coin spend
    // must be rejected by the lineage checker.
    use chia_l2_consensus::testing::LineageChecker;

    let checker = LineageChecker::new();
    let unknown_id = Bytes32::from([0xAB; 32]);

    assert!(
        !checker.is_network_coin_spend(&unknown_id),
        "REG-002: Unknown parent must not pass lineage check"
    );
}

#[test]
fn vv_req_reg_002_known_parent_accepted() {
    // REG-002: A coin whose parent IS a known network coin spend
    // must be accepted by the lineage checker.
    use chia_l2_consensus::testing::LineageChecker;

    let mut checker = LineageChecker::new();
    let spend_id = Bytes32::from([0xCD; 32]);
    checker.record_network_coin_spend(spend_id);

    assert!(
        checker.is_network_coin_spend(&spend_id),
        "REG-002: Known network coin spend must pass lineage check"
    );
}

#[test]
fn vv_req_reg_002_multiple_spends_tracked() {
    // REG-002: The checker must track multiple network coin spends
    // (one per registration) for the full history.
    use chia_l2_consensus::testing::LineageChecker;

    let mut checker = LineageChecker::new();
    let spend1 = Bytes32::from([0x01; 32]);
    let spend2 = Bytes32::from([0x02; 32]);
    let spend3 = Bytes32::from([0x03; 32]);

    checker.record_network_coin_spend(spend1);
    checker.record_network_coin_spend(spend2);
    checker.record_network_coin_spend(spend3);

    assert!(checker.is_network_coin_spend(&spend1));
    assert!(checker.is_network_coin_spend(&spend2));
    assert!(checker.is_network_coin_spend(&spend3));
    assert!(!checker.is_network_coin_spend(&Bytes32::from([0xFF; 32])));
}

#[test]
fn vv_req_reg_002_spend_count() {
    // REG-002: The checker should report how many network coin spends
    // it has recorded (for diagnostics).
    use chia_l2_consensus::testing::LineageChecker;

    let mut checker = LineageChecker::new();
    assert_eq!(checker.network_coin_spend_count(), 0);

    checker.record_network_coin_spend(Bytes32::from([0x01; 32]));
    assert_eq!(checker.network_coin_spend_count(), 1);

    checker.record_network_coin_spend(Bytes32::from([0x02; 32]));
    assert_eq!(checker.network_coin_spend_count(), 2);
}

#[test]
fn vv_req_reg_002_duplicate_spend_is_idempotent() {
    // REG-002: Recording the same spend ID twice must not create duplicates.
    use chia_l2_consensus::testing::LineageChecker;

    let mut checker = LineageChecker::new();
    let spend_id = Bytes32::from([0xAA; 32]);

    checker.record_network_coin_spend(spend_id);
    checker.record_network_coin_spend(spend_id);

    assert_eq!(
        checker.network_coin_spend_count(),
        1,
        "REG-002: Duplicate spend recording must be idempotent"
    );
}

#[test]
fn vv_req_reg_002_verify_registration_coin_valid() {
    // REG-002: verify_registration_coin_lineage returns true when
    // the parent_coin_id is a recorded network coin spend.
    use chia_l2_consensus::testing::LineageChecker;

    let mut checker = LineageChecker::new();
    let network_spend_id = Bytes32::from([0x11; 32]);
    checker.record_network_coin_spend(network_spend_id);

    assert!(
        checker.verify_registration_coin_lineage(&network_spend_id),
        "REG-002: Registration coin with valid parent must pass"
    );
}

#[test]
fn vv_req_reg_002_verify_registration_coin_invalid() {
    // REG-002: verify_registration_coin_lineage returns false when
    // the parent_coin_id is NOT a recorded network coin spend.
    use chia_l2_consensus::testing::LineageChecker;

    let mut checker = LineageChecker::new();
    // Record a different spend
    checker.record_network_coin_spend(Bytes32::from([0x11; 32]));

    let fake_parent = Bytes32::from([0x99; 32]);
    assert!(
        !checker.verify_registration_coin_lineage(&fake_parent),
        "REG-002: Registration coin with fake parent must fail"
    );
}

#[test]
fn vv_req_reg_002_error_type_exists() {
    // REG-002: ConsensusError::InvalidLineage must exist for lineage failures.
    let src = std::fs::read_to_string("src/error.rs").expect("Failed to read error.rs");

    assert!(
        src.contains("InvalidLineage"),
        "REG-002: ConsensusError must have InvalidLineage variant"
    );
}

#[test]
fn vv_req_reg_002_spec_file_exists() {
    // REG-002: Dedicated spec file must exist.
    assert!(
        std::path::Path::new("docs/requirements/domains/registration_coin/specs/REG-002.md")
            .exists(),
        "REG-002: Spec file must exist"
    );
}
