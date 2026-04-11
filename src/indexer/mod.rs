//! Chain indexer — tracks on-chain state for the L2 consensus system.
//!
//! The indexer queries the Chia full node for:
//! 1. Network coin singleton state (current coin, lineage)
//! 2. Checkpoint singleton state (epoch, roots, count)
//! 3. All registration coins (with lineage verification back to network coin)
//! 4. Checkpoint history for reorg handling
//!
//! ## Sub-modules
//!
//! | Module | Purpose | Requirements |
//! |--------|---------|-------------|
//! | `cache` | Persistent JSON cache for fast restarts | IDX-005 |
//! | `chain` | Raw Chia node RPC queries | IDX-001 |
//! | `reorg` | Blockchain reorg detection and rollback | IDX-004 |
//! | `validator_set` | Lineage verification + Merkle consistency | IDX-002, IDX-003 |
//!
//! ## Sync Algorithm
//!
//! ```text
//! sync() →
//!   1. Fetch network coin state (singleton lookup by launcher_id)
//!   2. Fetch checkpoint singleton state (epoch, roots)
//!   3. Find all coins with registration_coin puzzle hash
//!   4. For each: verify lineage (parent = network coin spend) — IDX-002
//!   5. Build sparse Merkle tree from verified validators
//!   6. Verify computed root == on-chain validator_merkle_root — IDX-003
//!   7. Return StateMismatch if roots differ → trigger full re-index
//! ```
//!
//! See [spec-consensus-crate.md Lines 1274-1579](../../docs/resources/spec-consensus-crate.md)
//! for the full indexer specification.
//! See [spec-indexer.md](../../docs/resources/spec-indexer.md) for the authoritative
//! indexer algorithm.

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

/// Indexer state — tracks on-chain consensus state between sync() calls.
///
/// Populated by `sync()` which drives the full indexer algorithm.
/// Cached to disk via `IndexerCache` for fast restarts.
///
/// See [spec-indexer.md Lines 1-50](../../docs/resources/spec-indexer.md).
#[derive(Debug)]
pub struct IndexerState {
    cache: IndexerCache,
}

impl IndexerState {
    /// Create a new indexer with the given cache.
    pub fn new(cache: IndexerCache) -> Self {
        Self { cache }
    }

    /// Sync with the Chia chain and return the current validator set.
    ///
    /// Runs the full indexer algorithm: fetch coins, verify lineage,
    /// build Merkle tree, verify root consistency.
    ///
    /// See [spec-indexer.md Lines 50-200](../../docs/resources/spec-indexer.md) —
    /// Sync Algorithm.
    pub async fn sync(&mut self) -> ConsensusResult<ValidatorSet> {
        // TODO: Implement — needs FullNodeClient for RPC queries
        // The algorithm is defined in spec-indexer.md Lines 50-200
        todo!()
    }
}
