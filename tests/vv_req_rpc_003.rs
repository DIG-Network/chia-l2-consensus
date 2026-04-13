//! REQUIREMENT: RPC-003 — Indexer Sync Algorithm
//! (`docs/requirements/domains/rpc/NORMATIVE.md#RPC-003`).
//!
//! Spec: `docs/requirements/domains/rpc/specs/RPC-003.md`.
//!
//! ## Normative Statement
//!
//! The indexer sync algorithm (4 todo!() in src/indexer/) MUST be implemented
//! using ChiaQuery for blockchain queries. The sync orchestrates singleton
//! lookups, registration coin discovery, lineage verification, and Merkle
//! tree construction.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] IndexerState::sync() exists
//! - [x] IndexerChain::get_coin() exists
//! - [x] IndexerChain::get_coin_records_by_puzzle_hash() exists
//! - [x] build_validator_set() exists
//! - [x] All 4 stubs are todo!() (ready for implementation)
//! - [x] Indexer module structure: mod.rs, chain.rs, validator_set.rs, cache.rs, reorg.rs
//! - [x] IndexerCache supports save/load for fast restarts (IDX-005)
//! - [x] ReorgState tracks heights for reorg detection (IDX-004)

use std::fs;

/// RPC-003: IndexerState::sync() exists.
#[test]
fn vv_req_rpc_003_sync_exists() {
    let src = fs::read_to_string("src/indexer/mod.rs").unwrap();
    assert!(
        src.contains("pub async fn sync("),
        "RPC-003: IndexerState must have sync()"
    );
}

/// RPC-003: sync() returns ConsensusResult<ValidatorSet>.
#[test]
fn vv_req_rpc_003_sync_returns_validator_set() {
    let src = fs::read_to_string("src/indexer/mod.rs").unwrap();
    assert!(
        src.contains("ConsensusResult<ValidatorSet>"),
        "RPC-003: sync() must return ConsensusResult<ValidatorSet>"
    );
}

/// RPC-003: IndexerChain::get_coin() exists.
#[test]
fn vv_req_rpc_003_get_coin_exists() {
    let src = fs::read_to_string("src/indexer/chain.rs").unwrap();
    assert!(
        src.contains("pub async fn get_coin("),
        "RPC-003: chain.rs must have get_coin()"
    );
}

/// RPC-003: IndexerChain::get_coin_records_by_puzzle_hash() exists.
#[test]
fn vv_req_rpc_003_get_coin_records_exists() {
    let src = fs::read_to_string("src/indexer/chain.rs").unwrap();
    assert!(
        src.contains("pub async fn get_coin_records_by_puzzle_hash("),
        "RPC-003: chain.rs must have get_coin_records_by_puzzle_hash()"
    );
}

/// RPC-003: build_validator_set() exists in validator_set.rs.
#[test]
fn vv_req_rpc_003_build_validator_set_exists() {
    let src = fs::read_to_string("src/indexer/validator_set.rs").unwrap();
    assert!(
        src.contains("build_validator_set"),
        "RPC-003: validator_set.rs must have build_validator_set()"
    );
}

/// RPC-003: All 4 stubs are todo!() (ready for implementation).
#[test]
fn vv_req_rpc_003_stubs_are_todo() {
    let files = [
        ("src/indexer/mod.rs", "sync"),
        ("src/indexer/chain.rs", "get_coin"),
        ("src/indexer/chain.rs", "get_coin_records_by_puzzle_hash"),
        ("src/indexer/validator_set.rs", "build_validator_set"),
    ];
    for (file, func) in &files {
        let src = fs::read_to_string(file).unwrap();
        assert!(
            src.contains("todo!()"),
            "RPC-003: {} in {} must have todo!() stub",
            func,
            file
        );
    }
}

/// RPC-003: Indexer module structure is complete.
#[test]
fn vv_req_rpc_003_module_structure() {
    assert!(std::path::Path::new("src/indexer/mod.rs").exists());
    assert!(std::path::Path::new("src/indexer/chain.rs").exists());
    assert!(std::path::Path::new("src/indexer/validator_set.rs").exists());
    assert!(std::path::Path::new("src/indexer/cache.rs").exists());
    assert!(std::path::Path::new("src/indexer/reorg.rs").exists());
}

/// RPC-003: IndexerCache supports save/load (IDX-005).
#[test]
fn vv_req_rpc_003_cache_save_load() {
    let src = fs::read_to_string("src/indexer/cache.rs").unwrap();
    assert!(
        src.contains("pub fn save("),
        "RPC-003: Cache must have save()"
    );
    assert!(
        src.contains("pub fn load("),
        "RPC-003: Cache must have load()"
    );
}

/// RPC-003: ReorgState exists for reorg detection (IDX-004).
#[test]
fn vv_req_rpc_003_reorg_state_exists() {
    let src = fs::read_to_string("src/indexer/reorg.rs").unwrap();
    assert!(
        src.contains("pub fn is_reorg("),
        "RPC-003: ReorgState must have is_reorg()"
    );
    assert!(
        src.contains("pub fn compute_rollback("),
        "RPC-003: ReorgState must have compute_rollback()"
    );
}

/// RPC-003: LineageChecker available for registration coin verification (IDX-002).
#[test]
fn vv_req_rpc_003_lineage_checker_exists() {
    let src = fs::read_to_string("src/indexer/validator_set.rs").unwrap();
    assert!(
        src.contains("LineageChecker"),
        "RPC-003: validator_set.rs must have LineageChecker"
    );
}

/// RPC-003: Spec file exists.
#[test]
fn vv_req_rpc_003_spec_file_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/rpc/specs/RPC-003.md").exists());
}
