//! REQUIREMENT: DEP-001 — Trusted Setup
//! (`docs/requirements/domains/deployment/NORMATIVE.md#DEP-001`).
//!
//! Spec: `docs/requirements/domains/deployment/specs/DEP-001.md`.
//!
//! Implementation: `src/prover/setup.rs`.
//!
//! Verifies that the trusted setup produces valid proving and verification
//! keys, that the VK has the correct structure (672 bytes, 7 IC points),
//! and that deployment artifact helpers (validation, hashing, serialization)
//! work correctly.

use chia_l2_consensus::testing::{
    deserialize_proving_key, deserialize_verification_key, extract_vk_components, generate_proof,
    run_test_setup, ConsensusCircuit, PUBLIC_INPUT_COUNT,
};

// ── Setup produces valid keys ──────────────────────────────────────

#[test]
fn vv_req_dep_001_setup_produces_valid_keys() {
    let (pk_bytes, vk_bytes) = run_test_setup().expect("DEP-001: Setup must succeed");

    assert!(
        !pk_bytes.is_empty(),
        "DEP-001: Proving key must be non-empty"
    );
    assert!(
        !vk_bytes.is_empty(),
        "DEP-001: Verification key must be non-empty"
    );

    // Both must deserialize
    let _pk = deserialize_proving_key(&pk_bytes).expect("DEP-001: PK must deserialize");
    let _vk = deserialize_verification_key(&vk_bytes).expect("DEP-001: VK must deserialize");
}

// ── VK has exactly 7 IC points ─────────────────────────────────────

#[test]
fn vv_req_dep_001_vk_has_seven_ic_points() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let components = extract_vk_components(&pk.vk).expect("VK components");

    assert_eq!(
        components.ic_points.len(),
        PUBLIC_INPUT_COUNT + 1, // 6 public inputs + 1 constant = 7
        "DEP-001: VK must have exactly 7 IC points (6 public inputs + 1 constant)"
    );
}

// ── VK component sizes match spec ──────────────────────────────────

#[test]
fn vv_req_dep_001_vk_component_sizes() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let components = extract_vk_components(&pk.vk).expect("VK components");

    assert_eq!(
        components.alpha_g1.len(),
        48,
        "DEP-001: alpha_g1 must be 48 bytes (G1 compressed)"
    );
    assert_eq!(
        components.beta_g2.len(),
        96,
        "DEP-001: beta_g2 must be 96 bytes (G2 compressed)"
    );
    assert_eq!(
        components.gamma_g2.len(),
        96,
        "DEP-001: gamma_g2 must be 96 bytes (G2 compressed)"
    );
    assert_eq!(
        components.delta_g2.len(),
        96,
        "DEP-001: delta_g2 must be 96 bytes (G2 compressed)"
    );

    for (i, ic) in components.ic_points.iter().enumerate() {
        assert_eq!(
            ic.len(),
            48,
            "DEP-001: IC point {} must be 48 bytes (G1 compressed)",
            i
        );
    }
}

// ── VK serializes to exactly 672 bytes ─────────────────────────────

#[test]
fn vv_req_dep_001_vk_total_672_bytes() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk_bytes = chia_l2_consensus::testing::vk_to_bytes(&pk.vk).expect("DEP-001: vk_to_bytes");

    // 48 (alpha) + 96 (beta) + 96 (gamma) + 96 (delta) + 7*48 (IC) = 672
    assert_eq!(
        vk_bytes.len(),
        672,
        "DEP-001: Full VK serialization must be exactly 672 bytes"
    );
}

// ── VK hash is deterministic ───────────────────────────────────────

#[test]
fn vv_req_dep_001_vk_hash_deterministic() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let hash1 = chia_l2_consensus::testing::compute_vk_hash(&pk.vk).expect("DEP-001: VK hash 1");
    let hash2 = chia_l2_consensus::testing::compute_vk_hash(&pk.vk).expect("DEP-001: VK hash 2");

    assert_eq!(hash1, hash2, "DEP-001: VK hash must be deterministic");
    assert_eq!(
        hash1.len(),
        32,
        "DEP-001: VK hash must be 32 bytes (SHA-256)"
    );
}

// ── VK validation succeeds for valid VK ────────────────────────────

#[test]
fn vv_req_dep_001_validate_vk_succeeds() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let result = chia_l2_consensus::testing::validate_vk(&pk.vk);
    assert!(
        result.is_ok(),
        "DEP-001: Valid VK must pass validation: {:?}",
        result.err()
    );
}

// ── PK round-trip serialization ────────────────────────────────────

#[test]
fn vv_req_dep_001_pk_roundtrip() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");

    // Deserialize
    let pk = deserialize_proving_key(&pk_bytes).expect("DEP-001: PK deserialize");

    // Re-serialize
    let mut re_serialized = Vec::new();
    use ark_serialize::CanonicalSerialize;
    pk.serialize_compressed(&mut re_serialized)
        .expect("DEP-001: PK re-serialize");

    assert_eq!(
        pk_bytes, re_serialized,
        "DEP-001: PK round-trip must produce identical bytes"
    );
}

// ── VK round-trip serialization ────────────────────────────────────

#[test]
fn vv_req_dep_001_vk_roundtrip() {
    let (_, vk_bytes) = run_test_setup().expect("Setup");

    // Deserialize
    let vk = deserialize_verification_key(&vk_bytes).expect("DEP-001: VK deserialize");

    // Re-serialize
    let mut re_serialized = Vec::new();
    use ark_serialize::CanonicalSerialize;
    vk.serialize_compressed(&mut re_serialized)
        .expect("DEP-001: VK re-serialize");

    assert_eq!(
        vk_bytes, re_serialized,
        "DEP-001: VK round-trip must produce identical bytes"
    );
}

// ── Test proof verifies with setup VK ──────────────────────────────

#[test]
fn vv_req_dep_001_proof_verifies_with_setup_vk() {
    let (pk_bytes, vk_bytes) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = deserialize_verification_key(&vk_bytes).expect("VK");

    // Generate proof with valid majority (k=1, n=1 → 2*1 > 1 → true)
    let circuit = ConsensusCircuit::with_public_inputs(
        [0xAA; 32], // validator_merkle_root
        1,          // validator_count
        [0xBB; 32], // new_validator_merkle_root
        1,          // new_validator_count
        [0xCC; 48], // agg_signers
        [0xDD; 32], // checkpoint_message
        1,          // actual_signers
    );

    let proof_bytes = generate_proof(circuit, &pk).expect("DEP-001: Proof generation");
    assert_eq!(proof_bytes.len(), 192, "DEP-001: Proof must be 192 bytes");

    // Verify with VK using arkworks
    use ark_bls12_381::Bls12_381;
    use ark_groth16::Groth16;
    use ark_serialize::CanonicalDeserialize;
    use chia_l2_consensus::testing::bytes_to_scalar;

    let proof = ark_groth16::Proof::<Bls12_381>::deserialize_compressed(&proof_bytes[..])
        .expect("DEP-001: Proof deserialize");

    let public_inputs = vec![
        bytes_to_scalar(&[0xAA; 32]),
        bytes_to_scalar(&1u64.to_be_bytes()),
        bytes_to_scalar(&[0xBB; 32]),
        bytes_to_scalar(&1u64.to_be_bytes()),
        bytes_to_scalar(&[0xCC; 48]),
        bytes_to_scalar(&[0xDD; 32]),
    ];

    let pvk = ark_groth16::prepare_verifying_key(&vk);
    let valid = Groth16::<Bls12_381>::verify_proof(&pvk, &proof, &public_inputs)
        .expect("DEP-001: Verification");
    assert!(valid, "DEP-001: Test proof must verify with setup VK");
}

// ── VK bytes match component concatenation ─────────────────────────

#[test]
fn vv_req_dep_001_vk_bytes_match_components() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let vk_bytes = chia_l2_consensus::testing::vk_to_bytes(&pk.vk).expect("vk_to_bytes");
    let components = extract_vk_components(&pk.vk).expect("components");

    // Manually concatenate components and compare
    let mut expected = Vec::with_capacity(672);
    expected.extend_from_slice(&components.alpha_g1);
    expected.extend_from_slice(&components.beta_g2);
    expected.extend_from_slice(&components.gamma_g2);
    expected.extend_from_slice(&components.delta_g2);
    for ic in &components.ic_points {
        expected.extend_from_slice(ic);
    }

    assert_eq!(
        vk_bytes, expected,
        "DEP-001: vk_to_bytes must match concatenated components"
    );
}

// ── VK hash matches manual computation ─────────────────────────────

#[test]
fn vv_req_dep_001_vk_hash_matches_manual() {
    use sha2::{Digest, Sha256};

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let vk_bytes = chia_l2_consensus::testing::vk_to_bytes(&pk.vk).expect("vk_to_bytes");
    let computed_hash =
        chia_l2_consensus::testing::compute_vk_hash(&pk.vk).expect("compute_vk_hash");

    // Manual SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&vk_bytes);
    let manual_hash: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        computed_hash, manual_hash,
        "DEP-001: compute_vk_hash must equal sha256(vk_to_bytes)"
    );
}

// ── Circuit parameters reflected in VK ─────────────────────────────

#[test]
fn vv_req_dep_001_circuit_parameters_in_vk() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // VK IC count = PUBLIC_INPUT_COUNT + 1 = 7
    // This indirectly confirms the circuit has the right number of public inputs
    assert_eq!(
        pk.vk.gamma_abc_g1.len(),
        PUBLIC_INPUT_COUNT + 1,
        "DEP-001: VK IC count must reflect circuit public input count"
    );

    // Non-trivial: IC points should not all be identity
    let components = extract_vk_components(&pk.vk).expect("components");
    let all_same = components.ic_points.windows(2).all(|w| w[0] == w[1]);
    assert!(
        !all_same,
        "DEP-001: IC points should not all be identical (circuit has distinct inputs)"
    );
}
