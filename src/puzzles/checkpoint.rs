//! Checkpoint singleton driver.
//!
//! See [spec-checkpoint-singleton.md](../../docs/resources/spec-checkpoint-singleton.md).

use chia_protocol::Bytes32;
use chia_protocol::SpendBundle;

use crate::error::ConsensusResult;
use crate::state::CheckpointSingletonState;

/// Spend the checkpoint singleton to update state.
pub fn spend_checkpoint_singleton(
    _current_state: &CheckpointSingletonState,
    _new_state_root: Bytes32,
    _proof: &[u8],
    _aggregate_signature: &[u8],
) -> ConsensusResult<SpendBundle> {
    // TODO: Implement using chia-wallet-sdk
    todo!()
}

/// Fetch the current checkpoint singleton state.
pub fn fetch_checkpoint_singleton_state(_launcher_id: Bytes32) -> ConsensusResult<CheckpointSingletonState> {
    // TODO: Implement
    todo!()
}

/// Query membership without state update.
pub fn spend_checkpoint_singleton_membership_query(
    _current_state: &CheckpointSingletonState,
    _pubkey: &[u8; 48],
) -> ConsensusResult<SpendBundle> {
    // TODO: Implement using chia-wallet-sdk
    todo!()
}
