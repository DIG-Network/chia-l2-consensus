//! REQUIREMENT: REG-005 — Collateral Return
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-005`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-005.md`.
//!
//! Implementation: `puzzles/registration_coin.rue` (compiled to CLVM).
//!
//! Verifies that upon valid spend, the registration coin creates a coin at
//! the specified collateral_destination with the full collateral_amount.

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

#[test]
fn vv_req_reg_005_create_coin_has_correct_amount() {
    let (_, a) =
        run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 1_000_000_000_000);
    assert_eq!(a, 1_000_000_000_000, "REG-005: Amount must be 1 XCH");
}

#[test]
fn vv_req_reg_005_small_amount() {
    let (_, a) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 1);
    assert_eq!(a, 1, "REG-005: 1 mojo must work");
}

#[test]
fn vv_req_reg_005_large_amount() {
    let (_, a) =
        run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 10_000_000_000_000);
    assert_eq!(a, 10_000_000_000_000, "REG-005: 10 XCH must work");
}

// ── Exactly one CREATE_COIN ────────────────────────────────────────

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

#[test]
fn vv_req_reg_005_destination_independent_of_pubkey() {
    let dest = [0xCC; 32];
    let (d1, _) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &dest, 1000);
    let (d2, _) = run_and_get_create_coin(&[0x11; 48], &[0xBB; 32], 1, &dest, 1000);
    assert_eq!(d1, d2, "REG-005: Same destination regardless of pubkey");
}

#[test]
fn vv_req_reg_005_amount_independent_of_epoch() {
    let (_, a1) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 5_000_000);
    let (_, a2) = run_and_get_create_coin(&[0xAA; 48], &[0xBB; 32], 999, &[0xCC; 32], 5_000_000);
    assert_eq!(a1, a2, "REG-005: Same amount regardless of epoch");
}

// ── Spec ───────────────────────────────────────────────────────────

#[test]
fn vv_req_reg_005_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/registration_coin/specs/REG-005.md")
            .exists()
    );
}

#[test]
fn vv_req_reg_005_puzzle_documents_collateral_return() {
    let src = std::fs::read_to_string("puzzles/registration_coin.rue").unwrap();
    assert!(src.contains("REG-005") || src.contains("collateral") || src.contains("CreateCoin"));
}
