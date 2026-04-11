//! REQUIREMENT: REG-003 — Collateral Lock
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-003`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-003.md`.
//!
//! Implementation: `puzzles/registration_coin.rue` (compiled to CLVM).
//!
//! Verifies that the registration coin holds collateral and MUST NOT be
//! spendable without a non-membership announcement from the checkpoint
//! singleton. Tests execute the compiled CLVM bytecode.
//!
//! NOTE: Full simulator cross-coin announcement testing (checkpoint +
//! registration in same bundle) is blocked until CHK-005 is implemented.
//! These tests verify the CLVM-level lock mechanism: the puzzle ALWAYS
//! emits ASSERT_COIN_ANNOUNCEMENT with the correct non-membership hash.

mod common;

use clvmr::Allocator;
use sha2::{Digest, Sha256};

use common::clvm::*;

const REG_COIN_HEX: &str = include_str!("../puzzles/compiled/registration_coin.hex");

/// Build registration coin env: (PK . (CKPT_ID . (epoch . (dest . (amt . (conds . nil))))))
fn build_env(
    a: &mut Allocator,
    pk: &[u8],
    ckpt_id: &[u8],
    epoch: u64,
    dest: &[u8],
    amt: u64,
) -> clvmr::NodePtr {
    let conds = a.nil();
    let nil = a.nil();
    let t = a.new_pair(conds, nil).unwrap();
    let amt_bytes: Vec<u8> = if amt == 0 {
        vec![]
    } else {
        let b = amt.to_be_bytes();
        b.iter().copied().skip_while(|&x| x == 0).collect()
    };
    let amt_node = a.new_atom(&amt_bytes).unwrap();
    let t = a.new_pair(amt_node, t).unwrap();
    let dest_node = a.new_atom(dest).unwrap();
    let t = a.new_pair(dest_node, t).unwrap();
    let epoch_bytes: Vec<u8> = if epoch == 0 {
        vec![]
    } else {
        let b = epoch.to_be_bytes();
        b.iter().copied().skip_while(|&x| x == 0).collect()
    };
    let epoch_node = a.new_atom(&epoch_bytes).unwrap();
    let t = a.new_pair(epoch_node, t).unwrap();
    let ckpt_node = a.new_atom(ckpt_id).unwrap();
    let t = a.new_pair(ckpt_node, t).unwrap();
    let pk_node = a.new_atom(pk).unwrap();
    a.new_pair(pk_node, t).unwrap()
}

/// Compute the expected announcement hash in Rust (cross-impl reference).
/// inner = sha256("membership" + epoch_be8 + pubkey + is_member_byte)
/// full  = sha256(checkpoint_singleton_id + inner)
fn expected_announcement_hash(ckpt_id: &[u8], epoch: u64, pk: &[u8], is_member: bool) -> [u8; 32] {
    let mut inner = Vec::new();
    inner.extend_from_slice(b"membership");
    inner.extend_from_slice(&epoch.to_be_bytes());
    inner.extend_from_slice(pk);
    inner.push(if is_member { 0x01 } else { 0x00 });
    let inner_hash: [u8; 32] = Sha256::digest(&inner).into();

    let mut full = Vec::new();
    full.extend_from_slice(ckpt_id);
    full.extend_from_slice(&inner_hash);
    Sha256::digest(&full).into()
}

// ── CLVM Execution: Lock is always present ─────────────────────────

#[test]
fn vv_req_reg_003_always_emits_assert_announcement() {
    // REG-003: The puzzle MUST always emit ASSERT_COIN_ANNOUNCEMENT.
    // There is no code path that skips it — this IS the lock.
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 1_000_000);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    assert!(
        has_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT),
        "REG-003: Puzzle MUST always emit ASSERT_COIN_ANNOUNCEMENT (61)"
    );
}

#[test]
fn vv_req_reg_003_exactly_one_assert_announcement() {
    // REG-003: Exactly one ASSERT_COIN_ANNOUNCEMENT — the non-membership check.
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &[0xAA; 48], &[0xBB; 32], 5, &[0xCC; 32], 1_000_000);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    let announcements = conditions_with_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT);
    assert_eq!(
        announcements.len(),
        1,
        "REG-003: Must have exactly 1 ASSERT_COIN_ANNOUNCEMENT"
    );
}

#[test]
fn vv_req_reg_003_announcement_is_non_membership() {
    // REG-003: The announcement hash must correspond to is_member=0x00
    // (non-membership). The puzzle hardcodes 0x00 — there's no way to
    // make it assert a membership=true announcement.
    let pk = [0xAA; 48];
    let ckpt_id = [0xBB; 32];
    let epoch: u64 = 5;

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &pk, &ckpt_id, epoch, &[0xCC; 32], 1_000_000);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let announcements = conditions_with_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT);
    let clvm_hash = &announcements[0].args[0];

    // Must match non-membership hash (is_member=false)
    let expected_non_member = expected_announcement_hash(&ckpt_id, epoch, &pk, false);
    assert_eq!(
        clvm_hash.as_slice(),
        expected_non_member.as_slice(),
        "REG-003: Announcement must be non-membership (is_member=0x00)"
    );

    // Must NOT match membership hash (is_member=true)
    let wrong_member = expected_announcement_hash(&ckpt_id, epoch, &pk, true);
    assert_ne!(
        clvm_hash.as_slice(),
        wrong_member.as_slice(),
        "REG-003: Announcement must NOT match membership=true hash"
    );
}

// ── CLVM Execution: Announcement binds to curried params ───────────

#[test]
fn vv_req_reg_003_announcement_binds_to_pubkey() {
    // REG-003: Different pubkeys produce different announcement hashes.
    // The lock is specific to the validator who registered.
    let mut a = Allocator::new();
    let ckpt_id = [0xBB; 32];

    let puzzle1 = load_puzzle(&mut a, REG_COIN_HEX);
    let env1 = build_env(&mut a, &[0xAA; 48], &ckpt_id, 5, &[0xCC; 32], 1_000_000);
    let (_, out1) = run_puzzle_ok(&mut a, puzzle1, env1);
    let hash1 = conditions_with_opcode(&parse_conditions(&a, out1), ASSERT_COIN_ANNOUNCEMENT)[0]
        .args[0]
        .clone();

    let puzzle2 = load_puzzle(&mut a, REG_COIN_HEX);
    let env2 = build_env(&mut a, &[0x11; 48], &ckpt_id, 5, &[0xCC; 32], 1_000_000);
    let (_, out2) = run_puzzle_ok(&mut a, puzzle2, env2);
    let hash2 = conditions_with_opcode(&parse_conditions(&a, out2), ASSERT_COIN_ANNOUNCEMENT)[0]
        .args[0]
        .clone();

    assert_ne!(
        hash1, hash2,
        "REG-003: Different pubkeys must produce different announcement hashes"
    );
}

#[test]
fn vv_req_reg_003_announcement_binds_to_checkpoint_id() {
    // REG-003: Different checkpoint singleton IDs produce different hashes.
    // The lock is bound to the specific checkpoint singleton.
    let mut a = Allocator::new();
    let pk = [0xAA; 48];

    let puzzle1 = load_puzzle(&mut a, REG_COIN_HEX);
    let env1 = build_env(&mut a, &pk, &[0xBB; 32], 5, &[0xCC; 32], 1_000_000);
    let (_, out1) = run_puzzle_ok(&mut a, puzzle1, env1);
    let hash1 = conditions_with_opcode(&parse_conditions(&a, out1), ASSERT_COIN_ANNOUNCEMENT)[0]
        .args[0]
        .clone();

    let puzzle2 = load_puzzle(&mut a, REG_COIN_HEX);
    let env2 = build_env(&mut a, &pk, &[0x22; 32], 5, &[0xCC; 32], 1_000_000);
    let (_, out2) = run_puzzle_ok(&mut a, puzzle2, env2);
    let hash2 = conditions_with_opcode(&parse_conditions(&a, out2), ASSERT_COIN_ANNOUNCEMENT)[0]
        .args[0]
        .clone();

    assert_ne!(
        hash1, hash2,
        "REG-003: Different checkpoint IDs must produce different announcement hashes"
    );
}

#[test]
fn vv_req_reg_003_announcement_binds_to_epoch() {
    // REG-003: Different epochs produce different announcement hashes.
    // This is part of the replay protection (see REG-006).
    let mut a = Allocator::new();
    let pk = [0xAA; 48];
    let ckpt_id = [0xBB; 32];

    let puzzle1 = load_puzzle(&mut a, REG_COIN_HEX);
    let env1 = build_env(&mut a, &pk, &ckpt_id, 5, &[0xCC; 32], 1_000_000);
    let (_, out1) = run_puzzle_ok(&mut a, puzzle1, env1);
    let hash1 = conditions_with_opcode(&parse_conditions(&a, out1), ASSERT_COIN_ANNOUNCEMENT)[0]
        .args[0]
        .clone();

    let puzzle2 = load_puzzle(&mut a, REG_COIN_HEX);
    let env2 = build_env(&mut a, &pk, &ckpt_id, 6, &[0xCC; 32], 1_000_000);
    let (_, out2) = run_puzzle_ok(&mut a, puzzle2, env2);
    let hash2 = conditions_with_opcode(&parse_conditions(&a, out2), ASSERT_COIN_ANNOUNCEMENT)[0]
        .args[0]
        .clone();

    assert_ne!(
        hash1, hash2,
        "REG-003: Different epochs must produce different announcement hashes"
    );
}

// ── CLVM Execution: Collateral output ──────────────────────────────

#[test]
fn vv_req_reg_003_always_emits_create_coin() {
    // REG-003: Alongside the lock assertion, the puzzle always creates
    // the collateral return coin. Both conditions are unconditional.
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 500_000);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    assert!(
        has_opcode(&conditions, CREATE_COIN),
        "REG-003: Puzzle must always emit CREATE_COIN for collateral return"
    );
}

#[test]
fn vv_req_reg_003_collateral_amount_passthrough() {
    // REG-003: The full collateral amount from the solution is passed
    // through to CREATE_COIN. The puzzle does not enforce a minimum —
    // the driver should set it to the coin's full amount.
    let amount: u64 = 1_000_000_000_000; // 1 XCH

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &[0xAA; 48], &[0xBB; 32], 5, &[0xCC; 32], amount);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let create_coins = conditions_with_opcode(&conditions, CREATE_COIN);
    assert_eq!(create_coins.len(), 1);

    let amt_bytes = &create_coins[0].args[1];
    let mut padded = vec![0u8; 8 - amt_bytes.len()];
    padded.extend_from_slice(amt_bytes);
    let actual = u64::from_be_bytes(padded.try_into().unwrap());
    assert_eq!(
        actual, amount,
        "REG-003: CREATE_COIN amount must equal solution collateral_amount"
    );
}

// ── CLVM Execution: Cross-implementation hash verification ─────────

#[test]
fn vv_req_reg_003_cross_impl_hash_epoch_0() {
    // REG-003: Cross-impl check at epoch 0 (edge case).
    let pk = [0x01; 48];
    let ckpt_id = [0xFF; 32];
    let epoch: u64 = 0;

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &pk, &ckpt_id, epoch, &[0xCC; 32], 1);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let clvm_hash = &conditions_with_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT)[0].args[0];

    let expected = expected_announcement_hash(&ckpt_id, epoch, &pk, false);
    assert_eq!(
        clvm_hash.as_slice(),
        expected.as_slice(),
        "REG-003: Cross-impl hash must match at epoch 0"
    );
}

#[test]
fn vv_req_reg_003_cross_impl_hash_large_epoch() {
    // REG-003: Cross-impl check with large epoch (tests int_to_8_bytes_be).
    let pk = [0x42; 48];
    let ckpt_id = [0xDE; 32];
    let epoch: u64 = 1_000_000;

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &pk, &ckpt_id, epoch, &[0xCC; 32], 1_000_000);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let clvm_hash = &conditions_with_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT)[0].args[0];

    let expected = expected_announcement_hash(&ckpt_id, epoch, &pk, false);
    assert_eq!(
        clvm_hash.as_slice(),
        expected.as_slice(),
        "REG-003: Cross-impl hash must match at epoch 1,000,000"
    );
}

#[test]
fn vv_req_reg_003_cross_impl_hash_max_epoch() {
    // REG-003: Cross-impl at near-max epoch (8-byte boundary).
    let pk = [0x99; 48];
    let ckpt_id = [0x77; 32];
    let epoch: u64 = u64::MAX - 1;

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &pk, &ckpt_id, epoch, &[0xCC; 32], 1);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let clvm_hash = &conditions_with_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT)[0].args[0];

    let expected = expected_announcement_hash(&ckpt_id, epoch, &pk, false);
    assert_eq!(
        clvm_hash.as_slice(),
        expected.as_slice(),
        "REG-003: Cross-impl hash must match at near-max epoch"
    );
}

// ── Exactly two conditions (lock + collateral) ─────────────────────

#[test]
fn vv_req_reg_003_exactly_two_conditions_with_empty_passthrough() {
    // REG-003: With empty conditions passthrough, puzzle produces exactly 2:
    // ASSERT_COIN_ANNOUNCEMENT + CREATE_COIN.
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, &[0xAA; 48], &[0xBB; 32], 1, &[0xCC; 32], 1_000);

    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    assert_eq!(
        conditions.len(),
        2,
        "REG-003: With empty conditions, puzzle must produce exactly 2 conditions, got {}",
        conditions.len()
    );
    assert_eq!(conditions[0].opcode, ASSERT_COIN_ANNOUNCEMENT as i64);
    assert_eq!(conditions[1].opcode, CREATE_COIN as i64);
}

// ── Spec and documentation ─────────────────────────────────────────

#[test]
fn vv_req_reg_003_puzzle_documents_collateral_lock() {
    let src = std::fs::read_to_string("puzzles/registration_coin.rue")
        .expect("Failed to read puzzle source");

    assert!(
        src.contains("REG-003") || src.contains("Collateral") || src.contains("collateral"),
        "REG-003: Puzzle must document the collateral lock mechanism"
    );
}

#[test]
fn vv_req_reg_003_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/registration_coin/specs/REG-003.md")
            .exists(),
        "REG-003: Spec file must exist"
    );
}
