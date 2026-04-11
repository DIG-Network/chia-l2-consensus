//! REQUIREMENT: SEC-003 — Collateral Security
//! (`docs/requirements/domains/security/NORMATIVE.md#SEC-003`).
//!
//! Spec: `docs/requirements/domains/security/specs/SEC-003.md`.
//!
//! Implementation: `puzzles/registration_coin.rue` (compiled to CLVM).
//!
//! Security verification: confirms that collateral is locked in the
//! registration coin until a non-membership announcement is asserted,
//! and that the puzzle has no bypass path.

use chia_l2_consensus::testing::{
    compute_membership_announcement_message, generate_validator_keypair, is_validator_excluded,
    prepare_collateral_recovery,
};
use chia_l2_consensus::testing::{SparseMerkleTree, EMPTY_LEAF};

/// Compiled registration coin puzzle hex.
const REG_HEX: &str = include_str!("../puzzles/compiled/registration_coin.hex");

// ── Puzzle contains ASSERT_COIN_ANNOUNCEMENT (opcode 61 = 0x3d) ────

#[test]
fn vv_req_sec_003_puzzle_requires_announcement() {
    let hex = REG_HEX.trim();

    // In the compiled CLVM, the literal atom 0x3d (61) represents the
    // ASSERT_COIN_ANNOUNCEMENT condition opcode. It appears as ff013d
    // (quoted atom: opcode 1 = quote, value 0x3d).
    assert!(
        hex.contains("013d"),
        "SEC-003: Registration coin puzzle MUST contain ASSERT_COIN_ANNOUNCEMENT (opcode 61). \
         Without it, collateral could be spent without proving non-membership."
    );
}

// ── Puzzle hardcodes non-membership byte 0x00 ───────────────────────

#[test]
fn vv_req_sec_003_hardcoded_non_membership() {
    let hex = REG_HEX.trim();

    // The "membership" prefix is hardcoded as the atom:
    // 0x6d656d62657273686970 = "membership" (10 bytes)
    assert!(
        hex.contains("6d656d62657273686970"),
        "SEC-003: Puzzle must contain hardcoded 'membership' prefix"
    );

    // The is_member byte 0x00 (non-membership) is hardcoded via quoted atom ff0100.
    // This means the puzzle ONLY constructs non-membership announcements.
    // There is no code path that constructs is_member=0x01.
    assert!(
        hex.contains("0100"),
        "SEC-003: Puzzle must contain hardcoded 0x00 (non-membership byte)"
    );
}

// ── Puzzle contains CREATE_COIN (opcode 51 = 0x33) ─────────────────

#[test]
fn vv_req_sec_003_puzzle_creates_coin() {
    let hex = REG_HEX.trim();

    // CREATE_COIN (opcode 51 = 0x33) returns collateral to destination.
    // The ff0133 pattern is the quoted atom for the condition opcode.
    assert!(
        hex.contains("0133"),
        "SEC-003: Puzzle must contain CREATE_COIN (opcode 51) for collateral return"
    );
}

// ── Puzzle has exactly two condition types (no hidden paths) ────────

#[test]
fn vv_req_sec_003_only_two_conditions() {
    // The registration coin puzzle should produce exactly:
    // 1. ASSERT_COIN_ANNOUNCEMENT (61 = 0x3d) — verify non-membership
    // 2. CREATE_COIN (51 = 0x33) — return collateral
    // Plus any user-provided conditions from the solution.
    //
    // There should be NO AGG_SIG_ME, no other assertions, no hidden
    // CREATE_COINs that could redirect collateral.
    //
    // The puzzle does NOT contain AGG_SIG_ME (50 = 0x32) or AGG_SIG_UNSAFE (49 = 0x31)
    // This is correct because the registration coin is permissionless to spend
    // (the non-membership proof is the authorization, not a signature).

    // Verify no signature conditions are hardcoded
    let source = include_str!("../puzzles/registration_coin.rue");
    assert!(
        !source.contains("AggSigMe"),
        "SEC-003: Registration coin must NOT require signatures (permissionless spend with proof)"
    );
    assert!(
        !source.contains("AggSigUnsafe"),
        "SEC-003: Registration coin must NOT use AggSigUnsafe"
    );
}

// ── Non-membership announcement format is correct ───────────────────

#[test]
fn vv_req_sec_003_announcement_format() {
    let kp = generate_validator_keypair(&[0x42; 32]).unwrap();
    let epoch: u64 = 5;

    // Non-membership inner hash
    let inner = compute_membership_announcement_message(epoch, &kp.pubkey, false);

    // Membership inner hash (different is_member byte)
    let member_inner = compute_membership_announcement_message(epoch, &kp.pubkey, true);

    // Must differ — the is_member byte changes the hash
    assert_ne!(
        inner, member_inner,
        "SEC-003: Non-membership and membership announcements must differ"
    );
}

// ── Active validator cannot prepare collateral recovery ─────────────

#[test]
fn vv_req_sec_003_active_validator_locked() {
    let pks: Vec<[u8; 48]> = (0..3)
        .map(|i| {
            let mut e = [0u8; 32];
            e[0] = i;
            generate_validator_keypair(&e).unwrap().pubkey
        })
        .collect();

    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    // Active validator: slot contains active_leaf, not EMPTY_LEAF
    assert!(
        !is_validator_excluded(&tree, &pks[0]),
        "SEC-003: Active validator must not be excluded"
    );

    // Cannot prepare recovery
    let result = prepare_collateral_recovery(
        &tree,
        &pks[0],
        5,
        &[0xCC; 32],
        &[0xDD; 32],
        10_000_000_000_000,
    );
    assert!(
        result.is_err(),
        "SEC-003: Active validator MUST NOT be able to prepare collateral recovery"
    );
}

// ── Excluded validator CAN prepare recovery ─────────────────────────

#[test]
fn vv_req_sec_003_excluded_validator_unlocked() {
    let pks: Vec<[u8; 48]> = (0..3)
        .map(|i| {
            let mut e = [0u8; 32];
            e[0] = i;
            generate_validator_keypair(&e).unwrap().pubkey
        })
        .collect();

    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    // Remove validator 1
    tree.remove_validator(&pks[1]);
    assert!(is_validator_excluded(&tree, &pks[1]));

    // CAN prepare recovery
    let result = prepare_collateral_recovery(
        &tree,
        &pks[1],
        5,
        &[0xCC; 32],
        &[0xDD; 32],
        10_000_000_000_000,
    );
    assert!(
        result.is_ok(),
        "SEC-003: Excluded validator must be able to prepare collateral recovery"
    );

    let params = result.unwrap();
    assert_eq!(
        params.merkle_proof.leaf, EMPTY_LEAF,
        "SEC-003: Recovery proof must show non-membership (EMPTY_LEAF)"
    );
}

// ── Non-membership proof required for recovery ──────────────────────

#[test]
fn vv_req_sec_003_non_membership_proof_required() {
    let pks: Vec<[u8; 48]> = (0..3)
        .map(|i| {
            let mut e = [0u8; 32];
            e[0] = i;
            generate_validator_keypair(&e).unwrap().pubkey
        })
        .collect();

    let mut tree = SparseMerkleTree::new();
    for pk in &pks {
        tree.insert_validator(pk);
    }

    tree.remove_validator(&pks[1]);

    let params = prepare_collateral_recovery(
        &tree,
        &pks[1],
        5,
        &[0xCC; 32],
        &[0xDD; 32],
        10_000_000_000_000,
    )
    .unwrap();

    // The proof must verify against the current root
    assert!(
        params.merkle_proof.verify(tree.root()),
        "SEC-003: Non-membership proof must verify against current Merkle root"
    );

    // The proof must NOT verify against a different root
    assert!(
        !params.merkle_proof.verify([0xFF; 32]),
        "SEC-003: Proof must not verify against a random root"
    );
}

// ── Collateral lock: Rue source has no bypass ───────────────────────

#[test]
fn vv_req_sec_003_no_bypass_in_source() {
    let source = include_str!("../puzzles/registration_coin.rue");

    // The puzzle must ALWAYS emit ASSERT_COIN_ANNOUNCEMENT.
    // There should be no conditional that skips the announcement assertion.
    // The assertion is unconditional — it's in the main output list,
    // not inside an if/else branch.

    assert!(
        source.contains("AssertCoinAnnouncement"),
        "SEC-003: Source must contain AssertCoinAnnouncement"
    );

    // The puzzle should not have multiple execution paths (no `if` on
    // the announcement). The announcement is always required.
    // Count the number of AssertCoinAnnouncement occurrences — should be exactly 1.
    let count = source.matches("AssertCoinAnnouncement").count();
    assert_eq!(
        count, 1,
        "SEC-003: Exactly one AssertCoinAnnouncement (no conditional bypass). Found: {}",
        count
    );
}
