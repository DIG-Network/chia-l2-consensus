//! Re-exports for integration testing.
//!
//! This module exposes internal types and functions that VV tests need
//! but which are NOT part of the stable public API for L2 consumers.
//!
//! L2 consumers should use only `ConsensusClient` and the types it returns.
//! Test code imports from `chia_l2_consensus::testing::*`.

// ── Wire format (spec-wire-format.md) ───────────────────────────────
pub use crate::prover::{
    ark_g1_to_zcash, ark_g2_to_zcash, bytes_to_scalar, compute_checkpoint_message,
    compute_membership_announcement_message, compute_registration_message, ClvmProof,
    ClvmVerificationKey, G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE, GROTH16_PROOF_SIZE,
    MEMBERSHIP_INPUT_SIZE, MEMBERSHIP_PREFIX, REGISTER_PREFIX, REGISTRATION_INPUT_SIZE,
};

// ── Circuit (spec-groth16-circuit.md) ───────────────────────────────
pub use crate::prover::{public_input_index, ConsensusCircuit, MAX_SIGNERS, PUBLIC_INPUT_COUNT};

// ── G1 aggregation (CIR-003) ───────────────────────────────────────
pub use crate::prover::{
    add_g1, aggregate_pubkeys, deserialize_g1, g1_identity, negate_g1, serialize_g1,
    verify_aggregate, AggregateError,
};

// ── Majority threshold (CIR-004) ───────────────────────────────────
pub use crate::prover::{is_at_least_half, is_majority, minimum_signers};

// ── Trusted setup + proof generation ────────────────────────────────
pub use crate::prover::{
    compute_vk_hash, deserialize_proving_key, deserialize_verification_key, extract_vk_components,
    extract_vk_components_from_bytes, generate_proof, run_test_setup, validate_vk,
    validate_vk_bytes, verify_vk_hash, vk_to_bytes, VkComponents, VK_BYTE_SIZE,
};

// ── Deployment (DEP-002) ────────────────────────────────────────────
pub use crate::puzzles::{deploy_both_singletons, derive_launcher_id};

// ── Validator operations (VAL-001 through VAL-005) ──────────────────
pub use crate::validator::{
    aggregate_checkpoint_signatures, compute_checkpoint_signing_message, compute_exit_announcement,
    compute_registration_signing_message, generate_validator_keypair, is_validator_excluded,
    prepare_collateral_recovery, prepare_forced_exit, pubkey_from_secret, sign_checkpoint,
    sign_message, sign_registration, verify_checkpoint_signature, verify_registration_signature,
    verify_signature, CollateralRecoveryParams, ForcedExitParams, ForcedExitReason,
    ValidatorKeypair,
};

// ── Merkle tree ─────────────────────────────────────────────────────
pub use crate::merkle::{
    active_leaf, compute_empty_nodes, compute_slot, MerkleProof, SparseMerkleTree, EMPTY_LEAF,
    EMPTY_TREE_ROOT, TREE_DEPTH,
};

// ── Merkle Poseidon ─────────────────────────────────────────────────
pub mod poseidon {
    pub use crate::merkle::poseidon::*;
}

// ── Indexer ─────────────────────────────────────────────────────────
pub use crate::indexer::{
    registration_coin_puzzle_hash, try_parse_registration_coin, verify_merkle_consistency,
    CheckpointRecord, IndexerCache, IndexerState, LineageChecker, RegistrationCoinRecord,
    ReorgState,
};

// ── State helpers ───────────────────────────────────────────────────
pub use crate::state::initial_checkpoint_state;
