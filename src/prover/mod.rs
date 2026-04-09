//! Groth16 prover module.
//!
//! See [spec-consensus-crate.md Lines 929-1026](../docs/resources/spec-consensus-crate.md).

mod circuit;
mod prove;
mod serialize;
mod setup;

pub use circuit::ConsensusCircuit;
pub use prove::generate_proof;
pub use serialize::{ClvmProof, ClvmVerificationKey};
pub use setup::{load_proving_key, load_verification_key};
