//! Checkpoint singleton driver.
//!
//! CHK-001: Inner puzzle loaded from compiled Rue artifact.
//! CHK-003: Groth16 verification TODO — Rue bls_pairing_identity not yet supported.
//!
//! The puzzle is authored in Rue (`puzzles/checkpoint_inner.rue`), compiled
//! to CLVM hex (`puzzles/compiled/checkpoint_inner.hex`), and embedded via
//! `include_str!`. The membership query path is fully functional; the
//! checkpoint path is structurally correct but lacks on-chain Groth16
//! verification until Rue supports `bls_pairing_identity` and `scalar()`.
//!
//! See [spec-checkpoint-singleton.md](../../docs/resources/spec-checkpoint-singleton.md).

use chia_protocol::Bytes32;
use chia_protocol::SpendBundle;

use crate::error::ConsensusResult;
use crate::state::CheckpointSingletonState;

/// Compiled CLVM hex for the checkpoint inner puzzle.
pub const CHECKPOINT_INNER_PUZZLE_HEX: &str =
    include_str!("../../puzzles/compiled/checkpoint_inner.hex");

/// Tree hash of the uncurried checkpoint inner puzzle.
pub const CHECKPOINT_INNER_MOD_HASH_HEX: &str =
    include_str!("../../puzzles/compiled/checkpoint_inner.hash");

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
pub fn fetch_checkpoint_singleton_state(
    _launcher_id: Bytes32,
) -> ConsensusResult<CheckpointSingletonState> {
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
