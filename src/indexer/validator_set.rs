//! Validator set building with lineage verification.
//!
//! See [spec-indexer.md](../../docs/resources/spec-indexer.md).

use crate::error::ConsensusResult;
use crate::indexer::IndexerCache;
use crate::state::ValidatorSet;

/// Build the current validator set from cached state.
pub fn build_validator_set(_cache: &IndexerCache) -> ConsensusResult<ValidatorSet> {
    // TODO: Implement with lineage verification
    todo!()
}

/// Verify lineage of a registration coin.
pub fn verify_lineage(_registration_coin_id: chia_protocol::Bytes32) -> ConsensusResult<bool> {
    // TODO: Implement
    todo!()
}
