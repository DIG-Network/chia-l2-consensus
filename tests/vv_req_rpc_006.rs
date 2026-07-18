//! REQUIREMENT: RPC-006 — Dependency Version Alignment
//! (`docs/requirements/domains/rpc/NORMATIVE.md#RPC-006`).
//!
//! Spec: `docs/requirements/domains/rpc/specs/RPC-006.md`.
//!
//! ## Normative Statement
//!
//! Cargo.toml MUST add chia-query and dig-wallet-backend as dependencies. The crate
//! MUST compile without type mismatch errors. Existing tests MUST still pass.
//! (#998/#1004: dig-wallet-backend replaces the deprecated dig-l1-wallet — consumers
//! migrate to its engine-seam interface rather than the reverse.)
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] chia-query added to Cargo.toml
//! - [x] dig-wallet-backend added to Cargo.toml (engine feature only — no signer)
//! - [x] cargo check passes
//! - [x] cargo check --tests passes
//! - [x] Existing tests still pass
//! - [x] chia-query types importable
//! - [x] dig-wallet-backend engine-seam types importable

/// RPC-006: chia-query is in Cargo.toml and importable.
#[test]
fn vv_req_rpc_006_chia_query_in_deps() {
    let toml = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    assert!(
        toml.contains("chia-query"),
        "RPC-006: Cargo.toml must include chia-query"
    );
}

/// RPC-006: dig-wallet-backend is in Cargo.toml and importable.
#[test]
fn vv_req_rpc_006_dig_wallet_backend_in_deps() {
    let toml = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    assert!(
        toml.contains("dig-wallet-backend"),
        "RPC-006: Cargo.toml must include dig-wallet-backend"
    );
    assert!(
        !toml.contains("dig-l1-wallet"),
        "RPC-006: dig-l1-wallet is deprecated (#998) and must not remain a dependency"
    );
}

/// RPC-006: chia-query ChiaQuery type is importable.
#[test]
fn vv_req_rpc_006_chia_query_importable() {
    // This compiles → the type exists and is accessible
    fn _assert_type_exists(_: &chia_query::ChiaQuery) {}
}

/// RPC-006: chia-query ChiaQueryConfig type is importable.
#[test]
fn vv_req_rpc_006_chia_query_config_importable() {
    fn _assert_type_exists(_: &chia_query::ChiaQueryConfig) {}
}

/// RPC-006: dig-wallet-backend's WalletEngine trait (engine seam) is importable.
#[test]
fn vv_req_rpc_006_wallet_engine_importable() {
    fn _assert_type_exists(_: &dyn dig_wallet_backend::engine::WalletEngine) {}
}

/// RPC-006: dig-wallet-backend's shared IdentityRef type is importable.
#[test]
fn vv_req_rpc_006_identity_ref_importable() {
    fn _assert_type_exists(_: &dig_wallet_backend::types::IdentityRef) {}
}

/// RPC-006: dig-wallet-backend's coin-selection outcome type is importable.
#[test]
fn vv_req_rpc_006_selection_outcome_importable() {
    fn _assert_type_exists(_: &dig_wallet_backend::engine::SelectionOutcome) {}
}

/// RPC-006: Our existing chia-protocol types still work.
#[test]
fn vv_req_rpc_006_existing_types_still_work() {
    let _: chia_protocol::Bytes32 = chia_protocol::Bytes32::default();
    let _: chia_protocol::Coin = chia_protocol::Coin::new(
        chia_protocol::Bytes32::default(),
        chia_protocol::Bytes32::default(),
        0,
    );
}

/// RPC-006: Our crate's public types still accessible.
#[test]
fn vv_req_rpc_006_crate_types_still_work() {
    let _: Option<chia_l2_consensus::ConsensusClient> = None;
    let _: Option<chia_l2_consensus::NetworkConfig> = None;
    let _: Option<chia_l2_consensus::ConsensusError> = None;
}

/// RPC-006: Spec file exists.
#[test]
fn vv_req_rpc_006_spec_file_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/rpc/specs/RPC-006.md").exists(),);
}
