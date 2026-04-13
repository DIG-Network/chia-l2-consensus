//! REQUIREMENT: WDC-010 — Destination Hint Memo
//! (`docs/requirements/domains/withdraw_delay/NORMATIVE.md#WDC-010`).
//!
//! Spec: `docs/requirements/domains/withdraw_delay/specs/WDC-010.md`.
//!
//! ## Normative Statement
//!
//! The registration coin puzzle MUST include sha256(CHECKPOINT_SINGLETON_ID +
//! collateral_destination) as a conflict-resistant memo (hint) on the CreateCoin
//! that creates the withdraw delay coin. The withdraw delay coin MUST include
//! "DIG Network Collateral Release" as a memo on its CreateCoin output.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Registration coin CreateCoin has conflict-resistant hint as memo
//! - [x] Memo is puzzle-enforced (in Rue source)
//! - [x] CLVM output includes the memo as 3rd element of CreateCoin
//! - [x] Withdraw delay coin CreateCoin has "DIG Network Collateral Release" memo
//! - [x] Both puzzles compile with memos
//! - [x] Registration coin hint matches Rust sha256(ckpt_id + dest)

mod common;

use clvmr::Allocator;
use sha2::{Digest, Sha256};

use chia_l2_consensus::testing::{REGISTRATION_COIN_PUZZLE_HEX, WITHDRAW_DELAY_COIN_PUZZLE_HEX};

use common::clvm::*;

/// Build registration coin env (4 curried + 3 solution).
fn build_reg_env(
    a: &mut Allocator,
    pk: &[u8],
    ckpt_id: &[u8],
    wdc_mod: &[u8],
    wdc_delay: u64,
    epoch: u64,
    dest: &[u8],
    amt: u64,
) -> clvmr::NodePtr {
    let nil = a.nil();
    let amt_n = u64_to_clvm(a, amt);
    let t = a.new_pair(amt_n, nil).unwrap();
    let dest_n = a.new_atom(dest).unwrap();
    let t = a.new_pair(dest_n, t).unwrap();
    let ep_n = u64_to_clvm(a, epoch);
    let t = a.new_pair(ep_n, t).unwrap();
    let del_n = u64_to_clvm(a, wdc_delay);
    let t = a.new_pair(del_n, t).unwrap();
    let wdm_n = a.new_atom(wdc_mod).unwrap();
    let t = a.new_pair(wdm_n, t).unwrap();
    let ck_n = a.new_atom(ckpt_id).unwrap();
    let t = a.new_pair(ck_n, t).unwrap();
    let pk_n = a.new_atom(pk).unwrap();
    a.new_pair(pk_n, t).unwrap()
}

/// Build withdraw delay coin env (3 curried, 0 solution).
fn build_wdc_env(a: &mut Allocator, dest: &[u8], amount: u64, delay: u64) -> clvmr::NodePtr {
    let d = u64_to_clvm(a, delay);
    let t = a.new_pair(d, a.nil()).unwrap();
    let am = u64_to_clvm(a, amount);
    let t = a.new_pair(am, t).unwrap();
    let de = a.new_atom(dest).unwrap();
    a.new_pair(de, t).unwrap()
}

// ── Registration coin memo (conflict-resistant hint) ─────────────────

/// WDC-010: Registration coin source has Memos with hint.
#[test]
fn vv_req_wdc_010_reg_source_has_memo() {
    let src = include_str!("../puzzles/registration_coin.rue");
    assert!(
        src.contains("Memos { value: hint }"),
        "WDC-010: Registration coin must have Memos with hint"
    );
}

/// WDC-010: Registration coin computes hint as sha256(ckpt_id + dest).
#[test]
fn vv_req_wdc_010_reg_source_hint_is_sha256() {
    let src = include_str!("../puzzles/registration_coin.rue");
    assert!(
        src.contains("sha256(checkpoint_bytes + dest_bytes)"),
        "WDC-010: Hint must be sha256(checkpoint_bytes + dest_bytes)"
    );
}

/// WDC-010: Registration coin CLVM output includes memo in CreateCoin.
#[test]
fn vv_req_wdc_010_reg_clvm_create_coin_has_memo() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let ckpt_id = [0xBB; 32];
    let dest = [0xCC; 32];
    let wdc_mod = [0x55; 32];
    let env = build_reg_env(
        &mut a,
        &[0xAA; 48],
        &ckpt_id,
        &wdc_mod,
        24_000,
        1,
        &dest,
        1_000_000,
    );
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    // CreateCoin is conditions[1] (after ASSERT_COIN_ANNOUNCEMENT)
    let create_coin = &conditions[1];
    assert_eq!(create_coin.opcode, CREATE_COIN as i64);

    // Must have at least 3 args: puzzle_hash, amount, memo
    assert!(
        create_coin.args.len() >= 3,
        "WDC-010: CreateCoin must have memo (3rd arg), got {} args",
        create_coin.args.len()
    );

    // Memo must be 32 bytes (sha256 hash)
    let memo = &create_coin.args[2];
    assert_eq!(
        memo.len(),
        32,
        "WDC-010: Memo must be 32 bytes (sha256 hash), got {}",
        memo.len()
    );
}

/// WDC-010: Registration coin memo matches Rust sha256(ckpt_id + dest).
#[test]
fn vv_req_wdc_010_reg_clvm_memo_cross_impl() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let ckpt_id = [0xBB; 32];
    let dest = [0xCC; 32];
    let wdc_mod = [0x55; 32];
    let env = build_reg_env(
        &mut a,
        &[0xAA; 48],
        &ckpt_id,
        &wdc_mod,
        24_000,
        1,
        &dest,
        1_000_000,
    );
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let memo = &conditions[1].args[2];

    // Compute expected hint in Rust
    let expected: [u8; 32] = Sha256::digest([ckpt_id.as_slice(), dest.as_slice()].concat()).into();

    assert_eq!(
        memo.as_slice(),
        &expected,
        "WDC-010: CLVM memo must match Rust sha256(ckpt_id + dest)"
    );
}

/// WDC-010: Different destinations produce different hints.
#[test]
fn vv_req_wdc_010_reg_different_dest_different_hint() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let ckpt_id = [0xBB; 32];
    let wdc_mod = [0x55; 32];

    let env1 = build_reg_env(
        &mut a,
        &[0xAA; 48],
        &ckpt_id,
        &wdc_mod,
        24_000,
        1,
        &[0x11; 32],
        1_000_000,
    );
    let env2 = build_reg_env(
        &mut a,
        &[0xAA; 48],
        &ckpt_id,
        &wdc_mod,
        24_000,
        1,
        &[0x22; 32],
        1_000_000,
    );

    let (_, o1) = run_puzzle_ok(&mut a, puzzle, env1);
    let (_, o2) = run_puzzle_ok(&mut a, puzzle, env2);
    let c1 = parse_conditions(&a, o1);
    let c2 = parse_conditions(&a, o2);

    assert_ne!(
        c1[1].args[2], c2[1].args[2],
        "WDC-010: Different destinations must produce different hints"
    );
}

/// WDC-010: Different checkpoint IDs produce different hints (network isolation).
#[test]
fn vv_req_wdc_010_reg_different_ckpt_different_hint() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let dest = [0xCC; 32];
    let wdc_mod = [0x55; 32];

    let env1 = build_reg_env(
        &mut a,
        &[0xAA; 48],
        &[0x11; 32],
        &wdc_mod,
        24_000,
        1,
        &dest,
        1_000_000,
    );
    let env2 = build_reg_env(
        &mut a,
        &[0xAA; 48],
        &[0x22; 32],
        &wdc_mod,
        24_000,
        1,
        &dest,
        1_000_000,
    );

    let (_, o1) = run_puzzle_ok(&mut a, puzzle, env1);
    let (_, o2) = run_puzzle_ok(&mut a, puzzle, env2);
    let c1 = parse_conditions(&a, o1);
    let c2 = parse_conditions(&a, o2);

    assert_ne!(
        c1[1].args[2], c2[1].args[2],
        "WDC-010: Different checkpoint IDs must produce different hints (network isolation)"
    );
}

// ── Withdraw delay coin memo ─────────────────────────────────────────

/// WDC-010: Withdraw delay coin source has "DIG Network Collateral Release" memo.
#[test]
fn vv_req_wdc_010_wdc_source_has_memo() {
    let src = include_str!("../puzzles/withdraw_delay_coin.rue");
    assert!(
        src.contains("DIG Network Collateral Release"),
        "WDC-010: Withdraw delay coin must have 'DIG Network Collateral Release' memo"
    );
}

/// WDC-010: Withdraw delay coin CLVM output includes memo in CreateCoin.
#[test]
fn vv_req_wdc_010_wdc_clvm_create_coin_has_memo() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
    let env = build_wdc_env(&mut a, &[0xDD; 32], 1_000_000, 100);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    // CreateCoin is conditions[1] (after ASSERT_HEIGHT_RELATIVE)
    let create_coin = &conditions[1];
    assert_eq!(create_coin.opcode, CREATE_COIN as i64);

    assert!(
        create_coin.args.len() >= 3,
        "WDC-010: Withdraw delay CreateCoin must have memo, got {} args",
        create_coin.args.len()
    );

    // Memo must be "DIG Network Collateral Release" as UTF-8 bytes
    let memo = &create_coin.args[2];
    let expected = b"DIG Network Collateral Release";
    assert_eq!(
        memo.as_slice(),
        expected.as_slice(),
        "WDC-010: Memo must be 'DIG Network Collateral Release', got {:?}",
        String::from_utf8_lossy(memo)
    );
}

// ── Compilation ──────────────────────────────────────────────────────

/// WDC-010: Both puzzles compile with memos (artifacts fresh).
#[test]
fn vv_req_wdc_010_both_compile() {
    // If .hex artifacts load, the puzzles compiled successfully with memos
    let mut a = Allocator::new();
    let _reg = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let _wdc = load_puzzle(&mut a, WITHDRAW_DELAY_COIN_PUZZLE_HEX);
}

/// WDC-010: Spec file exists.
#[test]
fn vv_req_wdc_010_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/withdraw_delay/specs/WDC-010.md").exists(),
    );
}
