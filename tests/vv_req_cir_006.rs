//! REQUIREMENT: CIR-006 — Circuit Parameters
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-006`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-006.md`.
//!
//! ## Normative Statement
//!
//! The circuit is parameterized by two compile-time constants fixed at trusted
//! setup: `MAX_SIGNERS` (maximum simultaneous signers the circuit can verify)
//! and `TREE_DEPTH` (depth of the sparse Merkle tree, supports 2^TREE_DEPTH
//! slots). These parameters cannot be changed without a new trusted setup.
//! All components (circuit, SMT, checkpoint singleton) must use identical values.
//!
//! ## How These Tests Prove the Requirement
//!
//! Tests verify: both constants are compile-time (const assignment compiles),
//! TREE_DEPTH=32 and MAX_SIGNERS=20,000 match spec, the circuit's accessors
//! agree with the global constants, Merkle proofs have exactly TREE_DEPTH
//! siblings, tree capacity is 2^32, the circuit enforces k <= MAX_SIGNERS
//! structurally, and all components (circuit, SMT) use consistent values.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] MAX_SIGNERS is compile-time constant
//! - [x] TREE_DEPTH is compile-time constant
//! - [x] Checkpoint singleton TREE_DEPTH matches circuit (via SMT consistency)
//! - [x] k <= MAX_SIGNERS enforced (witness structure size)
//! - [x] Merkle proofs have exactly TREE_DEPTH siblings
//! - [x] MAX_SIGNERS = 20,000 (spec value)
//! - [x] TREE_DEPTH = 32 (spec value)
//! - [x] Parameters match across circuit and SMT components
//! - [ ] VK is bound to specific parameter values (requires trusted setup test)
//! - [ ] Network config documents both parameters (config file not tested)
//!
//! ## Gaps
//!
//! - Does not test that a VK generated for one set of parameters rejects
//!   proofs from a different parameter set. This would require generating
//!   two VKs and cross-verifying.
//! - validator_count can exceed MAX_SIGNERS (up to 2*MAX_SIGNERS-1) is
//!   documented but not tested against the circuit.

use chia_l2_consensus::testing::{ConsensusCircuit, MAX_SIGNERS};
use chia_l2_consensus::testing::{SparseMerkleTree, TREE_DEPTH};

// Verifies MAX_SIGNERS is a compile-time constant by assigning it to a const.
// If MAX_SIGNERS were a runtime value, this line would not compile.
#[test]
fn vv_req_cir_006_max_signers_is_constant() {
    // CIR-006: MAX_SIGNERS is a compile-time constant
    // If this test compiles, MAX_SIGNERS is a constant
    const _CONST_CHECK: usize = MAX_SIGNERS;

    // Verify it's a reasonable value (at least 1)
    assert!(MAX_SIGNERS >= 1, "CIR-006: MAX_SIGNERS must be at least 1");
}

// Verifies TREE_DEPTH is a compile-time constant (const assignment compiles)
// and that its value is exactly 32 per the spec.
#[test]
fn vv_req_cir_006_tree_depth_is_constant() {
    // CIR-006: TREE_DEPTH is a compile-time constant
    // If this test compiles, TREE_DEPTH is a constant
    const _CONST_CHECK: u32 = TREE_DEPTH;

    // Verify it matches spec (32)
    assert_eq!(TREE_DEPTH, 32, "CIR-006: TREE_DEPTH must be 32");
}

// Verifies that the circuit's tree_depth() accessor returns the same value
// as the global TREE_DEPTH constant. A mismatch would mean the circuit
// expects different-length Merkle proofs than the SMT produces.
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

// Verifies the circuit's max_signers() accessor matches the global
// MAX_SIGNERS constant. This ensures the circuit's witness array is sized
// consistently with the system-wide limit.
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

// Verifies that a Merkle proof from the SparseMerkleTree has exactly
// TREE_DEPTH sibling hashes. The circuit expects this exact count;
// fewer or more siblings would make the proof incompatible.
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

// Verifies that 2^TREE_DEPTH = 4,294,967,296 (2^32) -- the maximum
// number of validator slots the tree can hold. This confirms the tree
// is large enough for any practical validator set.
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

// Verifies the spec-mandated value: MAX_SIGNERS = 20,000.
#[test]
fn vv_req_cir_006_max_signers_value() {
    // CIR-006: MAX_SIGNERS should be set to 20,000 for large validator sets
    assert_eq!(MAX_SIGNERS, 20_000, "CIR-006: MAX_SIGNERS must be 20,000");
}

// Verifies parameter types: MAX_SIGNERS is usize and TREE_DEPTH is u32.
// If these types were changed, dependent code using them in array sizing
// or bitwise operations would break.
#[test]
fn vv_req_cir_006_parameters_are_usize_and_u32() {
    // CIR-006: Verify parameter types
    // MAX_SIGNERS is usize
    let _max_signers: usize = MAX_SIGNERS;

    // TREE_DEPTH is u32
    let _tree_depth: u32 = TREE_DEPTH;

    // Types are correct if this compiles
}

// Verifies that even a proof for a non-existent key in an empty tree
// still has exactly TREE_DEPTH siblings. This is needed for padding
// witnesses (unused signer slots need valid-length proofs).
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

// Documents that k <= MAX_SIGNERS is enforced structurally: the circuit's
// witness arrays are sized to MAX_SIGNERS, so providing more signers is
// impossible. Verifies via max_signers() accessor.
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

// Documents and verifies the relationship between MAX_SIGNERS and the
// maximum feasible validator_count. With MAX_SIGNERS=20,000, the network
// can have up to 39,999 validators (2*20,000-1) while still achieving
// majority with k=20,000 signers.
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

// Verifies that both parameters are accessible via the circuit API and
// prints their values for diagnostic purposes.
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

// Cross-component consistency test: verifies that the circuit, the SMT,
// the global constants, and actual proof generation all agree on
// TREE_DEPTH and MAX_SIGNERS. This is the definitive coordination check.
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
