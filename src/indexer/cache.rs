//! Indexer persistent cache (IDX-005).
//!
//! JSON-based cache with atomic writes for fast restarts.
//! See [spec-indexer.md Lines 461-488](../../docs/resources/spec-indexer.md).

use serde::{Deserialize, Serialize};

use crate::error::{ConsensusError, ConsensusResult};

/// A checkpoint record stored in the cache.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerCache {
    last_synced_height: u32,
    checkpoint_history: Vec<CachedCheckpoint>,
}

impl IndexerCache {
    /// Create an in-memory cache (for testing or first run).
    pub fn in_memory() -> Self {
        Self {
            last_synced_height: 0,
            checkpoint_history: Vec::new(),
        }
    }

    /// Load cache from a JSON file.
    ///
    /// Returns `Ok(None)` if the file does not exist (first run).
    /// Returns `Err` if the file exists but is corrupted.
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
    /// Writes to a temporary file first, then renames to the target path.
    /// This prevents partial writes from corrupting the cache.
    pub fn save(&self, path: &str) -> ConsensusResult<()> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| ConsensusError::CacheError(format!("serialize: {}", e)))?;
        let tmp = format!("{}.tmp", path);
        std::fs::write(&tmp, &bytes)
            .map_err(|e| ConsensusError::CacheError(format!("write tmp: {}", e)))?;
        std::fs::rename(&tmp, path)
            .map_err(|e| ConsensusError::CacheError(format!("rename: {}", e)))?;
        Ok(())
    }

    /// Get the last synced block height.
    pub fn last_synced_height(&self) -> u32 {
        self.last_synced_height
    }

    /// Set the last synced block height.
    pub fn set_last_synced_height(&mut self, height: u32) {
        self.last_synced_height = height;
    }

    /// Number of cached checkpoint records.
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoint_history.len()
    }

    /// Add a checkpoint record to the cache.
    pub fn add_checkpoint_record(&mut self, epoch: u64, confirmed_at_height: u32) {
        self.checkpoint_history.push(CachedCheckpoint {
            epoch,
            confirmed_at_height,
        });
    }

    /// Get checkpoint history (for reorg rollback).
    pub fn checkpoint_history(&self) -> &[CachedCheckpoint] {
        &self.checkpoint_history
    }
}
