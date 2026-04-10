//! chia-l2-consensus — Groth16-based L2 consensus for Chia
//!
//! This crate packages:
//! - L1 puzzle implementations (network coin, registration coin, checkpoint singleton)
//! - Puzzle driver code
//! - Groth16 circuit and prover
//! - Sparse Merkle tree
//! - Chain indexer
//!
//! The L2 system uses only [`ConsensusClient`] and the types it returns.
//!
//! See [spec-consensus-crate.md](../docs/resources/spec-consensus-crate.md).

#![allow(unused)] // Temporary during initial development

// Public modules
mod client;
mod config;
mod error;
mod state;

// Internal modules
pub(crate) mod indexer;
pub mod merkle;
pub(crate) mod prover;
pub(crate) mod puzzles;

// Public re-exports (the only types exposed to L2 system)
pub use chia_protocol::Bytes32;
pub use chia_protocol::SpendBundle;

pub use client::ConsensusClient;
pub use config::NetworkConfig;
pub use error::{ConsensusError, ConsensusResult};
pub use state::{CheckpointSingletonState, ValidatorSet};

// Wire format functions and constants (spec-wire-format.md)
pub use prover::{
    bytes_to_scalar, compute_checkpoint_message, compute_membership_announcement_message,
    compute_registration_message, ClvmProof, G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE,
    GROTH16_PROOF_SIZE,
};

// Groth16 circuit (spec-groth16-circuit.md)
pub use prover::{ConsensusCircuit, MAX_SIGNERS};

// G1 pubkey aggregation (CIR-003)
pub use prover::{
    add_g1, aggregate_pubkeys, deserialize_g1, g1_identity, negate_g1, serialize_g1,
    verify_aggregate, AggregateError,
};

// Majority threshold (CIR-004)
pub use prover::{is_at_least_half, is_majority, minimum_signers};

// Re-export PublicKey type (48-byte BLS public key)
// TODO: Use proper BLS pubkey type from chia-bls when needed
