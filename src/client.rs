//! ConsensusClient — the single public entry point for L2 consensus operations.
//!
//! The L2 system imports ONLY this type and the types it returns.
//! Everything else (merkle, prover, puzzles, indexer, validator) is internal.
//!
//! ## Lifecycle
//!
//! 1. `ConsensusClient::new(config, cache)` — create client for an existing network
//! 2. `sync()` — fetch current on-chain state from the Chia node
//! 3. `register_validator()` / `build_checkpoint()` / `recover_collateral()` — operations
//!
//! All methods that produce coin spends return `SpendBundle`. The crate NEVER
//! broadcasts transactions. The importing project is responsible for submitting
//! bundles to a Chia full node via `push_tx()` or equivalent.
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
use chia_query::{ChiaQuery, ChiaQueryConfig};

use crate::config::NetworkConfig;
use crate::error::{ConsensusError, ConsensusResult};
use crate::indexer::{IndexerCache, IndexerState};
use crate::state::{CheckpointSingletonState, ValidatorSet};

/// Convert a Bytes32 to a 0x-prefixed hex string for chia-query API calls.
///
/// RPC-001: chia-query uses `&str` hex strings, not `Bytes32`.
pub fn bytes32_to_hex(b: &Bytes32) -> String {
    format!("0x{}", hex::encode(b.as_ref()))
}

/// Convert a 0x-prefixed hex string back to Bytes32.
///
/// RPC-001: Used when parsing chia-query responses back to our types.
pub fn hex_to_bytes32(s: &str) -> ConsensusResult<Bytes32> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s).map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    Bytes32::try_from(bytes.as_slice())
        .map_err(|_| ConsensusError::SerializationError("invalid Bytes32 length".into()))
}

/// Main client for L2 consensus operations.
///
/// The L2 system uses only this client and the types it returns.
/// All internal coordination (puzzles, merkle, prover, indexer) is hidden.
///
/// RPC-001: Uses `ChiaQuery` for all blockchain queries (decentralized
/// peers + coinset.org fallback). The crate NEVER broadcasts transactions.
///
/// See [spec-consensus-crate.md Lines 1579-1670](../docs/resources/spec-consensus-crate.md)
/// for the full ConsensusClient specification.
pub struct ConsensusClient {
    config: NetworkConfig,
    /// RPC-001: Blockchain query backend — shared with indexer and puzzle drivers.
    query: Option<ChiaQuery>,
    indexer: IndexerState,
    /// Cached checkpoint state from last sync(). None before first sync.
    state: Option<CheckpointSingletonState>,
    /// Path for indexer persistent cache.
    cache_path: Option<String>,
}

impl ConsensusClient {
    /// Create a new client with the given configuration and optional cache.
    ///
    /// Does NOT sync with the chain. Call `sync()` before any operation
    /// that depends on current on-chain state.
    ///
    /// The `ChiaQuery` backend is NOT initialized here — call
    /// `connect()` to establish blockchain connectivity.
    ///
    /// See [spec-consensus-crate.md Lines 1597-1602](../docs/resources/spec-consensus-crate.md).
    pub fn new(config: NetworkConfig, cache: IndexerCache) -> Self {
        Self {
            config,
            query: None,
            indexer: IndexerState::new(cache),
            state: None,
            cache_path: None,
        }
    }

    /// RPC-001: Connect to the Chia blockchain via chia-query.
    ///
    /// Initializes the `ChiaQuery` backend with decentralized peer
    /// connections and coinset.org fallback. Must be called before `sync()`.
    ///
    /// The `ChiaQueryConfig` controls network type (mainnet/testnet11),
    /// max peers, timeouts, and fallback settings.
    pub async fn connect(&mut self, query_config: ChiaQueryConfig) -> ConsensusResult<()> {
        let query = ChiaQuery::new(query_config)
            .await
            .map_err(|e| ConsensusError::RpcError(e.to_string()))?;
        self.query = Some(query);
        Ok(())
    }

    /// Get a reference to the ChiaQuery backend, or error if not connected.
    pub(crate) fn query(&self) -> ConsensusResult<&ChiaQuery> {
        self.query
            .as_ref()
            .ok_or_else(|| ConsensusError::RpcError("not connected — call connect() first".into()))
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
    /// Returns SpendBundle — the caller broadcasts it (API-008).
    ///
    /// RPC-005: Uses `dig-l1-wallet` for coin selection to fund collateral.
    /// The `L1Wallet` is passed per-call, NOT stored in ConsensusClient.
    ///
    /// Builds a spend bundle that:
    /// 1. Selects XCH coins from wallet to fund collateral + fee (RPC-005)
    /// 2. Spends the network coin singleton with the validator's BLS pubkey
    /// 3. Creates a registration coin with collateral (NET-003)
    /// 4. Includes AGG_SIG_ME proving the validator controls the pubkey (NET-002)
    /// 5. Includes pubkey memo for indexer detection (NET-005)
    ///
    /// Returns `InsufficientFunds` if wallet balance < collateral + fee.
    ///
    /// See [spec-consensus-crate.md](../docs/resources/spec-consensus-crate.md).
    /// See [spec-network-coin.md](../docs/resources/spec-network-coin.md).
    pub async fn register_validator(
        &self,
        _pubkey: &[u8; 48],
        _wallet: &dig_l1_wallet::L1Wallet,
        _wallet_name: &str,
        _account_index: u32,
        _fee: u64,
    ) -> ConsensusResult<SpendBundle> {
        // TODO: Implement (RPC-002 + RPC-005):
        // 1. wallet.select_coins(wallet_name, Some(account_index), collateral + fee, LargestFirst)
        // 2. Build network coin spend via puzzles/network_coin.rs
        // 3. Build funding spends from selected coins
        // 4. Combine into single SpendBundle
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
    pub async fn build_checkpoint(
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

    /// Initiate collateral recovery for an exited validator.
    /// Returns SpendBundle — the caller broadcasts it.
    ///
    /// WDC-004: This creates a WITHDRAW DELAY COIN that holds the collateral
    /// for withdraw_delay_blocks L1 blocks. After the delay, call
    /// release_collateral() to send the funds to the destination.
    ///
    /// Builds a two-spend atomic bundle:
    /// 1. Checkpoint singleton membership query (permissionless, CHK-005/CHK-006)
    ///    → emits non-membership announcement
    /// 2. Registration coin spend asserting the announcement (REG-004)
    ///    → creates withdraw delay coin (not direct destination)
    ///
    /// The validator must already be excluded from the Merkle tree
    /// (i.e., a checkpoint that excludes them must have been accepted).
    ///
    /// See [spec-consensus-crate.md](../docs/resources/spec-consensus-crate.md).
    /// See [spec-registration-coin.md](../docs/resources/spec-registration-coin.md).
    pub async fn recover_collateral(&self, _pubkey: &[u8; 48]) -> ConsensusResult<SpendBundle> {
        // TODO: Implement — see prepare_collateral_recovery() in validator/exit.rs
        // for the off-chain parameter computation
        todo!()
    }

    /// Release collateral from a withdraw delay coin after the delay period.
    /// Returns SpendBundle — the caller broadcasts it (API-008).
    ///
    /// WDC-005: This is the second step of two-phase collateral recovery:
    ///   1. `recover_collateral()` — creates the withdraw delay coin
    ///   2. [wait `withdraw_delay_blocks` L1 blocks (~5 days default)]
    ///   3. `release_collateral()` — releases funds to destination
    ///
    /// If the delay has not elapsed, the Chia node will reject the bundle
    /// (ASSERT_HEIGHT_RELATIVE failure).
    ///
    /// No signature required (WDC-007: permissionless after delay).
    /// CLVM cost: ~2.4M units.
    ///
    /// See [spec-withdraw-delay-coin.md](../docs/resources/spec-withdraw-delay-coin.md).
    pub async fn release_collateral(
        &self,
        _withdraw_delay_coin: Bytes32,
        _destination: Bytes32,
        _amount: u64,
    ) -> ConsensusResult<SpendBundle> {
        // TODO: Implement using puzzles::withdraw_delay::release_collateral()
        todo!()
    }

    // ================================================================
    // API-004: State Accessors
    // ================================================================

    /// Get the internal state, or NotDeployed if sync() hasn't been called.
    fn require_state(&self) -> ConsensusResult<&CheckpointSingletonState> {
        self.state
            .as_ref()
            .ok_or(crate::error::ConsensusError::NotDeployed)
    }

    /// Current epoch from the checkpoint singleton.
    /// Primary health signal — stalled epoch means checkpoints are stuck.
    ///
    /// See [spec-consensus-crate.md Lines 2129-2131](../docs/resources/spec-consensus-crate.md).
    pub fn epoch(&self) -> ConsensusResult<u64> {
        Ok(self.require_state()?.epoch)
    }

    /// Current L2 state root from the checkpoint singleton.
    ///
    /// See [spec-consensus-crate.md Lines 2132-2134](../docs/resources/spec-consensus-crate.md).
    pub fn state_root(&self) -> ConsensusResult<Bytes32> {
        Ok(self.require_state()?.state_root)
    }

    /// Current on-chain Merkle root of the active validator set.
    ///
    /// See [spec-consensus-crate.md Lines 2136-2140](../docs/resources/spec-consensus-crate.md).
    pub fn validator_merkle_root(&self) -> ConsensusResult<Bytes32> {
        Ok(self.require_state()?.validator_merkle_root)
    }

    /// On-chain validator count from the checkpoint singleton.
    ///
    /// See [spec-consensus-crate.md Lines 2145-2147](../docs/resources/spec-consensus-crate.md).
    pub fn validator_count(&self) -> ConsensusResult<u64> {
        Ok(self.require_state()?.validator_count)
    }

    /// Set the indexer cache file path for persistent storage.
    ///
    /// See [spec-consensus-crate.md Lines 1604-1608](../docs/resources/spec-consensus-crate.md).
    pub fn set_cache_path(&mut self, path: &str) {
        self.cache_path = Some(path.to_string());
    }

    // ================================================================
    // API-005: Message Computation Facades
    // ================================================================

    /// Compute the checkpoint message for a proposed state transition.
    ///
    /// Uses current epoch+1 as new_epoch. Delegates to `compute_checkpoint_message()`.
    ///
    /// See [spec-consensus-crate.md Lines 1903-1918](../docs/resources/spec-consensus-crate.md).
    pub fn checkpoint_message(
        &self,
        new_state_root: Bytes32,
        new_validator_merkle_root: Bytes32,
        new_validator_count: u64,
    ) -> ConsensusResult<[u8; 32]> {
        let state = self.require_state()?;
        let new_epoch = state.epoch + 1;
        Ok(crate::prover::compute_checkpoint_message(
            new_state_root.into(),
            new_validator_merkle_root.into(),
            new_validator_count,
            new_epoch,
            self.config.network_coin_launcher_id.into(),
        ))
    }

    /// Compute the full 96-byte message each validator signs.
    ///
    /// = checkpoint_message + genesis_challenge + checkpoint_singleton_coin_id
    ///
    /// See [spec-consensus-crate.md Lines 1920-1945](../docs/resources/spec-consensus-crate.md).
    pub fn validator_signing_message(
        &self,
        new_state_root: Bytes32,
        new_validator_merkle_root: Bytes32,
        new_validator_count: u64,
    ) -> ConsensusResult<[u8; 96]> {
        let msg = self.checkpoint_message(
            new_state_root,
            new_validator_merkle_root,
            new_validator_count,
        )?;
        let state = self.require_state()?;
        let gc: [u8; 32] = self.config.genesis_challenge.into();
        let cid: [u8; 32] = state.coin.coin_id().into();
        let mut result = [0u8; 96];
        result[0..32].copy_from_slice(&msg);
        result[32..64].copy_from_slice(&gc);
        result[64..96].copy_from_slice(&cid);
        Ok(result)
    }

    /// Fast local membership check. No RPC call. Call sync() first.
    ///
    /// See [spec-consensus-crate.md Lines 2060-2070](../docs/resources/spec-consensus-crate.md).
    pub fn is_active(&self, pubkey: &[u8; 48]) -> ConsensusResult<bool> {
        // Requires validator set from sync — for now, check state exists
        let _ = self.require_state()?;
        // TODO: Check against the local validator set when sync() populates it
        Ok(false)
    }

    /// Compute the membership announcement hash for AssertCoinAnnouncement.
    ///
    /// Uses current epoch and checkpoint coin ID.
    ///
    /// See [spec-consensus-crate.md Lines 2108-2120](../docs/resources/spec-consensus-crate.md).
    pub fn membership_announcement(
        &self,
        pubkey: &[u8; 48],
        is_member: bool,
    ) -> ConsensusResult<[u8; 32]> {
        let state = self.require_state()?;
        let inner =
            crate::prover::compute_membership_announcement_message(state.epoch, pubkey, is_member);
        // Full announcement = sha256(checkpoint_coin_id + inner)
        use sha2::{Digest, Sha256};
        let cid: [u8; 32] = state.coin.coin_id().into();
        let mut hasher = Sha256::new();
        hasher.update(cid);
        hasher.update(inner);
        Ok(hasher.finalize().into())
    }
}
