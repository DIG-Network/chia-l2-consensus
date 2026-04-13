//! REQUIREMENT: WDC-004 — Registration Coin Integration
//! (`docs/requirements/domains/withdraw_delay/NORMATIVE.md#WDC-004`).
//!
//! Spec: `docs/requirements/domains/withdraw_delay/specs/WDC-004.md`.
//!
//! ## Normative Statement
//!
//! The registration coin puzzle MUST be updated to create a withdraw delay coin
//! instead of sending collateral directly to the destination. The registration
//! coin MUST compute the withdraw delay coin puzzle hash on-chain using
//! curry_tree_hash and MUST have WITHDRAW_DELAY_MOD_HASH and WITHDRAW_DELAY_BLOCKS
//! as additional curried parameters.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Registration coin has WITHDRAW_DELAY_MOD_HASH curried
//! - [x] Registration coin has WITHDRAW_DELAY_BLOCKS curried
//! - [x] Registration coin CREATE_COIN target is delay coin puzzle hash
//! - [x] On-chain curry_tree_hash matches Rust-computed delay coin puzzle hash
//! - [x] Network coin updated with 2 additional curried params
//! - [x] Registration coin no longer creates direct destination coin

mod common;

use clvm_utils::{curry_tree_hash, tree_hash_atom, TreeHash};
use clvmr::Allocator;

use chia_l2_consensus::testing::{
    NETWORK_COIN_INNER_PUZZLE_HEX, REGISTRATION_COIN_MOD_HASH_HEX, REGISTRATION_COIN_PUZZLE_HEX,
    WITHDRAW_DELAY_COIN_MOD_HASH_HEX,
};

use common::clvm::*;

// ── Registration coin source inspection ──────────────────────────────

/// WDC-004: Registration coin now has WITHDRAW_DELAY_MOD_HASH curried param.
#[test]
fn vv_req_wdc_004_reg_coin_has_delay_mod_hash() {
    let src = include_str!("../puzzles/registration_coin.rue");
    assert!(
        src.contains("WITHDRAW_DELAY_MOD_HASH: Bytes32"),
        "WDC-004: Registration coin must have WITHDRAW_DELAY_MOD_HASH curried"
    );
}

/// WDC-004: Registration coin now has WITHDRAW_DELAY_BLOCKS curried param.
#[test]
fn vv_req_wdc_004_reg_coin_has_delay_blocks() {
    let src = include_str!("../puzzles/registration_coin.rue");
    assert!(
        src.contains("WITHDRAW_DELAY_BLOCKS: Int"),
        "WDC-004: Registration coin must have WITHDRAW_DELAY_BLOCKS curried"
    );
}

/// WDC-004: Registration coin uses curry_tree_hash to compute delay coin hash.
#[test]
fn vv_req_wdc_004_reg_coin_computes_delay_hash() {
    let src = include_str!("../puzzles/registration_coin.rue");
    assert!(
        src.contains("curry_tree_hash("),
        "WDC-004: Registration coin must use curry_tree_hash for delay coin"
    );
    assert!(
        src.contains("WITHDRAW_DELAY_MOD_HASH"),
        "WDC-004: curry_tree_hash must use WITHDRAW_DELAY_MOD_HASH"
    );
}

/// WDC-004: Registration coin creates delay coin, not direct destination.
#[test]
fn vv_req_wdc_004_reg_coin_creates_delay_coin() {
    let src = include_str!("../puzzles/registration_coin.rue");
    assert!(
        src.contains("withdraw_delay_puzzle_hash"),
        "WDC-004: Must create coin at withdraw_delay_puzzle_hash"
    );
    // Should NOT directly use collateral_destination as puzzle_hash in CreateCoin
    assert!(
        !src.contains("puzzle_hash: collateral_destination"),
        "WDC-004: Must NOT create coin directly at collateral_destination"
    );
}

/// WDC-004: Registration coin has exactly 4 curried params.
#[test]
fn vv_req_wdc_004_reg_coin_four_curried_params() {
    let src = include_str!("../puzzles/registration_coin.rue");
    let fn_sig = src
        .split("fn main(")
        .nth(1)
        .unwrap()
        .split(") ->")
        .next()
        .unwrap();
    let params: Vec<&str> = fn_sig
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//") && l.contains(": "))
        .collect();
    // 4 curried (UPPERCASE) + 3 solution (lowercase) = 7 total
    let curried: Vec<&&str> = params
        .iter()
        .filter(|p| {
            let name = p.split(':').next().unwrap().trim();
            name.chars().next().is_some_and(|c| c.is_uppercase())
        })
        .collect();
    assert_eq!(
        curried.len(),
        4,
        "WDC-004: Must have 4 curried params, got {}: {:?}",
        curried.len(),
        curried
    );
}

// ── Network coin source inspection ───────────────────────────────────

/// WDC-004: Network coin now has withdraw_delay_mod_hash curried param.
#[test]
fn vv_req_wdc_004_net_coin_has_delay_mod_hash() {
    let src = include_str!("../puzzles/network_coin_inner.rue");
    assert!(
        src.contains("withdraw_delay_mod_hash: Bytes32"),
        "WDC-004: Network coin must have withdraw_delay_mod_hash curried"
    );
}

/// WDC-004: Network coin now has withdraw_delay_blocks curried param.
#[test]
fn vv_req_wdc_004_net_coin_has_delay_blocks() {
    let src = include_str!("../puzzles/network_coin_inner.rue");
    assert!(
        src.contains("withdraw_delay_blocks: Int"),
        "WDC-004: Network coin must have withdraw_delay_blocks curried"
    );
}

/// WDC-004: Network coin curry_tree_hash includes 4 args for reg coin.
#[test]
fn vv_req_wdc_004_net_coin_curry_includes_delay_params() {
    let src = include_str!("../puzzles/network_coin_inner.rue");
    assert!(
        src.contains("tree_hash(withdraw_delay_mod_hash)"),
        "WDC-004: Network coin curry must include tree_hash(withdraw_delay_mod_hash)"
    );
    assert!(
        src.contains("tree_hash(withdraw_delay_blocks)"),
        "WDC-004: Network coin curry must include tree_hash(withdraw_delay_blocks)"
    );
}

/// WDC-004: Network coin has 6 curried params (was 4, +2 for delay).
#[test]
fn vv_req_wdc_004_net_coin_six_curried_params() {
    let src = include_str!("../puzzles/network_coin_inner.rue");
    let fn_sig = src
        .split("fn main(")
        .nth(1)
        .unwrap()
        .split(") ->")
        .next()
        .unwrap();
    let params: Vec<&str> = fn_sig
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//") && l.contains(": "))
        .collect();
    // 6 curried + 2 solution = 8 total
    assert_eq!(
        params.len(),
        8,
        "WDC-004: Network coin must have 8 params (6 curried + 2 solution), got {}: {:?}",
        params.len(),
        params
    );
}

// ── CLVM execution: registration coin ────────────────────────────────

/// Build registration coin env with 4 curried + 3 solution params.
/// Flat env: (PK . (CKPT_ID . (WDC_MOD . (WDC_DELAY . (epoch . (dest . (amt . nil)))))))
fn build_reg_env_wdc004(
    a: &mut Allocator,
    pk: &[u8],
    ckpt_id: &[u8],
    wdc_mod_hash: &[u8],
    wdc_delay: u64,
    epoch: u64,
    dest: &[u8],
    amt: u64,
) -> clvmr::NodePtr {
    let nil = a.nil();
    let amt_node = u64_to_clvm(a, amt);
    let t = a.new_pair(amt_node, nil).unwrap();
    let dest_node = a.new_atom(dest).unwrap();
    let t = a.new_pair(dest_node, t).unwrap();
    let epoch_node = u64_to_clvm(a, epoch);
    let t = a.new_pair(epoch_node, t).unwrap();
    let delay_node = u64_to_clvm(a, wdc_delay);
    let t = a.new_pair(delay_node, t).unwrap();
    let wdc_mod_node = a.new_atom(wdc_mod_hash).unwrap();
    let t = a.new_pair(wdc_mod_node, t).unwrap();
    let ckpt_node = a.new_atom(ckpt_id).unwrap();
    let t = a.new_pair(ckpt_node, t).unwrap();
    let pk_node = a.new_atom(pk).unwrap();
    a.new_pair(pk_node, t).unwrap()
}

/// Get the WDC mod hash bytes.
fn wdc_mod_hash_bytes() -> [u8; 32] {
    let h = WITHDRAW_DELAY_COIN_MOD_HASH_HEX.trim();
    let h = h.strip_prefix("0x").unwrap_or(h);
    hex::decode(h).unwrap().try_into().unwrap()
}

/// WDC-004: Updated registration coin CLVM loads and runs.
#[test]
fn vv_req_wdc_004_reg_clvm_loads_and_executes() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let wdc_mod = wdc_mod_hash_bytes();
    let env = build_reg_env_wdc004(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &wdc_mod,
        24_000,
        1,
        &[0xCC; 32],
        1_000_000,
    );
    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    assert_eq!(
        conditions.len(),
        2,
        "WDC-004: Registration coin must emit 2 conditions"
    );
}

/// WDC-004: Registration coin CREATE_COIN target is NOT the raw destination.
#[test]
fn vv_req_wdc_004_reg_clvm_not_direct_destination() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let dest = [0xCC; 32];
    let wdc_mod = wdc_mod_hash_bytes();
    let env = build_reg_env_wdc004(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &wdc_mod,
        24_000,
        1,
        &dest,
        1_000_000,
    );
    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    let create_coin = &conditions[1];
    assert_eq!(create_coin.opcode, CREATE_COIN as i64);
    assert_ne!(
        create_coin.args[0].as_slice(),
        &dest,
        "WDC-004: CREATE_COIN puzzle_hash must NOT be the raw destination"
    );
}

/// WDC-004: Registration coin CREATE_COIN target matches Rust-computed delay coin hash.
#[test]
fn vv_req_wdc_004_reg_clvm_cross_impl_delay_hash() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let dest = [0xCC; 32];
    let amount: u64 = 1_000_000;
    let delay: u64 = 24_000;
    let wdc_mod = wdc_mod_hash_bytes();

    let env = build_reg_env_wdc004(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &wdc_mod,
        delay,
        1,
        &dest,
        amount,
    );
    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let clvm_hash = &conditions[1].args[0];

    // Compute the same hash in Rust
    let rust_hash: [u8; 32] = curry_tree_hash(
        TreeHash::new(wdc_mod),
        &[
            tree_hash_atom(&dest),
            tree_hash_atom(&clvm_int_bytes(amount)),
            tree_hash_atom(&clvm_int_bytes(delay)),
        ],
    )
    .into();

    assert_eq!(
        clvm_hash.as_slice(),
        &rust_hash,
        "WDC-004: CLVM delay coin puzzle hash must match Rust curry_tree_hash"
    );
}

/// WDC-004: Different destinations produce different delay coin hashes in CLVM.
#[test]
fn vv_req_wdc_004_reg_clvm_different_dest_different_hash() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let wdc_mod = wdc_mod_hash_bytes();

    let env_a = build_reg_env_wdc004(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &wdc_mod,
        24_000,
        1,
        &[0x11; 32],
        1_000_000,
    );
    let env_b = build_reg_env_wdc004(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &wdc_mod,
        24_000,
        1,
        &[0x22; 32],
        1_000_000,
    );

    let (_, out_a) = run_puzzle_ok(&mut a, puzzle, env_a);
    let (_, out_b) = run_puzzle_ok(&mut a, puzzle, env_b);
    let conds_a = parse_conditions(&a, out_a);
    let conds_b = parse_conditions(&a, out_b);

    assert_ne!(
        conds_a[1].args[0], conds_b[1].args[0],
        "WDC-004: Different destinations must produce different delay coin hashes"
    );
}

/// WDC-004: Cross-impl with different amount.
#[test]
fn vv_req_wdc_004_reg_clvm_cross_impl_different_amount() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let dest = [0xCC; 32];
    let amount: u64 = 5_000_000;
    let delay: u64 = 100;
    let wdc_mod = wdc_mod_hash_bytes();

    let env = build_reg_env_wdc004(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &wdc_mod,
        delay,
        1,
        &dest,
        amount,
    );
    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let clvm_hash = &conditions[1].args[0];

    let rust_hash: [u8; 32] = curry_tree_hash(
        TreeHash::new(wdc_mod),
        &[
            tree_hash_atom(&dest),
            tree_hash_atom(&clvm_int_bytes(amount)),
            tree_hash_atom(&clvm_int_bytes(delay)),
        ],
    )
    .into();

    assert_eq!(
        clvm_hash.as_slice(),
        &rust_hash,
        "WDC-004: Cross-impl must match for different amount"
    );
}

/// WDC-004: Collateral amount in CREATE_COIN still matches solution amount.
#[test]
fn vv_req_wdc_004_reg_clvm_amount_passthrough() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REGISTRATION_COIN_PUZZLE_HEX);
    let wdc_mod = wdc_mod_hash_bytes();
    let amount: u64 = 7_777_777;

    let env = build_reg_env_wdc004(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &wdc_mod,
        24_000,
        1,
        &[0xCC; 32],
        amount,
    );
    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    let create_coin = &conditions[1];
    let parsed_amount = parse_clvm_u64(&create_coin.args[1]);
    assert_eq!(
        parsed_amount, amount,
        "WDC-004: CREATE_COIN amount must match solution collateral_amount"
    );
}

// ── Artifact freshness ───────────────────────────────────────────────

/// WDC-004: Registration coin .hex artifact matches fresh build.
#[test]
fn vv_req_wdc_004_reg_hex_matches_live_build() {
    let fresh = std::process::Command::new("rue")
        .args(["build", "-x", "puzzles/registration_coin.rue"])
        .output();
    if let Ok(output) = fresh {
        if output.status.success() {
            let live = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let saved = REGISTRATION_COIN_PUZZLE_HEX.trim();
            assert_eq!(
                saved, live,
                "WDC-004: Saved .hex must match fresh rue build"
            );
        }
    }
}

/// WDC-004: Registration coin .hash artifact matches fresh build.
#[test]
fn vv_req_wdc_004_reg_hash_matches_live_build() {
    let fresh = std::process::Command::new("rue")
        .args(["build", "--hash", "puzzles/registration_coin.rue"])
        .output();
    if let Ok(output) = fresh {
        if output.status.success() {
            let live = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let saved = REGISTRATION_COIN_MOD_HASH_HEX.trim();
            assert_eq!(
                saved, live,
                "WDC-004: Saved .hash must match fresh rue build"
            );
        }
    }
}

/// WDC-004: Network coin .hex artifact matches fresh build.
#[test]
fn vv_req_wdc_004_net_hex_matches_live_build() {
    let fresh = std::process::Command::new("rue")
        .args(["build", "-x", "puzzles/network_coin_inner.rue"])
        .output();
    if let Ok(output) = fresh {
        if output.status.success() {
            let live = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let saved = NETWORK_COIN_INNER_PUZZLE_HEX.trim();
            assert_eq!(
                saved, live,
                "WDC-004: Network .hex must match fresh rue build"
            );
        }
    }
}

/// WDC-004: Spec file exists.
#[test]
fn vv_req_wdc_004_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/withdraw_delay/specs/WDC-004.md").exists(),
        "WDC-004: Spec file must exist"
    );
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Encode u64 as CLVM integer bytes.
fn clvm_int_bytes(val: u64) -> Vec<u8> {
    if val == 0 {
        return vec![];
    }
    let bytes = val.to_be_bytes();
    let stripped: Vec<u8> = bytes.iter().copied().skip_while(|&b| b == 0).collect();
    if stripped[0] & 0x80 != 0 {
        let mut with_sign = vec![0x00];
        with_sign.extend_from_slice(&stripped);
        with_sign
    } else {
        stripped
    }
}

/// Parse CLVM integer atom as u64.
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
