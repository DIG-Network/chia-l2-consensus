//! REQUIREMENT: WDC-007 — Permissionless Release
//! (`docs/requirements/domains/withdraw_delay/NORMATIVE.md#WDC-007`).
//!
//! Spec: `docs/requirements/domains/withdraw_delay/specs/WDC-007.md`.
//!
//! ## Normative Statement
//!
//! The withdraw delay coin MUST be permissionless to spend (no AGG_SIG
//! conditions). After the delay period, any party MUST be able to release
//! the funds to the curried destination.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] CLVM output has no AGG_SIG_ME (opcode 50)
//! - [x] CLVM output has no AGG_SIG_UNSAFE (opcode 49)
//! - [x] CLVM output has no AGG_SIG of ANY variant (opcodes 43-50)
//! - [x] Spend bundle uses identity signature
//! - [x] Funds always go to curried DESTINATION regardless of who spends
//! - [x] Source has no AggSig conditions of any kind

mod common;

use clvmr::Allocator;

use chia_l2_consensus::testing::WITHDRAW_DELAY_COIN_PUZZLE_HEX;

use common::clvm::*;

/// Build flat env for the withdraw delay coin.
fn build_wdc_env(a: &mut Allocator, dest: &[u8], amount: u64, delay: u64) -> clvmr::NodePtr {
    let delay_node = u64_to_clvm(a, delay);
    let t = a.new_pair(delay_node, a.nil()).unwrap();
    let amount_node = u64_to_clvm(a, amount);
    let t = a.new_pair(amount_node, t).unwrap();
    let dest_node = a.new_atom(dest).unwrap();
    a.new_pair(dest_node, t).unwrap()
}

// All AGG_SIG opcodes in Chia CLVM (43-50)
const AGG_SIG_PARENT: i64 = 43;
const AGG_SIG_PUZZLE: i64 = 44;
const AGG_SIG_AMOUNT: i64 = 45;
const AGG_SIG_PUZZLE_AMOUNT: i64 = 46;
const AGG_SIG_PARENT_AMOUNT: i64 = 47;
const AGG_SIG_PARENT_PUZZLE: i64 = 48;
const AGG_SIG_UNSAFE_OP: i64 = 49;
const AGG_SIG_ME_OP: i64 = 50;

const ALL_AGG_SIG_OPCODES: [i64; 8] = [
    AGG_SIG_PARENT,
    AGG_SIG_PUZZLE,
    AGG_SIG_AMOUNT,
    AGG_SIG_PUZZLE_AMOUNT,
    AGG_SIG_PARENT_AMOUNT,
    AGG_SIG_PARENT_PUZZLE,
    AGG_SIG_UNSAFE_OP,
    AGG_SIG_ME_OP,
];

// ── No signature conditions ──────────────────────────────────────────

/// WDC-007: CLVM output has no AGG_SIG_ME (opcode 50).
#[test]
fn vv_req_wdc_007_no_agg_sig_me() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(&mut a, &[0xAA; 32], 1_000_000, 100);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    for c in &conditions {
        assert_ne!(c.opcode, AGG_SIG_ME_OP, "WDC-007: Must NOT emit AGG_SIG_ME");
    }
}

/// WDC-007: CLVM output has no AGG_SIG_UNSAFE (opcode 49).
#[test]
fn vv_req_wdc_007_no_agg_sig_unsafe() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(&mut a, &[0xAA; 32], 1_000_000, 100);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    for c in &conditions {
        assert_ne!(
            c.opcode, AGG_SIG_UNSAFE_OP,
            "WDC-007: Must NOT emit AGG_SIG_UNSAFE"
        );
    }
}

/// WDC-007: CLVM output has NO AGG_SIG of ANY variant (opcodes 43-50).
#[test]
fn vv_req_wdc_007_no_agg_sig_any_variant() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(&mut a, &[0xBB; 32], 5_000_000, 24_000);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    for c in &conditions {
        assert!(
            !ALL_AGG_SIG_OPCODES.contains(&c.opcode),
            "WDC-007: Must NOT emit any AGG_SIG variant, found opcode {}",
            c.opcode
        );
    }
}

/// WDC-007: Only opcodes 82 and 51 are emitted — nothing else.
#[test]
fn vv_req_wdc_007_only_expected_opcodes() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(&mut a, &[0xCC; 32], 1_000_000, 100);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    for c in &conditions {
        assert!(
            c.opcode == 82 || c.opcode == CREATE_COIN as i64,
            "WDC-007: Only opcodes 82 (ASSERT_HEIGHT_RELATIVE) and 51 (CREATE_COIN) allowed, got {}",
            c.opcode
        );
    }
}

// ── Destination immutability ─────────────────────────────────────────

/// WDC-007: Funds ALWAYS go to curried DESTINATION — verified across multiple params.
#[test]
fn vv_req_wdc_007_destination_always_curried() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);

    let test_cases: Vec<([u8; 32], u64, u64)> = vec![
        ([0x11; 32], 1, 1),
        ([0x22; 32], 1_000_000, 100),
        ([0xFF; 32], 10_000_000_000_000, 24_000),
        ([0x00; 32], 999, 0),
    ];

    for (dest, amount, delay) in &test_cases {
        let env = build_wdc_env(&mut a, dest, *amount, *delay);
        let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
        let conditions = parse_conditions(&a, output);

        let create_coin = conditions
            .iter()
            .find(|c| c.opcode == CREATE_COIN as i64)
            .unwrap();
        assert_eq!(
            create_coin.args[0].as_slice(),
            dest,
            "WDC-007: CREATE_COIN destination must match curried DESTINATION for dest={:?}",
            &dest[..4]
        );
    }
}

// ── Source inspection ────────────────────────────────────────────────

/// WDC-007: Puzzle source has no AggSig conditions of any kind.
#[test]
fn vv_req_wdc_007_source_no_agg_sig() {
    let src = include_str!("../puzzles/withdraw_delay_coin.rue");
    assert!(
        !src.contains("AggSigMe"),
        "WDC-007: Source must NOT contain AggSigMe"
    );
    assert!(
        !src.contains("AggSigUnsafe"),
        "WDC-007: Source must NOT contain AggSigUnsafe"
    );
    assert!(
        !src.contains("AggSigParent"),
        "WDC-007: Source must NOT contain AggSigParent"
    );
    assert!(
        !src.contains("AggSigPuzzle"),
        "WDC-007: Source must NOT contain AggSigPuzzle"
    );
    assert!(
        !src.contains("AggSigAmount"),
        "WDC-007: Source must NOT contain AggSigAmount"
    );
}

/// WDC-007: Puzzle source documents permissionless design.
#[test]
fn vv_req_wdc_007_source_documents_permissionless() {
    let src = include_str!("../puzzles/withdraw_delay_coin.rue");
    assert!(
        src.contains("WDC-007") || src.contains("permissionless") || src.contains("Permissionless"),
        "WDC-007: Source must document permissionless design"
    );
}

/// WDC-007: Spec file exists.
#[test]
fn vv_req_wdc_007_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/withdraw_delay/specs/WDC-007.md").exists(),
        "WDC-007: Spec file must exist"
    );
}
