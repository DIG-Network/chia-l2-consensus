//! Groth16 prover module — proof generation, verification key management, and wire format.
//!
//! ## Sub-modules
//!
//! | Module | Purpose | Spec |
//! |--------|---------|------|
//! | `circuit` | `ConsensusCircuit` struct, public inputs, constraints | [spec-groth16-circuit.md](../../docs/resources/spec-groth16-circuit.md) |
//! | `setup` | Trusted setup, VK validation, VK hashing | [spec-trusted-setup.md](../../docs/resources/spec-trusted-setup.md) |
//! | `prove` | `generate_proof()` — produces 192-byte proof | [spec-groth16-circuit.md Lines 400-450](../../docs/resources/spec-groth16-circuit.md) |
//! | `serialize` | Wire format: checkpoint message, announcements, scalars | [spec-wire-format.md](../../docs/resources/spec-wire-format.md) |
//! | `aggregate` | G1 pubkey aggregation (CIR-003) | [spec-wire-format.md Lines 466-544](../../docs/resources/spec-wire-format.md) |
//! | `majority` | Majority threshold (CIR-004): `2k > validator_count` | [spec-groth16-circuit.md Lines 325-357](../../docs/resources/spec-groth16-circuit.md) |
//!
//! See [spec-consensus-crate.md Lines 929-1026](../../docs/resources/spec-consensus-crate.md)
//! for the full prover module specification.

mod aggregate;
pub mod circuit;
pub mod g1_gadget;
mod majority;
mod prove;
mod serialize;
pub mod setup;

// ── CIR-003: G1 pubkey aggregation ─────────────────────────────────
pub use aggregate::{
    add_g1, aggregate_pubkeys, deserialize_g1, g1_identity, negate_g1, serialize_g1,
    verify_aggregate, AggregateError,
};

// ── CIR-001/005/006: Circuit definition ─────────────────────────────
pub use circuit::{public_input_index, ConsensusCircuit, MAX_SIGNERS, PUBLIC_INPUT_COUNT};

// ── CIR-004: Majority threshold ─────────────────────────────────────
pub use majority::{is_at_least_half, is_majority, minimum_signers};

// ── Proof generation ────────────────────────────────────────────────
pub use prove::generate_proof;

// ── Wire format (WIRE-001 through WIRE-006) ─────────────────────────
pub use serialize::{
    ark_g1_to_zcash, ark_g2_to_zcash, bytes_to_scalar, compute_checkpoint_message,
    compute_membership_announcement_message, compute_registration_message, ClvmProof,
    ClvmVerificationKey, G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE, GROTH16_PROOF_SIZE,
    MEMBERSHIP_INPUT_SIZE, MEMBERSHIP_PREFIX, REGISTER_PREFIX, REGISTRATION_INPUT_SIZE,
};

// ── Trusted setup (DEP-001, DEP-004) ────────────────────────────────
pub use setup::{
    compute_vk_hash, deserialize_proving_key, deserialize_verification_key, extract_vk_components,
    extract_vk_components_from_bytes, run_test_setup, validate_vk, validate_vk_bytes,
    verify_vk_hash, vk_to_bytes, VkComponents, VK_BYTE_SIZE,
};
