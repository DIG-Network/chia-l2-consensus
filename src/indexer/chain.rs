//! Raw chain query operations for the indexer.
//!
//! Provides low-level access to the Chia full node RPC for querying coins,
//! coin records, and puzzle reveals. These functions are called by the
//! indexer's sync loop to discover network coin spends, checkpoint
//! singleton state changes, and registration coin creation.
//!
//! Source: [spec-indexer.md Lines 50-200](../../docs/resources/spec-indexer.md)
//! (What the Indexer Tracks, Sync Algorithm, and process_additions).
//!
//! # What data is queried
//!
//! The indexer tracks four categories of on-chain data (spec-indexer.md Lines 37-56):
//!   1. **Network coin state** -- current unspent coin, lineage proof, last spend height
//!   2. **Checkpoint singleton state** -- epoch, state_root, validator_merkle_root, validator_count
//!   3. **Registration coins** -- unspent coins whose parent is a network coin spend
//!   4. **Checkpoint history** -- ordered list of all past checkpoint states
//!
//! # Cross-references
//!
//! - Sync algorithm: [spec-indexer.md Lines 107-160](../../docs/resources/spec-indexer.md)
//! - Registration coin parsing: [`crate::indexer::validator_set::try_parse_registration_coin`]
//! - Lineage verification: [`crate::indexer::validator_set::LineageChecker`]
//! - Reorg detection: [`crate::indexer::reorg::ReorgState::is_reorg`]

use chia_protocol::Bytes32;

use crate::error::ConsensusResult;

/// Query a coin by ID from the chain.
///
/// Used to look up specific coins during sync -- for example, to retrieve
/// the current network coin or checkpoint singleton by their known coin IDs.
///
/// Source: [spec-indexer.md Lines 149-151](../../docs/resources/spec-indexer.md)
/// (get_additions_and_removals_by_height).
pub async fn get_coin(_coin_id: Bytes32) -> ConsensusResult<Option<()>> {
    // TODO: Implement using chia full node RPC (`get_coin_record_by_name`)
    todo!()
}

/// Query coin records by puzzle hash.
///
/// Used to discover registration coins: the indexer computes the expected
/// puzzle hash for a registration coin (via
/// [`crate::indexer::validator_set::registration_coin_puzzle_hash`]) and
/// queries the chain for coins matching that hash.
///
/// Source: [spec-indexer.md Lines 50-56](../../docs/resources/spec-indexer.md)
/// (Registration coins tracked by the indexer).
pub async fn get_coin_records_by_puzzle_hash(_puzzle_hash: Bytes32) -> ConsensusResult<Vec<()>> {
    // TODO: Implement using chia full node RPC (`get_coin_records_by_puzzle_hash`)
    todo!()
}
