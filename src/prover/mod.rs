//! Groth16 prover module.
//!
//! See [spec-consensus-crate.md Lines 929-1026](../docs/resources/spec-consensus-crate.md).

mod circuit;
mod prove;
mod serialize;
mod setup;

pub use circuit::ConsensusCircuit;
pub use prove::generate_proof;
pub use serialize::{
    compute_checkpoint_message, ClvmProof, ClvmVerificationKey, G1_COMPRESSED_SIZE,
    G2_COMPRESSED_SIZE,
};
pub use setup::{load_proving_key, load_verification_key};
