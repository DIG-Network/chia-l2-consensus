//! Indexer persistent cache (IDX-005).
//!
//! JSON-based cache with atomic writes for fast restarts. The cache stores
//! the indexer's sync progress and checkpoint history so that restarts do
//! not require a full chain re-scan. Writes use a temp-file-then-rename
//! pattern to guarantee atomicity: either the full new state is persisted
//! or the previous state remains intact.
//!
//! Source: [spec-indexer.md Lines 461-488](../../docs/resources/spec-indexer.md)
//! (Persistent Cache section).
//!
//! # Cross-references
//!
//! - Reorg rollback uses cache history: [`crate::indexer::reorg::ReorgState`]
//! - Validator set rebuilds from cache: [`crate::indexer::validator_set::build_validator_set`]
//! - Merkle consistency check after load: [`crate::indexer::validator_set::verify_merkle_consistency`]

use serde::{Deserialize, Serialize};

use crate::error::{ConsensusError, ConsensusResult};

/// A checkpoint record stored in the cache.
///
/// Mirrors the on-chain checkpoint singleton state for a single epoch.
/// Used during reorg rollback to find the last safe sync point.
///
/// See also: [`crate::indexer::reorg::CheckpointRecord`] (the in-memory counterpart).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCheckpoint {
    pub epoch: u64,
    pub confirmed_at_height: u32,
}

/// Persistent cache for indexer state (IDX-005).
///
/// Stores the indexer's sync progress, checkpoint history, and registration
/// data. Serialized as JSON with atomic writes (write-to-tmp then rename)
/// to prevent corruption from interrupted writes.
///
/// The cache is the primary mechanism for fast restarts. On startup, the
/// indexer loads the cache and resumes syncing from `last_synced_height`
/// instead of scanning from genesis.
///
/// Source: [spec-indexer.md Lines 461-488](../../docs/resources/spec-indexer.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerCache {
    last_synced_height: u32,
    checkpoint_history: Vec<CachedCheckpoint>,
}

impl IndexerCache {
    /// Create an in-memory cache (for testing or first run).
    ///
    /// Starts at height 0 with no checkpoint history. Used when no cache
    /// file exists on disk (first run) or in test environments.
    pub fn in_memory() -> Self {
        Self {
            last_synced_height: 0,
            checkpoint_history: Vec::new(),
        }
    }

    /// Load cache from a JSON file.
    ///
    /// Returns `Ok(None)` if the file does not exist (first run).
    /// Returns `Err` if the file exists but contains invalid JSON (corruption).
    ///
    /// After loading, the caller should verify Merkle consistency via
    /// [`crate::indexer::validator_set::verify_merkle_consistency`] before
    /// trusting the cached validator set.
    ///
    /// Source: [spec-indexer.md Lines 474-478](../../docs/resources/spec-indexer.md).
    pub fn load(path: &str) -> ConsensusResult<Option<Self>> {
        let p = std::path::Path::new(path);
        if !p.exists() {
            return Ok(None);
        }
        let bytes =
            std::fs::read(p).map_err(|e| ConsensusError::CacheError(format!("read: {}", e)))?;
        let cache: Self = serde_json::from_slice(&bytes)
            .map_err(|e| ConsensusError::CacheError(format!("parse: {}", e)))?;
        Ok(Some(cache))
    }

    /// Save cache to a JSON file using atomic write.
    ///
    /// Uses a two-phase write to prevent corruption from interrupted writes:
    ///   1. Serialize to JSON and write to a temporary file (`{path}.tmp`)
    ///   2. Atomically rename the temp file to the target path
    ///
    /// On most filesystems, `rename` is an atomic operation, so the cache
    /// file is either the old version or the new version, never a partial
    /// write. If the process crashes between steps 1 and 2, only the `.tmp`
    /// file is left (ignored on next load).
    ///
    /// Source: [spec-indexer.md Lines 480-487](../../docs/resources/spec-indexer.md).
    pub fn save(&self, path: &str) -> ConsensusResult<()> {
        // Step 1: Serialize to JSON and write to temporary file
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| ConsensusError::CacheError(format!("serialize: {}", e)))?;
        let tmp = format!("{}.tmp", path);
        std::fs::write(&tmp, &bytes)
            .map_err(|e| ConsensusError::CacheError(format!("write tmp: {}", e)))?;
        // Step 2: Atomic rename replaces the target file in one operation
        std::fs::rename(&tmp, path)
            .map_err(|e| ConsensusError::CacheError(format!("rename: {}", e)))?;
        Ok(())
    }

    /// Get the last synced block height.
    ///
    /// This is the height up to which the indexer has processed all coin
    /// additions and removals. On restart, syncing resumes from this height + 1.
    pub fn last_synced_height(&self) -> u32 {
        self.last_synced_height
    }

    /// Set the last synced block height.
    ///
    /// Called at the end of each sync batch after all blocks in the range
    /// have been processed. Should be followed by [`save`](Self::save) to
    /// persist the progress.
    pub fn set_last_synced_height(&mut self, height: u32) {
        self.last_synced_height = height;
    }

    /// Number of cached checkpoint records.
    ///
    /// The checkpoint history grows monotonically during normal operation.
    /// During reorg rollback, entries beyond the rollback point are pruned.
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoint_history.len()
    }

    /// Add a checkpoint record to the cache.
    ///
    /// Called when the indexer observes a checkpoint singleton spend on-chain.
    /// The record is appended to the history (assumed to be in epoch order).
    ///
    /// See also: [`crate::indexer::reorg::ReorgState::record_checkpoint`].
    pub fn add_checkpoint_record(&mut self, epoch: u64, confirmed_at_height: u32) {
        self.checkpoint_history.push(CachedCheckpoint {
            epoch,
            confirmed_at_height,
        });
    }

    /// Get checkpoint history (for reorg rollback).
    ///
    /// During a reorg, the rollback logic scans this history to find the
    /// last checkpoint confirmed at or before the reorg point.
    ///
    /// See: [`crate::indexer::reorg::ReorgState::compute_rollback`].
    pub fn checkpoint_history(&self) -> &[CachedCheckpoint] {
        &self.checkpoint_history
    }
}
