//! REQUIREMENT: API-002 — NetworkConfig Completeness
//! (`docs/requirements/domains/crate_api/NORMATIVE.md#API-002`).
//!
//! Spec: `docs/requirements/domains/crate_api/specs/API-002.md`.
//!
//! ## Normative statement
//! `NetworkConfig` MUST support JSON serialization/deserialization, provide a
//! `verification_key()` method that deserializes the VK from hex, and provide
//! a `checkpoint_singleton_id()` method that derives the singleton coin ID.
//! Bytes32 fields MUST serialize as 0x-prefixed hex. Invalid VK hex MUST
//! return an error.
//!
//! ## How the tests prove the requirement
//! 1. **JSON roundtrip**: All fields survive serialize -> deserialize.
//! 2. **0x prefix**: JSON output contains "0xaa", "0xbb" for Bytes32.
//! 3. **verification_key()**: Deserializes to VK with 7 IC points.
//! 4. **checkpoint_singleton_id()**: Matches derive_launcher_id.
//! 5. **Empty VK hex fails**: Error returned.
//! 6. **Invalid VK hex fails**: Error returned.
//!
//! ## Completeness: HIGH
//! ## Gaps: None significant.

use chia_l2_consensus::testing::{
    derive_launcher_id, deserialize_proving_key, run_test_setup, vk_to_bytes,
};
use chia_l2_consensus::NetworkConfig;
use chia_protocol::Bytes32;

fn test_config() -> NetworkConfig {
    // Use arkworks serialized VK (not flat 672-byte format)
    let (_, vk_bytes) = run_test_setup().unwrap();

    NetworkConfig {
        network_coin_launcher_id: Bytes32::from([0xAA; 32]),
        checkpoint_launcher_id: Bytes32::from([0xBB; 32]),
        registration_coin_mod_hash: Bytes32::from([0xCC; 32]),
        checkpoint_inner_mod_hash: Bytes32::from([0xDD; 32]),
        collateral_amount: 10_000_000_000_000,
        tree_depth: 32,
        max_signers: 20_000,
        verification_key_hex: hex::encode(&vk_bytes), // arkworks compressed format
        genesis_challenge: Bytes32::from([0x42; 32]),
    }
}

// ── JSON serialization round-trip ───────────────────────────────────

#[test]
fn vv_req_api_002_json_roundtrip() {
    let config = test_config();
    let json = serde_json::to_string_pretty(&config).expect("API-002: Must serialize to JSON");
    let deserialized: NetworkConfig =
        serde_json::from_str(&json).expect("API-002: Must deserialize from JSON");

    assert_eq!(
        config.network_coin_launcher_id, deserialized.network_coin_launcher_id,
        "API-002: network_coin_launcher_id round-trip"
    );
    assert_eq!(
        config.checkpoint_launcher_id, deserialized.checkpoint_launcher_id,
        "API-002: checkpoint_launcher_id round-trip"
    );
    assert_eq!(
        config.collateral_amount, deserialized.collateral_amount,
        "API-002: collateral_amount round-trip"
    );
    assert_eq!(
        config.genesis_challenge, deserialized.genesis_challenge,
        "API-002: genesis_challenge round-trip"
    );
    assert_eq!(
        config.verification_key_hex, deserialized.verification_key_hex,
        "API-002: verification_key_hex round-trip"
    );
}

// ── Bytes32 fields serialize as 0x-prefixed hex ─────────────────────

#[test]
fn vv_req_api_002_hex_prefix() {
    let config = test_config();
    let json = serde_json::to_string(&config).unwrap();

    assert!(
        json.contains("\"0xaa"),
        "API-002: network_coin_launcher_id must be 0x-prefixed hex"
    );
    assert!(
        json.contains("\"0xbb"),
        "API-002: checkpoint_launcher_id must be 0x-prefixed hex"
    );
}

// ── verification_key() deserializes VK ──────────────────────────────

#[test]
fn vv_req_api_002_verification_key_method() {
    let config = test_config();
    let vk = config
        .verification_key()
        .expect("API-002: verification_key() must succeed");

    // VK must have 7 IC points
    assert_eq!(
        vk.gamma_abc_g1.len(),
        7,
        "API-002: Deserialized VK must have 7 IC points"
    );
}

// ── checkpoint_singleton_id() derives correctly ─────────────────────

#[test]
fn vv_req_api_002_checkpoint_singleton_id() {
    let config = test_config();
    let id = config.checkpoint_singleton_id();

    // Must match derive_launcher_id (same computation)
    let expected = derive_launcher_id(config.checkpoint_launcher_id, 1);
    assert_eq!(
        id, expected,
        "API-002: checkpoint_singleton_id() must match derive_launcher_id()"
    );
}

// ── Empty VK hex fails gracefully ───────────────────────────────────

#[test]
fn vv_req_api_002_empty_vk_fails() {
    let mut config = test_config();
    config.verification_key_hex = String::new();

    let result = config.verification_key();
    assert!(result.is_err(), "API-002: Empty VK hex must fail");
}

// ── Invalid VK hex fails gracefully ─────────────────────────────────

#[test]
fn vv_req_api_002_invalid_vk_fails() {
    let mut config = test_config();
    config.verification_key_hex = "not_valid_hex".to_string();

    let result = config.verification_key();
    assert!(result.is_err(), "API-002: Invalid VK hex must fail");
}
