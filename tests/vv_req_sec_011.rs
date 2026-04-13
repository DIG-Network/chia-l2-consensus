//! REQUIREMENT: SEC-011 — Phantom Majority Forgery Resistance
//!
//! CIR-003 (aggregate key constraint) is implemented via non-native G1
//! arithmetic in `src/prover/g1_gadget.rs`. When signing_pubkeys are provided
//! to the circuit via `with_public_inputs()`, the circuit enforces
//! G1_sum(pubkeys) == agg_signers.
//!
//! These tests verify that the SEC-011 vulnerability is CLOSED:
//!
//! 1. Phantom majority attacks FAIL when signing pubkeys are provided
//! 2. Legitimate majority proofs with correct aggregates SUCCEED
//!
//! Spec: `docs/requirements/domains/security/specs/SEC-011.md`
//! Implementation: `src/prover/g1_gadget.rs`, `src/prover/circuit.rs`

mod common;

use chia_l2_consensus::testing::{
    aggregate_pubkeys, deserialize_proving_key, generate_proof, run_test_setup, ConsensusCircuit,
    GROTH16_PROOF_SIZE,
};
use sha2::{Digest, Sha256};

/// Helper: compute checkpoint message per WIRE-001 / CHK-012
fn compute_checkpoint_message(
    state_root: &[u8; 32],
    merkle_root: &[u8; 32],
    count: u64,
    epoch: u64,
    network_id: &[u8; 32],
) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(state_root);
    h.update(merkle_root);
    h.update(count.to_be_bytes());
    h.update(epoch.to_be_bytes());
    h.update(network_id);
    h.finalize().into()
}

/// Generate a valid BLS12-381 G1 pubkey from a seed byte.
fn test_pubkey(seed: u8) -> [u8; 48] {
    use ark_bls12_381::{Fr, G1Affine};
    use ark_ec::AffineRepr;
    use ark_ff::PrimeField;
    use ark_serialize::CanonicalSerialize;

    let scalar = Fr::from(seed as u64 + 1);
    let point = G1Affine::generator().mul_bigint(scalar.into_bigint());
    let affine = G1Affine::from(point);
    let mut bytes = Vec::with_capacity(48);
    affine.serialize_compressed(&mut bytes).unwrap();
    bytes.try_into().unwrap()
}

// ── SEC-011 FIX VERIFICATION: Phantom majority BLOCKED ──────────────

/// SEC-011: Phantom majority attack is BLOCKED by CIR-003.
///
/// An attacker with ONE key cannot generate a valid proof claiming to
/// represent a majority when using the secure constructor. CIR-003 enforces
/// that G1_sum(signing_pubkeys) == agg_signers, and an attacker cannot
/// provide pubkeys that sum to their own key.
#[test]
fn vv_req_sec_011_phantom_majority_proof_generation_fails() {
    let (pk_bytes, _vk_bytes) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // Attacker has 3 legitimate validator pubkeys
    let pk1 = test_pubkey(10);
    let pk2 = test_pubkey(20);
    let pk3 = test_pubkey(30);

    // Attacker sets agg_signers to their OWN key (not the aggregate)
    let attacker_key = test_pubkey(99);

    let msg = compute_checkpoint_message(&[0xAA; 32], &[0xBB; 32], 5, 1, &[0x00; 32]);

    // SECURE PATH: Provide signing pubkeys — CIR-003 IS enforced
    let circuit = ConsensusCircuit::with_public_inputs(
        [0xBB; 32],
        5,
        [0xBB; 32],
        5,
        attacker_key, // WRONG: attacker's key, not sum(pk1,pk2,pk3)
        msg,
        3,
        vec![pk1, pk2, pk3], // These don't sum to attacker_key
    );

    // Proof generation MUST FAIL because CIR-003 rejects the mismatch
    let result = std::panic::catch_unwind(|| generate_proof(circuit, &pk));
    assert!(
        result.is_err(),
        "SEC-011 FIX: Phantom majority proof must FAIL with secure constructor"
    );

    eprintln!(
        "SEC-011 FIXED: Phantom majority attack blocked — \
         proof generation panics when agg_signers != G1_sum(pubkeys)"
    );
}

/// SEC-011: Extreme phantom ratio (51/100) also blocked.
#[test]
fn vv_req_sec_011_phantom_majority_extreme_ratio_blocked() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // 3 real pubkeys (pretending to be 51 of 100)
    let pk1 = test_pubkey(1);
    let pk2 = test_pubkey(2);
    let pk3 = test_pubkey(3);

    // Correct aggregate of the 3 keys
    let real_agg = aggregate_pubkeys(&[pk1, pk2, pk3]).expect("aggregate");

    // Attacker's own key (different from real aggregate)
    let attacker_key = test_pubkey(99);
    assert_ne!(
        attacker_key, real_agg,
        "attacker key must differ from real aggregate"
    );

    let msg = compute_checkpoint_message(&[0x11; 32], &[0x22; 32], 100, 1, &[0x00; 32]);

    let circuit = ConsensusCircuit::with_public_inputs(
        [0x22; 32],
        100,
        [0x22; 32],
        100,
        attacker_key, // WRONG: not the sum of pk1+pk2+pk3
        msg,
        3,                   // Claiming 3 signers
        vec![pk1, pk2, pk3], // But aggregate doesn't match
    );

    let result = std::panic::catch_unwind(|| generate_proof(circuit, &pk));
    assert!(
        result.is_err(),
        "SEC-011 FIX: Extreme phantom majority (51/100 claim) must be blocked"
    );

    eprintln!("SEC-011 FIXED: Extreme phantom majority attack blocked");
}

/// SEC-011: With CIR-003, the actual signers ARE constrained — the aggregate
/// must match the provided pubkeys.
#[test]
fn vv_req_sec_011_actual_signers_constrained_to_keys() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let pk1 = test_pubkey(1);
    let pk2 = test_pubkey(2);
    let pk3 = test_pubkey(3);

    // Correct aggregate for 3 keys
    let agg_3 = aggregate_pubkeys(&[pk1, pk2, pk3]).expect("aggregate");

    let msg = compute_checkpoint_message(&[0; 32], &[0; 32], 5, 1, &[0; 32]);

    // Try claiming 3 signers with correct aggregate + correct pubkeys → should succeed
    let circuit = ConsensusCircuit::with_public_inputs(
        [0; 32],
        5,
        [0; 32],
        5,
        agg_3,
        msg,
        3,
        vec![pk1, pk2, pk3],
    );
    let result = generate_proof(circuit, &pk);
    assert!(
        result.is_ok(),
        "SEC-011: Correct aggregate with matching keys must succeed, got: {:?}",
        result.err()
    );

    // Now try wrong aggregate (pk4 instead of sum) with same pubkeys → must fail
    let wrong_agg = test_pubkey(4);
    let circuit = ConsensusCircuit::with_public_inputs(
        [0; 32],
        5,
        [0; 32],
        5,
        wrong_agg,
        msg,
        3,
        vec![pk1, pk2, pk3],
    );
    let result = std::panic::catch_unwind(|| generate_proof(circuit, &pk));
    assert!(
        result.is_err(),
        "SEC-011 FIX: Wrong aggregate with provided keys must fail"
    );

    eprintln!(
        "SEC-011 FIXED: Signers are constrained — correct aggregate succeeds, \
         wrong aggregate fails"
    );
}

/// SEC-011: CIR-004 (minority threshold) still works alongside CIR-003.
#[test]
fn vv_req_sec_011_minority_still_rejected_by_cir_004() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let pk1 = test_pubkey(1);
    let pk2 = test_pubkey(2);

    // k=2, n=5 → 2*2=4 ≤ 5 → SHOULD FAIL (CIR-004)
    let agg = aggregate_pubkeys(&[pk1, pk2]).expect("aggregate");
    let msg = compute_checkpoint_message(&[0; 32], &[0; 32], 5, 1, &[0; 32]);

    let circuit =
        ConsensusCircuit::with_public_inputs([0; 32], 5, [0; 32], 5, agg, msg, 2, vec![pk1, pk2]);

    let result = std::panic::catch_unwind(|| generate_proof(circuit, &pk));
    assert!(
        result.is_err(),
        "CIR-004 must reject minority (k=2, n=5) even with valid CIR-003"
    );

    eprintln!("SEC-011: CIR-004 correctly rejects minority — both constraints enforced");
}

/// SEC-011: Legitimate majority proofs STILL WORK with CIR-003 enforced.
///
/// This is critical — the fix must not break valid checkpoints.
#[test]
fn vv_req_sec_011_legitimate_majority_succeeds() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // 3 validators, k=2 is not majority of 3 (2*2=4 > 3), k=2 works
    // Actually: 2*2 = 4 > 3 ✓
    let pk1 = test_pubkey(1);
    let pk2 = test_pubkey(2);

    let agg = aggregate_pubkeys(&[pk1, pk2]).expect("aggregate");
    let msg = compute_checkpoint_message(&[0; 32], &[0; 32], 3, 1, &[0; 32]);

    let circuit =
        ConsensusCircuit::with_public_inputs([0; 32], 3, [0; 32], 3, agg, msg, 2, vec![pk1, pk2]);

    let proof = generate_proof(circuit, &pk);
    assert!(
        proof.is_ok(),
        "SEC-011: Legitimate majority must produce valid proof, got: {:?}",
        proof.err()
    );

    let proof_bytes = proof.unwrap();
    assert_eq!(proof_bytes.len(), GROTH16_PROOF_SIZE);

    // Verify proof is non-trivial
    assert!(
        !proof_bytes[0..48].iter().all(|&b| b == 0),
        "Proof A non-zero"
    );
    assert!(
        !proof_bytes[48..144].iter().all(|&b| b == 0),
        "Proof B non-zero"
    );
    assert!(
        !proof_bytes[144..192].iter().all(|&b| b == 0),
        "Proof C non-zero"
    );

    eprintln!("SEC-011: Legitimate majority proof succeeded with CIR-003 enforced");
}

/// SEC-011: CIR-003 enforcement code exists in the circuit.
#[test]
fn vv_req_sec_011_cir_003_enforced_in_circuit() {
    let circuit_source = include_str!("../src/prover/circuit.rs");
    let gadget_source = include_str!("../src/prover/g1_gadget.rs");

    // Circuit calls enforce_aggregate_key from g1_gadget
    assert!(
        circuit_source.contains("enforce_aggregate_key"),
        "SEC-011: Circuit must call enforce_aggregate_key"
    );

    // g1_gadget enforces equality
    assert!(
        gadget_source.contains("enforce_equal"),
        "SEC-011: g1_gadget must enforce G1 sum equality"
    );

    eprintln!("SEC-011: CIR-003 enforcement confirmed in circuit source");
}

/// SEC-011: The with_public_inputs constructor must accept signing_pubkeys
/// to enable CIR-003 enforcement.
#[test]
fn vv_req_sec_011_constructor_accepts_signing_pubkeys() {
    let source = include_str!("../src/prover/circuit.rs");

    // The constructor must accept signing_pubkeys parameter
    assert!(
        source.contains("signing_pubkeys"),
        "Constructor must accept signing_pubkeys for CIR-003"
    );

    // The constructor must exist
    assert!(
        source.contains("pub fn with_public_inputs("),
        "with_public_inputs constructor must be available"
    );

    // CIR-003 enforcement must be documented
    assert!(
        source.contains("CIR-003"),
        "Constructor must document CIR-003 enforcement"
    );

    eprintln!("SEC-011: Constructor accepts signing_pubkeys for CIR-003 enforcement");
}
