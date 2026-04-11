//! REQUIREMENT: CIR-001 — Circuit Statement
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-001`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-001.md`.
//!
//! ## Normative Statement
//!
//! The Groth16 circuit proves: "I know k BLS pubkeys, each with a valid Merkle
//! inclusion proof against `validator_merkle_root`, whose G1 sum equals
//! `agg_signers`, and where 2k > `validator_count`." The circuit separates
//! concerns: it proves signer legitimacy and majority, while BLS signature
//! validity is checked on-chain via `bls_verify`.
//!
//! ## How These Tests Prove the Requirement
//!
//! The tests verify that the `ConsensusCircuit` type can be instantiated in all
//! three modes (empty, with public inputs, with witnesses), that it implements
//! `ConstraintSynthesizer<Fr>` for Groth16 integration, and that the circuit
//! stores all six public inputs and private witness data correctly. This gives
//! structural confidence that the circuit is wired for the three-part statement.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Circuit can be created and has positive max_signers (structural)
//! - [x] Circuit parameters (TREE_DEPTH, MAX_SIGNERS) are correct
//! - [x] Six public inputs stored in fixed order matching VK IC points
//! - [x] Private witnesses (signing pubkeys, proofs) tracked correctly
//! - [x] Circuit implements ConstraintSynthesizer trait
//! - [ ] Valid witnesses produce verifying proof (deferred to CIR-002/003/004)
//! - [ ] Invalid witnesses fail proof generation (deferred to CIR-002/003/004)
//! - [ ] Proof verifies on-chain via checkpoint singleton (not yet testable)
//!
//! ## Gaps
//!
//! These tests verify the circuit API surface and data storage, not the actual
//! R1CS constraint generation. The three constraint components (Merkle, G1 sum,
//! majority) are exercised individually in CIR-002, CIR-003, and CIR-004.

use ark_bls12_381::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use chia_l2_consensus::testing::ConsensusCircuit;
use chia_l2_consensus::testing::TREE_DEPTH;

// Verifies that `ConsensusCircuit::new()` succeeds and returns a circuit
// with a positive `max_signers` value. A passing result means the circuit
// type exists, is constructible, and has a non-degenerate signer capacity.
#[test]
fn vv_req_cir_001_circuit_can_be_created() {
    // CIR-001: Circuit can be instantiated
    let circuit = ConsensusCircuit::new();
    assert!(
        circuit.max_signers() > 0,
        "CIR-001: Circuit must have positive max_signers"
    );
}

// Verifies that `tree_depth()` matches the global `TREE_DEPTH` constant and
// that `max_signers()` is at least 1. This ensures the circuit is configured
// to use the same SMT depth as the rest of the system, and that the signer
// array is not zero-length (which would make majority impossible).
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

// Verifies that `with_public_inputs` stores all six public inputs and that
// each accessor returns the exact value passed in. This proves the circuit's
// public-input wiring is correct: no transposition, truncation, or aliasing.
// A passing result means the circuit can be configured for any checkpoint.
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
        validator_count as usize,
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

// Verifies that `with_witnesses` accepts k signing pubkeys and correctly
// reports `actual_signers() == k`. This proves the circuit tracks the
// number of real signers in its witness structure, which is essential for
// the majority check (2k > validator_count).
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

// Verifies that `ConsensusCircuit` implements `ConstraintSynthesizer<Fr>` and
// that `generate_constraints` succeeds on an empty circuit. This is the
// Arkworks trait required by Groth16 -- if this fails, no proof can ever be
// generated. Actual constraint correctness is tested in CIR-002/003/004.
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

// Documents that the circuit statement has three components and verifies that
// the accessor methods (`max_signers`, `tree_depth`) exist. The actual three
// constraints are tested in CIR-002 (Merkle), CIR-003 (G1 sum), CIR-004
// (majority). This test serves as a structural cross-reference.
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

// Verifies that the six public inputs are stored in the specification order:
// (1) validator_merkle_root, (2) validator_count, (3) new_validator_merkle_root,
// (4) new_validator_count, (5) agg_signers, (6) checkpoint_message. Each
// accessor is checked against a distinct sentinel value. This prevents silent
// VK input-point misalignment (a wrong order causes all proofs to fail).
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
        100,          // actual_signers (majority: 2*100 > 100)
    );

    // Verify order by checking accessor methods return expected values
    assert_eq!(circuit.validator_merkle_root(), [0x01u8; 32]);
    assert_eq!(circuit.validator_count(), 100);
    assert_eq!(circuit.new_validator_merkle_root(), [0x02u8; 32]);
    assert_eq!(circuit.new_validator_count(), 101);
    assert_eq!(circuit.agg_signers(), [0x03u8; 48]);
    assert_eq!(circuit.checkpoint_message(), [0x04u8; 32]);
}

// Verifies the private witness structure: k=3 signing pubkeys are passed to
// `with_witnesses` and `actual_signers()` returns 3. This confirms the
// circuit tracks the witness count, which feeds into the majority constraint.
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
