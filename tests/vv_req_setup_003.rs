//! REQUIREMENT: SETUP-003 — Project structure
//! (`docs/requirements/domains/setup/NORMATIVE.md#SETUP-003`).
//!
//! Spec: `docs/requirements/domains/setup/specs/SETUP-003.md`.
//!
//! Verifies that the project structure matches the specification.

use std::path::Path;

#[test]
fn vv_req_setup_003_directory_structure_exists() {
    // Verify src/ modules exist
    assert!(
        Path::new("src/lib.rs").exists(),
        "SETUP-003: src/lib.rs must exist"
    );
    assert!(
        Path::new("src/client.rs").exists(),
        "SETUP-003: src/client.rs must exist"
    );
    assert!(
        Path::new("src/config.rs").exists(),
        "SETUP-003: src/config.rs must exist"
    );
    assert!(
        Path::new("src/state.rs").exists(),
        "SETUP-003: src/state.rs must exist"
    );
    assert!(
        Path::new("src/error.rs").exists(),
        "SETUP-003: src/error.rs must exist"
    );

    // Verify submodules
    assert!(
        Path::new("src/puzzles/mod.rs").exists(),
        "SETUP-003: src/puzzles/mod.rs must exist"
    );
    assert!(
        Path::new("src/merkle/mod.rs").exists(),
        "SETUP-003: src/merkle/mod.rs must exist"
    );
    assert!(
        Path::new("src/prover/mod.rs").exists(),
        "SETUP-003: src/prover/mod.rs must exist"
    );
    assert!(
        Path::new("src/indexer/mod.rs").exists(),
        "SETUP-003: src/indexer/mod.rs must exist"
    );

    // Verify puzzles directory (Rue source files)
    assert!(
        Path::new("puzzles/network_coin_inner.rue").exists(),
        "SETUP-003: puzzles/network_coin_inner.rue must exist"
    );
    assert!(
        Path::new("puzzles/registration_coin.rue").exists(),
        "SETUP-003: puzzles/registration_coin.rue must exist"
    );
    assert!(
        Path::new("puzzles/checkpoint_inner.rue").exists(),
        "SETUP-003: puzzles/checkpoint_inner.rue must exist"
    );
    assert!(
        Path::new("puzzles/compiled").exists(),
        "SETUP-003: puzzles/compiled/ directory must exist"
    );
}

#[test]
fn vv_req_setup_003_public_api_compiles() {
    // Verify public types are accessible via re-exports
    use chia_l2_consensus::{
        Bytes32, ConsensusClient, ConsensusError, NetworkConfig, SpendBundle, ValidatorSet,
    };

    // Types should be usable (compile-time check via function signatures)
    fn _assert_send<T: Send>() {}
    fn _assert_sync<T: Sync>() {}
    fn _check_bytes32(_b: Bytes32) {}
    fn _check_consensus_client(_c: ConsensusClient) {}
    fn _check_network_config(_n: NetworkConfig) {}
    fn _check_validator_set(_v: ValidatorSet) {}
    fn _check_spend_bundle(_s: SpendBundle) {}

    // ConsensusError should be Send + Sync for async usage
    _assert_send::<ConsensusError>();
    _assert_sync::<ConsensusError>();
}
