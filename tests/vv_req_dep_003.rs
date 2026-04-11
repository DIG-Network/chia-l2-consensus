//! REQUIREMENT: DEP-003 — Initial State
//! (`docs/requirements/domains/deployment/NORMATIVE.md#DEP-003`).
//!
//! Spec: `docs/requirements/domains/deployment/specs/DEP-003.md`.
//!
//! Implementation: `src/state.rs`.
//!
//! Verifies that the checkpoint singleton is deployed with correct initial
//! state: epoch=0, validator_count=0, validator_merkle_root=EMPTY_TREE_ROOT,
//! and state_root as application-defined genesis.

use chia_l2_consensus::testing::initial_checkpoint_state;
use chia_l2_consensus::testing::{EMPTY_TREE_ROOT, TREE_DEPTH};

// ── Epoch initialized to 0 ─────────────────────────────────────────

#[test]
fn vv_req_dep_003_epoch_is_zero() {
    let state = initial_checkpoint_state([0u8; 32]);

    assert_eq!(state.epoch, 0, "DEP-003: Initial epoch must be 0");
}

// ── Validator count initialized to 0 ───────────────────────────────

#[test]
fn vv_req_dep_003_validator_count_is_zero() {
    let state = initial_checkpoint_state([0u8; 32]);

    assert_eq!(
        state.validator_count, 0,
        "DEP-003: Initial validator count must be 0"
    );
}

// ── Merkle root matches EMPTY_TREE_ROOT ─────────────────────────────

#[test]
fn vv_req_dep_003_merkle_root_is_empty_tree() {
    let state = initial_checkpoint_state([0u8; 32]);

    assert_eq!(
        state.validator_merkle_root.as_ref(),
        &EMPTY_TREE_ROOT,
        "DEP-003: Initial merkle root must be EMPTY_TREE_ROOT"
    );
}

// ── State root is application-defined genesis ───────────────────────

#[test]
fn vv_req_dep_003_state_root_is_genesis() {
    let genesis_root = [0x42u8; 32];
    let state = initial_checkpoint_state(genesis_root);

    assert_eq!(
        state.state_root.as_ref(),
        &genesis_root,
        "DEP-003: Initial state root must be the application-defined genesis"
    );
}

// ── Different genesis roots produce different states ────────────────

#[test]
fn vv_req_dep_003_different_genesis_roots() {
    let state_a = initial_checkpoint_state([0xAA; 32]);
    let state_b = initial_checkpoint_state([0xBB; 32]);

    assert_ne!(
        state_a.state_root, state_b.state_root,
        "DEP-003: Different genesis roots must produce different state roots"
    );
    assert_eq!(
        state_a.epoch, state_b.epoch,
        "DEP-003: Epoch must be 0 regardless of genesis root"
    );
    assert_eq!(
        state_a.validator_count, state_b.validator_count,
        "DEP-003: Validator count must be 0 regardless of genesis root"
    );
    assert_eq!(
        state_a.validator_merkle_root, state_b.validator_merkle_root,
        "DEP-003: Merkle root must be EMPTY_TREE_ROOT regardless of genesis root"
    );
}

// ── EMPTY_TREE_ROOT matches fresh computation ───────────────────────

#[test]
fn vv_req_dep_003_empty_tree_root_matches_computation() {
    use chia_l2_consensus::testing::compute_empty_nodes;

    let empty_nodes = compute_empty_nodes(TREE_DEPTH);
    let computed_root = empty_nodes[TREE_DEPTH as usize];

    assert_eq!(
        computed_root, EMPTY_TREE_ROOT,
        "DEP-003: EMPTY_TREE_ROOT constant must match fresh computation at depth {}",
        TREE_DEPTH
    );
}

// ── Initial state has no coin (pre-deployment) ─────────────────────

#[test]
fn vv_req_dep_003_no_coin_before_deploy() {
    let state = initial_checkpoint_state([0u8; 32]);

    // The coin field should be None or a placeholder before on-chain deployment
    assert_eq!(
        state.coin.amount, 0,
        "DEP-003: Initial state coin should be placeholder (amount 0) before deployment"
    );
}

// ── All zero genesis root is valid ──────────────────────────────────

#[test]
fn vv_req_dep_003_zero_genesis_root_valid() {
    let state = initial_checkpoint_state([0u8; 32]);

    assert_eq!(state.epoch, 0, "DEP-003: Zero genesis root is valid");
    assert_eq!(state.validator_count, 0);
    assert_eq!(state.state_root.as_ref(), &[0u8; 32]);
}

// ── Initial state matches deploy_both_singletons expectations ───────

#[test]
fn vv_req_dep_003_consistent_with_deployment() {
    use chia_l2_consensus::testing::EMPTY_LEAF;
    use sha2::{Digest, Sha256};

    // EMPTY_LEAF must be sha256([0u8; 48])
    let mut hasher = Sha256::new();
    hasher.update([0u8; 48]);
    let computed_empty_leaf: [u8; 32] = hasher.finalize().into();
    assert_eq!(
        EMPTY_LEAF, computed_empty_leaf,
        "DEP-003: EMPTY_LEAF must be sha256(48 zero bytes)"
    );

    // Initial merkle root must be built from EMPTY_LEAF
    let state = initial_checkpoint_state([0u8; 32]);
    assert_eq!(
        state.validator_merkle_root.as_ref(),
        &EMPTY_TREE_ROOT,
        "DEP-003: Initial merkle root uses EMPTY_LEAF-derived tree"
    );
}
