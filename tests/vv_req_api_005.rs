//! REQUIREMENT: API-005 — ConsensusClient Message Computation
//! (`docs/requirements/domains/crate_api/NORMATIVE.md#API-005`).
//!
//! Spec: `docs/requirements/domains/crate_api/specs/API-005.md`.
//!
//! ## Normative statement
//! `ConsensusClient` MUST provide facade methods: `checkpoint_message()`,
//! `validator_signing_message()`, `is_active()`, `membership_announcement()`.
//! Before sync, all MUST return `ConsensusError::NotDeployed`.
//!
//! ## How the tests prove the requirement
//! 1. **checkpoint_message NotDeployed**: Returns error before sync.
//! 2. **signing_message NotDeployed**: Returns error before sync.
//! 3. **is_active NotDeployed**: Returns error before sync.
//! 4. **membership_announcement NotDeployed**: Returns error before sync.
//! 5. **Methods exist**: All 4 methods callable (compile-time + runtime).
//!
//! ## Completeness: MODERATE
//! ## Gaps: Does not test methods after sync (requires simulator integration).

use chia_l2_consensus::testing::IndexerCache;
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
fn vv_req_api_005_checkpoint_message_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    let result = client.checkpoint_message(Bytes32::default(), Bytes32::default(), 0);
    assert!(matches!(result, Err(ConsensusError::NotDeployed)));
}

#[test]
fn vv_req_api_005_signing_message_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    let result = client.validator_signing_message(Bytes32::default(), Bytes32::default(), 0);
    assert!(matches!(result, Err(ConsensusError::NotDeployed)));
}

#[test]
fn vv_req_api_005_is_active_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    let result = client.is_active(&[0xAA; 48]);
    assert!(matches!(result, Err(ConsensusError::NotDeployed)));
}

#[test]
fn vv_req_api_005_membership_announcement_not_deployed() {
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    let result = client.membership_announcement(&[0xAA; 48], false);
    assert!(matches!(result, Err(ConsensusError::NotDeployed)));
}

#[test]
fn vv_req_api_005_methods_exist() {
    // Verify all 5 facade methods exist and are callable
    let client = ConsensusClient::new(dummy_config(), IndexerCache::in_memory());
    let _ = client.checkpoint_message(Bytes32::default(), Bytes32::default(), 0);
    let _ = client.validator_signing_message(Bytes32::default(), Bytes32::default(), 0);
    let _ = client.is_active(&[0; 48]);
    let _ = client.membership_announcement(&[0; 48], true);
}
