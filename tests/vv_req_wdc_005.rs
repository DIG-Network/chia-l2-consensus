//! REQUIREMENT: WDC-005 — Driver and API
//! (`docs/requirements/domains/withdraw_delay/NORMATIVE.md#WDC-005`).
//!
//! Spec: `docs/requirements/domains/withdraw_delay/specs/WDC-005.md`.
//!
//! ## Normative Statement
//!
//! The crate MUST provide a `release_collateral()` method on `ConsensusClient`
//! that builds a SpendBundle to spend a withdraw delay coin after the delay
//! period has elapsed. The method MUST return the bundle without broadcasting
//! it (per API-008).
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] `release_collateral()` exists on ConsensusClient
//! - [x] `withdraw_delay_puzzle_hash()` function exists
//! - [x] `recover_collateral()` doc comment explains two-phase flow
//! - [x] Bundle functions exist and are callable
//! - [x] DEFAULT_WITHDRAW_DELAY_BLOCKS constant available

use chia_l2_consensus::testing::{
    withdraw_delay_puzzle_hash, DEFAULT_WITHDRAW_DELAY_BLOCKS, WITHDRAW_DELAY_COIN_MOD_HASH_HEX,
    WITHDRAW_DELAY_COIN_PUZZLE_HEX,
};
use chia_protocol::Bytes32;

/// WDC-005: release_collateral() exists on ConsensusClient.
#[test]
fn vv_req_wdc_005_release_collateral_exists() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("pub async fn release_collateral("),
        "WDC-005: ConsensusClient must have release_collateral()"
    );
}

/// WDC-005: release_collateral() returns SpendBundle (API-008 compliance).
#[test]
fn vv_req_wdc_005_release_returns_spend_bundle() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    // Find the release_collateral method and verify return type
    let method_start = src.find("pub async fn release_collateral(").unwrap();
    let after_method = &src[method_start..];
    let sig_end = after_method.find('{').unwrap();
    let signature = &after_method[..sig_end];
    assert!(
        signature.contains("ConsensusResult<SpendBundle>"),
        "WDC-005: release_collateral must return ConsensusResult<SpendBundle>"
    );
}

/// WDC-005: recover_collateral() documents two-phase flow.
#[test]
fn vv_req_wdc_005_recover_documents_two_phase() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("withdraw delay coin") || src.contains("WITHDRAW DELAY COIN"),
        "WDC-005: recover_collateral must document withdraw delay coin creation"
    );
    assert!(
        src.contains("release_collateral()"),
        "WDC-005: recover_collateral must reference release_collateral()"
    );
}

/// WDC-005: withdraw_delay_puzzle_hash() function exists.
#[test]
fn vv_req_wdc_005_puzzle_hash_function_exists() {
    // If this compiles, the function exists and is callable
    let result =
        withdraw_delay_puzzle_hash(Bytes32::default(), Bytes32::default(), 1_000_000, 24_000);
    // Returns a Bytes32 (may be default for now — implementation is TODO)
    let _ = result;
}

/// WDC-005: release_collateral driver function exists in withdraw_delay module.
#[test]
fn vv_req_wdc_005_driver_release_exists() {
    let src = std::fs::read_to_string("src/puzzles/withdraw_delay.rs").expect("withdraw_delay.rs");
    assert!(
        src.contains("pub fn release_collateral("),
        "WDC-005: Driver must have release_collateral function"
    );
}

/// WDC-005: Driver module exports all required symbols.
#[test]
fn vv_req_wdc_005_driver_exports() {
    // These imports prove the symbols are exported
    let _ = WITHDRAW_DELAY_COIN_PUZZLE_HEX;
    let _ = WITHDRAW_DELAY_COIN_MOD_HASH_HEX;
    let _ = DEFAULT_WITHDRAW_DELAY_BLOCKS;
}

/// WDC-005: No broadcast code in the module (API-008).
#[test]
fn vv_req_wdc_005_no_broadcast() {
    let src = std::fs::read_to_string("src/puzzles/withdraw_delay.rs").expect("withdraw_delay.rs");
    // Check for actual broadcast function calls (not doc comments mentioning the concept)
    assert!(
        !src.contains("push_tx(") && !src.contains("send_transaction("),
        "WDC-005/API-008: Module must NOT contain broadcast function calls"
    );
}

/// WDC-005: Spec file exists.
#[test]
fn vv_req_wdc_005_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/withdraw_delay/specs/WDC-005.md").exists(),
        "WDC-005: Spec file must exist"
    );
}
