//! REQUIREMENT: DEP-004 — Verification Key Verification
//! (`docs/requirements/domains/deployment/NORMATIVE.md#DEP-004`).
//!
//! Spec: `docs/requirements/domains/deployment/specs/DEP-004.md`.
//!
//! Implementation: `src/prover/setup.rs`.
//!
//! ## Normative statement
//! Operators MUST be able to verify a deployed VK by: (1) validating its
//! structural correctness (size, field sizes), (2) hash-matching against
//! a published VK hash, and (3) extracting individual components (alpha_g1,
//! beta_g2, gamma_g2, delta_g2, 7 IC points) for comparison.
//!
//! ## How the tests prove the requirement
//! 1. **Hash matches**: verify_vk_hash succeeds for matching VK.
//! 2. **Hash mismatch detected**: Wrong hash returns false.
//! 3. **Corrupted VK detected**: Flipping one byte fails hash check.
//! 4. **Validation succeeds**: Valid VK bytes pass validate_vk_bytes.
//! 5. **Wrong length rejected**: 671 bytes and 673 bytes both fail.
//! 6. **Empty rejected**: Zero bytes fails.
//! 7. **VK_BYTE_SIZE = 672**: Constant verified.
//! 8. **Components match**: Extracted components from bytes match original.
//! 9. **Full workflow**: setup -> serialize -> verify end-to-end.
//! 10. **Different setup detected**: Modified VK fails hash check.
//!
//! ## Completeness: HIGH
//! ## Gaps: Does not test against a real on-chain singleton read.

use chia_l2_consensus::testing::{
    compute_vk_hash, deserialize_proving_key, extract_vk_components, run_test_setup,
    validate_vk_bytes, verify_vk_hash, vk_to_bytes, VK_BYTE_SIZE,
};

// ── VK hash verification succeeds for matching VK ───────────────────

#[test]
fn vv_req_dep_004_hash_matches() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let vk_bytes = vk_to_bytes(&pk.vk).expect("vk_to_bytes");
    let expected_hash = compute_vk_hash(&pk.vk).expect("compute_vk_hash");

    assert!(
        verify_vk_hash(&vk_bytes, &expected_hash),
        "DEP-004: VK hash must match for identical VK"
    );
}

// ── VK hash verification fails for wrong hash ──────────────────────

#[test]
fn vv_req_dep_004_hash_mismatch_detected() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let vk_bytes = vk_to_bytes(&pk.vk).expect("vk_to_bytes");
    let wrong_hash = [0xFF; 32];

    assert!(
        !verify_vk_hash(&vk_bytes, &wrong_hash),
        "DEP-004: VK hash must NOT match for wrong expected hash"
    );
}

// ── VK hash verification fails for corrupted VK ────────────────────

#[test]
fn vv_req_dep_004_hash_fails_for_corrupted_vk() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let vk_bytes = vk_to_bytes(&pk.vk).expect("vk_to_bytes");
    let correct_hash = compute_vk_hash(&pk.vk).expect("hash");

    // Corrupt one byte
    let mut corrupted = vk_bytes.clone();
    corrupted[100] ^= 0xFF;

    assert!(
        !verify_vk_hash(&corrupted, &correct_hash),
        "DEP-004: Corrupted VK must not match original hash"
    );
}

// ── validate_vk_bytes succeeds for valid VK ─────────────────────────

#[test]
fn vv_req_dep_004_validate_bytes_succeeds() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let vk_bytes = vk_to_bytes(&pk.vk).expect("vk_to_bytes");

    let result = validate_vk_bytes(&vk_bytes);
    assert!(
        result.is_ok(),
        "DEP-004: Valid VK bytes must pass validation: {:?}",
        result.err()
    );
}

// ── validate_vk_bytes fails for wrong length ────────────────────────

#[test]
fn vv_req_dep_004_validate_bytes_wrong_length() {
    let too_short = vec![0u8; 671]; // 1 byte short
    let result = validate_vk_bytes(&too_short);
    assert!(
        result.is_err(),
        "DEP-004: VK bytes with wrong length must fail validation"
    );

    let too_long = vec![0u8; 673]; // 1 byte extra
    let result = validate_vk_bytes(&too_long);
    assert!(
        result.is_err(),
        "DEP-004: VK bytes with wrong length must fail validation"
    );
}

// ── validate_vk_bytes fails for empty input ─────────────────────────

#[test]
fn vv_req_dep_004_validate_bytes_empty() {
    let result = validate_vk_bytes(&[]);
    assert!(
        result.is_err(),
        "DEP-004: Empty VK bytes must fail validation"
    );
}

// ── validate_vk_bytes checks VK_BYTE_SIZE = 672 ────────────────────

#[test]
fn vv_req_dep_004_vk_byte_size_is_672() {
    assert_eq!(
        VK_BYTE_SIZE, 672,
        "DEP-004: VK_BYTE_SIZE must be 672 (48+96+96+96+7*48)"
    );
}

// ── Components extracted from bytes match original ──────────────────

#[test]
fn vv_req_dep_004_components_from_bytes_match() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let original = extract_vk_components(&pk.vk).expect("original components");
    let vk_bytes = vk_to_bytes(&pk.vk).expect("vk_to_bytes");
    let extracted =
        chia_l2_consensus::testing::extract_vk_components_from_bytes(&vk_bytes).expect("extracted");

    assert_eq!(
        original.alpha_g1, extracted.alpha_g1,
        "DEP-004: alpha_g1 must match"
    );
    assert_eq!(
        original.beta_g2, extracted.beta_g2,
        "DEP-004: beta_g2 must match"
    );
    assert_eq!(
        original.gamma_g2, extracted.gamma_g2,
        "DEP-004: gamma_g2 must match"
    );
    assert_eq!(
        original.delta_g2, extracted.delta_g2,
        "DEP-004: delta_g2 must match"
    );
    assert_eq!(
        original.ic_points.len(),
        extracted.ic_points.len(),
        "DEP-004: IC count must match"
    );
    for (i, (orig, ext)) in original
        .ic_points
        .iter()
        .zip(extracted.ic_points.iter())
        .enumerate()
    {
        assert_eq!(orig, ext, "DEP-004: IC point {} must match", i);
    }
}

// ── Full verification workflow: setup → serialize → verify ──────────

#[test]
fn vv_req_dep_004_full_verification_workflow() {
    // Simulate operator workflow:
    // 1. Run trusted setup
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // 2. Publish VK hash
    let published_hash = compute_vk_hash(&pk.vk).expect("hash");

    // 3. Serialize VK (as it would be curried into singleton)
    let deployed_vk_bytes = vk_to_bytes(&pk.vk).expect("serialize");

    // 4. Operator verifies: extract from deployed singleton (simulated)
    assert!(
        verify_vk_hash(&deployed_vk_bytes, &published_hash),
        "DEP-004: Deployed VK hash must match published hash"
    );
    let result = validate_vk_bytes(&deployed_vk_bytes);
    assert!(
        result.is_ok(),
        "DEP-004: Deployed VK must pass structure validation"
    );

    // 5. Extract components and verify IC count
    let components =
        chia_l2_consensus::testing::extract_vk_components_from_bytes(&deployed_vk_bytes)
            .expect("components");
    assert_eq!(
        components.ic_points.len(),
        7,
        "DEP-004: Deployed VK must have 7 IC points"
    );
}

// ── Wrong VK from different setup is detected ───────────────────────

#[test]
fn vv_req_dep_004_different_setup_detected() {
    // Two setups produce different VKs (different random seeds via the prover)
    let (pk_bytes_1, _) = run_test_setup().expect("Setup 1");
    let pk1 = deserialize_proving_key(&pk_bytes_1).expect("PK1");
    let hash1 = compute_vk_hash(&pk1.vk).expect("hash1");
    let bytes1 = vk_to_bytes(&pk1.vk).expect("bytes1");

    // The test setup is deterministic (seed=42), so re-running gives same result.
    // To test mismatch, fabricate a different VK by flipping a byte.
    let mut fake_vk = bytes1.clone();
    fake_vk[0] ^= 0x01;

    assert!(
        !verify_vk_hash(&fake_vk, &hash1),
        "DEP-004: Modified VK must not match original hash"
    );
}
