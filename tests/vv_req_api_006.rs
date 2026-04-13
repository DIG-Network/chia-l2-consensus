//! REQUIREMENT: API-006 — Module Visibility
//! (`docs/requirements/domains/crate_api/NORMATIVE.md#API-006`).
//!
//! Spec: `docs/requirements/domains/crate_api/specs/API-006.md`.
//!
//! ## Normative statement
//! Internal modules MUST be `pub(crate)`. The `testing` module MUST
//! re-export all internal types VV tests need. Test imports use
//! `chia_l2_consensus::testing::` prefix.
//!
//! ## How the tests prove the requirement
//! Tests verify the testing module re-exports by importing types from each
//! internal module category (prover, merkle, indexer, validator, puzzles)
//! and using them. Also verifies lib.rs source declares pub(crate) modules.

use chia_l2_consensus::testing;

// ── Wire format re-exports (prover::serialize) ────────────────────

#[test]
fn vv_req_api_006_testing_wire_format_exports() {
    // All wire format types/functions must be accessible via testing::
    let _ = testing::G1_COMPRESSED_SIZE;
    let _ = testing::G2_COMPRESSED_SIZE;
    let _ = testing::GROTH16_PROOF_SIZE;
    let _ = testing::MEMBERSHIP_PREFIX;
    let _ = testing::REGISTER_PREFIX;
    let _ = testing::REGISTRATION_INPUT_SIZE;
    let _ = testing::MEMBERSHIP_INPUT_SIZE;

    // Functions exist (verified by calling with test values)
    let _ = testing::compute_checkpoint_message([0u8; 32], [0u8; 32], 0, 0, [0u8; 32]);
    let _ = testing::compute_membership_announcement_message(0, &[0u8; 48], false);
    let _ = testing::compute_registration_message(&[0u8; 48]);
}

// ── Circuit re-exports (prover::circuit) ──────────────────────────

#[test]
fn vv_req_api_006_testing_circuit_exports() {
    let _ = testing::MAX_SIGNERS;
    let _ = testing::PUBLIC_INPUT_COUNT;
    let _: Option<testing::ConsensusCircuit> = None;

    // public_input_index submodule
    let _ = testing::public_input_index::VALIDATOR_MERKLE_ROOT;
}

// ── Aggregation re-exports (prover::aggregate) ────────────────────

#[test]
fn vv_req_api_006_testing_aggregation_exports() {
    // Type exists
    let _: Option<testing::AggregateError> = None;

    // Functions exist
    let _ = testing::g1_identity;
    let _ = testing::add_g1;
    let _ = testing::negate_g1;
    let _ = testing::serialize_g1;
    let _ = testing::deserialize_g1;
    let _ = testing::aggregate_pubkeys;
    let _ = testing::verify_aggregate;
}

// ── Majority re-exports (prover::majority) ────────────────────────

#[test]
fn vv_req_api_006_testing_majority_exports() {
    // Verify functions are callable
    assert!(testing::is_majority(3, 5));
    assert!(!testing::is_majority(2, 5));
    assert_eq!(testing::minimum_signers(10), 6);
    assert!(!testing::is_at_least_half(4, 10));
}

// ── Setup re-exports (prover::setup) ──────────────────────────────

#[test]
fn vv_req_api_006_testing_setup_exports() {
    let _ = testing::VK_BYTE_SIZE;
    let _: Option<testing::VkComponents> = None;

    // Functions exist (type-level reference)
    let _ = testing::run_test_setup;
    let _ = testing::validate_vk;
    let _ = testing::compute_vk_hash;
    let _ = testing::vk_to_bytes;
}

// ── Merkle re-exports ─────────────────────────────────────────────

#[test]
fn vv_req_api_006_testing_merkle_exports() {
    let _ = testing::TREE_DEPTH;
    let _ = testing::EMPTY_LEAF;
    let _ = testing::EMPTY_TREE_ROOT;

    let _: Option<testing::SparseMerkleTree> = None;
    let _: Option<testing::MerkleProof> = None;

    let _ = testing::compute_slot;
    let _ = testing::active_leaf;
    let _ = testing::compute_empty_nodes;
}

// ── Indexer re-exports ────────────────────────────────────────────

#[test]
fn vv_req_api_006_testing_indexer_exports() {
    let _: Option<testing::LineageChecker> = None;
    let _: Option<testing::IndexerState> = None;
    let _: Option<testing::IndexerCache> = None;
    let _: Option<testing::CheckpointRecord> = None;
    let _: Option<testing::ReorgState> = None;

    let _ = testing::registration_coin_puzzle_hash;
    let _ = testing::try_parse_registration_coin;
    let _ = testing::verify_merkle_consistency;
}

// ── Validator re-exports ──────────────────────────────────────────

#[test]
fn vv_req_api_006_testing_validator_exports() {
    let _: Option<testing::ValidatorKeypair> = None;
    let _: Option<testing::CollateralRecoveryParams> = None;
    let _: Option<testing::ForcedExitParams> = None;
    let _: Option<testing::ForcedExitReason> = None;

    let _ = testing::generate_validator_keypair;
    let _ = testing::sign_checkpoint;
    let _ = testing::sign_registration;
    let _ = testing::verify_checkpoint_signature;
    let _ = testing::is_validator_excluded;
    let _ = testing::prepare_collateral_recovery;
    let _ = testing::prepare_forced_exit;
}

// ── Deployment re-exports ─────────────────────────────────────────

#[test]
fn vv_req_api_006_testing_deployment_exports() {
    let _ = testing::deploy_both_singletons;
    let _ = testing::derive_launcher_id;
}

// ── Source-level visibility check ─────────────────────────────────

#[test]
fn vv_req_api_006_pub_crate_modules() {
    let lib_src =
        std::fs::read_to_string("src/lib.rs").expect("API-006: src/lib.rs must be readable");

    // All internal modules MUST be pub(crate)
    for module in &["indexer", "merkle", "prover", "puzzles", "validator"] {
        let pattern = format!("pub(crate) mod {}", module);
        assert!(
            lib_src.contains(&pattern),
            "API-006: '{}' must be pub(crate), found different visibility",
            module
        );
    }
}

#[test]
fn vv_req_api_006_testing_module_documented() {
    let testing_src = std::fs::read_to_string("src/testing.rs")
        .expect("API-006: src/testing.rs must be readable");

    assert!(
        testing_src.contains("NOT a stable public API")
            || testing_src.contains("not stable API")
            || testing_src.contains("NOT stable"),
        "API-006: testing module must document that it is NOT a stable public API"
    );
}
