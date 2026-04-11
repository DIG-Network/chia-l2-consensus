//! Reorg handling for the indexer (IDX-004).
//!
//! On blockchain reorganization, the indexer rolls back to the last safe
//! checkpoint before the reorg point and re-indexes forward. If no safe
//! point exists, full re-index from genesis is required.
//!
//! See [spec-indexer.md Lines 405-457](../../docs/resources/spec-indexer.md).

use chia_protocol::Bytes32;

/// A recorded checkpoint with its confirmation height.
#[derive(Debug, Clone)]
pub struct CheckpointRecord {
    /// Epoch number of this checkpoint.
    pub epoch: u64,
    /// Block height at which the checkpoint was confirmed on-chain.
    pub confirmed_at_height: u32,
}

/// Reorg-aware indexer state machine (IDX-004).
///
/// Tracks checkpoint history, registration coins, and sync progress.
/// Provides rollback operations for reorg handling without requiring
/// a full node connection — the rollback logic is pure state manipulation.
#[derive(Debug)]
pub struct ReorgState {
    /// Last block height the indexer has processed.
    last_synced_height: u32,
    /// Checkpoint history ordered by epoch (ascending).
    checkpoint_history: Vec<CheckpointRecord>,
    /// Registration coin IDs currently tracked.
    registration_coins: Vec<Bytes32>,
}

impl ReorgState {
    /// Create a new empty reorg state.
    pub fn new() -> Self {
        Self {
            last_synced_height: 0,
            checkpoint_history: Vec::new(),
            registration_coins: Vec::new(),
        }
    }

    /// Current last synced height.
    pub fn last_synced_height(&self) -> u32 {
        self.last_synced_height
    }

    /// Set the last synced height (called during normal sync).
    pub fn set_last_synced_height(&mut self, height: u32) {
        self.last_synced_height = height;
    }

    /// Number of recorded checkpoints.
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoint_history.len()
    }

    /// Number of tracked registration coins.
    pub fn registration_count(&self) -> usize {
        self.registration_coins.len()
    }

    /// Record a new checkpoint (called when indexer observes a checkpoint spend).
    pub fn record_checkpoint(&mut self, record: CheckpointRecord) {
        self.checkpoint_history.push(record);
    }

    /// Record a registration coin (called when indexer observes a valid registration).
    pub fn record_registration(&mut self, coin_id: Bytes32) {
        self.registration_coins.push(coin_id);
    }

    /// Detect whether a reorg has occurred.
    ///
    /// A reorg is detected when the chain's peak height is STRICTLY lower
    /// than the indexer's last synced height.
    pub fn is_reorg(&self, peak_height: u32) -> bool {
        peak_height < self.last_synced_height
    }

    /// Find the last safe checkpoint at or before the given height.
    ///
    /// Returns `None` if no checkpoint exists at or before `reorg_height`,
    /// meaning a full re-index is needed.
    pub fn compute_rollback(&self, reorg_height: u32) -> Option<CheckpointRecord> {
        self.checkpoint_history
            .iter()
            .rev()
            .find(|c| c.confirmed_at_height <= reorg_height)
            .cloned()
    }

    /// Apply a rollback to a safe checkpoint.
    ///
    /// - Truncates checkpoint history to epochs <= the safe checkpoint's epoch
    /// - Clears all registration coins (they will be re-indexed)
    /// - Sets last_synced_height to the safe checkpoint's height
    pub fn apply_rollback(&mut self, safe: &CheckpointRecord) {
        self.checkpoint_history.retain(|c| c.epoch <= safe.epoch);
        self.registration_coins.clear();
        self.last_synced_height = safe.confirmed_at_height;
    }

    /// Apply a full re-index (no safe checkpoint found).
    ///
    /// Clears ALL state — checkpoint history, registration coins, and
    /// resets sync height to 0. The caller must then re-sync from genesis.
    pub fn apply_full_reindex(&mut self) {
        self.checkpoint_history.clear();
        self.registration_coins.clear();
        self.last_synced_height = 0;
    }
}

impl Default for ReorgState {
    fn default() -> Self {
        Self::new()
    }
}
