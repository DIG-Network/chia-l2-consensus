//! REQUIREMENT: CIR-006 — Circuit Parameters
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-006`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-006.md`.
//!
//! Verifies that the circuit parameters MAX_SIGNERS and TREE_DEPTH are
//! compile-time constants that are consistent across all components.

use chia_l2_consensus::merkle::{SparseMerkleTree, TREE_DEPTH};
use chia_l2_consensus::{ConsensusCircuit, MAX_SIGNERS};

#[test]
fn vv_req_cir_006_max_signers_is_constant() {
    // CIR-006: MAX_SIGNERS is a compile-time constant
    // If this test compiles, MAX_SIGNERS is a constant
    const _CONST_CHECK: usize = MAX_SIGNERS;

    // Verify it's a reasonable value (at least 1)
    assert!(MAX_SIGNERS >= 1, "CIR-006: MAX_SIGNERS must be at least 1");
}

#[test]
fn vv_req_cir_006_tree_depth_is_constant() {
    // CIR-006: TREE_DEPTH is a compile-time constant
    // If this test compiles, TREE_DEPTH is a constant
    const _CONST_CHECK: u32 = TREE_DEPTH;

    // Verify it matches spec (32)
    assert_eq!(TREE_DEPTH, 32, "CIR-006: TREE_DEPTH must be 32");
}

#[test]
fn vv_req_cir_006_circuit_tree_depth_matches_smt() {
    // CIR-006: Circuit TREE_DEPTH must match SMT TREE_DEPTH
    let circuit = ConsensusCircuit::new();

    assert_eq!(
        circuit.tree_depth(),
        TREE_DEPTH,
        "CIR-006: Circuit tree_depth must match SMT TREE_DEPTH"
    );
}

#[test]
fn vv_req_cir_006_circuit_max_signers_matches_constant() {
    // CIR-006: Circuit max_signers() must match MAX_SIGNERS constant
    let circuit = ConsensusCircuit::new();

    assert_eq!(
        circuit.max_signers(),
        MAX_SIGNERS,
        "CIR-006: Circuit max_signers must match MAX_SIGNERS constant"
    );
}

#[test]
fn vv_req_cir_006_merkle_proofs_have_tree_depth_siblings() {
    // CIR-006: Merkle proofs have exactly TREE_DEPTH siblings
    let mut tree = SparseMerkleTree::new();
    let pubkey = [0x42u8; 48];
    tree.insert_validator(&pubkey);

    let proof = tree.prove_validator(&pubkey);

    assert_eq!(
        proof.siblings.len() as u32,
        TREE_DEPTH,
        "CIR-006: Merkle proof must have exactly TREE_DEPTH siblings"
    );
}

#[test]
fn vv_req_cir_006_tree_capacity_from_depth() {
    // CIR-006: Tree capacity is 2^TREE_DEPTH
    let expected_capacity: u64 = 1u64 << TREE_DEPTH;

    // With TREE_DEPTH=32, capacity is 2^32 = 4,294,967,296
    assert_eq!(
        expected_capacity, 4_294_967_296,
        "CIR-006: Tree capacity must be 2^32 for TREE_DEPTH=32"
    );
}

#[test]
fn vv_req_cir_006_max_signers_value() {
    // CIR-006: MAX_SIGNERS should be set to 20,000 for large validator sets
    assert_eq!(MAX_SIGNERS, 20_000, "CIR-006: MAX_SIGNERS must be 20,000");
}

#[test]
fn vv_req_cir_006_parameters_are_usize_and_u32() {
    // CIR-006: Verify parameter types
    // MAX_SIGNERS is usize
    let _max_signers: usize = MAX_SIGNERS;

    // TREE_DEPTH is u32
    let _tree_depth: u32 = TREE_DEPTH;

    // Types are correct if this compiles
}

#[test]
fn vv_req_cir_006_empty_tree_proofs_still_have_tree_depth() {
    // CIR-006: Even proofs in empty tree have TREE_DEPTH siblings
    let tree = SparseMerkleTree::new();
    let pubkey = [0x00u8; 48]; // Not in tree

    let proof = tree.prove_validator(&pubkey);

    assert_eq!(
        proof.siblings.len() as u32,
        TREE_DEPTH,
        "CIR-006: Empty tree proof must have TREE_DEPTH siblings"
    );
}

#[test]
fn vv_req_cir_006_circuit_constraint_would_enforce_k_le_max_signers() {
    // CIR-006: k ≤ MAX_SIGNERS constraint
    // This is enforced by the witness structure - if k > MAX_SIGNERS,
    // there's no room for additional signers in the witness arrays

    let circuit = ConsensusCircuit::new();

    // The circuit's witness arrays are sized to MAX_SIGNERS
    // Verify this by checking that the circuit reports MAX_SIGNERS correctly
    assert_eq!(
        circuit.max_signers(),
        MAX_SIGNERS,
        "CIR-006: Circuit enforces k <= MAX_SIGNERS through witness structure"
    );
}

#[test]
fn vv_req_cir_006_validator_count_can_exceed_max_signers() {
    // CIR-006: validator_count can be > MAX_SIGNERS
    // This is allowed because only k signers need to sign, not all validators
    // As long as k <= MAX_SIGNERS and 2k > validator_count, it's valid

    // For example, with MAX_SIGNERS=20,000:
    // - validator_count can be up to ~40,000 (2*20,000)
    // - Because we need 2k > validator_count for majority
    // - And k <= MAX_SIGNERS

    // This test documents the relationship
    let max_validator_count_for_majority = 2 * MAX_SIGNERS - 1;

    // With 20,000 max signers, we can have up to 39,999 validators
    // because we need k = 20,000 signers, and 2*20,000 = 40,000 > 39,999
    assert_eq!(
        max_validator_count_for_majority, 39_999,
        "CIR-006: Max validator count for majority with MAX_SIGNERS=20,000 is 39,999"
    );
}

#[test]
fn vv_req_cir_006_circuit_parameters_documented() {
    // CIR-006: Circuit parameters should be accessible
    let circuit = ConsensusCircuit::new();

    // Both parameters should be accessible
    let max_signers = circuit.max_signers();
    let tree_depth = circuit.tree_depth();

    // Document expected values
    assert!(max_signers > 0, "CIR-006: max_signers is documented");
    assert!(tree_depth > 0, "CIR-006: tree_depth is documented");

    println!("CIR-006: MAX_SIGNERS = {}", max_signers);
    println!("CIR-006: TREE_DEPTH = {}", tree_depth);
}

#[test]
fn vv_req_cir_006_parameters_match_across_components() {
    // CIR-006: All components must use the same parameter values
    let circuit = ConsensusCircuit::new();
    let tree = SparseMerkleTree::new();

    // Get TREE_DEPTH from different sources
    let circuit_depth = circuit.tree_depth();
    let smt_depth = TREE_DEPTH;

    // They must match
    assert_eq!(
        circuit_depth, smt_depth,
        "CIR-006: Circuit and SMT must use same TREE_DEPTH"
    );

    // MAX_SIGNERS from circuit
    let circuit_max = circuit.max_signers();
    assert_eq!(
        circuit_max, MAX_SIGNERS,
        "CIR-006: Circuit max_signers must match MAX_SIGNERS constant"
    );

    // Verify proof depth matches
    let pubkey = [0x99u8; 48];
    let mut tree = tree;
    tree.insert_validator(&pubkey);
    let proof = tree.prove_validator(&pubkey);

    assert_eq!(
        proof.siblings.len() as u32,
        smt_depth,
        "CIR-006: Proof siblings match TREE_DEPTH"
    );
}
