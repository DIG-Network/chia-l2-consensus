//! REQUIREMENT: NET-006 — End-to-End Simulator Test
//! (`docs/requirements/domains/network_coin/NORMATIVE.md#NET-006`).
//!
//! Spec: `docs/requirements/domains/network_coin/specs/NET-006.md`.
//!
//! ## Normative Statement
//!
//! All prior NET requirements (NET-001 through NET-005) must be verified
//! end-to-end using the chia-sdk-test Simulator. The inner puzzle is stored
//! UNCURRIED; all params (curried + solution) are passed as a flat list in
//! inner_solution. The singleton wrapper calls `(a MODULE inner_solution)`.
//! This is the definitive integration test that proves the inner puzzle works
//! correctly inside a real singleton on a real (simulated) blockchain.
//!
//! ## How These Tests Prove the Requirement
//!
//! Three tests at increasing levels:
//! 1. `inner_puzzle_flat_env` -- Executes compiled CLVM with a flat env,
//!    verifying AGG_SIG_ME + 2 CREATE_COINs (registration + recreation).
//! 2. `deploy_singleton` -- Deploys the network coin via Launcher, verifying
//!    the singleton exists with 1 mojo and only 1 child from the launcher.
//! 3. `register_validator` -- Full registration flow: singleton spend with
//!    eve proof, BLS signature, collateral funding. Verifies recreation (1 mojo
//!    child), registration coin (COLLATERAL_AMOUNT child), and old singleton
//!    spent.
//! 4. `sequential_registrations` -- Three consecutive registrations with
//!    lineage proofs, verifying continuous singleton recreation.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Network coin deployed as singleton via chia-wallet-sdk
//! - [x] Validator registration spend accepted with real BLS signature
//! - [x] Registration coin created with correct collateral amount
//! - [x] Network coin recreated as new singleton coin (1 mojo)
//! - [x] Sequential registrations succeed with continuous lineage
//! - [x] Old singleton spent after registration
//! - [x] Inner puzzle flat-env test: AGG_SIG_ME + 2 CREATE_COINs
//! - [ ] Pubkey memo visible in coin record (memo not checked)
//! - [ ] Wrong signature rejected (not tested)
//! - [ ] Insufficient collateral rejected (not tested)
//!
//! ## Gaps
//!
//! - Negative cases (wrong signature, insufficient collateral) are not tested.
//! - Pubkey memo in the coin record is not explicitly checked.
//! - The puzzle hash of the recreated singleton is not explicitly compared
//!   to the original (only amount=1 is verified).

mod common;

use chia_protocol::Bytes32;
use chia_puzzles::singleton::{SingletonArgs, SingletonSolution, SingletonStruct};
use chia_puzzles::{EveProof, LineageProof, Proof};
use chia_sdk_driver::{Launcher, Spend, SpendContext, StandardLayer};
use chia_sdk_test::Simulator;
use chia_sdk_types::Conditions;
use clvm_traits::ToClvm;
use clvm_utils::CurriedProgram;
use clvmr::serde::node_from_bytes;

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

/// Build flat env for the network coin inner puzzle (all params as right-linked list).
/// WDC-004: Now 6 curried + 2 solution = 8 params.
/// Layout: (INNER_MOD_HASH reg_mod_hash collateral ckpt_id wdc_mod wdc_delay pubkey . conditions)
fn build_flat_env(
    a: &mut clvmr::Allocator,
    inner_mod_hash: &[u8],
    reg_hash: &[u8],
    collateral: u64,
    ckpt_id: &[u8],
    wdc_mod_hash: &[u8],
    wdc_delay: u64,
    pubkey: &[u8],
) -> clvmr::NodePtr {
    // Build right-to-left.
    // CRITICAL: Rue spread params (conditions: List<Condition>) compile to
    // a FIRST access, so empty conditions must be a pair (nil . nil), not bare nil.
    // This matches the harness_smoke_test pattern.
    let nil = a.nil();
    let conds = a.new_pair(nil, nil).unwrap(); // (nil . nil) for empty conditions
    let pk = a.new_atom(pubkey).unwrap();
    let t = a.new_pair(pk, conds).unwrap();
    // WDC-004: Add withdraw delay params
    let delay = common::clvm::u64_to_clvm(a, wdc_delay);
    let t = a.new_pair(delay, t).unwrap();
    let wdm = a.new_atom(wdc_mod_hash).unwrap();
    let t = a.new_pair(wdm, t).unwrap();
    let ck = a.new_atom(ckpt_id).unwrap();
    let t = a.new_pair(ck, t).unwrap();
    // Collateral as CLVM signed integer
    let col = common::clvm::u64_to_clvm(a, collateral);
    let t = a.new_pair(col, t).unwrap();
    let rm = a.new_atom(reg_hash).unwrap();
    let t = a.new_pair(rm, t).unwrap();
    let im = a.new_atom(inner_mod_hash).unwrap();
    a.new_pair(im, t).unwrap()
}

// ── Step 2: Verify puzzle with flat env (no singleton) ──────────────

// Executes the compiled inner puzzle CLVM with a flat environment containing
// all parameters. Verifies the output includes AGG_SIG_ME (opcode 50) and
// exactly 2 CREATE_COINs (opcode 51): one with COLLATERAL_AMOUNT (registration)
// and one with amount=1 (singleton recreation). This proves the inner puzzle
// produces the correct conditions before wrapping.
#[test]
fn vv_req_net_006_inner_puzzle_flat_env() {
    let mut a = clvmr::Allocator::new();
    let puzzle = node_from_bytes(&mut a, &net_inner_hex()).unwrap();

    let imh: [u8; 32] = net_inner_mod_hash().into();
    let rmh: [u8; 32] = reg_mod_hash().into();

    let env = build_flat_env(
        &mut a,
        &imh,
        &rmh,
        COLLATERAL_AMOUNT,
        &[0x33u8; 32],
        &[0x55u8; 32], // WDC-004: withdraw_delay_mod_hash
        24_000,        // WDC-004: withdraw_delay_blocks
        &[0x44u8; 48],
    );

    let result = clvmr::run_program(
        &mut a,
        &clvmr::ChiaDialect::new(0),
        puzzle,
        env,
        11_000_000_000,
    );
    match &result {
        Ok(clvmr::reduction::Reduction(cost, output)) => {
            let conditions = common::clvm::parse_conditions(&a, *output);
            // Should have: AGG_SIG_ME (50), 2x CREATE_COIN (51)
            assert!(
                common::clvm::has_opcode(&conditions, 50),
                "NET-006: Must emit AGG_SIG_ME"
            );
            let create_coins = common::clvm::conditions_with_opcode(&conditions, 51);
            assert_eq!(
                create_coins.len(),
                2,
                "NET-006: Must emit 2 CREATE_COINs (registration + recreation)"
            );
            // One CREATE_COIN with amount=COLLATERAL (registration)
            // One CREATE_COIN with amount=1 (singleton recreation)
            let amounts: Vec<u64> = create_coins
                .iter()
                .map(|c| {
                    let bytes = &c.args[1];
                    if bytes.is_empty() {
                        return 0;
                    }
                    let mut padded = vec![0u8; 8 - bytes.len()];
                    padded.extend_from_slice(bytes);
                    u64::from_be_bytes(padded.try_into().unwrap())
                })
                .collect();
            assert!(
                amounts.contains(&COLLATERAL_AMOUNT),
                "NET-006: Registration coin amount"
            );
            assert!(amounts.contains(&1), "NET-006: Recreation amount = 1");
            eprintln!(
                "Inner puzzle OK! Cost: {}, conditions: {}",
                cost,
                conditions.len()
            );
        }
        Err(e) => panic!("NET-006: Inner puzzle FAILED: {}", e.1),
    }
}

// ── Step 3: Deploy singleton ────────────────────────────────────────

// Deploys the network coin as a singleton via the chia-wallet-sdk Launcher.
// Verifies: the singleton coin exists with 1 mojo, the launcher coin was
// spent and produced exactly 1 child. This proves NET-001 (singleton identity)
// at the simulator level.
#[test]
fn vv_req_net_006_deploy_singleton() -> anyhow::Result<()> {
    let mut sim = Simulator::new();
    let ctx = &mut SpendContext::new();

    // Inner puzzle = uncurried module. Hash = INNER_MOD_HASH.
    let inner_ph = net_inner_mod_hash();

    let (sk, pk, _, p2_coin) = sim.new_p2(1)?;
    let launcher = Launcher::new(p2_coin.coin_id(), 1);
    let launcher_id = launcher.coin().coin_id();
    let (conds, singleton) = launcher.spend(ctx, inner_ph, ())?;
    StandardLayer::new(pk).spend(ctx, p2_coin, conds)?;
    sim.spend_coins(ctx.take(), &[sk])?;

    assert!(
        sim.coin_state(singleton.coin_id()).is_some(),
        "NET-006: Singleton exists"
    );
    assert_eq!(singleton.amount, 1);
    assert_eq!(sim.children(launcher_id).len(), 1);
    Ok(())
}

// ── Step 4: Register validator via singleton ────────────────────────

// Full registration flow: deploy singleton, then spend it with a validator's
// BLS key to register. Uses the eve proof for lineage, builds the singleton
// outer puzzle via CurriedProgram, constructs the flat inner solution, signs
// with the validator's secret key, and funds collateral. Verifies: singleton
// recreated (1 mojo child), registration coin created (COLLATERAL_AMOUNT
// child), and original singleton spent. This is the definitive end-to-end
// proof of NET-001 through NET-004.
#[test]
fn vv_req_net_006_register_validator() -> anyhow::Result<()> {
    let mut sim = Simulator::new();
    let checkpoint_id = Bytes32::from([0x22u8; 32]);

    // Deploy
    let ctx = &mut SpendContext::new();
    let inner_ph = net_inner_mod_hash();
    let (p2_sk, p2_pk, _, p2_coin) = sim.new_p2(1)?;
    let launcher = Launcher::new(p2_coin.coin_id(), 1);
    let launcher_id = launcher.coin().coin_id();
    let (conds, singleton) = launcher.spend(ctx, inner_ph, ())?;
    StandardLayer::new(p2_pk).spend(ctx, p2_coin, conds)?;
    sim.spend_coins(ctx.take(), &[p2_sk])?;

    // ── Register ────────────────────────────────────────────────────
    let ctx = &mut SpendContext::new();
    let validator_sk = chia_sdk_test::test_secret_key()?;
    let pk_bytes = validator_sk.public_key().to_bytes();

    // Load uncurried inner puzzle
    let inner_mod = node_from_bytes(&mut ctx.allocator, &net_inner_hex())?;

    // Build singleton outer puzzle: singleton_top_layer curried with (STRUCT, inner_mod)
    let singleton_mod = ctx.singleton_top_layer()?;
    let singleton_puzzle = CurriedProgram {
        program: singleton_mod,
        args: SingletonArgs {
            singleton_struct: SingletonStruct::new(launcher_id),
            inner_puzzle: inner_mod,
        },
    }
    .to_clvm(&mut ctx.allocator)?;

    // Build inner solution: flat list of ALL params
    let imh: [u8; 32] = net_inner_mod_hash().into();
    let rmh: [u8; 32] = reg_mod_hash().into();
    let ckpt: [u8; 32] = checkpoint_id.into();
    let inner_sol = build_flat_env(
        &mut ctx.allocator,
        &imh,
        &rmh,
        COLLATERAL_AMOUNT,
        &ckpt,
        &[0x55u8; 32], // WDC-004: withdraw_delay_mod_hash
        24_000,        // WDC-004: withdraw_delay_blocks
        &pk_bytes,
    );

    // Build singleton solution: eve proof
    let singleton_sol = SingletonSolution {
        lineage_proof: Proof::Eve(EveProof {
            parent_parent_coin_info: p2_coin.coin_id(),
            parent_amount: 1,
        }),
        amount: 1,
        inner_solution: inner_sol,
    }
    .to_clvm(&mut ctx.allocator)?;

    ctx.spend(singleton, Spend::new(singleton_puzzle, singleton_sol))?;

    // Fund collateral
    let (fund_sk, fund_pk, _, fund_coin) = sim.new_p2(COLLATERAL_AMOUNT)?;
    StandardLayer::new(fund_pk).spend(ctx, fund_coin, Conditions::new())?;

    let result = sim.spend_coins(ctx.take(), &[validator_sk.clone(), fund_sk]);
    assert!(
        result.is_ok(),
        "NET-006: Registration must succeed: {:?}",
        result.err()
    );

    // Verify children of singleton spend
    let children = sim.children(singleton.coin_id());
    assert!(
        children.iter().any(|cs| cs.coin.amount == 1),
        "NET-006: Singleton recreated with 1 mojo"
    );
    assert!(
        children
            .iter()
            .any(|cs| cs.coin.amount == COLLATERAL_AMOUNT),
        "NET-006: Registration coin created"
    );

    // Verify old singleton spent
    let old = sim.coin_state(singleton.coin_id()).unwrap();
    assert!(
        old.spent_height.is_some(),
        "NET-006: Original singleton spent"
    );

    Ok(())
}

// ── Sequential registrations ────────────────────────────────────────

// Registers 3 validators sequentially, each using the recreated singleton
// from the previous spend. Builds correct lineage proofs (eve for first,
// LineageProof for subsequent). Verifies each registration succeeds and the
// singleton is recreated each time. This proves NET-004 (continuous self-
// recreation) and the singleton's indefinite availability.
#[test]
fn vv_req_net_006_sequential_registrations() -> anyhow::Result<()> {
    let mut sim = Simulator::new();
    let checkpoint_id = Bytes32::from([0x22u8; 32]);

    // Deploy
    let ctx = &mut SpendContext::new();
    let inner_ph = net_inner_mod_hash();
    let (p2_sk, p2_pk, _, p2_coin) = sim.new_p2(1)?;
    let launcher = Launcher::new(p2_coin.coin_id(), 1);
    let launcher_id = launcher.coin().coin_id();
    let (conds, singleton) = launcher.spend(ctx, inner_ph, ())?;
    StandardLayer::new(p2_pk).spend(ctx, p2_coin, conds)?;
    sim.spend_coins(ctx.take(), &[p2_sk])?;

    let mut current = singleton;
    let mut parent_proof: Proof = Proof::Eve(EveProof {
        parent_parent_coin_info: p2_coin.coin_id(),
        parent_amount: 1,
    });

    for i in 1..=3u32 {
        let ctx = &mut SpendContext::new();
        let val_sk = {
            let seed = [i as u8; 32];
            chia_bls::SecretKey::from_seed(&seed)
        };
        let pk_bytes = val_sk.public_key().to_bytes();

        let inner_mod = node_from_bytes(&mut ctx.allocator, &net_inner_hex())?;
        let singleton_mod = ctx.singleton_top_layer()?;
        let singleton_puzzle = CurriedProgram {
            program: singleton_mod,
            args: SingletonArgs {
                singleton_struct: SingletonStruct::new(launcher_id),
                inner_puzzle: inner_mod,
            },
        }
        .to_clvm(&mut ctx.allocator)?;

        let imh: [u8; 32] = net_inner_mod_hash().into();
        let rmh: [u8; 32] = reg_mod_hash().into();
        let ckpt: [u8; 32] = checkpoint_id.into();
        let inner_sol = build_flat_env(
            &mut ctx.allocator,
            &imh,
            &rmh,
            COLLATERAL_AMOUNT,
            &ckpt,
            &[0x55u8; 32], // WDC-004: withdraw_delay_mod_hash
            24_000,        // WDC-004: withdraw_delay_blocks
            &pk_bytes,
        );

        let singleton_sol = SingletonSolution {
            lineage_proof: parent_proof,
            amount: 1,
            inner_solution: inner_sol,
        }
        .to_clvm(&mut ctx.allocator)?;

        ctx.spend(current, Spend::new(singleton_puzzle, singleton_sol))?;

        let (fund_sk, fund_pk, _, fund_coin) = sim.new_p2(COLLATERAL_AMOUNT)?;
        StandardLayer::new(fund_pk).spend(ctx, fund_coin, Conditions::new())?;

        let result = sim.spend_coins(ctx.take(), &[val_sk, fund_sk]);
        assert!(
            result.is_ok(),
            "NET-006: Registration {} must succeed: {:?}",
            i,
            result.err()
        );

        let children = sim.children(current.coin_id());
        let recreated = children
            .iter()
            .find(|cs| cs.coin.amount == 1)
            .unwrap_or_else(|| panic!("NET-006: Singleton recreated after registration {}", i));

        // Build lineage proof for next spend
        parent_proof = Proof::Lineage(LineageProof {
            parent_parent_coin_info: current.parent_coin_info,
            parent_inner_puzzle_hash: inner_ph,
            parent_amount: 1,
        });

        current = recreated.coin;
    }

    Ok(())
}
