//! REQUIREMENT: WDC-002 — Time Lock Enforcement
//! (`docs/requirements/domains/withdraw_delay/NORMATIVE.md#WDC-002`).
//!
//! Spec: `docs/requirements/domains/withdraw_delay/specs/WDC-002.md`.
//!
//! ## Normative Statement
//!
//! The withdraw delay coin MUST emit `ASSERT_HEIGHT_RELATIVE(WITHDRAW_DELAY_BLOCKS)`
//! to enforce that the configured number of L1 blocks have passed since coin
//! creation before funds can be released.
//!
//! ## How These Tests Prove the Requirement
//!
//! CLVM execution tests verify ASSERT_HEIGHT_RELATIVE (opcode 82) is emitted
//! with the exact delay value for various configurations: default 24,000
//! (~5 days), boundary values (0, 1, 2), large values, and different delays
//! producing distinct puzzle outputs. Cross-implementation verification
//! confirms the delay value encoding is consistent between Rust and CLVM.
//!
//! ## Simulator Note
//!
//! The chia-sdk-test Simulator (v0.18) does NOT enforce ASSERT_HEIGHT_RELATIVE
//! at the consensus level — it validates CLVM execution and signatures only.
//! Height-based conditions are enforced by the full node at block inclusion
//! time. Tests here verify the CONDITION IS EMITTED CORRECTLY; enforcement
//! is by the Chia network. This is the correct boundary: our puzzle emits
//! the right condition, Chia enforces it.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] CLVM output contains ASSERT_HEIGHT_RELATIVE (opcode 82)
//! - [x] Delay value matches curried WITHDRAW_DELAY_BLOCKS
//! - [x] Different delay values produce different CLVM outputs
//! - [x] Default 24,000-block delay correctly encoded
//! - [x] Boundary: delay=0 emits opcode 82 with value 0
//! - [x] Boundary: delay=1 emits opcode 82 with value 1
//! - [x] Large delay values correctly encoded (no truncation/overflow)
//! - [ ] Simulator rejects spend before delay (not enforceable — see note)
//! - [ ] Simulator accepts spend after delay (not enforceable — see note)

mod common;

use clvmr::Allocator;

use chia_l2_consensus::testing::{DEFAULT_WITHDRAW_DELAY_BLOCKS, WITHDRAW_DELAY_COIN_PUZZLE_HEX};

use common::clvm::*;

/// Build flat CLVM environment for the withdraw delay coin puzzle.
/// (DESTINATION . (AMOUNT . (WITHDRAW_DELAY_BLOCKS . nil)))
fn build_wdc_env(a: &mut Allocator, dest: &[u8], amount: u64, delay: u64) -> clvmr::NodePtr {
    let delay_node = u64_to_clvm(a, delay);
    let t = a.new_pair(delay_node, a.nil()).unwrap();
    let amount_node = u64_to_clvm(a, amount);
    let t = a.new_pair(amount_node, t).unwrap();
    let dest_node = a.new_atom(dest).unwrap();
    a.new_pair(dest_node, t).unwrap()
}

/// Parse a CLVM integer atom as u64 (big-endian, unsigned).
fn parse_clvm_u64(bytes: &[u8]) -> u64 {
    if bytes.is_empty() {
        return 0;
    }
    let mut padded = [0u8; 8];
    let start = 8usize.saturating_sub(bytes.len());
    let copy_len = bytes.len().min(8);
    padded[start..start + copy_len].copy_from_slice(&bytes[..copy_len]);
    u64::from_be_bytes(padded)
}

/// Helper: extract the ASSERT_HEIGHT_RELATIVE delay value from puzzle output.
fn extract_delay(a: &mut Allocator, delay: u64) -> u64 {
    let puzzle = load_puzzle(a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(a, &[0xAA; 32], 1_000_000, delay);
    let (_, output) = run_puzzle_ok(a, puzzle, env);
    let conditions = parse_conditions(a, output);
    assert_eq!(
        conditions[0].opcode, 82,
        "First condition must be ASSERT_HEIGHT_RELATIVE"
    );
    parse_clvm_u64(&conditions[0].args[0])
}

// ── Default delay (5 days / 24,000 blocks) ───────────────────────────

/// WDC-002: Default delay constant is 24,000 blocks.
#[test]
fn vv_req_wdc_002_default_delay_is_24000_blocks() {
    assert_eq!(DEFAULT_WITHDRAW_DELAY_BLOCKS, 24_000);
    // 24,000 blocks × 18 seconds/block = 432,000 seconds = 5 days
    let seconds = 24_000u64 * 18;
    let days = seconds / 86_400;
    assert_eq!(days, 5, "24,000 blocks at 18s/block must equal 5 days");
}

/// WDC-002: CLVM emits delay=24,000 when curried with default.
#[test]
fn vv_req_wdc_002_clvm_default_delay_value() {
    let mut a = Allocator::new();
    let parsed = extract_delay(&mut a, DEFAULT_WITHDRAW_DELAY_BLOCKS);
    assert_eq!(
        parsed, 24_000,
        "WDC-002: ASSERT_HEIGHT_RELATIVE must be 24,000 for default delay"
    );
}

// ── Boundary values ──────────────────────────────────────────────────

/// WDC-002: Delay=0 emits ASSERT_HEIGHT_RELATIVE(0) — immediate spend.
#[test]
fn vv_req_wdc_002_clvm_delay_zero() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(&mut a, &[0xAA; 32], 1_000_000, 0);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    assert_eq!(conditions[0].opcode, 82);
    // Delay 0 in CLVM: the arg may be empty bytes (nil = 0) or [0x00]
    let val = parse_clvm_u64(&conditions[0].args[0]);
    assert_eq!(val, 0, "WDC-002: Delay=0 must emit 0");
}

/// WDC-002: Delay=1 requires exactly 1 block to pass.
#[test]
fn vv_req_wdc_002_clvm_delay_one() {
    let mut a = Allocator::new();
    let parsed = extract_delay(&mut a, 1);
    assert_eq!(parsed, 1, "WDC-002: Delay=1 must emit value 1");
}

/// WDC-002: Delay=2 — smallest non-trivial delay.
#[test]
fn vv_req_wdc_002_clvm_delay_two() {
    let mut a = Allocator::new();
    let parsed = extract_delay(&mut a, 2);
    assert_eq!(parsed, 2, "WDC-002: Delay=2 must emit value 2");
}

// ── Various delay values ─────────────────────────────────────────────

/// WDC-002: Delay=10 — testnet fast delay.
#[test]
fn vv_req_wdc_002_clvm_delay_10() {
    let mut a = Allocator::new();
    let parsed = extract_delay(&mut a, 10);
    assert_eq!(parsed, 10);
}

/// WDC-002: Delay=256 — ~80 minutes.
#[test]
fn vv_req_wdc_002_clvm_delay_256() {
    let mut a = Allocator::new();
    let parsed = extract_delay(&mut a, 256);
    assert_eq!(parsed, 256);
}

/// WDC-002: Delay=4608 — ~24 hours.
#[test]
fn vv_req_wdc_002_clvm_delay_4608() {
    let mut a = Allocator::new();
    let parsed = extract_delay(&mut a, 4608);
    assert_eq!(parsed, 4608);
}

/// WDC-002: Large delay (100,000 blocks ≈ 20 days) — no truncation.
#[test]
fn vv_req_wdc_002_clvm_large_delay() {
    let mut a = Allocator::new();
    let parsed = extract_delay(&mut a, 100_000);
    assert_eq!(
        parsed, 100_000,
        "WDC-002: Large delay values must not be truncated"
    );
}

/// WDC-002: Very large delay (u32::MAX) — tests encoding limit.
#[test]
fn vv_req_wdc_002_clvm_u32_max_delay() {
    let mut a = Allocator::new();
    let delay = u32::MAX as u64;
    let parsed = extract_delay(&mut a, delay);
    assert_eq!(
        parsed, delay,
        "WDC-002: u32::MAX delay must be correctly encoded"
    );
}

// ── Different delays produce different outputs ───────────────────────

/// WDC-002: Different delay values produce different ASSERT_HEIGHT_RELATIVE args.
#[test]
fn vv_req_wdc_002_different_delays_different_conditions() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);

    let delays = [0u64, 1, 10, 100, 1000, 24_000, 100_000];
    let mut results = Vec::new();

    for &delay in &delays {
        let env = build_wdc_env(&mut a, &[0xAA; 32], 1_000_000, delay);
        let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
        let conditions = parse_conditions(&a, output);
        results.push(conditions[0].args[0].clone());
    }

    // All should be distinct
    for i in 0..results.len() {
        for j in (i + 1)..results.len() {
            assert_ne!(
                results[i], results[j],
                "WDC-002: Delay {} and {} must produce different conditions",
                delays[i], delays[j]
            );
        }
    }
}

/// WDC-002: Same delay always produces same ASSERT_HEIGHT_RELATIVE arg.
#[test]
fn vv_req_wdc_002_same_delay_deterministic() {
    let mut a = Allocator::new();
    let v1 = extract_delay(&mut a, 24_000);
    let v2 = extract_delay(&mut a, 24_000);
    assert_eq!(
        v1, v2,
        "WDC-002: Same delay must produce same condition value"
    );
}

// ── Condition structure ──────────────────────────────────────────────

/// WDC-002: ASSERT_HEIGHT_RELATIVE is always the FIRST condition emitted.
#[test]
fn vv_req_wdc_002_assert_height_is_first_condition() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(&mut a, &[0xBB; 32], 5_000_000, 24_000);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    assert_eq!(conditions.len(), 2, "Must be exactly 2 conditions");
    assert_eq!(
        conditions[0].opcode, 82,
        "WDC-002: First condition must be ASSERT_HEIGHT_RELATIVE (82)"
    );
    assert_eq!(
        conditions[1].opcode, CREATE_COIN as i64,
        "WDC-002: Second condition must be CREATE_COIN (51)"
    );
}

/// WDC-002: ASSERT_HEIGHT_RELATIVE has exactly 1 argument (the delay value).
#[test]
fn vv_req_wdc_002_assert_height_has_one_arg() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(&mut a, &[0xCC; 32], 1_000_000, 500);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    assert_eq!(
        conditions[0].args.len(),
        1,
        "WDC-002: ASSERT_HEIGHT_RELATIVE must have exactly 1 argument"
    );
}

/// WDC-002: The delay cannot be bypassed — no alternative code path exists.
/// Verified by confirming the puzzle always emits exactly 2 conditions
/// regardless of parameters.
#[test]
fn vv_req_wdc_002_no_bypass() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);

    // Try various parameter combinations — all must produce 2 conditions with opcode 82 first
    let test_cases: Vec<(&[u8], u64, u64)> = vec![
        (&[0x00; 32], 0, 0),
        (&[0xFF; 32], u64::MAX / 2, 1),
        (&[0x42; 32], 1, 24_000),
        (&[0x01; 32], 999_999_999, 100_000),
    ];

    for (dest, amount, delay) in test_cases {
        let env = build_wdc_env(&mut a, dest, amount, delay);
        let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
        let conditions = parse_conditions(&a, output);

        assert_eq!(
            conditions.len(),
            2,
            "WDC-002: Must always emit exactly 2 conditions"
        );
        assert_eq!(
            conditions[0].opcode, 82,
            "WDC-002: ASSERT_HEIGHT_RELATIVE must always be first (no bypass)"
        );
    }
}

/// WDC-002: Spec file exists.
#[test]
fn vv_req_wdc_002_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/withdraw_delay/specs/WDC-002.md").exists(),
        "WDC-002: Spec file must exist"
    );
}
