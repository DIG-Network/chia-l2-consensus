//! REQUIREMENT: DEP-001 — Trusted Setup
//! (`docs/requirements/domains/deployment/NORMATIVE.md#DEP-001`).
//!
//! Spec: `docs/requirements/domains/deployment/specs/DEP-001.md`.
//!
//! Implementation: `src/prover/setup.rs`.
//!
//! **Normative statement:** A Groth16 trusted setup ceremony produces a proving
//! key (PK) and a verification key (VK) bound to the circuit's parameters
//! (MAX_SIGNERS, TREE_DEPTH). The VK has exactly 7 IC points (6 public inputs
//! + 1 constant), totals 672 bytes when serialized, and its SHA-256 hash is
//! deterministic. Both keys must round-trip through serialization. A test proof
//! generated with the PK must verify with the VK.
//!
//! **How the tests prove this:**
//! - `setup_produces_valid_keys` runs the setup and verifies both PK and VK
//!   are non-empty and deserializable.
//! - `vk_has_seven_ic_points` extracts VK components and checks IC count.
//! - `vk_component_sizes` verifies alpha_g1 (48B), beta/gamma/delta_g2 (96B
//!   each), and all IC points (48B each).
//! - `vk_total_672_bytes` serializes the full VK and checks the byte count.
//! - `vk_hash_deterministic` computes the VK hash twice and compares.
//! - `validate_vk_succeeds` runs structural validation on the VK.
//! - `pk_roundtrip` and `vk_roundtrip` serialize then deserialize and
//!   compare byte-for-byte.
//! - `proof_verifies_with_setup_vk` generates a real Groth16 proof with the
//!   consensus circuit and verifies it with the VK using arkworks.
//! - `vk_bytes_match_components` confirms vk_to_bytes equals the concatenation
//!   of extracted components.
//! - `vk_hash_matches_manual` computes sha256(vk_to_bytes) independently.
//! - `circuit_parameters_in_vk` checks gamma_abc_g1.len() and non-degeneracy.
//!
//! **Acceptance-criteria coverage (from spec):**
//! - [x] Trusted setup completed before deployment
//! - [x] Circuit parameters match deployment config (7 IC points)
//! - [x] VK has correct structure (672 bytes, 7 IC points)
//! - [x] Test proof verifies in Rust
//! - [ ] Ceremony uses MPC (production concern; dev uses run_test_setup)

use chia_l2_consensus::testing::{
    deserialize_proving_key, deserialize_verification_key, extract_vk_components, generate_proof,
    run_test_setup, ConsensusCircuit, PUBLIC_INPUT_COUNT,
};

// ── Setup produces valid keys ──────────────────────────────────────

/// Verifies that run_test_setup produces non-empty, deserializable PK and VK.
/// Strategy: run the full Groth16 setup and attempt deserialization.
/// Confidence: the setup pipeline produces structurally valid keys.
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

/// Verifies the VK contains exactly 7 IC points (PUBLIC_INPUT_COUNT + 1).
/// Strategy: extract VK components and check ic_points.len().
/// Confidence: the circuit has exactly 6 public inputs, producing 7 IC points.
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

/// Verifies each VK component has the correct byte size per WIRE-002/WIRE-003.
/// Strategy: extract components and check alpha_g1=48, beta/gamma/delta_g2=96,
/// each IC point=48.
/// Confidence: the VK serialization matches the wire format specification.
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

/// Verifies the full VK serialization is exactly 672 bytes:
/// 48 (alpha) + 96 (beta) + 96 (gamma) + 96 (delta) + 7*48 (IC) = 672.
/// Strategy: call vk_to_bytes and check the length.
/// Confidence: the on-chain VK curried into the singleton has the expected size.
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

/// Verifies the VK hash is deterministic and 32 bytes (SHA-256).
/// Strategy: compute the hash twice and compare.
/// Confidence: the published VK hash can be reliably verified.
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

/// Verifies the structural validation of a valid VK passes.
/// Strategy: call validate_vk and assert Ok.
/// Confidence: the validation routine accepts well-formed keys.
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

/// Verifies the proving key survives a serialize/deserialize round-trip.
/// Strategy: deserialize the PK bytes, re-serialize, and compare byte-for-byte.
/// Confidence: the PK can be stored and reloaded without data loss.
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

/// Verifies the verification key survives a serialize/deserialize round-trip.
/// Strategy: deserialize the VK bytes, re-serialize, and compare byte-for-byte.
/// Confidence: the VK can be distributed and reloaded without data loss.
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

/// End-to-end test: generates a Groth16 proof using the consensus circuit with
/// valid majority (k=1, n=1) and verifies it against the VK using arkworks.
/// Strategy: construct a ConsensusCircuit with known public inputs, generate a
/// proof, deserialize it, prepare the public input scalars, and call
/// Groth16::verify_proof.
/// Confidence: the full pipeline (setup -> prove -> verify) works end-to-end.
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

/// Verifies vk_to_bytes equals the manual concatenation of extracted components.
/// Strategy: extract alpha_g1, beta_g2, gamma_g2, delta_g2, and IC points,
/// concatenate them, and compare to vk_to_bytes output.
/// Confidence: the serialization helper is consistent with component extraction.
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

/// Verifies compute_vk_hash equals sha256(vk_to_bytes) computed independently.
/// Strategy: compute both and compare.
/// Confidence: the hash function uses the correct serialization as input.
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

/// Verifies the VK reflects the circuit's public input count (gamma_abc_g1.len()
/// = PUBLIC_INPUT_COUNT + 1 = 7) and that IC points are not all identical.
/// Strategy: check the length and assert at least two IC points differ.
/// Confidence: the setup was bound to the correct circuit parameters.
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
