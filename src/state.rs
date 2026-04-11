//! On-chain state types for chia-l2-consensus.
//!
//! See [spec-consensus-crate.md Lines 319-414](../docs/resources/spec-consensus-crate.md).

use chia_protocol::{Bytes32, Coin};

use crate::merkle::EMPTY_TREE_ROOT;

/// Current on-chain state of the network coin singleton.
#[derive(Debug, Clone)]
pub struct NetworkCoinState {
    /// The current unspent network coin.
    pub coin: Coin,
    // TODO: Add inner_puzzle and lineage_proof when implementing puzzles
}

/// Current on-chain state of the checkpoint singleton.
#[derive(Debug, Clone)]
pub struct CheckpointSingletonState {
    /// The current unspent checkpoint singleton coin.
    pub coin: Coin,

    /// Current epoch number.
    pub epoch: u64,

    /// Number of registered validators.
    pub validator_count: u64,

    /// Merkle root of the validator set.
    pub validator_merkle_root: Bytes32,

    /// Current L2 state root.
    pub state_root: Bytes32,
}

/// Create the initial checkpoint singleton state for deployment.
///
/// The initial state has:
/// - `epoch`: 0
/// - `validator_count`: 0
/// - `validator_merkle_root`: EMPTY_TREE_ROOT (all-empty sparse Merkle tree)
/// - `state_root`: application-defined genesis root
/// - `coin`: placeholder (no on-chain coin yet)
///
/// This state is curried into the checkpoint singleton inner puzzle at
/// deployment time.
///
/// See spec-deployment-runbook.md — Step 3 and DEP-003.
pub fn initial_checkpoint_state(genesis_state_root: [u8; 32]) -> CheckpointSingletonState {
    CheckpointSingletonState {
        coin: Coin::new(Bytes32::default(), Bytes32::default(), 0),
        epoch: 0,
        validator_count: 0,
        validator_merkle_root: EMPTY_TREE_ROOT.into(),
        state_root: genesis_state_root.into(),
    }
}

/// A validator in the active set.
#[derive(Debug, Clone)]
pub struct Validator {
    /// BLS public key (48 bytes).
    pub pubkey: Vec<u8>,

    /// Slot in the sparse Merkle tree.
    pub slot: u32,

    /// Registration coin ID.
    pub registration_coin_id: Bytes32,
}

/// The complete validator set with Merkle tree.
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    /// All active validators.
    pub validators: Vec<Validator>,

    /// Current epoch.
    pub epoch: u64,

    /// Merkle root of the validator set.
    pub merkle_root: Bytes32,
}
