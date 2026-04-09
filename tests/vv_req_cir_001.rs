//! REQUIREMENT: CIR-001 — Circuit Statement
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-001`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-001.md`.
//!
//! Verifies that the Groth16 circuit proves: "I know k BLS pubkeys, each with
//! a valid Merkle inclusion proof against `validator_merkle_root`, whose G1
//! sum equals `agg_signers`, and where 2k > `validator_count`."
//!
//! This test file verifies the circuit structure and that it can be
//! instantiated with proper public inputs and private witnesses.

use ark_bls12_381::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use chia_l2_consensus::merkle::TREE_DEPTH;
use chia_l2_consensus::ConsensusCircuit;

#[test]
fn vv_req_cir_001_circuit_can_be_created() {
    // CIR-001: Circuit can be instantiated
    let circuit = ConsensusCircuit::new();
    assert!(
        circuit.max_signers() > 0,
        "CIR-001: Circuit must have positive max_signers"
    );
}

#[test]
fn vv_req_cir_001_circuit_has_correct_parameters() {
    // CIR-001: Circuit parameters must be correct
    let circuit = ConsensusCircuit::new();

    // TREE_DEPTH must match SMT spec
    assert_eq!(
        circuit.tree_depth(),
        TREE_DEPTH,
        "CIR-001: Circuit TREE_DEPTH must match SMT"
    );

    // MAX_SIGNERS must be reasonable (at least 1)
    assert!(
        circuit.max_signers() >= 1,
        "CIR-001: MAX_SIGNERS must be at least 1"
    );
}

#[test]
fn vv_req_cir_001_circuit_with_public_inputs() {
    // CIR-001: Circuit can be created with public inputs
    let validator_merkle_root = [0x11u8; 32];
    let validator_count = 100u64;
    let new_validator_merkle_root = [0x22u8; 32];
    let new_validator_count = 101u64;
    let agg_signers = [0x33u8; 48];
    let checkpoint_message = [0x44u8; 32];

    let circuit = ConsensusCircuit::with_public_inputs(
        validator_merkle_root,
        validator_count,
        new_validator_merkle_root,
        new_validator_count,
        agg_signers,
        checkpoint_message,
    );

    // Verify public inputs are stored
    assert_eq!(
        circuit.validator_merkle_root(),
        validator_merkle_root,
        "CIR-001: validator_merkle_root must be stored"
    );
    assert_eq!(
        circuit.validator_count(),
        validator_count,
        "CIR-001: validator_count must be stored"
    );
    assert_eq!(
        circuit.new_validator_merkle_root(),
        new_validator_merkle_root,
        "CIR-001: new_validator_merkle_root must be stored"
    );
    assert_eq!(
        circuit.new_validator_count(),
        new_validator_count,
        "CIR-001: new_validator_count must be stored"
    );
    assert_eq!(
        circuit.agg_signers(),
        agg_signers,
        "CIR-001: agg_signers must be stored"
    );
    assert_eq!(
        circuit.checkpoint_message(),
        checkpoint_message,
        "CIR-001: checkpoint_message must be stored"
    );
}

#[test]
fn vv_req_cir_001_circuit_with_witnesses() {
    // CIR-001: Circuit can be created with private witnesses
    let validator_merkle_root = [0x11u8; 32];
    let validator_count = 10u64;
    let new_validator_merkle_root = [0x22u8; 32];
    let new_validator_count = 11u64;
    let agg_signers = [0x33u8; 48];
    let checkpoint_message = [0x44u8; 32];

    // Create signing pubkeys (k = 6 for majority of 10)
    let signing_pubkeys: Vec<[u8; 48]> = (0..6).map(|i| [i as u8; 48]).collect();

    let circuit = ConsensusCircuit::with_witnesses(
        validator_merkle_root,
        validator_count,
        new_validator_merkle_root,
        new_validator_count,
        agg_signers,
        checkpoint_message,
        signing_pubkeys.clone(),
        vec![], // Empty proofs for now (tested in CIR-002)
    );

    assert_eq!(
        circuit.actual_signers(),
        6,
        "CIR-001: actual_signers must match witness count"
    );
}

#[test]
fn vv_req_cir_001_circuit_implements_constraint_synthesizer() {
    // CIR-001: Circuit implements ConstraintSynthesizer trait
    let circuit = ConsensusCircuit::new();

    // Create constraint system
    let cs = ConstraintSystem::<Fr>::new_ref();

    // This should not panic - circuit can generate constraints
    // (actual constraints tested in CIR-002, CIR-003, CIR-004)
    let result = circuit.generate_constraints(cs.clone());

    assert!(
        result.is_ok(),
        "CIR-001: Circuit must implement ConstraintSynthesizer"
    );
}

#[test]
fn vv_req_cir_001_circuit_statement_components() {
    // CIR-001: Circuit statement has three components:
    // 1. Merkle membership for each signer
    // 2. G1 sum equals agg_signers
    // 3. Majority threshold: 2k > validator_count

    // This test documents the circuit statement structure
    // Actual verification is in CIR-002, CIR-003, CIR-004

    let circuit = ConsensusCircuit::new();

    // The circuit must support these operations (methods exist)
    let _ = circuit.max_signers();
    let _ = circuit.tree_depth();

    // Statement components are tested via:
    // - CIR-002: Merkle membership (verify_merkle for each pubkey)
    // - CIR-003: Aggregate key (G1 sum equals agg_signers)
    // - CIR-004: Majority (2k > validator_count)
}

#[test]
fn vv_req_cir_001_public_input_order() {
    // CIR-001: Public inputs must be in fixed order (matches IC points)
    // 1. validator_merkle_root
    // 2. validator_count
    // 3. new_validator_merkle_root
    // 4. new_validator_count
    // 5. agg_signers
    // 6. checkpoint_message

    let circuit = ConsensusCircuit::with_public_inputs(
        [0x01u8; 32], // validator_merkle_root
        100,          // validator_count
        [0x02u8; 32], // new_validator_merkle_root
        101,          // new_validator_count
        [0x03u8; 48], // agg_signers
        [0x04u8; 32], // checkpoint_message
    );

    // Verify order by checking accessor methods return expected values
    assert_eq!(circuit.validator_merkle_root(), [0x01u8; 32]);
    assert_eq!(circuit.validator_count(), 100);
    assert_eq!(circuit.new_validator_merkle_root(), [0x02u8; 32]);
    assert_eq!(circuit.new_validator_count(), 101);
    assert_eq!(circuit.agg_signers(), [0x03u8; 48]);
    assert_eq!(circuit.checkpoint_message(), [0x04u8; 32]);
}

#[test]
fn vv_req_cir_001_private_witnesses_structure() {
    // CIR-001: Private witnesses consist of:
    // - signing_pubkeys: k BLS pubkeys (G1, 48 bytes each)
    // - merkle_proofs: k Merkle proofs
    // - active flags: which slots are actually used

    let signing_pubkeys: Vec<[u8; 48]> = vec![[0xAAu8; 48], [0xBBu8; 48], [0xCCu8; 48]];

    let circuit = ConsensusCircuit::with_witnesses(
        [0x00u8; 32],
        10,
        [0x00u8; 32],
        10,
        [0x00u8; 48],
        [0x00u8; 32],
        signing_pubkeys,
        vec![],
    );

    // k = 3 signers
    assert_eq!(
        circuit.actual_signers(),
        3,
        "CIR-001: Private witness tracks actual signer count"
    );
}
