//! Chain indexer module.
//!
//! See [spec-consensus-crate.md Lines 1274-1579](../docs/resources/spec-consensus-crate.md).

mod cache;
mod chain;
mod reorg;
mod validator_set;

pub use cache::IndexerCache;
pub use chain::*;
pub use reorg::*;
pub use validator_set::*;

use crate::error::ConsensusResult;
use crate::state::ValidatorSet;

/// Indexer state tracking on-chain consensus state.
#[derive(Debug)]
pub struct IndexerState {
    cache: IndexerCache,
}

impl IndexerState {
    /// Create a new indexer.
    pub fn new(cache: IndexerCache) -> Self {
        Self { cache }
    }

    /// Sync with the chain and return current validator set.
    pub async fn sync(&mut self) -> ConsensusResult<ValidatorSet> {
        // TODO: Implement
        todo!()
    }
}
