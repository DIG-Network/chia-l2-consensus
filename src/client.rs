//! ConsensusClient — main entry point for L2 consensus operations.
//!
//! See [spec-consensus-crate.md Lines 1579-2154](../docs/resources/spec-consensus-crate.md).

use chia_protocol::Bytes32;
use chia_protocol::SpendBundle;

use crate::config::NetworkConfig;
use crate::error::ConsensusResult;
use crate::indexer::{IndexerCache, IndexerState};
use crate::state::{CheckpointSingletonState, ValidatorSet};

/// Main client for L2 consensus operations.
///
/// The L2 system uses only this client and the types it returns.
pub struct ConsensusClient {
    config: NetworkConfig,
    indexer: IndexerState,
}

impl ConsensusClient {
    /// Create a new client with the given configuration.
    pub fn new(config: NetworkConfig, cache: IndexerCache) -> Self {
        Self {
            config,
            indexer: IndexerState::new(cache),
        }
    }

    /// Get the network configuration.
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Sync with the chain and return current validator set.
    pub async fn sync(&mut self) -> ConsensusResult<ValidatorSet> {
        self.indexer.sync().await
    }

    /// Deploy a new L2 network (genesis).
    pub async fn deploy(&self) -> ConsensusResult<SpendBundle> {
        // TODO: Implement
        todo!()
    }

    /// Register a new validator.
    pub async fn register_validator(&self, _pubkey: &[u8; 48]) -> ConsensusResult<SpendBundle> {
        // TODO: Implement
        todo!()
    }

    /// Submit a checkpoint with proof.
    pub async fn submit_checkpoint(
        &self,
        _new_state_root: Bytes32,
        _signers: &[[u8; 48]],
        _signatures: &[[u8; 96]],
    ) -> ConsensusResult<SpendBundle> {
        // TODO: Implement
        todo!()
    }

    /// Get the current checkpoint state.
    pub async fn get_checkpoint_state(&self) -> ConsensusResult<CheckpointSingletonState> {
        // TODO: Implement
        todo!()
    }

    /// Recover collateral for a validator.
    pub async fn recover_collateral(&self, _pubkey: &[u8; 48]) -> ConsensusResult<SpendBundle> {
        // TODO: Implement
        todo!()
    }
}
