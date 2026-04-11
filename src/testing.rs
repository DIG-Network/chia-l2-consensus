//! Re-exports for integration testing.
//!
//! **WARNING: This module is NOT a stable public API.**
//!
//! It exposes internal types and functions that Verification & Validation (VV)
//! tests need, but which may change without notice between releases. L2
//! consumers should use only `ConsensusClient` and the types it returns.
//! Test code imports from `chia_l2_consensus::testing::*`.
//!
//! The re-exports below are organized by the source module they come from.
//! Each section header references the spec document or requirement ID that
//! governs the exported symbols.

// ============================================================================
// Prover: Wire format (src/prover/serialize.rs)
// Spec: spec-wire-format.md — WIRE-001 through WIRE-006
// ============================================================================
pub use crate::prover::{
    ark_g1_to_zcash, ark_g2_to_zcash, bytes_to_scalar, compute_checkpoint_message,
    compute_membership_announcement_message, compute_registration_message, ClvmProof,
    ClvmVerificationKey, G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE, GROTH16_PROOF_SIZE,
    MEMBERSHIP_INPUT_SIZE, MEMBERSHIP_PREFIX, REGISTER_PREFIX, REGISTRATION_INPUT_SIZE,
};

// ============================================================================
// Prover: Circuit definition (src/prover/circuit.rs)
// Spec: spec-groth16-circuit.md — CIR-001, CIR-005, CIR-006
// ============================================================================
pub use crate::prover::{public_input_index, ConsensusCircuit, MAX_SIGNERS, PUBLIC_INPUT_COUNT};

// ============================================================================
// Prover: G1 aggregation (src/prover/aggregate.rs)
// Spec: spec-groth16-circuit.md Lines 277-323 — CIR-003
// ============================================================================
pub use crate::prover::{
    add_g1, aggregate_pubkeys, deserialize_g1, g1_identity, negate_g1, serialize_g1,
    verify_aggregate, AggregateError,
};

// ============================================================================
// Prover: Majority threshold (src/prover/majority.rs)
// Spec: spec-groth16-circuit.md Lines 327-357 — CIR-004
// ============================================================================
pub use crate::prover::{is_at_least_half, is_majority, minimum_signers};

// ============================================================================
// Prover: Trusted setup + proof generation (src/prover/setup.rs, src/prover/prove.rs)
// Spec: spec-trusted-setup.md — DEP-001, DEP-004
// Spec: spec-groth16-circuit.md Lines 560-581 — Proof Generation
// ============================================================================
pub use crate::prover::{
    compute_vk_hash, deserialize_proving_key, deserialize_verification_key, extract_vk_components,
    extract_vk_components_from_bytes, generate_proof, run_test_setup, validate_vk,
    validate_vk_bytes, verify_vk_hash, vk_to_bytes, VkComponents, VK_BYTE_SIZE,
};

// ============================================================================
// Puzzles: Deployment (src/puzzles/)
// Spec: spec-deployment-runbook.md — DEP-002
// ============================================================================
pub use crate::puzzles::{deploy_both_singletons, derive_launcher_id};

// ============================================================================
// Validator operations (src/validator/)
// Spec: spec-validator-onboarding.md — VAL-001 through VAL-005
// ============================================================================
pub use crate::validator::{
    aggregate_checkpoint_signatures, compute_checkpoint_signing_message, compute_exit_announcement,
    compute_registration_signing_message, generate_validator_keypair, is_validator_excluded,
    prepare_collateral_recovery, prepare_forced_exit, pubkey_from_secret, sign_checkpoint,
    sign_message, sign_registration, verify_checkpoint_signature, verify_registration_signature,
    verify_signature, CollateralRecoveryParams, ForcedExitParams, ForcedExitReason,
    ValidatorKeypair,
};

// ============================================================================
// Merkle: SHA-256 sparse Merkle tree (src/merkle/sparse.rs, src/merkle/proof.rs)
// Spec: spec-sparse-merkle-tree.md — SMT-001 through SMT-006
// ============================================================================
pub use crate::merkle::{
    active_leaf, compute_empty_nodes, compute_slot, MerkleProof, SparseMerkleTree, EMPTY_LEAF,
    EMPTY_TREE_ROOT, TREE_DEPTH,
};

// ============================================================================
// Merkle: Poseidon tree for in-circuit use (src/merkle/poseidon.rs)
// Spec: DESIGN_DECISIONS.md Decision 1 — CIR-002
// ============================================================================
pub mod poseidon {
    //! Re-export of `crate::merkle::poseidon` for test access.
    pub use crate::merkle::poseidon::*;
}

// ============================================================================
// Indexer (src/indexer/)
// Spec: spec-indexer.md — IDX-001 through IDX-005
// ============================================================================
pub use crate::indexer::{
    registration_coin_puzzle_hash, try_parse_registration_coin, verify_merkle_consistency,
    CheckpointRecord, IndexerCache, IndexerState, LineageChecker, RegistrationCoinRecord,
    ReorgState,
};

// ============================================================================
// State helpers (src/state.rs)
// ============================================================================
pub use crate::state::initial_checkpoint_state;
