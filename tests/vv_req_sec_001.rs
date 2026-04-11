//! REQUIREMENT: SEC-001 — Majority Assumption
//! (`docs/requirements/domains/security/NORMATIVE.md#SEC-001`).
//!
//! Spec: `docs/requirements/domains/security/specs/SEC-001.md`.
//!
//! ## Normative statement
//! A checkpoint is valid only if signed by a strict majority of validators:
//! `2k > validator_count` (not `>=`). This must hold at all layers: the
//! `is_majority()` helper, the `minimum_signers()` function, and the Groth16
//! circuit constraint.
//!
//! ## How the tests prove the requirement
//! 1. **Strict majority (not GTE)**: 50/100 is NOT majority; 51/100 IS.
//! 2. **Odd counts**: 49/99 not majority; 50/99 is. 1/1 is.
//! 3. **Small counts**: Boundary values 0-3 tested exhaustively.
//! 4. **minimum_signers correctness**: For n=0..200, verifies min_k is the
//!    smallest k where is_majority(k, n) is true.
//! 5. **Large counts**: 10000/20000 not majority; 10001/20000 is.
//! 6. **No overflow**: Near u64::MAX values do not panic or overflow.
//! 7. **Circuit rejects minority**: Groth16 circuit panics (unsatisfied
//!    constraints) for 2/5 signers.
//! 8. **Circuit accepts majority**: Groth16 circuit produces valid proof
//!    for 3/5 signers.
//! 9. **Circuit rejects exact half**: 3/6 rejected (strict majority).
//!
//! ## Completeness: HIGH
//! ## Gaps: None significant -- all layers tested.

use chia_l2_consensus::testing::{
    deserialize_proving_key, generate_proof, is_majority, minimum_signers, run_test_setup,
    ConsensusCircuit,
};

// ── Strict majority: 2k > n (not >=) ───────────────────────────────

/// Core property: strict majority means 2k > n, NOT 2k >= n. 50/100 fails
/// (2*50 = 100, 100 > 100 is FALSE), 51/100 passes (2*51 = 102 > 100).
#[test]
fn vv_req_sec_001_strict_majority_not_gte() {
    // 2*50 = 100, 100 > 100 is FALSE → strict majority requires >50%
    assert!(
        !is_majority(50, 100),
        "SEC-001: Exactly half (50/100) must NOT be majority (strict >)"
    );

    // 2*51 = 102, 102 > 100 is TRUE
    assert!(is_majority(51, 100), "SEC-001: 51/100 must be majority");
}

// ── Boundary: odd validator counts ──────────────────────────────────

#[test]
fn vv_req_sec_001_odd_count_boundary() {
    // n=99: need 2k > 99, so k >= 50
    assert!(!is_majority(49, 99), "SEC-001: 49/99 not majority");
    assert!(is_majority(50, 99), "SEC-001: 50/99 is majority");

    // n=1: need 2k > 1, so k >= 1
    assert!(is_majority(1, 1), "SEC-001: 1/1 is majority");
    assert!(!is_majority(0, 1), "SEC-001: 0/1 not majority");
}

// ── Boundary: small counts ──────────────────────────────────────────

#[test]
fn vv_req_sec_001_small_counts() {
    assert!(is_majority(1, 0), "SEC-001: 1/0 is majority (degenerate)");
    assert!(is_majority(1, 1), "SEC-001: 1/1 is majority");
    assert!(is_majority(2, 2), "SEC-001: 2/2 is majority");
    assert!(is_majority(2, 3), "SEC-001: 2/3 is majority");
    assert!(!is_majority(1, 2), "SEC-001: 1/2 not majority");
    assert!(!is_majority(1, 3), "SEC-001: 1/3 not majority");
}

// ── minimum_signers computes correct threshold ──────────────────────

/// Exhaustive verification: for every n from 0 to 200, minimum_signers(n)
/// is the smallest k where is_majority(k, n) is true, and k-1 is NOT.
/// This proves the function computes the exact threshold.
#[test]
fn vv_req_sec_001_minimum_signers_correctness() {
    // For every count from 0 to 200, verify minimum_signers is the
    // smallest k where is_majority(k, n) is true.
    for n in 0..=200u64 {
        let min_k = minimum_signers(n);

        // min_k must be majority
        assert!(
            is_majority(min_k, n),
            "SEC-001: minimum_signers({}) = {} but is_majority fails",
            n,
            min_k
        );

        // min_k - 1 must NOT be majority (unless min_k is 1 and n is 0)
        if min_k > 1 {
            assert!(
                !is_majority(min_k - 1, n),
                "SEC-001: minimum_signers({}) = {} but {} also passes — not minimal!",
                n,
                min_k,
                min_k - 1
            );
        }
    }
}

// ── Large validator counts ──────────────────────────────────────────

#[test]
fn vv_req_sec_001_large_counts() {
    assert!(
        is_majority(10_001, 20_000),
        "SEC-001: 10001/20000 is majority"
    );
    assert!(
        !is_majority(10_000, 20_000),
        "SEC-001: 10000/20000 not majority"
    );

    assert_eq!(minimum_signers(20_000), 10_001);
    assert_eq!(minimum_signers(19_999), 10_000);
}

// ── No overflow at u64 max ──────────────────────────────────────────

#[test]
fn vv_req_sec_001_no_overflow() {
    // Near u64::MAX — must not panic or overflow
    let big = u64::MAX / 2;
    let result = is_majority(big, u64::MAX - 1);
    // 2 * (u64::MAX/2) = u64::MAX - 1 (due to truncation)
    // u64::MAX - 1 > u64::MAX - 1 → false
    assert!(!result, "SEC-001: Boundary near u64::MAX must not overflow");

    let result2 = is_majority(big + 1, u64::MAX - 1);
    assert!(result2, "SEC-001: big+1 should be majority of MAX-1");
}

// ── Circuit rejects minority signers ────────────────────────────────

/// Circuit-level enforcement: Groth16 circuit panics (unsatisfied constraints)
/// when only 2/5 validators sign (minority). The catch_unwind confirms
/// the circuit enforces the majority constraint, not just the Rust helper.
#[test]
fn vv_req_sec_001_circuit_rejects_minority() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // Minority: k=2, n=5 → 2*2=4, 4 > 5 is FALSE
    // Arkworks panics when constraints are unsatisfied rather than returning Err
    let result = std::panic::catch_unwind(|| {
        let circuit = ConsensusCircuit::with_public_inputs(
            [0xAA; 32], 5, [0xBB; 32], 5, [0xCC; 48], [0xDD; 32],
            2, // actual_signers = 2 (minority of 5)
        );
        generate_proof(circuit, &pk)
    });

    assert!(
        result.is_err(),
        "SEC-001: Circuit must reject minority signers (2/5) — constraint unsatisfied"
    );
}

// ── Circuit accepts majority signers ────────────────────────────────

/// Circuit-level acceptance: Groth16 circuit produces a valid proof when
/// 3/5 validators sign (majority). This proves the circuit constraint
/// is satisfiable for legitimate majority inputs.
#[test]
fn vv_req_sec_001_circuit_accepts_majority() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // Majority: k=3, n=5 → 2*3=6, 6 > 5 is TRUE
    let circuit = ConsensusCircuit::with_public_inputs(
        [0xAA; 32], 5, [0xBB; 32], 5, [0xCC; 48], [0xDD; 32],
        3, // actual_signers = 3 (majority of 5)
    );

    let result = generate_proof(circuit, &pk);
    assert!(
        result.is_ok(),
        "SEC-001: Circuit must accept majority signers (3/5): {:?}",
        result.err()
    );
}

// ── Circuit rejects exact half ──────────────────────────────────────

#[test]
fn vv_req_sec_001_circuit_rejects_exact_half() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // Exactly half: k=3, n=6 → 2*3=6, 6 > 6 is FALSE
    let result = std::panic::catch_unwind(|| {
        let circuit = ConsensusCircuit::with_public_inputs(
            [0xAA; 32], 6, [0xBB; 32], 6, [0xCC; 48], [0xDD; 32],
            3, // actual_signers = 3 (exactly half of 6)
        );
        generate_proof(circuit, &pk)
    });

    assert!(
        result.is_err(),
        "SEC-001: Circuit must reject exactly half (3/6) — strict majority required"
    );
}
