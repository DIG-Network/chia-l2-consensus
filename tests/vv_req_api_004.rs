//! REQUIREMENT: API-004 — ConsensusClient State Accessors
//! (`docs/requirements/domains/crate_api/NORMATIVE.md#API-004`).
//!
//! Spec: `docs/requirements/domains/crate_api/specs/API-004.md`.
//!
//! ## Normative statement
//! `ConsensusClient` MUST provide state accessor methods: `epoch()`,
//! `state_root()`, `validator_merkle_root()`, `validator_count()`. Before
//! sync, all MUST return `ConsensusError::NotDeployed`. The client MUST
//! also provide `set_cache_path()` and `config()`.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] epoch() exists and returns NotDeployed before sync
//! - [x] state_root() exists and returns NotDeployed before sync
//! - [x] validator_merkle_root() exists and returns NotDeployed before sync
//! - [x] validator_count() exists and returns NotDeployed before sync
//! - [x] set_cache_path() callable without panic
//! - [x] config() returns deployment parameters
//! - [x] All accessor methods exist in source
//! - [x] checkpoint_message() returns NotDeployed before sync
//! - [x] is_active() returns NotDeployed before sync

use chia_l2_consensus::testing::IndexerCache;
use chia_l2_consensus::{ConsensusClient, ConsensusError, NetworkConfig};
use chia_protocol::Bytes32;

fn dummy_config() -> NetworkConfig {
    NetworkConfig {
        network_coin_launcher_id: Bytes32::default(),
        checkpoint_launcher_id: Bytes32::default(),
        registration_coin_mod_hash: Bytes32::default(),
        checkpoint_inner_mod_hash: Bytes32::default(),
        collateral_amount: 1_000_000,
        tree_depth: 32,
        max_signers: 20_000,
        verification_key_hex: String::new(),
        genesis_challenge: Bytes32::default(),
        withdraw_delay_blocks: 24_000,
        withdraw_delay_mod_hash: Bytes32::default(),
    }
}

/// API-004: epoch() returns NotDeployed before sync.
#[test]
fn vv_req_api_004_epoch_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    assert!(matches!(client.epoch(), Err(ConsensusError::NotDeployed)));
}

/// API-004: state_root() returns NotDeployed before sync.
#[test]
fn vv_req_api_004_state_root_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    assert!(matches!(
        client.state_root(),
        Err(ConsensusError::NotDeployed)
    ));
}

/// API-004: validator_merkle_root() returns NotDeployed before sync.
#[test]
fn vv_req_api_004_merkle_root_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    assert!(matches!(
        client.validator_merkle_root(),
        Err(ConsensusError::NotDeployed)
    ));
}

/// API-004: validator_count() returns NotDeployed before sync.
#[test]
fn vv_req_api_004_count_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    assert!(matches!(
        client.validator_count(),
        Err(ConsensusError::NotDeployed)
    ));
}

/// API-004: set_cache_path() callable without panic.
#[test]
fn vv_req_api_004_set_cache_path() {
    let mut client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    client.set_cache_path("/tmp/test_cache.json");
}

/// API-004: config() returns deployment parameters.
#[test]
fn vv_req_api_004_config_accessible() {
    let config = dummy_config();
    let client = ConsensusClient::new(config.clone(), IndexerCache::in_memory());
    assert_eq!(client.config().tree_depth, 32);
    assert_eq!(client.config().collateral_amount, 1_000_000);
    assert_eq!(client.config().max_signers, 20_000);
    assert_eq!(client.config().withdraw_delay_blocks, 24_000);
}

/// API-004: All accessor methods exist in source.
#[test]
fn vv_req_api_004_all_accessors_exist() {
    let src = std::fs::read_to_string("src/client.rs").unwrap();
    assert!(src.contains("pub fn epoch("));
    assert!(src.contains("pub fn state_root("));
    assert!(src.contains("pub fn validator_merkle_root("));
    assert!(src.contains("pub fn validator_count("));
    assert!(src.contains("pub fn set_cache_path("));
    assert!(src.contains("pub fn config("));
}

/// API-004: checkpoint_message() returns NotDeployed before sync.
#[test]
fn vv_req_api_004_checkpoint_message_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    assert!(matches!(
        client.checkpoint_message(Bytes32::default(), Bytes32::default(), 0),
        Err(ConsensusError::NotDeployed)
    ));
}

/// API-004: is_active() returns NotDeployed before sync.
#[test]
fn vv_req_api_004_is_active_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    assert!(matches!(
        client.is_active(&[0u8; 48]),
        Err(ConsensusError::NotDeployed)
    ));
}
