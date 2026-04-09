//! Raw chain query operations.
//!
//! See [spec-indexer.md](../../docs/resources/spec-indexer.md).

use chia_protocol::Bytes32;

use crate::error::ConsensusResult;

/// Query a coin by ID from the chain.
pub async fn get_coin(_coin_id: Bytes32) -> ConsensusResult<Option<()>> {
    // TODO: Implement using chia RPC
    todo!()
}

/// Query coin records by puzzle hash.
pub async fn get_coin_records_by_puzzle_hash(_puzzle_hash: Bytes32) -> ConsensusResult<Vec<()>> {
    // TODO: Implement using chia RPC
    todo!()
}
