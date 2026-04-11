//! ConsensusClient — the single public entry point for L2 consensus operations.
//!
//! The L2 system imports ONLY this type and the types it returns.
//! Everything else (merkle, prover, puzzles, indexer, validator) is internal.
//!
//! ## Lifecycle
//!
//! 1. `ConsensusClient::new(config, cache)` — create client for an existing network
//! 2. `sync()` — fetch current on-chain state from the Chia node
//! 3. `register_validator()` / `submit_checkpoint()` / `recover_collateral()` — operations
//!
//! ## Architecture
//!
//! ```text
//! L2 system
//!     │ uses: ConsensusClient, NetworkConfig, ValidatorSet, SpendBundle, Bytes32
//!     v
//! ConsensusClient (this file)
//!     │ coordinates internally:
//!     │   puzzles/   → spend bundle construction (spec-network-coin, spec-registration-coin, spec-checkpoint-singleton)
//!     │   merkle/    → validator set Merkle tree (spec-sparse-merkle-tree)
//!     │   prover/    → Groth16 proof generation (spec-groth16-circuit)
//!     │   indexer/   → on-chain state tracking (spec-indexer)
//!     v
//! Chia full node RPC
//! ```
//!
//! See [spec-consensus-crate.md Lines 23-57](../docs/resources/spec-consensus-crate.md)
//! for the full crate architecture and public API contract.

use chia_protocol::Bytes32;
use chia_protocol::SpendBundle;

use crate::config::NetworkConfig;
use crate::error::ConsensusResult;
use crate::indexer::{IndexerCache, IndexerState};
use crate::state::{CheckpointSingletonState, ValidatorSet};

/// Main client for L2 consensus operations.
///
/// The L2 system uses only this client and the types it returns.
/// All internal coordination (puzzles, merkle, prover, indexer) is hidden.
///
/// See [spec-consensus-crate.md Lines 1579-1670](../docs/resources/spec-consensus-crate.md)
/// for the full ConsensusClient specification.
pub struct ConsensusClient {
    config: NetworkConfig,
    indexer: IndexerState,
}

impl ConsensusClient {
    /// Create a new client with the given configuration and optional cache.
    ///
    /// Does NOT sync with the chain. Call `sync()` before any operation
    /// that depends on current on-chain state.
    ///
    /// See [spec-consensus-crate.md Lines 1597-1602](../docs/resources/spec-consensus-crate.md).
    pub fn new(config: NetworkConfig, cache: IndexerCache) -> Self {
        Self {
            config,
            indexer: IndexerState::new(cache),
        }
    }

    /// Get the network configuration (deployment parameters).
    ///
    /// Returns the immutable config set at deployment time: launcher IDs,
    /// collateral amount, tree depth, VK, genesis challenge.
    ///
    /// See [spec-consensus-crate.md Lines 226-314](../docs/resources/spec-consensus-crate.md).
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Sync with the Chia node to update local state.
    ///
    /// Drives the indexer to:
    /// 1. Fetch current network coin and checkpoint singleton state
    /// 2. Detect and verify all registration coins (lineage check per IDX-002)
    /// 3. Rebuild the sparse Merkle tree from verified validators
    /// 4. Verify Merkle root matches on-chain `validator_merkle_root` (IDX-003)
    ///
    /// Returns `StateMismatch` if local tree root ≠ on-chain root.
    ///
    /// See [spec-consensus-crate.md Lines 1628-1670](../docs/resources/spec-consensus-crate.md).
    /// See [spec-indexer.md Lines 1-50](../docs/resources/spec-indexer.md) for sync algorithm.
    pub async fn sync(&mut self) -> ConsensusResult<ValidatorSet> {
        self.indexer.sync().await
    }

    /// Deploy a new L2 network (genesis).
    ///
    /// Creates both singletons (network coin + checkpoint) in one atomic
    /// spend bundle from the genesis coin. Returns the bundle to submit
    /// and the resulting NetworkConfig for future use.
    ///
    /// See [spec-consensus-crate.md Lines 1684-1729](../docs/resources/spec-consensus-crate.md).
    /// See [spec-deployment-runbook.md Lines 1-50](../docs/resources/spec-deployment-runbook.md).
    pub async fn deploy(&self) -> ConsensusResult<SpendBundle> {
        // TODO: Implement — needs FullNodeClient parameter, genesis coin, VK
        // See deploy_both_singletons() in puzzles/deploy.rs for the bundle construction
        todo!()
    }

    /// Register a new validator by spending the network coin.
    ///
    /// Builds a spend bundle that:
    /// 1. Spends the network coin singleton with the validator's BLS pubkey
    /// 2. Creates a registration coin with collateral (NET-003)
    /// 3. Includes AGG_SIG_ME proving the validator controls the pubkey (NET-002)
    /// 4. Includes pubkey memo for indexer detection (NET-005)
    ///
    /// See [spec-consensus-crate.md Lines 1742-1790](../docs/resources/spec-consensus-crate.md).
    /// See [spec-network-coin.md Lines 100-200](../docs/resources/spec-network-coin.md).
    pub async fn register_validator(&self, _pubkey: &[u8; 48]) -> ConsensusResult<SpendBundle> {
        // TODO: Implement — needs current network coin state from sync()
        // See register_validator() in puzzles/network_coin.rs
        todo!()
    }

    /// Submit a checkpoint with Groth16 proof and BLS aggregate signature.
    ///
    /// This is the core L2 → L1 settlement operation. It:
    /// 1. Computes the checkpoint message: sha256(state_root ‖ merkle_root ‖ count ‖ epoch)
    /// 2. Generates Groth16 proof (off-chain, 5-15 minutes)
    /// 3. Aggregates BLS signatures from k validators
    /// 4. Builds the checkpoint singleton spend bundle
    ///
    /// The proof proves membership + majority (CIR-001). The BLS signature
    /// proves the majority actually signed (CHK-003). Both are required.
    ///
    /// See [spec-consensus-crate.md Lines 1806-1900](../docs/resources/spec-consensus-crate.md).
    /// See [spec-checkpoint-singleton.md Lines 1-100](../docs/resources/spec-checkpoint-singleton.md).
    pub async fn submit_checkpoint(
        &self,
        _new_state_root: Bytes32,
        _signers: &[[u8; 48]],
        _signatures: &[[u8; 96]],
    ) -> ConsensusResult<SpendBundle> {
        // TODO: Implement — needs proving key loaded via load_proving_key()
        // See generate_proof() in prover/prove.rs
        // See build_chk_path_env() in tests/vv_req_chk_008.rs for reference
        todo!()
    }

    /// Get the current checkpoint singleton state.
    ///
    /// Returns the on-chain state: epoch, state_root, validator_merkle_root,
    /// validator_count. Requires sync() to have been called.
    ///
    /// See [spec-consensus-crate.md Lines 2129-2149](../docs/resources/spec-consensus-crate.md).
    pub async fn get_checkpoint_state(&self) -> ConsensusResult<CheckpointSingletonState> {
        // TODO: Implement — read from indexer cache
        todo!()
    }

    /// Recover collateral for an exited validator.
    ///
    /// Builds a two-spend atomic bundle:
    /// 1. Checkpoint singleton membership query (permissionless, CHK-005/CHK-006)
    ///    → emits non-membership announcement
    /// 2. Registration coin spend asserting the announcement (REG-004)
    ///    → returns collateral to destination
    ///
    /// The validator must already be excluded from the Merkle tree
    /// (i.e., a checkpoint that excludes them must have been accepted).
    ///
    /// See [spec-consensus-crate.md Lines 2011-2055](../docs/resources/spec-consensus-crate.md).
    /// See [spec-registration-coin.md Lines 200-300](../docs/resources/spec-registration-coin.md).
    pub async fn recover_collateral(&self, _pubkey: &[u8; 48]) -> ConsensusResult<SpendBundle> {
        // TODO: Implement — see prepare_collateral_recovery() in validator/exit.rs
        // for the off-chain parameter computation
        todo!()
    }
}
