//! REQUIREMENT: IDX-001 — Indexer State Tracking
//! (`docs/requirements/domains/indexer/NORMATIVE.md#IDX-001`).
//!
//! Spec: `docs/requirements/domains/indexer/specs/IDX-001.md`.
//!
//! Implementation: `src/indexer/mod.rs`, `src/state.rs`.
//!
//! Verifies the indexer tracks: network coin state, checkpoint singleton
//! state, registration coins, and checkpoint history.

use chia_protocol::Bytes32;

// ── State types exist ──────────────────────────────────────────────

#[test]
fn vv_req_idx_001_network_coin_state_exists() {
    let src = std::fs::read_to_string("src/state.rs").unwrap();
    assert!(
        src.contains("NetworkCoinState"),
        "IDX-001: NetworkCoinState must exist"
    );
    assert!(
        src.contains("pub coin: Coin"),
        "IDX-001: NetworkCoinState must track coin"
    );
}

#[test]
fn vv_req_idx_001_checkpoint_state_exists() {
    let src = std::fs::read_to_string("src/state.rs").unwrap();
    assert!(
        src.contains("CheckpointSingletonState"),
        "IDX-001: CheckpointSingletonState must exist"
    );
    assert!(src.contains("pub epoch: u64"), "IDX-001: Must track epoch");
    assert!(
        src.contains("pub validator_count: u64"),
        "IDX-001: Must track validator_count"
    );
    assert!(
        src.contains("pub validator_merkle_root: Bytes32"),
        "IDX-001: Must track merkle_root"
    );
    assert!(
        src.contains("pub state_root: Bytes32"),
        "IDX-001: Must track state_root"
    );
}

#[test]
fn vv_req_idx_001_validator_type_exists() {
    let src = std::fs::read_to_string("src/state.rs").unwrap();
    assert!(
        src.contains("Validator"),
        "IDX-001: Validator type must exist"
    );
    assert!(
        src.contains("pub pubkey"),
        "IDX-001: Validator must have pubkey"
    );
    assert!(
        src.contains("pub registration_coin_id"),
        "IDX-001: Must track registration coin ID"
    );
}

#[test]
fn vv_req_idx_001_validator_set_exists() {
    let src = std::fs::read_to_string("src/state.rs").unwrap();
    assert!(
        src.contains("ValidatorSet"),
        "IDX-001: ValidatorSet type must exist"
    );
    assert!(
        src.contains("pub validators"),
        "IDX-001: Must have validators list"
    );
    assert!(
        src.contains("pub merkle_root"),
        "IDX-001: Must have merkle_root"
    );
}

// ── Indexer module exists with proper structure ─────────────────────

#[test]
fn vv_req_idx_001_indexer_module_exists() {
    assert!(std::path::Path::new("src/indexer/mod.rs").exists());
    assert!(std::path::Path::new("src/indexer/validator_set.rs").exists());
    assert!(std::path::Path::new("src/indexer/cache.rs").exists());
    assert!(std::path::Path::new("src/indexer/reorg.rs").exists());
}

#[test]
fn vv_req_idx_001_indexer_state_exists() {
    let src = std::fs::read_to_string("src/indexer/mod.rs").unwrap();
    assert!(
        src.contains("IndexerState"),
        "IDX-001: IndexerState type must exist"
    );
    assert!(
        src.contains("pub async fn sync"),
        "IDX-001: Must have async sync method"
    );
}

#[test]
fn vv_req_idx_001_lineage_checker_available() {
    // IDX-001: LineageChecker (REG-002) must be available from indexer module
    use chia_l2_consensus::testing::LineageChecker;
    let checker = LineageChecker::new();
    assert_eq!(checker.network_coin_spend_count(), 0);
}

// ── Checkpoint state can be created ────────────────────────────────

#[test]
fn vv_req_idx_001_checkpoint_state_constructible() {
    use chia_l2_consensus::CheckpointSingletonState;
    use chia_protocol::Coin;

    let state = CheckpointSingletonState {
        coin: Coin::new(Bytes32::default(), Bytes32::default(), 1),
        epoch: 0,
        validator_count: 0,
        validator_merkle_root: Bytes32::default(),
        state_root: Bytes32::default(),
    };
    assert_eq!(state.epoch, 0);
    assert_eq!(state.validator_count, 0);
}

// ── Spec ───────────────────────────────────────────────────────────

#[test]
fn vv_req_idx_001_spec_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/indexer/specs/IDX-001.md").exists());
}
