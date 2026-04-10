//! Majority threshold verification for the Groth16 circuit.
//!
//! See [spec-groth16-circuit.md](../../docs/resources/spec-groth16-circuit.md).
//!
//! Implements CIR-004: Majority threshold constraint.
//! The circuit enforces that 2k > validator_count where k is the number of signers.

/// Check if k signers form a strict majority of validator_count validators.
///
/// Returns true if 2k > validator_count.
///
/// This is the off-chain equivalent of the CIR-004 circuit constraint.
/// The circuit enforces this constraint to prevent minority attacks.
///
/// # Arguments
///
/// * `k` - Number of signers
/// * `validator_count` - Total number of validators
///
/// # Examples
///
/// ```
/// use chia_l2_consensus::is_majority;
///
/// assert!(is_majority(51, 100));  // 102 > 100
/// assert!(!is_majority(50, 100)); // 100 > 100 is false
/// assert!(is_majority(1, 1));     // 2 > 1
/// ```
///
/// Source: spec-groth16-circuit.md Lines 325-357 (Constraint 3: Majority threshold)
pub fn is_majority(k: u64, validator_count: u64) -> bool {
    // 2k > validator_count
    // To avoid overflow, we check: k > validator_count / 2
    // But we need strict majority, so: 2k > validator_count
    // This is equivalent to: k > validator_count / 2 (with proper rounding)
    //
    // Safe implementation using checked arithmetic to avoid overflow:
    // 2k > validator_count is equivalent to k > (validator_count - 1) / 2
    // when validator_count > 0, or k >= 1 when validator_count == 0

    // Use u128 to avoid overflow when computing 2*k
    let two_k = (k as u128) * 2;
    two_k > (validator_count as u128)
}

/// Compute the minimum number of signers needed for strict majority.
///
/// Returns the smallest k such that 2k > validator_count.
///
/// # Arguments
///
/// * `validator_count` - Total number of validators
///
/// # Examples
///
/// ```
/// use chia_l2_consensus::minimum_signers;
///
/// assert_eq!(minimum_signers(100), 51);
/// assert_eq!(minimum_signers(99), 50);
/// assert_eq!(minimum_signers(1), 1);
/// assert_eq!(minimum_signers(0), 1); // At least one signer needed
/// ```
///
/// Source: spec-groth16-circuit.md Lines 325-357
pub fn minimum_signers(validator_count: u64) -> u64 {
    // We need smallest k such that 2k > validator_count
    // 2k > validator_count
    // k > validator_count / 2
    // k >= floor(validator_count / 2) + 1
    //
    // Equivalently: k = (validator_count / 2) + 1 if even
    //               k = (validator_count + 1) / 2 if odd
    //
    // Simplest formula: k = (validator_count / 2) + 1

    if validator_count == 0 {
        // Edge case: at least one signer is needed
        return 1;
    }

    (validator_count / 2) + 1
}

/// Check if k signers form at least half (non-strict majority) of validator_count.
///
/// Returns true if 2k >= validator_count.
///
/// Note: The circuit uses strict majority (>), not this. This function is
/// provided for comparison and edge case analysis.
pub fn is_at_least_half(k: u64, validator_count: u64) -> bool {
    let two_k = (k as u128) * 2;
    two_k >= (validator_count as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_majority_basic() {
        assert!(is_majority(51, 100));
        assert!(!is_majority(50, 100));
    }

    #[test]
    fn test_minimum_signers_basic() {
        assert_eq!(minimum_signers(100), 51);
        assert_eq!(minimum_signers(99), 50);
    }

    #[test]
    fn test_edge_cases() {
        assert!(is_majority(1, 1));
        assert!(is_majority(1, 0));
        assert!(!is_majority(0, 1));
    }
}
