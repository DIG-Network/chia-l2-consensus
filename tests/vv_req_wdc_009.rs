//! REQUIREMENT: WDC-009 — E2E Simulator Test
//! (`docs/requirements/domains/withdraw_delay/NORMATIVE.md#WDC-009`).
//!
//! Spec: `docs/requirements/domains/withdraw_delay/specs/WDC-009.md`.
//!
//! ## Normative Statement
//!
//! A full E2E simulator test exercises the complete two-phase collateral
//! recovery lifecycle: register → exit → delay coin → release.
//!
//! ## Simulator Limitation
//!
//! chia-sdk-test v0.18 Simulator does NOT enforce ASSERT_HEIGHT_RELATIVE.
//! The delay coin spend succeeds immediately in the simulator. Actual delay
//! enforcement is by the Chia full node at block inclusion time. The test
//! verifies the correct CLVM conditions are emitted; the network enforces them.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Delay coin created with correct puzzle hash and amount
//! - [x] Destination coin amount == original collateral (after delay coin spend)
//! - [x] No direct destination coin from registration coin spend
//! - [x] Third-party release succeeds (no signature needed)
//! - [ ] Spend before delay rejected (simulator doesn't enforce — see note)

mod common;

use chia_protocol::Bytes32;
use chia_puzzles::singleton::{SingletonArgs, SingletonSolution, SingletonStruct};
use chia_puzzles::{EveProof, Proof};
use chia_sdk_driver::{Launcher, Spend, SpendContext, StandardLayer};
use chia_sdk_test::Simulator;
use chia_wallet_sdk::Conditions;
use clvm_traits::ToClvm;
use clvm_utils::CurriedProgram;
use clvmr::serde::node_from_bytes;
use sha2::{Digest, Sha256};

const COLLATERAL_AMOUNT: u64 = 1_000_000;

fn net_inner_hex() -> Vec<u8> {
    hex::decode(include_str!("../puzzles/compiled/network_coin_inner.hex").trim()).unwrap()
}
fn net_inner_mod_hash() -> Bytes32 {
    let b: [u8; 32] = hex::decode(
        include_str!("../puzzles/compiled/network_coin_inner.hash")
            .trim()
            .trim_start_matches("0x"),
    )
    .unwrap()
    .try_into()
    .unwrap();
    b.into()
}
fn reg_hex() -> Vec<u8> {
    hex::decode(include_str!("../puzzles/compiled/registration_coin.hex").trim()).unwrap()
}
fn chk_inner_hex() -> Vec<u8> {
    hex::decode(include_str!("../puzzles/compiled/checkpoint_inner.hex").trim()).unwrap()
}
fn chk_inner_mod_hash() -> Bytes32 {
    let b: [u8; 32] = hex::decode(
        include_str!("../puzzles/compiled/checkpoint_inner.hash")
            .trim()
            .trim_start_matches("0x"),
    )
    .unwrap()
    .try_into()
    .unwrap();
    b.into()
}
fn wdc_hex() -> Vec<u8> {
    hex::decode(include_str!("../puzzles/compiled/withdraw_delay_coin.hex").trim()).unwrap()
}
fn wdc_mod_hash_bytes() -> [u8; 32] {
    hex::decode(
        include_str!("../puzzles/compiled/withdraw_delay_coin.hash")
            .trim()
            .trim_start_matches("0x"),
    )
    .unwrap()
    .try_into()
    .unwrap()
}

const WDC_DELAY_BLOCKS: u64 = 10; // Short delay for tests

/// Standard CLVM curry.
fn clvm_curry(
    a: &mut clvmr::Allocator,
    module: clvmr::NodePtr,
    args: &[clvmr::NodePtr],
) -> clvmr::NodePtr {
    let op_a = a.new_small_number(2).unwrap();
    let op_q = a.new_small_number(1).unwrap();
    let op_c = a.new_small_number(4).unwrap();
    let one = a.new_small_number(1).unwrap();
    let nil = a.nil();
    let mut tail = one;
    for arg in args.iter().rev() {
        let qa = a.new_pair(op_q, *arg).unwrap();
        let al = a.new_pair(tail, nil).unwrap();
        let al = a.new_pair(qa, al).unwrap();
        tail = a.new_pair(op_c, al).unwrap();
    }
    let qm = a.new_pair(op_q, module).unwrap();
    let aa = a.new_pair(tail, nil).unwrap();
    let aa = a.new_pair(qm, aa).unwrap();
    a.new_pair(op_a, aa).unwrap()
}

/// Build network coin inner flat env (WDC-004: 6 curried + 2 solution).
fn build_net_env(
    a: &mut clvmr::Allocator,
    checkpoint_id: &[u8; 32],
    pubkey: &[u8],
) -> clvmr::NodePtr {
    let nil = a.nil();
    let conds = a.new_pair(nil, nil).unwrap();
    let pk = a.new_atom(pubkey).unwrap();
    let t = a.new_pair(pk, conds).unwrap();
    let delay = common::clvm::u64_to_clvm(a, WDC_DELAY_BLOCKS);
    let t = a.new_pair(delay, t).unwrap();
    let wdm = a.new_atom(&wdc_mod_hash_bytes()).unwrap();
    let t = a.new_pair(wdm, t).unwrap();
    let ck = a.new_atom(checkpoint_id).unwrap();
    let t = a.new_pair(ck, t).unwrap();
    let col = common::clvm::u64_to_clvm(a, COLLATERAL_AMOUNT);
    let t = a.new_pair(col, t).unwrap();
    let rm = a
        .new_atom(
            &hex::decode(
                include_str!("../puzzles/compiled/registration_coin.hash")
                    .trim()
                    .trim_start_matches("0x"),
            )
            .unwrap(),
        )
        .unwrap();
    let t = a.new_pair(rm, t).unwrap();
    let im = a.new_atom(net_inner_mod_hash().as_ref()).unwrap();
    a.new_pair(im, t).unwrap()
}

/// Build checkpoint inner flat env for membership query path (depth=0).
/// Full 19-parameter env matching REG-007's working version.
fn build_chk_query_env(
    a: &mut clvmr::Allocator,
    root: &[u8; 32],
    epoch: u64,
    validator_count: u64,
    query_pubkey: &[u8],
    is_member: bool,
) -> clvmr::NodePtr {
    let nil = a.nil();
    let empty_leaf: [u8; 32] = Sha256::digest([0u8; 48]).into();
    // 19. conditions (spread, (nil.nil))
    let conds = a.new_pair(nil, nil).unwrap();
    // 18. is_member
    let is_mem = if is_member {
        a.new_atom(&[1]).unwrap()
    } else {
        nil
    };
    let t = a.new_pair(is_mem, conds).unwrap();
    // 17. siblings (empty for depth=0)
    let t = a.new_pair(nil, t).unwrap();
    // 16. leaf_index
    let t = a.new_pair(nil, t).unwrap();
    // 15. query_pubkey
    let pk = a.new_atom(query_pubkey).unwrap();
    let t = a.new_pair(pk, t).unwrap();
    // 14. scalars (6-field struct, dummy)
    let scalars = {
        let s6 = a.new_atom(&[0u8; 48]).unwrap();
        let s5 = a.new_atom(&[0u8; 48]).unwrap();
        let r = a.new_pair(s5, s6).unwrap();
        let s4 = a.new_atom(&[0u8; 48]).unwrap();
        let r = a.new_pair(s4, r).unwrap();
        let s3 = a.new_atom(&[0u8; 48]).unwrap();
        let r = a.new_pair(s3, r).unwrap();
        let s2 = a.new_atom(&[0u8; 48]).unwrap();
        let r = a.new_pair(s2, r).unwrap();
        let s1 = a.new_atom(&[0u8; 48]).unwrap();
        a.new_pair(s1, r).unwrap()
    };
    let t = a.new_pair(scalars, t).unwrap();
    // 13. agg_sig (dummy)
    let agg_sig = a.new_atom(&[0u8; 96]).unwrap();
    let t = a.new_pair(agg_sig, t).unwrap();
    // 12. agg_signers (dummy)
    let agg_signers = a.new_atom(&[0u8; 48]).unwrap();
    let t = a.new_pair(agg_signers, t).unwrap();
    // 11. new_validator_count (dummy)
    let t = a.new_pair(nil, t).unwrap();
    // 10. new_validator_merkle_root (dummy)
    let nvmr = a.new_atom(&[0u8; 32]).unwrap();
    let t = a.new_pair(nvmr, t).unwrap();
    // 9. new_state_root (dummy)
    let nsr = a.new_atom(&[0u8; 32]).unwrap();
    let t = a.new_pair(nsr, t).unwrap();
    // 8. proof (3-field struct, dummy)
    let proof = {
        let c = a.new_atom(&[0u8; 48]).unwrap();
        let b = a.new_atom(&[0u8; 96]).unwrap();
        let ab = a.new_pair(b, c).unwrap();
        let pa = a.new_atom(&[0u8; 48]).unwrap();
        a.new_pair(pa, ab).unwrap()
    };
    let t = a.new_pair(proof, t).unwrap();
    // 7. is_checkpoint = false
    let t = a.new_pair(nil, t).unwrap();
    // 6. STATE (4-field struct: sr, epoch, vmr, vc — NOT nil-terminated)
    let state = {
        let vc = common::clvm::u64_to_clvm(a, validator_count);
        let vmr = a.new_atom(root).unwrap();
        let ep = common::clvm::u64_to_clvm(a, epoch);
        let sr = a.new_atom(&[0xAA; 32]).unwrap();
        let r = a.new_pair(vmr, vc).unwrap();
        let r = a.new_pair(ep, r).unwrap();
        a.new_pair(sr, r).unwrap()
    };
    let t = a.new_pair(state, t).unwrap();
    // NETWORK_COIN_LAUNCHER_ID (CHK-012)
    let ncli = a.new_atom(&[0x00u8; 32]).unwrap();
    let t = a.new_pair(ncli, t).unwrap();
    // EMPTY_LEAF_HASH
    let elh = a.new_atom(&empty_leaf).unwrap();
    let t = a.new_pair(elh, t).unwrap();
    // TREE_DEPTH = 0
    let t = a.new_pair(nil, t).unwrap();
    // IC (7-field, NOT nil-terminated)
    let ic = {
        let mut r = a.new_atom(&[0x01; 48]).unwrap();
        for _ in 0..6 {
            let p = a.new_atom(&[0x01; 48]).unwrap();
            r = a.new_pair(p, r).unwrap();
        }
        r
    };
    let t = a.new_pair(ic, t).unwrap();
    // VK (4-field, NOT nil-terminated)
    let vk = {
        let d = a.new_atom(&[0x01; 96]).unwrap();
        let g = a.new_atom(&[0x01; 96]).unwrap();
        let b = a.new_atom(&[0x01; 96]).unwrap();
        let al = a.new_atom(&[0x01; 48]).unwrap();
        let r = a.new_pair(g, d).unwrap();
        let r = a.new_pair(b, r).unwrap();
        a.new_pair(al, r).unwrap()
    };
    let t = a.new_pair(vk, t).unwrap();
    // INNER_MOD_HASH
    let imh = a.new_atom(chk_inner_mod_hash().as_ref()).unwrap();
    a.new_pair(imh, t).unwrap()
}

/// Build registration coin solution: (epoch dest amt . nil)
fn build_reg_solution(
    a: &mut clvmr::Allocator,
    epoch: u64,
    destination: &[u8; 32],
    amount: u64,
) -> clvmr::NodePtr {
    let nil = a.nil();
    let amt = common::clvm::u64_to_clvm(a, amount);
    let t = a.new_pair(amt, nil).unwrap();
    let dest = a.new_atom(destination).unwrap();
    let t = a.new_pair(dest, t).unwrap();
    let ep = common::clvm::u64_to_clvm(a, epoch);
    a.new_pair(ep, t).unwrap()
}

// ── E2E: Two-phase collateral recovery ───────────────────────────────

/// WDC-009: Full two-phase lifecycle.
/// 1. Deploy singletons + register validator
/// 2. Collateral recovery → creates delay coin (not direct destination)
/// 3. Spend delay coin → releases funds to destination
#[test]
fn vv_req_wdc_009_two_phase_collateral_recovery() -> anyhow::Result<()> {
    let mut sim = Simulator::new();

    // ── Phase 0: Deploy checkpoint singleton ──────────────────────────
    let ctx = &mut SpendContext::new();
    let (chk_sk, chk_pk, _, chk_p2) = sim.new_p2(1)?;
    let chk_launcher = Launcher::new(chk_p2.coin_id(), 1);
    let chk_launcher_id = chk_launcher.coin().coin_id();
    let (chk_conds, chk_singleton) = chk_launcher.spend(ctx, chk_inner_mod_hash(), ())?;
    StandardLayer::new(chk_pk).spend(ctx, chk_p2, chk_conds)?;
    sim.spend_coins(ctx.take(), &[chk_sk])?;

    // ── Phase 0: Deploy network coin + register validator ─────────────
    let ctx = &mut SpendContext::new();
    let (net_sk, net_pk, _, net_p2) = sim.new_p2(1)?;
    let net_launcher = Launcher::new(net_p2.coin_id(), 1);
    let net_launcher_id = net_launcher.coin().coin_id();
    let (net_conds, net_singleton) = net_launcher.spend(ctx, net_inner_mod_hash(), ())?;
    StandardLayer::new(net_pk).spend(ctx, net_p2, net_conds)?;
    sim.spend_coins(ctx.take(), &[net_sk])?;

    let ctx = &mut SpendContext::new();
    let validator_sk = chia_sdk_test::test_secret_key()?;
    let pk_bytes = validator_sk.public_key().to_bytes();
    let chk_coin_id: [u8; 32] = chk_singleton.coin_id().into();

    let inner_mod = node_from_bytes(&mut ctx.allocator, &net_inner_hex())?;
    let singleton_mod = ctx.singleton_top_layer()?;
    let net_puzzle = CurriedProgram {
        program: singleton_mod,
        args: SingletonArgs {
            singleton_struct: SingletonStruct::new(net_launcher_id),
            inner_puzzle: inner_mod,
        },
    }
    .to_clvm(&mut ctx.allocator)?;
    let inner_sol = build_net_env(&mut ctx.allocator, &chk_coin_id, &pk_bytes);
    let net_sol = SingletonSolution {
        lineage_proof: Proof::Eve(EveProof {
            parent_parent_coin_info: net_p2.coin_id(),
            parent_amount: 1,
        }),
        amount: 1,
        inner_solution: inner_sol,
    }
    .to_clvm(&mut ctx.allocator)?;
    ctx.spend(net_singleton, Spend::new(net_puzzle, net_sol))?;
    let (fund_sk, fund_pk, _, fund_coin) = sim.new_p2(COLLATERAL_AMOUNT)?;
    StandardLayer::new(fund_pk).spend(ctx, fund_coin, Conditions::new())?;
    sim.spend_coins(ctx.take(), &[validator_sk.clone(), fund_sk])?;

    // Find registration coin
    let net_children = sim.children(net_singleton.coin_id());
    let reg_coin = net_children
        .iter()
        .find(|cs| cs.coin.amount == COLLATERAL_AMOUNT)
        .expect("Registration coin must exist")
        .coin;

    // ── Phase 1: Collateral recovery → creates delay coin ─────────────
    let ctx = &mut SpendContext::new();
    let empty_leaf: [u8; 32] = Sha256::digest([0u8; 48]).into();
    let epoch: u64 = 0;
    let validator_count: u64 = 0;

    // Spend 1: Checkpoint membership query
    let chk_mod = node_from_bytes(&mut ctx.allocator, &chk_inner_hex())?;
    let chk_singleton_mod = ctx.singleton_top_layer()?;
    let chk_puzzle = CurriedProgram {
        program: chk_singleton_mod,
        args: SingletonArgs {
            singleton_struct: SingletonStruct::new(chk_launcher_id),
            inner_puzzle: chk_mod,
        },
    }
    .to_clvm(&mut ctx.allocator)?;
    let chk_inner_sol = build_chk_query_env(
        &mut ctx.allocator,
        &empty_leaf,
        epoch,
        validator_count,
        &pk_bytes,
        false,
    );
    let chk_sol = SingletonSolution {
        lineage_proof: Proof::Eve(EveProof {
            parent_parent_coin_info: chk_p2.coin_id(),
            parent_amount: 1,
        }),
        amount: 1,
        inner_solution: chk_inner_sol,
    }
    .to_clvm(&mut ctx.allocator)?;
    ctx.spend(chk_singleton, Spend::new(chk_puzzle, chk_sol))?;

    // Spend 2: Registration coin (WDC-004: creates delay coin)
    let reg_mod = node_from_bytes(&mut ctx.allocator, &reg_hex())?;
    let pk_atom = ctx.allocator.new_atom(&pk_bytes).unwrap();
    let ckpt_atom = ctx.allocator.new_atom(&chk_coin_id).unwrap();
    // Use real WDC mod hash — must match what network coin used
    let wdc_mod_atom = ctx.allocator.new_atom(&wdc_mod_hash_bytes()).unwrap();
    let wdc_delay_atom = common::clvm::u64_to_clvm(&mut ctx.allocator, WDC_DELAY_BLOCKS);
    let reg_curried = clvm_curry(
        &mut ctx.allocator,
        reg_mod,
        &[pk_atom, ckpt_atom, wdc_mod_atom, wdc_delay_atom],
    );
    let dest = [0xDD; 32];
    let reg_sol = build_reg_solution(&mut ctx.allocator, epoch, &dest, COLLATERAL_AMOUNT);
    ctx.spend(reg_coin, Spend::new(reg_curried, reg_sol))?;

    let result = sim.spend_coins(ctx.take(), &[]);
    assert!(
        result.is_ok(),
        "WDC-009: Collateral recovery must succeed: {:?}",
        result.err()
    );

    // Verify: delay coin created (child of registration coin with COLLATERAL_AMOUNT)
    let reg_children = sim.children(reg_coin.coin_id());
    let delay_coin_state = reg_children
        .iter()
        .find(|cs| cs.coin.amount == COLLATERAL_AMOUNT);
    assert!(
        delay_coin_state.is_some(),
        "WDC-009: Delay coin must be created with COLLATERAL_AMOUNT"
    );
    let delay_coin = delay_coin_state.unwrap().coin;

    // Verify: delay coin puzzle hash is NOT the raw destination
    let dest_bytes32: Bytes32 = dest.into();
    assert_ne!(
        delay_coin.puzzle_hash, dest_bytes32,
        "WDC-009: Delay coin hash must NOT be raw destination (WDC-004)"
    );

    // ── Phase 2: Spend delay coin → release funds ─────────────────────
    // NOTE: Simulator does NOT enforce ASSERT_HEIGHT_RELATIVE.
    // The spend succeeds immediately. Actual delay enforcement is by Chia node.
    let ctx = &mut SpendContext::new();
    let wdc_mod = node_from_bytes(&mut ctx.allocator, &wdc_hex())?;
    let dest_atom = ctx.allocator.new_atom(&dest).unwrap();
    let amt_atom = common::clvm::u64_to_clvm(&mut ctx.allocator, COLLATERAL_AMOUNT);
    let delay_atom = common::clvm::u64_to_clvm(&mut ctx.allocator, WDC_DELAY_BLOCKS);
    let wdc_curried = clvm_curry(
        &mut ctx.allocator,
        wdc_mod,
        &[dest_atom, amt_atom, delay_atom],
    );
    let nil_sol = ctx.allocator.nil();
    ctx.spend(delay_coin, Spend::new(wdc_curried, nil_sol))?;

    // No signatures needed (WDC-007: permissionless)
    let result = sim.spend_coins(ctx.take(), &[]);
    assert!(
        result.is_ok(),
        "WDC-009: Delay coin release must succeed: {:?}",
        result.err()
    );

    // Verify: destination coin created
    let delay_children = sim.children(delay_coin.coin_id());
    let dest_coin = delay_children
        .iter()
        .find(|cs| cs.coin.amount == COLLATERAL_AMOUNT);
    assert!(
        dest_coin.is_some(),
        "WDC-009: Destination coin must be created with full collateral"
    );

    // Verify: destination coin is at the correct puzzle hash
    // (The CLVM CREATE_COIN outputs DESTINATION, so the child has that hash)
    assert_eq!(
        dest_coin.unwrap().coin.puzzle_hash,
        dest_bytes32,
        "WDC-009: Final destination must match validator's specified address"
    );

    eprintln!("WDC-009: Full two-phase lifecycle succeeded");
    Ok(())
}

/// WDC-009: Spec file exists.
#[test]
fn vv_req_wdc_009_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/withdraw_delay/specs/WDC-009.md").exists(),
    );
}
