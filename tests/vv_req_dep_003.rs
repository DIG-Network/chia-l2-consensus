//! REQUIREMENT: DEP-003 — Initial State
//! (`docs/requirements/domains/deployment/NORMATIVE.md#DEP-003`).
//!
//! Spec: `docs/requirements/domains/deployment/specs/DEP-003.md`.
//!
//! Implementation: `src/state.rs`.
//!
//! **Normative statement:** The checkpoint singleton MUST be deployed with
//! initial state: epoch=0, validator_count=0, validator_merkle_root=EMPTY_TREE_ROOT,
//! and state_root as an application-defined genesis value. EMPTY_TREE_ROOT is
//! the root of a depth-32 sparse Merkle tree with all slots empty.
//!
//! **How the tests prove this:**
//! - `epoch_is_zero` checks state.epoch == 0.
//! - `validator_count_is_zero` checks state.validator_count == 0.
//! - `merkle_root_is_empty_tree` checks state.validator_merkle_root == EMPTY_TREE_ROOT.
//! - `state_root_is_genesis` passes a custom genesis root and confirms it appears
//!   in the state unchanged.
//! - `different_genesis_roots` shows two different genesis roots produce different
//!   state_root values but identical epoch/count/merkle_root.
//! - `empty_tree_root_matches_computation` re-derives EMPTY_TREE_ROOT from scratch
//!   via compute_empty_nodes.
//! - `no_coin_before_deploy` confirms the pre-deployment coin placeholder has
//!   amount 0.
//! - `zero_genesis_root_valid` tests the all-zeros genesis root as valid input.
//! - `consistent_with_deployment` verifies EMPTY_LEAF derivation and ties it
//!   to EMPTY_TREE_ROOT in the initial state.
//!
//! **Acceptance-criteria coverage (from spec):**
//! - [x] Epoch is initialized to 0
//! - [x] Validator count is initialized to 0
//! - [x] Merkle root matches EMPTY_TREE_ROOT for configured depth
//! - [x] State root is documented application genesis
//! - [x] All values are correctly set in state struct

use chia_l2_consensus::testing::initial_checkpoint_state;
use chia_l2_consensus::testing::{EMPTY_TREE_ROOT, TREE_DEPTH};

// ── Epoch initialized to 0 ─────────────────────────────────────────

/// Verifies the initial epoch is 0.
/// Strategy: construct the initial state and check the epoch field.
/// Confidence: epochs start from zero as specified.
#[test]
fn vv_req_dep_003_epoch_is_zero() {
    let state = initial_checkpoint_state([0u8; 32]);

    assert_eq!(state.epoch, 0, "DEP-003: Initial epoch must be 0");
}

// ── Validator count initialized to 0 ───────────────────────────────

/// Verifies the initial validator count is 0 (no validators registered yet).
/// Strategy: construct the initial state and check the validator_count field.
/// Confidence: the starting count matches the empty validator set.
#[test]
fn vv_req_dep_003_validator_count_is_zero() {
    let state = initial_checkpoint_state([0u8; 32]);

    assert_eq!(
        state.validator_count, 0,
        "DEP-003: Initial validator count must be 0"
    );
}

// ── Merkle root matches EMPTY_TREE_ROOT ─────────────────────────────

/// Verifies the initial validator_merkle_root equals EMPTY_TREE_ROOT.
/// Strategy: construct the initial state and compare the merkle root to the
/// precomputed empty tree constant.
/// Confidence: the initial state corresponds to a tree with no validators.
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

/// Verifies the state_root is set to the caller-provided genesis root.
/// Strategy: pass a known genesis root and check the state field.
/// Confidence: the application-defined genesis is correctly stored.
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

/// Verifies that different genesis roots produce different state_root values,
/// while epoch, count, and merkle_root remain constant.
/// Strategy: create two initial states with different genesis roots and compare
/// all fields.
/// Confidence: the genesis root is the only application-varying parameter.
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

/// Re-derives EMPTY_TREE_ROOT from compute_empty_nodes and verifies it matches
/// the constant.
/// Strategy: compute the chain at TREE_DEPTH and compare the top element.
/// Confidence: the constant is mathematically correct for the configured depth.
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

/// Verifies the pre-deployment coin placeholder has amount 0.
/// Strategy: construct the initial state and check the coin's amount field.
/// Confidence: the state does not reference a real on-chain coin before deploy.
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

/// Verifies that an all-zeros genesis root is accepted without error.
/// Strategy: construct initial state with [0u8;32] and check all fields.
/// Confidence: zero is a valid genesis root (no special-case rejection).
#[test]
fn vv_req_dep_003_zero_genesis_root_valid() {
    let state = initial_checkpoint_state([0u8; 32]);

    assert_eq!(state.epoch, 0, "DEP-003: Zero genesis root is valid");
    assert_eq!(state.validator_count, 0);
    assert_eq!(state.state_root.as_ref(), &[0u8; 32]);
}

// ── Initial state matches deploy_both_singletons expectations ───────

/// Verifies the initial state is consistent with the deployment pipeline:
/// EMPTY_LEAF = sha256(48 zeros) and the initial merkle root uses the
/// EMPTY_LEAF-derived tree.
/// Strategy: re-derive EMPTY_LEAF and check the initial state's merkle root.
/// Confidence: the state module and the SMT module agree on empty-tree values.
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
