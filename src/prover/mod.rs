//! Groth16 prover module.
//!
//! See [spec-consensus-crate.md Lines 929-1026](../docs/resources/spec-consensus-crate.md).

mod aggregate;
pub mod circuit;
mod majority;
mod prove;
mod serialize;
pub mod setup;

pub use aggregate::{
    add_g1, aggregate_pubkeys, deserialize_g1, g1_identity, negate_g1, serialize_g1,
    verify_aggregate, AggregateError,
};
pub use circuit::{public_input_index, ConsensusCircuit, MAX_SIGNERS, PUBLIC_INPUT_COUNT};
pub use majority::{is_at_least_half, is_majority, minimum_signers};
pub use prove::generate_proof;
pub use serialize::{
    ark_g1_to_zcash, ark_g2_to_zcash, bytes_to_scalar, compute_checkpoint_message,
    compute_membership_announcement_message, compute_registration_message, ClvmProof,
    ClvmVerificationKey, G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE, GROTH16_PROOF_SIZE,
    MEMBERSHIP_INPUT_SIZE, MEMBERSHIP_PREFIX, REGISTER_PREFIX, REGISTRATION_INPUT_SIZE,
};
pub use setup::{
    compute_vk_hash, deserialize_proving_key, deserialize_verification_key, extract_vk_components,
    extract_vk_components_from_bytes, run_test_setup, validate_vk, validate_vk_bytes,
    verify_vk_hash, vk_to_bytes, VkComponents, VK_BYTE_SIZE,
};
