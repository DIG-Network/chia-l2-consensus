//! On-chain state types for chia-l2-consensus.
//!
//! These types represent the current on-chain state of the L2 network,
//! populated by `sync()` via the indexer. The L2 system reads these
//! through `ConsensusClient` accessor methods.
//!
//! ## State Hierarchy
//!
//! ```text
//! NetworkCoinState     — current unspent network coin singleton
//! CheckpointSingletonState — current checkpoint with epoch, roots, count
//! ValidatorSet         — all active validators with registration coins
//! ```
//!
//! See [spec-consensus-crate.md Lines 319-414](../docs/resources/spec-consensus-crate.md)
//! for the full state type specifications.

use chia_protocol::{Bytes32, Coin};

use crate::merkle::EMPTY_TREE_ROOT;

/// Current on-chain state of the network coin singleton.
///
/// The network coin is the registration authority for the L2 network.
/// There is exactly one at any time (singleton pattern).
///
/// See [spec-consensus-crate.md Lines 325-335](../docs/resources/spec-consensus-crate.md).
/// See [spec-network-coin.md Lines 1-50](../docs/resources/spec-network-coin.md).
#[derive(Debug, Clone)]
pub struct NetworkCoinState {
    /// The current unspent network coin.
    /// Its coin_id is the parent of the next registration coin.
    pub coin: Coin,
    // TODO (API-003): Add inner_puzzle: NodePtr and lineage_proof: Proof
    // when implementing ConsensusClient puzzle driver methods.
    // See spec-consensus-crate.md Lines 327-334 for required fields.
}

/// Current on-chain state of the checkpoint singleton.
///
/// This is the canonical L2 state on the Chia L1. It changes with every
/// checkpoint spend (epoch increments, roots update). The membership query
/// path reads this state but does not change it.
///
/// See [spec-consensus-crate.md Lines 337-358](../docs/resources/spec-consensus-crate.md).
/// See [spec-checkpoint-singleton.md Lines 50-100](../docs/resources/spec-checkpoint-singleton.md) —
/// Singleton State.
#[derive(Debug, Clone)]
pub struct CheckpointSingletonState {
    /// The current unspent checkpoint singleton coin.
    /// Its coin_id is used in announcement assertions (REG-004) and
    /// AGG_SIG_ME message construction (VAL-003).
    pub coin: Coin,

    /// Current epoch number (monotonically increasing).
    /// Incremented by exactly 1 on every checkpoint spend (CHK-004).
    /// Used for replay protection in membership announcements (SEC-006).
    pub epoch: u64,

    /// Number of active validators.
    /// Used for majority threshold: 2k > validator_count (CIR-004).
    pub validator_count: u64,

    /// Sparse Merkle root of the current active validator set.
    /// See [spec-sparse-merkle-tree.md](../docs/resources/spec-sparse-merkle-tree.md).
    pub validator_merkle_root: Bytes32,

    /// Current L2 state root (application-defined).
    /// Committed to by the majority BLS signature in the checkpoint message.
    pub state_root: Bytes32,
}

/// Create the initial checkpoint singleton state for deployment (DEP-003).
///
/// Initial values:
/// - `epoch`: 0 (first checkpoint will be epoch 1)
/// - `validator_count`: 0 (no validators registered yet)
/// - `validator_merkle_root`: EMPTY_TREE_ROOT (all-empty sparse Merkle tree)
/// - `state_root`: application-defined genesis root
/// - `coin`: placeholder (no on-chain coin until deployment completes)
///
/// See [spec-deployment-runbook.md Lines 100-130](../docs/resources/spec-deployment-runbook.md) — Step 3.
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
///
/// Identified by their BLS pubkey, which determines their slot in the
/// sparse Merkle tree via `compute_slot(pubkey)`.
///
/// See [spec-consensus-crate.md Lines 389-396](../docs/resources/spec-consensus-crate.md).
#[derive(Debug, Clone)]
pub struct Validator {
    /// BLS12-381 G1 public key (48 bytes compressed, ZCash format).
    /// Determines Merkle tree slot via sha256(pubkey) mod 2^TREE_DEPTH.
    ///
    /// See [spec-wire-format.md Lines 46-80](../docs/resources/spec-wire-format.md) — G1 Points.
    pub pubkey: Vec<u8>,

    /// Slot in the sparse Merkle tree.
    /// Computed as: first_8_bytes_be(sha256(pubkey)) mod 2^TREE_DEPTH.
    ///
    /// See [spec-sparse-merkle-tree.md Lines 63-104](../docs/resources/spec-sparse-merkle-tree.md) —
    /// Slot Assignment.
    pub slot: u32,

    /// The unspent registration coin holding this validator's collateral.
    /// Its existence proves the validator went through the network coin
    /// registration process (lineage verification, REG-002).
    pub registration_coin_id: Bytes32,
}

/// The complete validator set derived from on-chain state.
///
/// Populated by `sync()` after the indexer verifies lineage for all
/// registration coins and rebuilds the sparse Merkle tree.
///
/// See [spec-consensus-crate.md Lines 383-408](../docs/resources/spec-consensus-crate.md).
/// See [spec-indexer.md Lines 200-260](../docs/resources/spec-indexer.md) — Validator Set Construction.
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    /// All active validators whose registration coins passed lineage check.
    /// Sorted by pubkey bytes for deterministic Merkle slot ordering.
    pub validators: Vec<Validator>,

    /// Current epoch from the checkpoint singleton.
    pub epoch: u64,

    /// Merkle root computed from the validators list.
    /// Verified against on-chain `validator_merkle_root` (IDX-003).
    pub merkle_root: Bytes32,
}
