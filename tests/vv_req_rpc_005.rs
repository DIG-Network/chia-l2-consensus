//! REQUIREMENT: RPC-005 — Wallet Integration for Collateral Funding
//! (`docs/requirements/domains/rpc/NORMATIVE.md#RPC-005`).
//!
//! Spec: `docs/requirements/domains/rpc/specs/RPC-005.md`.
//!
//! ## Normative Statement
//!
//! For validator registration, the crate MUST use dig-l1-wallet coin selection
//! to fund the collateral amount. The L1Wallet is passed per-call, NOT stored
//! in ConsensusClient.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] dig-l1-wallet in Cargo.toml (verified by RPC-006)
//! - [x] register_validator() accepts &L1Wallet parameter
//! - [x] register_validator() accepts wallet_name, account_index, fee
//! - [x] InsufficientFunds error variant exists
//! - [x] L1Wallet NOT stored in ConsensusClient
//! - [x] dig-l1-wallet types importable

use std::fs;

/// RPC-005: register_validator() accepts &L1Wallet parameter.
#[test]
fn vv_req_rpc_005_register_accepts_wallet() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("L1Wallet"),
        "RPC-005: register_validator must accept L1Wallet"
    );
    // Check it's a parameter, not a stored field
    let method = src.find("pub async fn register_validator(").unwrap();
    let sig_end = src[method..].find('{').unwrap();
    let signature = &src[method..method + sig_end];
    assert!(
        signature.contains("L1Wallet"),
        "RPC-005: L1Wallet must be in register_validator signature"
    );
}

/// RPC-005: register_validator() accepts wallet_name parameter.
#[test]
fn vv_req_rpc_005_register_accepts_wallet_name() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    let method = src.find("pub async fn register_validator(").unwrap();
    let sig_end = src[method..].find('{').unwrap();
    let signature = &src[method..method + sig_end];
    assert!(
        signature.contains("wallet_name"),
        "RPC-005: register_validator must accept wallet_name"
    );
}

/// RPC-005: register_validator() accepts account_index parameter.
#[test]
fn vv_req_rpc_005_register_accepts_account_index() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    let method = src.find("pub async fn register_validator(").unwrap();
    let sig_end = src[method..].find('{').unwrap();
    let signature = &src[method..method + sig_end];
    assert!(
        signature.contains("account_index"),
        "RPC-005: register_validator must accept account_index"
    );
}

/// RPC-005: register_validator() accepts fee parameter.
#[test]
fn vv_req_rpc_005_register_accepts_fee() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    let method = src.find("pub async fn register_validator(").unwrap();
    let sig_end = src[method..].find('{').unwrap();
    let signature = &src[method..method + sig_end];
    assert!(
        signature.contains("fee"),
        "RPC-005: register_validator must accept fee"
    );
}

/// RPC-005: InsufficientFunds error variant exists.
#[test]
fn vv_req_rpc_005_insufficient_funds_error() {
    let src = fs::read_to_string("src/error.rs").expect("error.rs");
    assert!(
        src.contains("InsufficientFunds"),
        "RPC-005: ConsensusError must have InsufficientFunds variant"
    );
}

/// RPC-005: L1Wallet NOT stored in ConsensusClient struct.
#[test]
fn vv_req_rpc_005_wallet_not_stored() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    // Find the struct definition
    let struct_start = src.find("pub struct ConsensusClient").unwrap();
    let struct_end = src[struct_start..].find("\n}").unwrap() + struct_start;
    let struct_def = &src[struct_start..struct_end];
    assert!(
        !struct_def.contains("L1Wallet") && !struct_def.contains("wallet:"),
        "RPC-005: ConsensusClient must NOT store L1Wallet"
    );
}

/// RPC-005: dig-l1-wallet L1Wallet type importable.
#[test]
fn vv_req_rpc_005_wallet_type_importable() {
    fn _assert(_: &dig_l1_wallet::L1Wallet) {}
}

/// RPC-005: dig-l1-wallet CoinSelectionStrategy type importable.
#[test]
fn vv_req_rpc_005_coin_selection_strategy_importable() {
    fn _assert(_: dig_l1_wallet::CoinSelectionStrategy) {}
}

/// RPC-005: Method documents coin selection usage.
#[test]
fn vv_req_rpc_005_documents_coin_selection() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("coin selection") || src.contains("select_coins"),
        "RPC-005: register_validator must document coin selection"
    );
}

/// RPC-005: Spec file exists.
#[test]
fn vv_req_rpc_005_spec_file_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/rpc/specs/RPC-005.md").exists(),);
}
