//! CIR-003 constraint tests — verify non-native G1 aggregate key binding.
//!
//! These tests verify that when signing_pubkeys are provided to the circuit,
//! proof generation succeeds ONLY if G1_sum(pubkeys) == agg_signers.
//! This closes SEC-011 (phantom majority attack).

use chia_l2_consensus::testing::{
    aggregate_pubkeys, deserialize_proving_key, generate_proof, run_test_setup, ConsensusCircuit,
};
use sha2::{Digest, Sha256};

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

    let scalar = Fr::from(seed as u64 + 1); // +1 to avoid zero
    let point = G1Affine::generator().mul_bigint(scalar.into_bigint());
    let affine = G1Affine::from(point);
    let mut bytes = Vec::with_capacity(48);
    affine.serialize_compressed(&mut bytes).unwrap();
    bytes.try_into().unwrap()
}

// ── Test: CIR-003 accepts correct aggregate ────────────────────────

/// When signing_pubkeys are provided and their G1 sum equals agg_signers,
/// proof generation MUST succeed.
#[test]
fn vv_req_cir_003_correct_aggregate_succeeds() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // 3 real pubkeys
    let pk1 = test_pubkey(1);
    let pk2 = test_pubkey(2);
    let pk3 = test_pubkey(3);

    // Correct aggregate: G1_sum(pk1, pk2, pk3)
    let agg = aggregate_pubkeys(&[pk1, pk2, pk3]).expect("aggregate");

    let msg = compute_checkpoint_message(&[0; 32], &[0; 32], 5, 1, &[0; 32]);

    let circuit = ConsensusCircuit::with_public_inputs(
        [0; 32],
        5,
        [0; 32],
        5,
        agg,
        msg,
        3,
        vec![pk1, pk2, pk3],
    );

    let proof = generate_proof(circuit, &pk);
    assert!(
        proof.is_ok(),
        "CIR-003: Correct aggregate must produce valid proof, got: {:?}",
        proof.err()
    );
    eprintln!("CIR-003: Proof with correct G1 sum succeeded");
}

/// When signing_pubkeys are provided but agg_signers is a DIFFERENT key
/// (not the G1 sum), proof generation MUST FAIL.
///
/// This is the core SEC-011 fix validation.
#[test]
fn vv_req_cir_003_wrong_aggregate_fails() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // 3 real pubkeys
    let pk1 = test_pubkey(1);
    let pk2 = test_pubkey(2);
    let pk3 = test_pubkey(3);

    // WRONG aggregate: use a different key (pk4), NOT the sum of pk1+pk2+pk3
    let wrong_agg = test_pubkey(4);

    let msg = compute_checkpoint_message(&[0; 32], &[0; 32], 5, 1, &[0; 32]);

    let circuit = ConsensusCircuit::with_public_inputs(
        [0; 32],
        5,
        [0; 32],
        5,
        wrong_agg, // WRONG: this is pk4, not sum(pk1,pk2,pk3)
        msg,
        3,
        vec![pk1, pk2, pk3],
    );

    let result = std::panic::catch_unwind(|| generate_proof(circuit, &pk));
    assert!(
        result.is_err(),
        "CIR-003: Wrong aggregate must fail proof generation (SEC-011 fix)"
    );
    eprintln!("CIR-003: Proof with wrong aggregate correctly rejected");
}

/// The phantom majority attack from SEC-011: attacker uses their own key
/// as agg_signers but provides OTHER validators' pubkeys. Must fail.
#[test]
fn vv_req_cir_003_phantom_majority_blocked() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // 3 legitimate validator pubkeys
    let pk1 = test_pubkey(10);
    let pk2 = test_pubkey(20);
    let pk3 = test_pubkey(30);

    // Attacker's own key (NOT the sum of pk1+pk2+pk3)
    let attacker_key = test_pubkey(99);

    let msg = compute_checkpoint_message(&[0xAA; 32], &[0xBB; 32], 5, 1, &[0; 32]);

    let circuit = ConsensusCircuit::with_public_inputs(
        [0xBB; 32],
        5,
        [0xBB; 32],
        5,
        attacker_key, // Attacker's key, not aggregate of validators
        msg,
        3,
        vec![pk1, pk2, pk3], // Legitimate validator pubkeys
    );

    let result = std::panic::catch_unwind(|| generate_proof(circuit, &pk));
    assert!(
        result.is_err(),
        "CIR-003/SEC-011: Phantom majority MUST be blocked when pubkeys are provided"
    );
    eprintln!("CIR-003/SEC-011: Phantom majority attack successfully blocked!");
}

/// Single signer: agg_signers == the single pubkey. Must succeed.
#[test]
fn vv_req_cir_003_single_signer() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let pk1 = test_pubkey(42);
    // For single signer, aggregate is just the key itself
    let agg = aggregate_pubkeys(&[pk1]).expect("aggregate");
    assert_eq!(agg, pk1, "Single key aggregate should be itself");

    let msg = compute_checkpoint_message(&[0; 32], &[0; 32], 1, 1, &[0; 32]);

    let circuit =
        ConsensusCircuit::with_public_inputs([0; 32], 1, [0; 32], 1, agg, msg, 1, vec![pk1]);

    let proof = generate_proof(circuit, &pk);
    assert!(
        proof.is_ok(),
        "CIR-003: Single signer with matching aggregate must succeed"
    );
    eprintln!("CIR-003: Single signer proof succeeded");
}

/// When signing_pubkeys is empty, CIR-003 is not activated.
/// This is valid for tests that verify properties other than CIR-003,
/// but production proofs MUST always supply signing pubkeys.
#[test]
fn vv_req_cir_003_empty_pubkeys_skips_constraint() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let msg = compute_checkpoint_message(&[0; 32], &[0; 32], 5, 1, &[0; 32]);

    // Empty signing_pubkeys → CIR-003 not activated
    let circuit = ConsensusCircuit::with_public_inputs(
        [0; 32],
        5,
        [0; 32],
        5,
        [0xAB; 48],
        msg,
        3,
        Vec::new(),
    );

    // Succeeds because no signing_pubkeys → CIR-003 not activated
    let proof = generate_proof(circuit, &pk);
    assert!(
        proof.is_ok(),
        "Empty signing_pubkeys should still produce a valid proof"
    );
    eprintln!(
        "CIR-003: Empty signing_pubkeys skips CIR-003 — \
         production proofs must always supply real pubkeys"
    );
}
