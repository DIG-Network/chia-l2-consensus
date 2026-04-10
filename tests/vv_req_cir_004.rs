//! REQUIREMENT: CIR-004 — Majority Threshold Constraint
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-004`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-004.md`.
//!
//! Verifies that the circuit enforces 2k > validator_count where k is the
//! number of signing validators. This prevents minority attacks.

use chia_l2_consensus::{is_at_least_half, is_majority, minimum_signers};

#[test]
fn vv_req_cir_004_majority_threshold_enforced() {
    // CIR-004: 2k > validator_count must be enforced
    // k=51, count=100: 2*51 = 102 > 100 ✓
    assert!(
        is_majority(51, 100),
        "CIR-004: k=51, count=100 must pass (102 > 100)"
    );

    // k=50, count=100: 2*50 = 100 > 100 is false
    assert!(
        !is_majority(50, 100),
        "CIR-004: k=50, count=100 must fail (100 > 100 is false)"
    );
}

#[test]
fn vv_req_cir_004_strict_majority_not_half() {
    // CIR-004: Must be strict majority (>), not half (>=)
    // This test ensures we don't accidentally use >= instead of >

    // At exactly half, is_majority should fail
    assert!(
        !is_majority(50, 100),
        "CIR-004: Exactly half must NOT be majority"
    );

    // But is_at_least_half should pass
    assert!(
        is_at_least_half(50, 100),
        "CIR-004: Exactly half is at_least_half"
    );

    // Strict majority requires one more
    assert!(is_majority(51, 100), "CIR-004: 51/100 is strict majority");
}

#[test]
fn vv_req_cir_004_boundary_51_100_passes() {
    // CIR-004: k=51, count=100 → passes (boundary test)
    assert!(is_majority(51, 100), "CIR-004: k=51, count=100 must pass");
    assert_eq!(
        minimum_signers(100),
        51,
        "CIR-004: minimum signers for 100 validators is 51"
    );
}

#[test]
fn vv_req_cir_004_boundary_50_100_fails() {
    // CIR-004: k=50, count=100 → fails (boundary test)
    assert!(!is_majority(50, 100), "CIR-004: k=50, count=100 must fail");
}

#[test]
fn vv_req_cir_004_edge_case_count_1_k_1() {
    // CIR-004: count=1, k=1 → passes (2 > 1)
    assert!(is_majority(1, 1), "CIR-004: k=1, count=1 must pass (2 > 1)");
    assert_eq!(
        minimum_signers(1),
        1,
        "CIR-004: minimum signers for 1 validator is 1"
    );
}

#[test]
fn vv_req_cir_004_edge_case_count_2() {
    // CIR-004: count=2, k=2 → passes (4 > 2)
    assert!(is_majority(2, 2), "CIR-004: k=2, count=2 must pass (4 > 2)");

    // count=2, k=1 → fails (2 > 2 is false)
    assert!(
        !is_majority(1, 2),
        "CIR-004: k=1, count=2 must fail (2 > 2 is false)"
    );

    assert_eq!(
        minimum_signers(2),
        2,
        "CIR-004: minimum signers for 2 validators is 2"
    );
}

#[test]
fn vv_req_cir_004_edge_case_count_3() {
    // CIR-004: count=3, k=2 → passes (4 > 3)
    assert!(is_majority(2, 3), "CIR-004: k=2, count=3 must pass (4 > 3)");

    // count=3, k=1 → fails (2 > 3 is false)
    assert!(
        !is_majority(1, 3),
        "CIR-004: k=1, count=3 must fail (2 > 3 is false)"
    );

    assert_eq!(
        minimum_signers(3),
        2,
        "CIR-004: minimum signers for 3 validators is 2"
    );
}

#[test]
fn vv_req_cir_004_edge_case_count_0() {
    // CIR-004: count=0 is degenerate but k=1 should technically pass
    // 2*1 = 2 > 0
    assert!(is_majority(1, 0), "CIR-004: k=1, count=0 passes (2 > 0)");

    // Even k=0 with count=0 should fail (0 > 0 is false)
    assert!(
        !is_majority(0, 0),
        "CIR-004: k=0, count=0 fails (0 > 0 is false)"
    );

    // Minimum signers for 0 should be at least 1
    assert_eq!(
        minimum_signers(0),
        1,
        "CIR-004: minimum signers for 0 validators is 1 (at least one signer needed)"
    );
}

#[test]
fn vv_req_cir_004_zero_signers_fails() {
    // CIR-004: k=0 (no signers) → fails for any positive validator count
    assert!(
        !is_majority(0, 1),
        "CIR-004: k=0, count=1 must fail (0 > 1 is false)"
    );
    assert!(!is_majority(0, 100), "CIR-004: k=0, count=100 must fail");
    assert!(
        !is_majority(0, 20000),
        "CIR-004: k=0, count=20000 must fail"
    );
}

#[test]
fn vv_req_cir_004_minimum_signers_formula() {
    // CIR-004: Verify minimum_signers formula
    // k = (validator_count / 2) + 1

    // Even counts
    assert_eq!(minimum_signers(100), 51); // 100/2 + 1 = 51
    assert_eq!(minimum_signers(200), 101); // 200/2 + 1 = 101
    assert_eq!(minimum_signers(1000), 501); // 1000/2 + 1 = 501

    // Odd counts
    assert_eq!(minimum_signers(99), 50); // 99/2 + 1 = 49 + 1 = 50
    assert_eq!(minimum_signers(101), 51); // 101/2 + 1 = 50 + 1 = 51
    assert_eq!(minimum_signers(999), 500); // 999/2 + 1 = 499 + 1 = 500
}

#[test]
fn vv_req_cir_004_minimum_signers_is_sufficient() {
    // CIR-004: minimum_signers result must pass is_majority
    for count in [1, 2, 3, 10, 50, 99, 100, 101, 1000, 10000, 20000] {
        let min_k = minimum_signers(count);
        assert!(
            is_majority(min_k, count),
            "CIR-004: minimum_signers({}) = {} must pass is_majority",
            count,
            min_k
        );

        // One less than minimum should fail (except for count=0 edge case)
        if min_k > 1 {
            assert!(
                !is_majority(min_k - 1, count),
                "CIR-004: minimum_signers({}) - 1 = {} must fail is_majority",
                count,
                min_k - 1
            );
        }
    }
}

#[test]
fn vv_req_cir_004_large_validator_counts() {
    // CIR-004: Test with large validator counts (up to MAX_SIGNERS = 20,000)
    let count = 20000u64;
    let min_k = minimum_signers(count);

    assert_eq!(min_k, 10001); // 20000/2 + 1 = 10001
    assert!(
        is_majority(min_k, count),
        "CIR-004: 10001/20000 must be majority"
    );
    assert!(
        !is_majority(min_k - 1, count),
        "CIR-004: 10000/20000 must not be majority"
    );
}

#[test]
fn vv_req_cir_004_overflow_protection() {
    // CIR-004: Test that large values don't cause overflow
    // Using u64::MAX / 2 to test near boundary
    let large_count = u64::MAX / 2;
    let large_k = large_count / 2 + 1;

    // Should not panic or overflow
    let result = is_majority(large_k, large_count);
    assert!(result, "CIR-004: Large values must not cause overflow");

    // Also test is_at_least_half
    let result2 = is_at_least_half(large_k, large_count);
    assert!(result2, "CIR-004: is_at_least_half with large values");
}

#[test]
fn vv_req_cir_004_table_from_spec() {
    // CIR-004: Verify the edge case table from the spec
    // | validator_count | Minimum k | 2k | Valid? |
    // |-----------------|-----------|-----|--------|
    // | 1 | 1 | 2 | ✓ (2 > 1) |
    // | 2 | 2 | 4 | ✓ (4 > 2) |
    // | 3 | 2 | 4 | ✓ (4 > 3) |
    // | 100 | 51 | 102 | ✓ (102 > 100) |
    // | 100 | 50 | 100 | ✗ (100 > 100 false) |

    assert!(is_majority(1, 1), "CIR-004: 2 > 1");
    assert!(is_majority(2, 2), "CIR-004: 4 > 2");
    assert!(is_majority(2, 3), "CIR-004: 4 > 3");
    assert!(is_majority(51, 100), "CIR-004: 102 > 100");
    assert!(!is_majority(50, 100), "CIR-004: 100 > 100 is false");
}

#[test]
fn vv_req_cir_004_all_signers_is_majority() {
    // CIR-004: If all validators sign, it's definitely a majority
    for count in [1, 10, 100, 1000, 20000] {
        assert!(
            is_majority(count, count),
            "CIR-004: All {} validators signing is majority",
            count
        );
    }
}

#[test]
fn vv_req_cir_004_more_than_all_signers() {
    // CIR-004: k > validator_count is technically possible (shouldn't happen in practice)
    // but should still count as majority
    assert!(
        is_majority(150, 100),
        "CIR-004: k > count should be majority"
    );
    assert!(
        is_majority(1000, 100),
        "CIR-004: k >> count should be majority"
    );
}
