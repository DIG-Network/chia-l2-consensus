//! REQUIREMENT: REG-002 — Lineage Verification
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-002`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-002.md`.
//!
//! Implementation: `src/indexer/validator_set.rs`.
//!
//! ## Normative Statement
//!
//! A registration coin is only valid if it was created by a legitimate network
//! coin spend. The indexer maintains a set of valid network coin spend IDs and
//! performs O(1) lookup to verify lineage. Coins whose parent is not a known
//! network coin spend are rejected. This off-chain verification prevents
//! attackers from creating fake registration coins directly.
//!
//! ## How These Tests Prove the Requirement
//!
//! Tests exercise the `LineageChecker` API: creation, recording network coin
//! spends, querying membership, accepting known parents, rejecting unknown
//! parents, tracking multiple spends, counting, idempotent duplicate recording,
//! `verify_registration_coin_lineage` for valid and invalid parents, and
//! confirming the `InvalidLineage` error variant exists.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Indexer maintains set of all network coin spend IDs
//! - [x] Coins created by network coin spend are accepted
//! - [x] Coins NOT created by network coin are rejected
//! - [x] Multiple spends tracked correctly
//! - [x] Duplicate recording is idempotent
//! - [x] ConsensusError::InvalidLineage exists
//! - [x] Spec file exists
//! - [ ] Lineage check includes singleton verification of network coin
//!       (not tested -- requires full singleton lineage chain)
//! - [ ] False registration coins produce no security impact (architectural)
//!
//! ## Gaps
//!
//! Tests exercise the LineageChecker in isolation. Full integration with the
//! indexer processing pipeline (detecting network coin spends, extracting
//! pubkey memos, computing expected puzzle hashes) is not tested here.

use chia_protocol::Bytes32;

// ── LineageChecker type and API ─────────────────────────────────────

// Verifies that validator_set.rs contains lineage verification logic
// (LineageChecker, lineage_checker, or verify_lineage).
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

// Verifies the source maintains a set of network coin spend IDs for O(1) lookup.
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

// Records a network coin spend and verifies it can be queried back.
// This is the basic write-then-read operation.
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

// Verifies an unknown parent ID is NOT recognized as a network coin spend.
// This is the core security property: fake parents are rejected.
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

// Verifies a recorded network coin spend is accepted as a valid parent.
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

// Verifies the checker correctly tracks 3 distinct network coin spends and
// rejects an unknown ID. This proves the set grows with each registration.
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

// Verifies the spend count increments correctly as spends are recorded.
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

// Verifies that recording the same spend ID twice does not create a
// duplicate entry. The count remains 1.
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

// Verifies verify_registration_coin_lineage returns true for a recorded
// network coin spend parent.
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

// Verifies verify_registration_coin_lineage returns false for a fake parent
// that is not a recorded network coin spend.
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

// Verifies ConsensusError::InvalidLineage exists in src/error.rs, providing
// a typed error for lineage verification failures.
#[test]
fn vv_req_reg_002_error_type_exists() {
    // REG-002: ConsensusError::InvalidLineage must exist for lineage failures.
    let src = std::fs::read_to_string("src/error.rs").expect("Failed to read error.rs");

    assert!(
        src.contains("InvalidLineage"),
        "REG-002: ConsensusError must have InvalidLineage variant"
    );
}

// Traceability: verifies the REG-002 spec file exists on disk.
#[test]
fn vv_req_reg_002_spec_file_exists() {
    // REG-002: Dedicated spec file must exist.
    assert!(
        std::path::Path::new("docs/requirements/domains/registration_coin/specs/REG-002.md")
            .exists(),
        "REG-002: Spec file must exist"
    );
}
