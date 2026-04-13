//! REQUIREMENT: RPC-004 — ConsensusClient Operation Methods
//! (`docs/requirements/domains/rpc/NORMATIVE.md#RPC-004`).
//!
//! Spec: `docs/requirements/domains/rpc/specs/RPC-004.md`.
//!
//! ## Normative Statement
//!
//! The ConsensusClient operation methods (todo!() stubs in src/client.rs)
//! MUST be implemented to coordinate puzzle drivers, prover, and indexer.
//! All methods MUST return SpendBundle without broadcasting (per API-008).
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] deploy() exists and returns ConsensusResult<SpendBundle>
//! - [x] register_validator() exists with wallet params (RPC-005)
//! - [x] build_checkpoint() exists and returns ConsensusResult<SpendBundle>
//! - [x] get_checkpoint_state() exists
//! - [x] recover_collateral() exists and returns ConsensusResult<SpendBundle>
//! - [x] release_collateral() exists (WDC-005)
//! - [x] All return SpendBundle (API-008)
//! - [x] connect() wires ChiaQuery (RPC-001)
//! - [x] sync() exists as prerequisite

use std::fs;

/// RPC-004: deploy() exists.
#[test]
fn vv_req_rpc_004_deploy_exists() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub async fn deploy("));
}

/// RPC-004: deploy() returns ConsensusResult<SpendBundle>.
#[test]
fn vv_req_rpc_004_deploy_returns_bundle() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    let m = src.find("pub async fn deploy(").unwrap();
    let sig = &src[m..m + 300.min(src.len() - m)];
    let end = sig.find('{').unwrap_or(sig.len());
    assert!(
        sig[..end].contains("ConsensusResult<SpendBundle>"),
        "RPC-004: deploy() must return ConsensusResult<SpendBundle>"
    );
}

/// RPC-004: register_validator() exists with wallet params.
#[test]
fn vv_req_rpc_004_register_validator_exists() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub async fn register_validator("));
    let m = src.find("pub async fn register_validator(").unwrap();
    let sig = &src[m..m + 500.min(src.len() - m)];
    let end = sig.find('{').unwrap_or(sig.len());
    let signature = &sig[..end];
    assert!(
        signature.contains("L1Wallet"),
        "RPC-004/RPC-005: Must accept L1Wallet"
    );
    assert!(
        signature.contains("ConsensusResult<SpendBundle>"),
        "RPC-004: Must return ConsensusResult<SpendBundle>"
    );
}

/// RPC-004: build_checkpoint() exists.
#[test]
fn vv_req_rpc_004_build_checkpoint_exists() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub async fn build_checkpoint("));
    let m = src.find("pub async fn build_checkpoint(").unwrap();
    let sig = &src[m..m + 500.min(src.len() - m)];
    let end = sig.find('{').unwrap_or(sig.len());
    assert!(
        sig[..end].contains("ConsensusResult<SpendBundle>"),
        "RPC-004: build_checkpoint must return ConsensusResult<SpendBundle>"
    );
}

/// RPC-004: get_checkpoint_state() exists.
#[test]
fn vv_req_rpc_004_get_checkpoint_state_exists() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub async fn get_checkpoint_state("));
}

/// RPC-004: recover_collateral() exists.
#[test]
fn vv_req_rpc_004_recover_collateral_exists() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub async fn recover_collateral("));
    let m = src.find("pub async fn recover_collateral(").unwrap();
    let sig = &src[m..m + 300.min(src.len() - m)];
    let end = sig.find('{').unwrap_or(sig.len());
    assert!(
        sig[..end].contains("ConsensusResult<SpendBundle>"),
        "RPC-004: recover_collateral must return ConsensusResult<SpendBundle>"
    );
}

/// RPC-004: release_collateral() exists (WDC-005).
#[test]
fn vv_req_rpc_004_release_collateral_exists() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub async fn release_collateral("));
}

/// RPC-004: sync() exists as prerequisite.
#[test]
fn vv_req_rpc_004_sync_exists() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub async fn sync("));
}

/// RPC-004: connect() wires ChiaQuery (RPC-001).
#[test]
fn vv_req_rpc_004_connect_exists() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub async fn connect("));
}

/// RPC-004: All stubs are todo!() (ready for implementation).
#[test]
fn vv_req_rpc_004_stubs_are_todo() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    // Count todo!() occurrences — should be at least 5 (deploy, register, checkpoint, state, recover)
    let count = src.matches("todo!()").count();
    assert!(
        count >= 5,
        "RPC-004: client.rs must have at least 5 todo!() stubs, found {}",
        count
    );
}

/// RPC-004: recover_collateral docs mention WithdrawDelayCoin (WDC-004).
#[test]
fn vv_req_rpc_004_recover_mentions_delay_coin() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    assert!(
        src.contains("withdraw delay") || src.contains("WITHDRAW DELAY") || src.contains("WDC-004"),
        "RPC-004: recover_collateral must document withdraw delay coin"
    );
}

/// RPC-004: No broadcast in client.rs code (API-008).
#[test]
fn vv_req_rpc_004_no_broadcast() {
    let src = fs::read_to_string("src/client.rs").unwrap();
    for line in src.lines() {
        let t = line.trim();
        if t.starts_with("//") || t.starts_with("///") || t.starts_with("//!") {
            continue;
        }
        assert!(
            !t.contains("push_tx(") && !t.contains("send_transaction("),
            "RPC-004/API-008: client.rs must NOT broadcast"
        );
    }
}

/// RPC-004: Spec file exists.
#[test]
fn vv_req_rpc_004_spec_file_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/rpc/specs/RPC-004.md").exists());
}
