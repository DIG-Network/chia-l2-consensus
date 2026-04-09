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
    compute_checkpoint_message, ClvmProof, G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE,
    GROTH16_PROOF_SIZE,
};

// Re-export PublicKey type (48-byte BLS public key)
// TODO: Use proper BLS pubkey type from chia-bls when needed
