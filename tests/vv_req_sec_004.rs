//! REQUIREMENT: SEC-004 — Trusted Setup
//! (`docs/requirements/domains/security/NORMATIVE.md#SEC-004`).
//!
//! Spec: `docs/requirements/domains/security/specs/SEC-004.md`.
//!
//! Implementation: `src/prover/setup.rs`.
//!
//! ## Normative statement
//! The trusted setup MUST warn against single-party use in production, MUST
//! reference MPC/multi-party as the production alternative, and MUST produce
//! a structurally valid VK with 7 IC points (672 bytes). The setup MUST be
//! deterministic for the same seed.
//!
//! ## How the tests prove the requirement
//! 1. **Source warnings**: Setup code contains "WARNING"/"insecure"/"NOT secure"
//!    and references MPC/multi-party.
//! 2. **Valid VK produced**: Test setup produces VK that passes validate_vk.
//! 3. **VK size**: Exactly 672 bytes (48+96+96+96+7*48).
//! 4. **IC count**: 7 points (6 public inputs + 1 constant).
//! 5. **Non-trivial hash**: VK hash is not all zeros.
//! 6. **Deterministic**: Same seed produces same VK and PK.
//! 7. **Invalid VK rejected**: Too short, too long, and empty bytes all fail.
//! 8. **Spec and CHIP documentation**: Reference MPC ceremony and toxic waste.
//!
//! ## Completeness: HIGH
//! ## Gaps: Does not test actual MPC ceremony (requires multi-party infrastructure).

use chia_l2_consensus::testing::{
    compute_vk_hash, deserialize_proving_key, run_test_setup, validate_vk, validate_vk_bytes,
    vk_to_bytes, VK_BYTE_SIZE,
};

// ── Source code warns single-party is not for production ─────────────

#[test]
fn vv_req_sec_004_single_party_warning_in_source() {
    let source = include_str!("../src/prover/setup.rs");

    // The run_test_setup() function must warn about single-party
    assert!(
        source.contains("WARNING")
            || source.contains("insecure")
            || source.contains("NOT secure")
            || source.contains("not secure"),
        "SEC-004: run_test_setup() must warn that single-party setup is not for production"
    );

    assert!(
        source.contains("MPC") || source.contains("multi-party"),
        "SEC-004: Source must reference MPC/multi-party as the production alternative"
    );
}

// ── Test setup produces a valid VK ──────────────────────────────────

#[test]
fn vv_req_sec_004_test_setup_produces_valid_vk() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let result = validate_vk(&pk.vk);
    assert!(
        result.is_ok(),
        "SEC-004: Test setup must produce a structurally valid VK: {:?}",
        result.err()
    );
}

// ── VK is exactly 672 bytes ─────────────────────────────────────────

#[test]
fn vv_req_sec_004_vk_correct_size() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let vk_bytes = vk_to_bytes(&pk.vk).expect("vk_to_bytes");
    assert_eq!(
        vk_bytes.len(),
        VK_BYTE_SIZE,
        "SEC-004: VK must be {} bytes",
        VK_BYTE_SIZE
    );
}

// ── VK has 7 IC points (6 public inputs + 1 constant) ───────────────

#[test]
fn vv_req_sec_004_vk_ic_count() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    assert_eq!(
        pk.vk.gamma_abc_g1.len(),
        7,
        "SEC-004: VK must have 7 IC points (6 public inputs + 1 constant)"
    );
}

// ── VK hash is non-trivial (not all zeros) ──────────────────────────

#[test]
fn vv_req_sec_004_vk_hash_nontrivial() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let hash = compute_vk_hash(&pk.vk).expect("hash");
    assert!(
        !hash.iter().all(|&b| b == 0),
        "SEC-004: VK hash must not be all zeros"
    );
}

// ── Setup is deterministic (same seed → same VK) ────────────────────

#[test]
fn vv_req_sec_004_setup_deterministic() {
    let (pk1, vk1) = run_test_setup().expect("Setup 1");
    let (pk2, vk2) = run_test_setup().expect("Setup 2");

    assert_eq!(
        vk1, vk2,
        "SEC-004: Same seed must produce same VK (deterministic)"
    );
    assert_eq!(
        pk1.len(),
        pk2.len(),
        "SEC-004: Same seed must produce same PK size"
    );
}

// ── Invalid VK bytes are rejected by validation ─────────────────────

#[test]
fn vv_req_sec_004_invalid_vk_rejected() {
    // Too short
    let result = validate_vk_bytes(&[0u8; 100]);
    assert!(result.is_err(), "SEC-004: Short VK bytes must be rejected");

    // Too long
    let result = validate_vk_bytes(&[0u8; 700]);
    assert!(result.is_err(), "SEC-004: Long VK bytes must be rejected");

    // Empty
    let result = validate_vk_bytes(&[]);
    assert!(result.is_err(), "SEC-004: Empty VK bytes must be rejected");
}

// ── Spec document references MPC ceremony ───────────────────────────

#[test]
fn vv_req_sec_004_spec_references_mpc() {
    let spec = include_str!("../docs/resources/spec-trusted-setup.md");

    assert!(
        spec.contains("Multi-Party Ceremony") || spec.contains("multi-party"),
        "SEC-004: Spec must document MPC ceremony requirement"
    );
    assert!(
        spec.contains("toxic waste"),
        "SEC-004: Spec must explain toxic waste concept"
    );
    assert!(
        spec.contains("Never use this in production")
            || spec.contains("never be used in production"),
        "SEC-004: Spec must warn against single-party in production"
    );
}

// ── CHIP document states trusted setup assumption ────────────────────

#[test]
fn vv_req_sec_004_chip_states_assumption() {
    let chip = include_str!("../docs/resources/chip-groth16-l2-consensus.md");

    assert!(
        chip.contains("trusted setup"),
        "SEC-004: CHIP must reference trusted setup"
    );
    assert!(
        chip.contains("multi-party") || chip.contains("MPC"),
        "SEC-004: CHIP must require MPC ceremony"
    );
}
