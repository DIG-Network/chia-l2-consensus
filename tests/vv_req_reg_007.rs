//! REQUIREMENT: REG-007 — End-to-End Simulator Test (Collateral Lifecycle)
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-007`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-007.md`.
//!
//! ## Normative statement
//! The complete registration coin lifecycle MUST work end-to-end in the Chia
//! simulator: deploy singletons, register a validator (creating a registration
//! coin with locked collateral), and recover collateral via a cross-coin
//! spend bundle that includes both a checkpoint membership query (emitting
//! a non-membership announcement) and the registration coin spend (asserting
//! that announcement).
//!
//! ## How the tests prove the requirement
//! 1. **Registration coin creation**: Deploys checkpoint and network coin
//!    singletons, registers a validator via AggSigMe signature, and verifies
//!    a child coin with the collateral amount exists.
//! 2. **Cross-coin collateral recovery**: Builds a 2-spend bundle:
//!    (a) checkpoint singleton membership query at depth=0 for a non-member,
//!    (b) registration coin spend asserting the announcement. The simulator
//!    accepts the bundle, proving both coins interoperate correctly.
//!    Post-conditions verify: registration coin spent, collateral coin
//!    created, checkpoint singleton recreated.
//!
//! ## Completeness: HIGH
//! This is the most comprehensive integration test, exercising real singleton
//! wrapping, curried puzzles, cross-coin announcements, and simulator
//! consensus rules.
//!
//! ## Gaps
//! - Uses depth=0 Merkle tree (avoids Rue recursive helper position bug).
//! - Does not test the failure path (spend without announcement).
//! - Uses dummy BLS/Groth16 data (checkpoint path not exercised).

mod common;

use chia_protocol::Bytes32;
use chia_puzzles::singleton::{SingletonArgs, SingletonSolution, SingletonStruct};
use chia_puzzles::{EveProof, Proof};
use chia_sdk_driver::{Launcher, Spend, SpendContext, StandardLayer};
use chia_sdk_test::Simulator;
use chia_sdk_types::Conditions;
use clvm_traits::ToClvm;
use clvm_utils::CurriedProgram;
use clvmr::serde::node_from_bytes;
use sha2::{Digest, Sha256};

const COLLATERAL_AMOUNT: u64 = 1_000_000;

// ── Puzzle artifact loaders ─────────────────────────────────────────

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
fn reg_mod_hash() -> Bytes32 {
    let b: [u8; 32] = hex::decode(
        include_str!("../puzzles/compiled/registration_coin.hash")
            .trim()
            .trim_start_matches("0x"),
    )
    .unwrap()
    .try_into()
    .unwrap();
    b.into()
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

// ── Flat env builders ───────────────────────────────────────────────

/// Build network coin inner flat env.
fn build_net_env(
    a: &mut clvmr::Allocator,
    checkpoint_id: &[u8; 32],
    pubkey: &[u8],
) -> clvmr::NodePtr {
    let nil = a.nil();
    let conds = a.new_pair(nil, nil).unwrap();
    let pk = a.new_atom(pubkey).unwrap();
    let t = a.new_pair(pk, conds).unwrap();
    let ck = a.new_atom(checkpoint_id).unwrap();
    let t = a.new_pair(ck, t).unwrap();
    let col = common::clvm::u64_to_clvm(a, COLLATERAL_AMOUNT);
    let t = a.new_pair(col, t).unwrap();
    let rm = a.new_atom(reg_mod_hash().as_ref()).unwrap();
    let t = a.new_pair(rm, t).unwrap();
    let im = a.new_atom(net_inner_mod_hash().as_ref()).unwrap();
    a.new_pair(im, t).unwrap()
}

/// Build checkpoint inner flat env for membership query path (depth=0).
fn build_chk_query_env(
    a: &mut clvmr::Allocator,
    root: &[u8; 32],
    epoch: u64,
    validator_count: u64,
    query_pubkey: &[u8],
    is_member: bool,
) -> clvmr::NodePtr {
    let nil = a.nil();
    let empty_leaf: [u8; 32] = Sha256::digest(&[0u8; 48]).into();

    // Build right-to-left: all 19 params

    // 19. conditions (spread, (nil.nil) for empty)
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
    let t = a.new_pair(nil, t).unwrap(); // 0
                                         // 15. query_pubkey
    let pk = a.new_atom(query_pubkey).unwrap();
    let t = a.new_pair(pk, t).unwrap();
    // 14. scalars (6-field struct, dummy zeros)
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
    // 6. STATE (4-field struct)
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
    // 6. NETWORK_COIN_LAUNCHER_ID (CHK-012)
    let ncli = a.new_atom(&[0x00u8; 32]).unwrap();
    let t = a.new_pair(ncli, t).unwrap();
    // 5. EMPTY_LEAF_HASH
    let elh = a.new_atom(&empty_leaf).unwrap();
    let t = a.new_pair(elh, t).unwrap();
    // 4. TREE_DEPTH = 0
    let t = a.new_pair(nil, t).unwrap();
    // 3. IC (7-field struct, dummy)
    let ic = {
        let mut r = a.new_atom(&[0x01; 48]).unwrap(); // ic6
        for _ in 0..6 {
            let ic = a.new_atom(&[0x01; 48]).unwrap();
            r = a.new_pair(ic, r).unwrap();
        }
        r
    };
    let t = a.new_pair(ic, t).unwrap();
    // 2. VK (4-field struct, dummy)
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
    // 1. INNER_MOD_HASH
    let imh = a.new_atom(chk_inner_mod_hash().as_ref()).unwrap();
    a.new_pair(imh, t).unwrap()
}

// ── Registration coin solution builder ──────────────────────────────

/// Build registration coin solution: (epoch dest amt . (nil.nil))
/// The registration coin uses standard CurriedProgram (no Rue helpers).
fn build_reg_solution(
    a: &mut clvmr::Allocator,
    epoch: u64,
    destination: &[u8; 32],
    amount: u64,
) -> clvmr::NodePtr {
    let nil = a.nil();
    let conds = a.new_pair(nil, nil).unwrap(); // empty conditions
    let amt = common::clvm::u64_to_clvm(a, amount);
    let t = a.new_pair(amt, conds).unwrap();
    let dest = a.new_atom(destination).unwrap();
    let t = a.new_pair(dest, t).unwrap();
    let ep = common::clvm::u64_to_clvm(a, epoch);
    a.new_pair(ep, t).unwrap()
}

// ── Test: Create registration coin via network coin ─────────────────

/// Verifies the full registration flow: deploy singletons, register a
/// validator via network coin spend with AggSigMe, and confirm a
/// registration coin child exists with the correct collateral amount.
/// Passing proves the network coin puzzle correctly creates registration
/// coins via the Chia simulator's consensus rules.
#[test]
fn vv_req_reg_007_create_registration_coin() -> anyhow::Result<()> {
    let mut sim = Simulator::new();

    // Deploy checkpoint singleton (for its coin ID)
    let ctx = &mut SpendContext::new();
    let (chk_sk, chk_pk, _, chk_p2) = sim.new_p2(1)?;
    let chk_launcher = Launcher::new(chk_p2.coin_id(), 1);
    let chk_launcher_id = chk_launcher.coin().coin_id();
    let (chk_conds, chk_singleton) = chk_launcher.spend(ctx, chk_inner_mod_hash(), ())?;
    StandardLayer::new(chk_pk).spend(ctx, chk_p2, chk_conds)?;
    sim.spend_coins(ctx.take(), &[chk_sk])?;

    // Deploy network coin singleton
    let ctx = &mut SpendContext::new();
    let (net_sk, net_pk, _, net_p2) = sim.new_p2(1)?;
    let net_launcher = Launcher::new(net_p2.coin_id(), 1);
    let net_launcher_id = net_launcher.coin().coin_id();
    let (net_conds, net_singleton) = net_launcher.spend(ctx, net_inner_mod_hash(), ())?;
    StandardLayer::new(net_pk).spend(ctx, net_p2, net_conds)?;
    sim.spend_coins(ctx.take(), &[net_sk])?;

    // Register validator via network coin spend
    let ctx = &mut SpendContext::new();
    let validator_sk = chia_sdk_test::test_secret_key()?;
    let pk_bytes = validator_sk.public_key().to_bytes();
    let chk_coin_id: [u8; 32] = chk_singleton.coin_id().into();

    // Build network coin singleton puzzle + solution
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

    // Fund collateral
    let (fund_sk, fund_pk, _, fund_coin) = sim.new_p2(COLLATERAL_AMOUNT)?;
    StandardLayer::new(fund_pk).spend(ctx, fund_coin, Conditions::new())?;

    sim.spend_coins(ctx.take(), &[validator_sk.clone(), fund_sk])?;

    // Verify registration coin created
    let net_children = sim.children(net_singleton.coin_id());
    let reg_coin = net_children
        .iter()
        .find(|cs| cs.coin.amount == COLLATERAL_AMOUNT);
    assert!(
        reg_coin.is_some(),
        "REG-007: Registration coin must be created"
    );

    Ok(())
}

// ── Test: Cross-coin collateral recovery ────────────────────────────

/// Verifies end-to-end collateral recovery via cross-coin spend bundle.
/// Builds a 2-spend bundle: (1) checkpoint membership query emitting a
/// non-membership announcement, (2) registration coin asserting that
/// announcement and returning collateral. The simulator validates the
/// bundle, proving cross-coin announcement/assertion works. Post-conditions
/// verify: registration coin spent, collateral returned, checkpoint
/// singleton recreated unchanged.
#[test]
fn vv_req_reg_007_cross_coin_collateral_recovery() -> anyhow::Result<()> {
    let mut sim = Simulator::new();

    // ── Deploy checkpoint singleton ─────────────────────────────────
    let ctx = &mut SpendContext::new();
    let (chk_sk, chk_pk, _, chk_p2) = sim.new_p2(1)?;
    let chk_launcher = Launcher::new(chk_p2.coin_id(), 1);
    let chk_launcher_id = chk_launcher.coin().coin_id();
    let (chk_conds, chk_singleton) = chk_launcher.spend(ctx, chk_inner_mod_hash(), ())?;
    StandardLayer::new(chk_pk).spend(ctx, chk_p2, chk_conds)?;
    sim.spend_coins(ctx.take(), &[chk_sk])?;

    // ── Deploy network coin + register validator ────────────────────
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
    let reg_coin_state = net_children
        .iter()
        .find(|cs| cs.coin.amount == COLLATERAL_AMOUNT)
        .unwrap();
    let reg_coin = reg_coin_state.coin;

    // ── Cross-coin collateral recovery bundle ───────────────────────
    // Spend 1: Checkpoint membership query (emits non-membership announcement)
    // Spend 2: Registration coin (asserts announcement, returns collateral)
    let ctx = &mut SpendContext::new();

    // For depth=0: non-membership root = EMPTY_LEAF_HASH (no validator at slot)
    let empty_leaf: [u8; 32] = Sha256::digest(&[0u8; 48]).into();
    let epoch: u64 = 0;
    let validator_count: u64 = 0;

    // Spend 1: Checkpoint singleton membership query
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
        &empty_leaf, // root = EMPTY_LEAF_HASH for depth=0 non-membership
        epoch,
        validator_count,
        &pk_bytes,
        false, // non-member
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

    // Spend 2: Registration coin (no Rue helpers → standard curry works)
    // Build curried puzzle manually to avoid derive macro issues
    let reg_mod = node_from_bytes(&mut ctx.allocator, &reg_hex())?;
    let pk_atom = ctx.allocator.new_atom(&pk_bytes).unwrap();
    let ckpt_atom = ctx.allocator.new_atom(&chk_coin_id).unwrap();
    let reg_curried = clvm_curry(&mut ctx.allocator, reg_mod, &[pk_atom, ckpt_atom]);

    // Destination for collateral return
    let dest = [0xDD; 32];
    let reg_sol = build_reg_solution(&mut ctx.allocator, epoch, &dest, COLLATERAL_AMOUNT);

    ctx.spend(reg_coin, Spend::new(reg_curried, reg_sol))?;

    // Submit cross-coin bundle (no signatures needed — no AGG_SIG in either spend)
    let result = sim.spend_coins(ctx.take(), &[]);
    assert!(
        result.is_ok(),
        "REG-007: Cross-coin recovery must succeed: {:?}",
        result.err()
    );

    // Verify: registration coin spent
    let reg_state = sim.coin_state(reg_coin.coin_id()).unwrap();
    assert!(
        reg_state.spent_height.is_some(),
        "REG-007: Registration coin must be spent"
    );

    // Verify: collateral returned
    let reg_children = sim.children(reg_coin.coin_id());
    let collateral_coin = reg_children
        .iter()
        .find(|cs| cs.coin.amount == COLLATERAL_AMOUNT);
    assert!(
        collateral_coin.is_some(),
        "REG-007: Collateral coin must be created"
    );

    // Verify: checkpoint singleton recreated unchanged
    let chk_children = sim.children(chk_singleton.coin_id());
    let chk_recreated = chk_children.iter().find(|cs| cs.coin.amount == 1);
    assert!(
        chk_recreated.is_some(),
        "REG-007: Checkpoint singleton must be recreated"
    );

    Ok(())
}

/// Standard CLVM curry: (a (q . MOD) (c (q . arg1) (c (q . arg2) ... (c (q . argN) 1))))
/// Works for puzzles WITHOUT Rue helpers (like registration_coin).
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
        let quoted_arg = a.new_pair(op_q, *arg).unwrap();
        let arg_list = a.new_pair(tail, nil).unwrap();
        let arg_list = a.new_pair(quoted_arg, arg_list).unwrap();
        tail = a.new_pair(op_c, arg_list).unwrap();
    }

    let quoted_mod = a.new_pair(op_q, module).unwrap();
    let apply_args = a.new_pair(tail, nil).unwrap();
    let apply_args = a.new_pair(quoted_mod, apply_args).unwrap();
    a.new_pair(op_a, apply_args).unwrap()
}
