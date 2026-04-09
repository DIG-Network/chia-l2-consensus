//! chia-l2-consensus — Groth16-based L2 consensus for Chia
//!
//! This crate packages:
//! - L1 puzzle implementations (network coin, registration coin, checkpoint singleton)
//! - Puzzle driver code
//! - Groth16 circuit and prover
//! - Sparse Merkle tree
//! - Chain indexer
//!
//! The L2 system uses only `ConsensusClient` and the types it returns.

#![allow(unused)] // Temporary during initial setup
