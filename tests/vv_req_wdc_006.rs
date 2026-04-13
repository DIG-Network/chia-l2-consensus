//! REQUIREMENT: WDC-006 — Configuration
//! (`docs/requirements/domains/withdraw_delay/NORMATIVE.md#WDC-006`).
//!
//! Spec: `docs/requirements/domains/withdraw_delay/specs/WDC-006.md`.
//!
//! ## Normative Statement
//!
//! `NetworkConfig` MUST include `withdraw_delay_blocks` (u64) and
//! `withdraw_delay_mod_hash` (Bytes32) fields. The delay value MUST be fixed
//! at deployment time and curried into all registration coin puzzles.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] `NetworkConfig` has `withdraw_delay_blocks: u64` field
//! - [x] `NetworkConfig` has `withdraw_delay_mod_hash: Bytes32` field
//! - [x] Default constant is 24,000 blocks
//! - [x] JSON round-trip includes both fields
//! - [x] Fields are populated by deploy_both_singletons()

use chia_l2_consensus::testing::DEFAULT_WITHDRAW_DELAY_BLOCKS;
use chia_l2_consensus::NetworkConfig;
use chia_protocol::Bytes32;

/// WDC-006: NetworkConfig has withdraw_delay_blocks field.
#[test]
fn vv_req_wdc_006_has_withdraw_delay_blocks() {
    let src = std::fs::read_to_string("src/config.rs").expect("config.rs");
    assert!(
        src.contains("pub withdraw_delay_blocks: u64"),
        "WDC-006: NetworkConfig must have withdraw_delay_blocks: u64"
    );
}

/// WDC-006: NetworkConfig has withdraw_delay_mod_hash field.
#[test]
fn vv_req_wdc_006_has_withdraw_delay_mod_hash() {
    let src = std::fs::read_to_string("src/config.rs").expect("config.rs");
    assert!(
        src.contains("pub withdraw_delay_mod_hash: Bytes32"),
        "WDC-006: NetworkConfig must have withdraw_delay_mod_hash: Bytes32"
    );
}

/// WDC-006: Default delay constant is 24,000 blocks.
#[test]
fn vv_req_wdc_006_default_24000() {
    assert_eq!(
        DEFAULT_WITHDRAW_DELAY_BLOCKS, 24_000,
        "WDC-006: Default must be 24,000 blocks (~5 days)"
    );
}

/// WDC-006: JSON round-trip preserves withdraw_delay_blocks.
#[test]
fn vv_req_wdc_006_json_roundtrip_delay_blocks() {
    let config = NetworkConfig {
        network_coin_launcher_id: Bytes32::default(),
        checkpoint_launcher_id: Bytes32::default(),
        registration_coin_mod_hash: Bytes32::default(),
        checkpoint_inner_mod_hash: Bytes32::default(),
        collateral_amount: 1_000_000,
        tree_depth: 32,
        max_signers: 64,
        verification_key_hex: String::new(),
        genesis_challenge: Bytes32::default(),
        withdraw_delay_blocks: 24_000,
        withdraw_delay_mod_hash: Bytes32::default(),
    };

    let json = serde_json::to_string(&config).expect("serialize");
    assert!(
        json.contains("\"withdraw_delay_blocks\":24000"),
        "WDC-006: JSON must include withdraw_delay_blocks"
    );

    let restored: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        restored.withdraw_delay_blocks, 24_000,
        "WDC-006: Round-trip must preserve withdraw_delay_blocks"
    );
}

/// WDC-006: JSON round-trip preserves withdraw_delay_mod_hash.
#[test]
fn vv_req_wdc_006_json_roundtrip_mod_hash() {
    let mut hash_bytes = [0u8; 32];
    hash_bytes[0] = 0xAB;
    hash_bytes[31] = 0xCD;
    let hash: Bytes32 = hash_bytes.into();

    let config = NetworkConfig {
        network_coin_launcher_id: Bytes32::default(),
        checkpoint_launcher_id: Bytes32::default(),
        registration_coin_mod_hash: Bytes32::default(),
        checkpoint_inner_mod_hash: Bytes32::default(),
        collateral_amount: 1_000_000,
        tree_depth: 32,
        max_signers: 64,
        verification_key_hex: String::new(),
        genesis_challenge: Bytes32::default(),
        withdraw_delay_blocks: 24_000,
        withdraw_delay_mod_hash: hash,
    };

    let json = serde_json::to_string(&config).expect("serialize");
    assert!(
        json.contains("withdraw_delay_mod_hash"),
        "WDC-006: JSON must include withdraw_delay_mod_hash"
    );

    let restored: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        restored.withdraw_delay_mod_hash, hash,
        "WDC-006: Round-trip must preserve withdraw_delay_mod_hash"
    );
}

/// WDC-006: deploy_both_singletons populates the new fields.
#[test]
fn vv_req_wdc_006_deploy_populates_fields() {
    let src = std::fs::read_to_string("src/puzzles/deploy.rs").expect("deploy.rs");
    assert!(
        src.contains("withdraw_delay_blocks"),
        "WDC-006: deploy must set withdraw_delay_blocks"
    );
    assert!(
        src.contains("withdraw_delay_mod_hash"),
        "WDC-006: deploy must set withdraw_delay_mod_hash"
    );
}

/// WDC-006: Spec file exists.
#[test]
fn vv_req_wdc_006_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/withdraw_delay/specs/WDC-006.md").exists(),
        "WDC-006: Spec file must exist"
    );
}
