//! Indexer persistent cache.
//!
//! See [spec-indexer.md](../../docs/resources/spec-indexer.md).

use std::path::Path;

use crate::error::ConsensusResult;

/// Persistent cache for indexer state.
#[derive(Debug)]
pub struct IndexerCache {
    // TODO: Add cache storage
    _placeholder: (),
}

impl IndexerCache {
    /// Open or create a cache at the given path.
    pub fn open(_path: &Path) -> ConsensusResult<Self> {
        Ok(Self { _placeholder: () })
    }

    /// Create an in-memory cache (for testing).
    pub fn in_memory() -> Self {
        Self { _placeholder: () }
    }
}
