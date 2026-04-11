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
//! Internal types for integration testing are in the [`testing`] module.
//!
//! See [spec-consensus-crate.md](../docs/resources/spec-consensus-crate.md).

#![allow(unused)] // Temporary during initial development

// ============================================================================
// Public API — types L2 consumers import directly
// ============================================================================

mod client;
mod config;
mod error;
mod state;

pub use client::ConsensusClient;
pub use config::{DeploymentArtifacts, NetworkConfig, VkJson};
pub use error::{ConsensusError, ConsensusResult};
pub use state::{CheckpointSingletonState, NetworkCoinState, ValidatorSet};

// Re-exported Chia protocol types
pub use chia_protocol::Bytes32;
pub use chia_protocol::SpendBundle;

// ============================================================================
// Internal modules — pub(crate) for use within the crate only
// ============================================================================

pub(crate) mod indexer;
pub(crate) mod merkle;
pub(crate) mod prover;
pub(crate) mod puzzles;
pub(crate) mod validator;

// ============================================================================
// Testing module — re-exports internal types for VV integration tests
//
// L2 consumers should NOT depend on this module. It is not stable API.
// VV tests import via: `use chia_l2_consensus::testing::{...};`
// ============================================================================

pub mod testing;
