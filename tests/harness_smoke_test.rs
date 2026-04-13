//! Smoke test for the CLVM test harness.
//!
//! KEY INSIGHT: Rue-compiled puzzles expect ALL parameters (curried + solution)
//! as a single flat right-linked list. Do NOT use CurriedProgram from clvm-utils.
//!
//! NOTE: Vec<u8> in #[clvm(list)] serializes as a CLVM list of bytes, NOT a byte atom.
//! Must build environments manually with Allocator for correct atom encoding.

mod common;

use clvmr::Allocator;
use sha2::{Digest, Sha256};

use common::clvm::*;

/// Build registration coin environment as flat list.
/// WDC-004: Now 4 curried + 3 solution = 7 params.
/// (PK . (CKPT_ID . (WDC_MOD . (WDC_DELAY . (epoch . (dest . (amt . nil)))))))
fn build_reg_coin_env(
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

/// Build network coin inner environment as flat list.
/// WDC-004: Now 6 curried + 2 solution = 8 params.
/// (inner_mod_hash . (mod_hash . (collateral . (ckpt_id . (wdc_mod . (wdc_delay . (pubkey . (conds . nil))))))))
fn build_net_coin_env(
    a: &mut Allocator,
    inner_mod_hash: &[u8],
    mod_hash: &[u8],
    collateral: u64,
    ckpt_id: &[u8],
    wdc_mod_hash: &[u8],
    wdc_delay: u64,
    pubkey: &[u8],
) -> clvmr::NodePtr {
    let conds = a.nil();
    let nil = a.nil();
    let t = a.new_pair(conds, nil).unwrap();
    let pk_node = a.new_atom(pubkey).unwrap();
    let t = a.new_pair(pk_node, t).unwrap();
    let delay_node = u64_to_clvm(a, wdc_delay);
    let t = a.new_pair(delay_node, t).unwrap();
    let wdc_mod_node = a.new_atom(wdc_mod_hash).unwrap();
    let t = a.new_pair(wdc_mod_node, t).unwrap();
    let ckpt_node = a.new_atom(ckpt_id).unwrap();
    let t = a.new_pair(ckpt_node, t).unwrap();
    let col_node = u64_to_clvm(a, collateral);
    let t = a.new_pair(col_node, t).unwrap();
    let hash_node = a.new_atom(mod_hash).unwrap();
    let t = a.new_pair(hash_node, t).unwrap();
    let imh_node = a.new_atom(inner_mod_hash).unwrap();
    a.new_pair(imh_node, t).unwrap()
}

// Dummy WDC mod hash for smoke tests
fn dummy_wdc_mod_hash() -> [u8; 32] {
    [0x44; 32]
}

// ── Registration coin tests ────────────────────────────────────────

#[test]
fn harness_smoke_load_registration_coin() {
    let mut a = Allocator::new();
    let hex = include_str!("../puzzles/compiled/registration_coin.hex");
    let puzzle = load_puzzle(&mut a, hex);
    assert_ne!(puzzle, a.nil(), "Puzzle should not be nil");
}

#[test]
fn harness_smoke_run_registration_coin() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(
        &mut a,
        include_str!("../puzzles/compiled/registration_coin.hex"),
    );
    let env = build_reg_coin_env(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &dummy_wdc_mod_hash(),
        24_000,
        5,
        &[0xCC; 32],
        1_000_000,
    );

    let (cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    assert!(cost > 0, "Execution should have non-zero cost");

    let conditions = parse_conditions(&a, output);
    assert!(
        has_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT),
        "Must emit ASSERT_COIN_ANNOUNCEMENT (61)"
    );
    assert!(
        has_opcode(&conditions, CREATE_COIN),
        "Must emit CREATE_COIN (51)"
    );
}

#[test]
fn harness_smoke_verify_create_coin_args() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(
        &mut a,
        include_str!("../puzzles/compiled/registration_coin.hex"),
    );
    let dest = [0xCC; 32];
    let amount: u64 = 1_000_000;
    let env = build_reg_coin_env(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &dummy_wdc_mod_hash(),
        24_000,
        5,
        &dest,
        amount,
    );

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    let create_coins = conditions_with_opcode(&conditions, CREATE_COIN);
    assert_eq!(create_coins.len(), 1, "Exactly 1 CREATE_COIN");

    // WDC-004: The puzzle hash is now the delay coin hash, NOT the raw destination
    assert_ne!(
        create_coins[0].args[0],
        dest.to_vec(),
        "Puzzle hash must NOT be raw destination (WDC-004: creates delay coin)"
    );

    let amount_bytes = &create_coins[0].args[1];
    let mut padded = vec![0u8; 8 - amount_bytes.len()];
    padded.extend_from_slice(amount_bytes);
    let actual = u64::from_be_bytes(padded.try_into().unwrap());
    assert_eq!(actual, amount, "Amount must match");
}

#[test]
fn harness_smoke_cross_impl_announcement_hash() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(
        &mut a,
        include_str!("../puzzles/compiled/registration_coin.hex"),
    );

    let pk = [0xAA; 48];
    let ckpt_id = [0xBB; 32];
    let epoch: u64 = 5;
    let env = build_reg_coin_env(
        &mut a,
        &pk,
        &ckpt_id,
        &dummy_wdc_mod_hash(),
        24_000,
        epoch,
        &[0xCC; 32],
        1_000_000,
    );

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let announcements = conditions_with_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT);
    assert_eq!(announcements.len(), 1);
    let clvm_hash = &announcements[0].args[0];

    // Compute expected hash in Rust
    let mut inner = Vec::new();
    inner.extend_from_slice(b"membership");
    inner.extend_from_slice(&epoch.to_be_bytes());
    inner.extend_from_slice(&pk);
    inner.push(0x00);
    let inner_hash: [u8; 32] = Sha256::digest(&inner).into();

    let mut full = Vec::new();
    full.extend_from_slice(&ckpt_id);
    full.extend_from_slice(&inner_hash);
    let expected: [u8; 32] = Sha256::digest(&full).into();

    assert_eq!(
        clvm_hash.as_slice(),
        expected.as_slice(),
        "CLVM announcement hash must match Rust wire format hash"
    );
}

#[test]
fn harness_smoke_different_params_different_output() {
    let mut a = Allocator::new();
    let hex = include_str!("../puzzles/compiled/registration_coin.hex");

    // Run with two different pubkeys
    let puzzle1 = load_puzzle(&mut a, hex);
    let env1 = build_reg_coin_env(
        &mut a,
        &[0xAA; 48],
        &[0xBB; 32],
        &dummy_wdc_mod_hash(),
        24_000,
        5,
        &[0xCC; 32],
        1_000_000,
    );
    let (_, out1) = run_puzzle_ok(&mut a, puzzle1, env1);
    let conds1 = parse_conditions(&a, out1);
    let hash1 = &conditions_with_opcode(&conds1, ASSERT_COIN_ANNOUNCEMENT)[0].args[0];

    let puzzle2 = load_puzzle(&mut a, hex);
    let env2 = build_reg_coin_env(
        &mut a,
        &[0x11; 48],
        &[0xBB; 32],
        &dummy_wdc_mod_hash(),
        24_000,
        5,
        &[0xCC; 32],
        1_000_000,
    );
    let (_, out2) = run_puzzle_ok(&mut a, puzzle2, env2);
    let conds2 = parse_conditions(&a, out2);
    let hash2 = &conditions_with_opcode(&conds2, ASSERT_COIN_ANNOUNCEMENT)[0].args[0];

    assert_ne!(
        hash1, hash2,
        "Different pubkeys must produce different announcement hashes"
    );
}

// ── Network coin tests ─────────────────────────────────────────────

#[test]
fn harness_smoke_run_network_coin() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(
        &mut a,
        include_str!("../puzzles/compiled/network_coin_inner.hex"),
    );
    let env = build_net_coin_env(
        &mut a,
        &[0x00; 32], // INNER_MOD_HASH
        &[0x11; 32], // registration_coin_mod_hash
        1_000_000_000_000,
        &[0x22; 32],           // checkpoint_singleton_id
        &dummy_wdc_mod_hash(), // WDC-004: withdraw_delay_mod_hash
        24_000,                // WDC-004: withdraw_delay_blocks
        &[0x33; 48],           // pubkey
    );

    let result = run_puzzle(&mut a, puzzle, env);
    match result {
        Ok((cost, output)) => {
            assert!(cost > 0);
            let conditions = parse_conditions(&a, output);
            assert!(
                has_opcode(&conditions, AGG_SIG_ME),
                "Must emit AGG_SIG_ME (50)"
            );
            assert!(
                has_opcode(&conditions, CREATE_COIN),
                "Must emit CREATE_COIN (51)"
            );
        }
        Err(e) => panic!("Network coin CLVM failed: {}", e.1),
    }
}
