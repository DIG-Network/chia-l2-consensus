//! REQUIREMENT: RPC-005 — Wallet Integration for Collateral Funding
//! (`docs/requirements/domains/rpc/NORMATIVE.md#RPC-005`).
//!
//! Spec: `docs/requirements/domains/rpc/specs/RPC-005.md`.
//!
//! ## Normative Statement
//!
//! For validator registration, the crate MUST use `dig-wallet-backend`'s engine seam
//! (`select_for_spend`, `WalletStore::coins`) to fund the collateral amount. The
//! `WalletEngine` handle is passed per-call, NOT stored in ConsensusClient — the crate
//! never holds wallet state or a signing key (#998, epic migration off `dig-l1-wallet`).
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] dig-wallet-backend in Cargo.toml (verified by RPC-006)
//! - [x] register_validator() accepts &dyn WalletEngine parameter
//! - [x] register_validator() accepts an IdentityRef (replaces wallet_name/account_index)
//! - [x] InsufficientFunds error variant exists
//! - [x] WalletEngine NOT stored in ConsensusClient
//! - [x] dig-wallet-backend engine-seam types importable

use std::fs;

/// RPC-005: register_validator() accepts a `&dyn WalletEngine` parameter.
#[test]
fn vv_req_rpc_005_register_accepts_wallet_engine() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("WalletEngine"),
        "RPC-005: register_validator must accept a WalletEngine handle"
    );
    // Check it's a parameter, not a stored field.
    let method = src.find("pub async fn register_validator(").unwrap();
    let sig_end = src[method..].find('{').unwrap();
    let signature = &src[method..method + sig_end];
    assert!(
        signature.contains("WalletEngine"),
        "RPC-005: WalletEngine must be in register_validator's signature"
    );
    assert!(
        signature.contains("IdentityRef"),
        "RPC-005: IdentityRef must replace wallet_name/account_index in the signature"
    );
}

/// RPC-005: ConsensusClient does not store a WalletEngine field (passed per-call only).
#[test]
fn vv_req_rpc_005_wallet_not_stored_in_client() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    let struct_start = src.find("pub struct ConsensusClient {").unwrap();
    let struct_end = src[struct_start..].find('}').unwrap();
    let struct_body = &src[struct_start..struct_start + struct_end];
    assert!(
        !struct_body.contains("WalletEngine"),
        "RPC-005: ConsensusClient must NOT store a WalletEngine — it is passed per-call"
    );
}

/// RPC-005: InsufficientFunds error variant exists for a shortfall from coin selection.
#[test]
fn vv_req_rpc_005_insufficient_funds_variant_exists() {
    let src = fs::read_to_string("src/error.rs").expect("error.rs");
    assert!(
        src.contains("InsufficientFunds(String)"),
        "RPC-005: ConsensusError must carry an InsufficientFunds(String) variant"
    );
}
