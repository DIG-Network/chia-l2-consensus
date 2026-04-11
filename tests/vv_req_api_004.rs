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
//! ## How the tests prove the requirement
//! 1. **NotDeployed before sync**: All 4 accessors return NotDeployed.
//! 2. **set_cache_path**: Callable without panic.
//! 3. **config accessible**: tree_depth matches.
//!
//! ## Completeness: MODERATE
//! ## Gaps: Does not test accessors after successful sync (requires simulator).

use chia_l2_consensus::testing::{initial_checkpoint_state, IndexerCache};
use chia_l2_consensus::{ConsensusClient, ConsensusError, NetworkConfig};
use chia_protocol::Bytes32;

fn dummy_config() -> NetworkConfig {
    NetworkConfig {
        network_coin_launcher_id: Bytes32::default(),
        checkpoint_launcher_id: Bytes32::default(),
        registration_coin_mod_hash: Bytes32::default(),
        checkpoint_inner_mod_hash: Bytes32::default(),
        collateral_amount: 0,
        tree_depth: 32,
        max_signers: 20_000,
        verification_key_hex: String::new(),
        genesis_challenge: Bytes32::default(),
    }
}

#[test]
fn vv_req_api_004_not_deployed_before_sync() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());

    assert!(matches!(client.epoch(), Err(ConsensusError::NotDeployed)));
    assert!(matches!(
        client.state_root(),
        Err(ConsensusError::NotDeployed)
    ));
    assert!(matches!(
        client.validator_merkle_root(),
        Err(ConsensusError::NotDeployed)
    ));
    assert!(matches!(
        client.validator_count(),
        Err(ConsensusError::NotDeployed)
    ));
}

#[test]
fn vv_req_api_004_set_cache_path() {
    let mut client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    client.set_cache_path("/tmp/cache.json");
    // No panic = success (cache_path is internal state)
}

#[test]
fn vv_req_api_004_config_accessible() {
    let config = dummy_config();
    let client = ConsensusClient::new(config.clone(), IndexerCache::in_memory());
    assert_eq!(client.config().tree_depth, 32);
}
