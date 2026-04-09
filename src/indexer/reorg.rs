//! Reorg handling for the indexer.
//!
//! See [spec-indexer.md](../../docs/resources/spec-indexer.md).

use crate::error::ConsensusResult;
use crate::indexer::IndexerCache;

/// Handle a chain reorganization.
pub async fn handle_reorg(_cache: &mut IndexerCache, _reorg_height: u32) -> ConsensusResult<()> {
    // TODO: Implement
    todo!()
}

/// Perform a full reindex from genesis.
pub async fn full_reindex(_cache: &mut IndexerCache) -> ConsensusResult<()> {
    // TODO: Implement
    todo!()
}
