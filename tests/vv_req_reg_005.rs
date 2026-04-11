//! REQUIREMENT: REG-005 — Collateral Return
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-005`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-005.md`.
//!
//! Implementation: `puzzles/registration_coin.rue` (compiled to CLVM).
//!
//! ## Normative statement
//! Upon valid spend, the registration coin MUST emit exactly one CREATE_COIN
//! condition with `puzzle_hash = collateral_destination` and `amount =
//! collateral_amount`, both taken from the solution. This returns the locked
//! collateral to the address specified by the spender.
//!
//! ## How the tests prove the requirement
//! 1. **Destination correctness**: CLVM execution extracts the CREATE_COIN
//!    puzzle_hash and asserts it matches the solution's destination.
//! 2. **Amount correctness**: CLVM execution extracts the CREATE_COIN amount
//!    and asserts it matches the solution's collateral_amount for small (1),
//!    medium (1 XCH), and large (10 XCH) values.
//! 3. **Exactly one CREATE_COIN**: Counts CREATE_COIN conditions in the
//!    output and asserts exactly 1, ruling out hidden outputs.
//! 4. **Independence from curried params**: Same destination regardless of
//!    pubkey; same amount regardless of epoch. Confirms the CREATE_COIN
//!    depends only on solution fields.
//!
//! ## Completeness: HIGH
//! Covers destination correctness, amount correctness, exact condition count,
//! and independence. Boundary values exercised.
//!
//! ## Gaps
//! - Does not test zero-amount collateral (may be disallowed by consensus).
//! - End-to-end simulator test covered by REG-007.

mod common;

use clvmr::Allocator;
use common::clvm::*;

const REG_COIN_HEX: &str = include_str!("../puzzles/compiled/registration_coin.hex");

fn parse_u64(bytes: &[u8]) -> u64 {
    if bytes.is_empty() {
        return 0;
    }
    let n = bytes.len().min(8);
    let mut padded = vec![0u8; 8 - n];
    padded.extend_from_slice(&bytes[..n]);
    u64::from_be_bytes(padded.try_into().unwrap())
}

fn run_and_get_create_coin(
    pk: &[u8],
    ckpt: &[u8],
    epoch: u64,
    dest: &[u8],
    amt: u64,
) -> (Vec<u8>, u64) {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_reg_coin_env(&mut a, pk, ckpt, epoch, dest, amt);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conds = parse_conditions(&a, output);
    let ccs = conditions_with_opcode(&conds, CREATE_COIN);
    let dest_out = ccs[0].args[0].clone();
    let amt_out = parse_u64(&ccs[0].args[1]);
    (dest_out, amt_out)
}

// ── CREATE_COIN destination ────────────────────────────────────────

/// Verifies the CREATE_COIN puzzle_hash equals the solution's destination.
/// Runs the compiled puzzle and extracts the first CREATE_COIN condition.
/// Passing proves the collateral goes to the address the spender specified.
#[test]
fn vv_req_reg_005_create_coin_has_correct_destination() {
    let dest = [0xDD; 32];
    let (d, _) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &dest, 1_000_000);
    assert_eq!(
        d,
        dest.to_vec(),
        "REG-005: CREATE_COIN puzzle_hash must match destination"
    );
}

/// Verifies that different destinations in the solution produce different
/// CREATE_COIN outputs. Proves the puzzle passes the destination through
/// faithfully rather than using a hardcoded value.
#[test]
fn vv_req_reg_005_destination_changes_with_solution() {
    let (d1, _) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0x11; 32], 1000);
    let (d2, _) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0x22; 32], 1000);
    assert_ne!(
        d1, d2,
        "REG-005: Different destinations produce different CREATE_COIN"
    );
}

// ── CREATE_COIN amount ─────────────────────────────────────────────

/// Verifies the CREATE_COIN amount equals the solution's collateral_amount
/// for a 1 XCH value. Proves the puzzle does not modify, cap, or truncate
/// the amount.
#[test]
fn vv_req_reg_005_create_coin_has_correct_amount() {
    let (_, a) =
        run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 1_000_000_000_000);
    assert_eq!(a, 1_000_000_000_000, "REG-005: Amount must be 1 XCH");
}

/// Boundary: smallest non-zero amount (1 mojo). Proves the puzzle handles
/// single-byte CLVM atoms for small values.
#[test]
fn vv_req_reg_005_small_amount() {
    let (_, a) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 1);
    assert_eq!(a, 1, "REG-005: 1 mojo must work");
}

/// Boundary: large amount (10 XCH = 10 trillion mojos). Proves the puzzle
/// handles multi-byte CLVM atoms for large values without truncation.
#[test]
fn vv_req_reg_005_large_amount() {
    let (_, a) =
        run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 10_000_000_000_000);
    assert_eq!(a, 10_000_000_000_000, "REG-005: 10 XCH must work");
}

// ── Exactly one CREATE_COIN ────────────────────────────────────────

/// Verifies the puzzle emits exactly 1 CREATE_COIN condition, ruling out
/// hidden extra outputs that could redirect collateral. This is a critical
/// security property: no surplus coins can be created.
#[test]
fn vv_req_reg_005_exactly_one_create_coin() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_reg_coin_env(&mut a, &[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 1_000_000);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conds = parse_conditions(&a, output);
    assert_eq!(
        conditions_with_opcode(&conds, CREATE_COIN).len(),
        1,
        "REG-005: Exactly 1 CREATE_COIN"
    );
}

// ── Independence from curried params ───────────────────────────────

/// Independence test: same destination regardless of which pubkey is
/// curried. Proves the CREATE_COIN destination comes from the solution,
/// not from curried parameters.
#[test]
fn vv_req_reg_005_destination_independent_of_pubkey() {
    let dest = [0xCC; 32];
    let (d1, _) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &dest, 1000);
    let (d2, _) = run_and_get_create_coin(&[0x11; 48], &[0xBB; 32], 1, &dest, 1000);
    assert_eq!(d1, d2, "REG-005: Same destination regardless of pubkey");
}

/// Independence test: same amount regardless of epoch. Proves the
/// CREATE_COIN amount comes from the solution, not from the epoch.
#[test]
fn vv_req_reg_005_amount_independent_of_epoch() {
    let (_, a1) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 5_000_000);
    let (_, a2) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 999, &[0xCC; 32], 5_000_000);
    assert_eq!(a1, a2, "REG-005: Same amount regardless of epoch");
}

// ── Spec ───────────────────────────────────────────────────────────

/// Traceability: confirms the REG-005 spec file exists.
#[test]
fn vv_req_reg_005_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/registration_coin/specs/REG-005.md")
            .exists()
    );
}

/// Structural check: the Rue source mentions collateral return via
/// REG-005 reference, "collateral" keyword, or CreateCoin usage.
#[test]
fn vv_req_reg_005_puzzle_documents_collateral_return() {
    let src = std::fs::read_to_string("puzzles/registration_coin.rue").unwrap();
    assert!(src.contains("REG-005") || src.contains("collateral") || src.contains("CreateCoin"));
}
