//! Groth16 prover module.
//!
//! See [spec-consensus-crate.md Lines 929-1026](../docs/resources/spec-consensus-crate.md).

mod aggregate;
mod circuit;
mod prove;
mod serialize;
mod setup;

pub use aggregate::{
    add_g1, aggregate_pubkeys, deserialize_g1, g1_identity, negate_g1, serialize_g1,
    verify_aggregate, AggregateError,
};
pub use circuit::{ConsensusCircuit, MAX_SIGNERS};
pub use prove::generate_proof;
pub use serialize::{
    bytes_to_scalar, compute_checkpoint_message, compute_membership_announcement_message,
    compute_registration_message, ClvmProof, ClvmVerificationKey, G1_COMPRESSED_SIZE,
    G2_COMPRESSED_SIZE, GROTH16_PROOF_SIZE, MEMBERSHIP_INPUT_SIZE, MEMBERSHIP_PREFIX,
    REGISTER_PREFIX, REGISTRATION_INPUT_SIZE,
};
pub use setup::{load_proving_key, load_verification_key};
