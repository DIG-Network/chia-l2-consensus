//! REQUIREMENT: SMT-005 — Cross-Implementation Consistency (Rust ↔ CLVM)
//!
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-005`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-005.md`.
//!
//! **These tests prove cross-implementation consistency** by:
//! 1. Building a SparseMerkleTree in Rust
//! 2. Generating membership/non-membership proofs in Rust
//! 3. Running the compiled CLVM checkpoint puzzle's membership query path
//!    with those proofs at full TREE_DEPTH=32
//! 4. Verifying the CLVM puzzle accepts the proofs (assert passes) and
//!    emits the correct membership announcement
//!
//! If the Rust tree and the Rue/CLVM `verify_merkle_path` disagree on
//! sibling ordering, leaf hashing, or parent concatenation, the CLVM
//! assert will fail, proving a divergence.
//!
//! Implementation:
//!   Rust: `src/merkle/sparse.rs`, `src/merkle/proof.rs`
//!   Rue:  `puzzles/checkpoint_inner.rue` → `verify_merkle_path()`

mod common;

use chia_l2_consensus::testing::{
    active_leaf, compute_slot, SparseMerkleTree, EMPTY_LEAF, EMPTY_TREE_ROOT, TREE_DEPTH,
};
use common::clvm::{
    has_opcode, load_puzzle, parse_conditions, run_puzzle_ok, u64_to_clvm, CREATE_COIN,
    CREATE_COIN_ANNOUNCEMENT,
};
use sha2::{Digest, Sha256};

const CHK_HEX: &str = include_str!("../puzzles/compiled/checkpoint_inner.hex");

fn compute_empty_leaf_hash() -> [u8; 32] {
    Sha256::digest([0u8; 48]).into()
}

/// Build CLVM env for membership query at full TREE_DEPTH with real siblings.
///
/// Matches `fn main()` param order in `checkpoint_inner.rue` exactly.
/// Includes NETWORK_COIN_LAUNCHER_ID per CHK-012.
fn build_full_query_env(
    a: &mut clvmr::Allocator,
    pubkey: &[u8; 48],
    root: &[u8; 32],
    validator_count: u64,
    epoch: u64,
    leaf_index: u64,
    siblings: &[[u8; 32]],
    is_member: bool,
) -> clvmr::NodePtr {
    let nil = a.nil();
    let empty_leaf = compute_empty_leaf_hash();

    // Build right-to-left (param 19 first, param 1 last)

    // 19. conditions (spread): (nil . nil)
    let conds = a.new_pair(nil, nil).unwrap();

    // 18. is_member: Bool
    let is_mem = if is_member {
        a.new_atom(&[1]).unwrap()
    } else {
        nil
    };
    let t = a.new_pair(is_mem, conds).unwrap();

    // 17. siblings: HashCons — Rue struct (head . (tail . nil)) nested
    //
    // Rue's HashCons { head: Bytes32, tail: HashCons } compiles to
    // (head . (tail . nil)) — nil-terminated proper list per struct.
    // For 32 siblings: (s0 . ((s1 . ((s2 . (... (s31 . (nil . nil)) ...)) . nil)) . nil))
    // The innermost tail is nil (atom) since depth_remaining=0 short-circuits.
    let mut sib_list = nil;
    for s in siblings.iter().rev() {
        let sn = a.new_atom(s).unwrap();
        // HashCons struct: (head . (tail . nil))
        let tail_wrapped = a.new_pair(sib_list, nil).unwrap();
        sib_list = a.new_pair(sn, tail_wrapped).unwrap();
    }
    let t = a.new_pair(sib_list, t).unwrap();

    // 16. leaf_index: Int
    let li = u64_to_clvm(a, leaf_index);
    let t = a.new_pair(li, t).unwrap();

    // 15. query_pubkey: PublicKey
    let pk = a.new_atom(pubkey).unwrap();
    let t = a.new_pair(pk, t).unwrap();

    // 14. scalars: Scalars (6 dummy fields, unused in query path)
    let z48 = [0u8; 48];
    let s6 = a.new_atom(&z48).unwrap();
    let s5 = a.new_atom(&z48).unwrap();
    let s4 = a.new_atom(&z48).unwrap();
    let s3 = a.new_atom(&z48).unwrap();
    let s2 = a.new_atom(&z48).unwrap();
    let s1 = a.new_atom(&z48).unwrap();
    let sc = a.new_pair(s6, nil).unwrap();
    let sc = a.new_pair(s5, sc).unwrap();
    let sc = a.new_pair(s4, sc).unwrap();
    let sc = a.new_pair(s3, sc).unwrap();
    let sc = a.new_pair(s2, sc).unwrap();
    let sc = a.new_pair(s1, sc).unwrap();
    let t = a.new_pair(sc, t).unwrap();

    // 13. agg_sig: Signature (dummy)
    let asig = a.new_atom(&[0u8; 96]).unwrap();
    let t = a.new_pair(asig, t).unwrap();

    // 12. agg_signers: PublicKey (dummy)
    let asgn = a.new_atom(&[0u8; 48]).unwrap();
    let t = a.new_pair(asgn, t).unwrap();

    // 11. new_validator_count: Int (dummy)
    let t = a.new_pair(nil, t).unwrap();

    // 10. new_validator_merkle_root: Bytes32 (dummy)
    let nvmr = a.new_atom(&[0u8; 32]).unwrap();
    let t = a.new_pair(nvmr, t).unwrap();

    // 9. new_state_root: Bytes32 (dummy)
    let nsr = a.new_atom(&[0u8; 32]).unwrap();
    let t = a.new_pair(nsr, t).unwrap();

    // 8. proof: Proof struct (dummy)
    let pc = a.new_atom(&[0u8; 48]).unwrap();
    let pb = a.new_atom(&[0u8; 96]).unwrap();
    let pa = a.new_atom(&[0u8; 48]).unwrap();
    let pr = a.new_pair(pc, nil).unwrap();
    let pr = a.new_pair(pb, pr).unwrap();
    let pr = a.new_pair(pa, pr).unwrap();
    let t = a.new_pair(pr, t).unwrap();

    // 7. is_checkpoint: Bool = false (membership query path)
    let t = a.new_pair(nil, t).unwrap();

    // 6. STATE: State { state_root, epoch, validator_merkle_root, validator_count }
    let sr = a.new_atom(&[0xAA; 32]).unwrap();
    let ep = u64_to_clvm(a, epoch);
    let vmr = a.new_atom(root).unwrap();
    let vc = u64_to_clvm(a, validator_count);
    let st = a.new_pair(vc, nil).unwrap();
    let st = a.new_pair(vmr, st).unwrap();
    let st = a.new_pair(ep, st).unwrap();
    let st = a.new_pair(sr, st).unwrap();
    let t = a.new_pair(st, t).unwrap();

    // 5b. NETWORK_COIN_LAUNCHER_ID (CHK-012)
    let ncli = a.new_atom(&[0x00u8; 32]).unwrap();
    let t = a.new_pair(ncli, t).unwrap();

    // 5. EMPTY_LEAF_HASH: Bytes32
    let elh = a.new_atom(&empty_leaf).unwrap();
    let t = a.new_pair(elh, t).unwrap();

    // 4. TREE_DEPTH: Int
    let td = u64_to_clvm(a, TREE_DEPTH as u64);
    let t = a.new_pair(td, t).unwrap();

    // 3. IC struct (7 dummy G1 points)
    let ic_atom = a.new_atom(&[0x01; 48]).unwrap();
    let mut ic_list = nil;
    for _ in 0..7 {
        let ic_n = a.new_atom(&[0x01; 48]).unwrap();
        ic_list = a.new_pair(ic_n, ic_list).unwrap();
    }
    // Wait - need to build as proper nil-terminated list: (ic0.(ic1.(ic2...(ic6.nil))))
    // The above loop builds it backwards, which is correct since we push onto front
    // Actually, let me build explicitly:
    let _ = ic_atom; // unused
    let _ = ic_list; // unused
    let ic6 = a.new_atom(&[0x01; 48]).unwrap();
    let ic5 = a.new_atom(&[0x01; 48]).unwrap();
    let ic4 = a.new_atom(&[0x01; 48]).unwrap();
    let ic3 = a.new_atom(&[0x01; 48]).unwrap();
    let ic2 = a.new_atom(&[0x01; 48]).unwrap();
    let ic1 = a.new_atom(&[0x01; 48]).unwrap();
    let ic0 = a.new_atom(&[0x01; 48]).unwrap();
    let ics = a.new_pair(ic6, nil).unwrap();
    let ics = a.new_pair(ic5, ics).unwrap();
    let ics = a.new_pair(ic4, ics).unwrap();
    let ics = a.new_pair(ic3, ics).unwrap();
    let ics = a.new_pair(ic2, ics).unwrap();
    let ics = a.new_pair(ic1, ics).unwrap();
    let ics = a.new_pair(ic0, ics).unwrap();
    let t = a.new_pair(ics, t).unwrap();

    // 2. VK struct (4 dummy points)
    let vd = a.new_atom(&[0x01; 96]).unwrap();
    let vg = a.new_atom(&[0x01; 96]).unwrap();
    let vb = a.new_atom(&[0x01; 96]).unwrap();
    let va = a.new_atom(&[0x01; 48]).unwrap();
    let vk = a.new_pair(vd, nil).unwrap();
    let vk = a.new_pair(vg, vk).unwrap();
    let vk = a.new_pair(vb, vk).unwrap();
    let vk = a.new_pair(va, vk).unwrap();
    let t = a.new_pair(vk, t).unwrap();

    // 1. INNER_MOD_HASH: Bytes32
    let imh = a.new_atom(&[0x11; 32]).unwrap();
    a.new_pair(imh, t).unwrap()
}

// ── TEST 1: Single validator membership proof (Rust → CLVM) ──────────

/// SMT-005: Insert one validator in the Rust tree, generate a depth-32
/// membership proof, then verify the CLVM puzzle accepts it.
///
/// This proves: Rust's SparseMerkleTree and Rue's verify_merkle_path
/// agree on slot assignment, leaf hashing, sibling ordering, and parent
/// hash concatenation for a single-validator tree at full depth.
#[test]
fn vv_req_smt_005_cross_impl_single_validator_membership() {
    let pubkey: [u8; 48] = [0xAA; 48];

    // 1. Build tree in Rust
    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey);
    let root = tree.root();
    let slot = compute_slot(&pubkey);

    // 2. Generate Rust proof
    let proof = tree.prove(slot);
    assert_eq!(proof.leaf, active_leaf(&pubkey));
    assert!(proof.verify(root), "Rust proof must verify in Rust");
    assert_eq!(proof.siblings.len(), TREE_DEPTH as usize);

    // 3. Run CLVM puzzle with Rust proof
    let mut a = clvmr::Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_full_query_env(&mut a, &pubkey, &root, 1, 5, slot, &proof.siblings, true);

    let (cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    assert!(cost > 0, "CLVM must execute");

    let conditions = parse_conditions(&a, output);
    assert!(
        has_opcode(&conditions, CREATE_COIN),
        "SMT-005: CLVM must emit CREATE_COIN (singleton recreation)"
    );
    assert!(
        has_opcode(&conditions, CREATE_COIN_ANNOUNCEMENT),
        "SMT-005: CLVM must emit CREATE_COIN_ANNOUNCEMENT"
    );

    eprintln!(
        "SMT-005 CROSS-IMPL: Single validator membership proof verified in CLVM at depth={} (cost={})",
        TREE_DEPTH, cost
    );
}

// ── TEST 2: Non-membership proof (Rust → CLVM) ──────────────────────

/// SMT-005: Generate a non-membership proof for a pubkey NOT in the tree,
/// then verify the CLVM puzzle accepts it (slot is empty → EMPTY_LEAF).
#[test]
fn vv_req_smt_005_cross_impl_non_membership_proof() {
    let active_pubkey: [u8; 48] = [0xAA; 48];
    let absent_pubkey: [u8; 48] = [0xBB; 48];

    // 1. Build tree with one validator
    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&active_pubkey);
    let root = tree.root();

    // 2. Generate non-membership proof for absent pubkey
    let absent_slot = compute_slot(&absent_pubkey);
    let proof = tree.prove(absent_slot);
    assert_eq!(proof.leaf, EMPTY_LEAF, "Absent pubkey slot must be empty");
    assert!(proof.verify(root), "Rust non-membership proof must verify");

    // 3. Run CLVM puzzle — is_member=false means leaf = EMPTY_LEAF_HASH
    let mut a = clvmr::Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_full_query_env(
        &mut a,
        &absent_pubkey,
        &root,
        1,
        5,
        absent_slot,
        &proof.siblings,
        false,
    );

    let (cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    assert!(has_opcode(&conditions, CREATE_COIN));
    assert!(has_opcode(&conditions, CREATE_COIN_ANNOUNCEMENT));

    // 4. Verify announcement matches Rust wire format (WIRE-004)
    let anns = common::clvm::conditions_with_opcode(&conditions, CREATE_COIN_ANNOUNCEMENT);
    let mut inner = Vec::new();
    inner.extend_from_slice(b"membership");
    inner.extend_from_slice(&5u64.to_be_bytes()); // epoch=5
    inner.extend_from_slice(&absent_pubkey);
    inner.push(0x00); // non-member
    let expected_hash: [u8; 32] = Sha256::digest(&inner).into();
    assert_eq!(
        anns[0].args[0].as_slice(),
        expected_hash.as_slice(),
        "SMT-005: Non-membership announcement must match Rust wire format"
    );

    eprintln!(
        "SMT-005 CROSS-IMPL: Non-membership proof verified in CLVM at depth={} (cost={})",
        TREE_DEPTH, cost
    );
}

// ── TEST 3: Multiple validators (Rust → CLVM) ───────────────────────

/// SMT-005: Insert 5 validators, then verify membership proofs for all 5
/// in the CLVM puzzle. This stress-tests sibling ordering and slot placement
/// with multiple occupied slots in the depth-32 tree.
#[test]
fn vv_req_smt_005_cross_impl_multiple_validators() {
    // 5 test pubkeys with diverse patterns
    let pubkeys: [[u8; 48]; 5] = [[0x01; 48], [0x02; 48], [0x03; 48], [0x04; 48], [0x05; 48]];

    // 1. Build tree in Rust
    let mut tree = SparseMerkleTree::new();
    for pk in &pubkeys {
        tree.insert_validator(pk);
    }
    let root = tree.root();

    // 2. Verify all Rust proofs first
    for pk in &pubkeys {
        let slot = compute_slot(pk);
        let proof = tree.prove(slot);
        assert!(
            proof.verify(root),
            "Rust proof must verify for {:02x}",
            pk[0]
        );
    }

    // 3. Verify each proof in CLVM
    for pk in &pubkeys {
        let slot = compute_slot(pk);
        let proof = tree.prove(slot);

        let mut a = clvmr::Allocator::new();
        let puzzle = load_puzzle(&mut a, CHK_HEX);
        let env = build_full_query_env(&mut a, pk, &root, 5, 10, slot, &proof.siblings, true);

        let result = common::clvm::run_puzzle(&mut a, puzzle, env);
        assert!(
            result.is_ok(),
            "SMT-005: CLVM must accept membership proof for pubkey {:02x}..., slot={}, error={:?}",
            pk[0],
            slot,
            result.err()
        );

        let (_, output) = result.unwrap();
        let conditions = parse_conditions(&a, output);
        assert!(has_opcode(&conditions, CREATE_COIN));
        assert!(has_opcode(&conditions, CREATE_COIN_ANNOUNCEMENT));
    }

    eprintln!("SMT-005 CROSS-IMPL: All 5 validator membership proofs verified in CLVM");
}

// ── TEST 4: Empty tree root matches ──────────────────────────────────

/// SMT-005: Verify Rust EMPTY_TREE_ROOT matches what the CLVM puzzle
/// expects by proving non-membership in an empty tree.
#[test]
fn vv_req_smt_005_cross_impl_empty_tree_root() {
    let tree = SparseMerkleTree::new();
    let root = tree.root();
    assert_eq!(root, EMPTY_TREE_ROOT, "Empty tree root must match constant");

    // Any pubkey is absent in empty tree
    let pubkey: [u8; 48] = [0xFF; 48];
    let slot = compute_slot(&pubkey);
    let proof = tree.prove(slot);
    assert_eq!(proof.leaf, EMPTY_LEAF);
    assert!(proof.verify(root));

    // CLVM should accept this non-membership proof
    let mut a = clvmr::Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_full_query_env(&mut a, &pubkey, &root, 0, 0, slot, &proof.siblings, false);

    let result = common::clvm::run_puzzle(&mut a, puzzle, env);
    assert!(
        result.is_ok(),
        "SMT-005: CLVM must accept non-membership in empty tree, error={:?}",
        result.err()
    );

    eprintln!("SMT-005 CROSS-IMPL: Empty tree root cross-verified in CLVM");
}

// ── TEST 5: Announcement hash cross-check ────────────────────────────

/// SMT-005: Verify the membership announcement produced by CLVM matches
/// the Rust wire format exactly. This tests the full chain:
/// Rust tree → Rust proof → CLVM verify → CLVM announcement == Rust announcement.
#[test]
fn vv_req_smt_005_cross_impl_announcement_hash() {
    let pubkey: [u8; 48] = [0xCC; 48];
    let epoch: u64 = 42;

    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey);
    let root = tree.root();
    let slot = compute_slot(&pubkey);
    let proof = tree.prove(slot);

    // CLVM
    let mut a = clvmr::Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_full_query_env(
        &mut a,
        &pubkey,
        &root,
        1,
        epoch,
        slot,
        &proof.siblings,
        true,
    );

    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let anns = common::clvm::conditions_with_opcode(&conditions, CREATE_COIN_ANNOUNCEMENT);
    assert_eq!(anns.len(), 1, "Exactly one announcement");

    // Rust wire format: sha256("membership" + epoch_be8 + pubkey + 0x01)
    let mut inner = Vec::new();
    inner.extend_from_slice(b"membership");
    inner.extend_from_slice(&epoch.to_be_bytes());
    inner.extend_from_slice(&pubkey);
    inner.push(0x01); // member
    let expected: [u8; 32] = Sha256::digest(&inner).into();

    assert_eq!(
        anns[0].args[0].as_slice(),
        expected.as_slice(),
        "SMT-005: Membership announcement Rust==CLVM at depth={}, epoch={}",
        TREE_DEPTH,
        epoch
    );

    eprintln!("SMT-005 CROSS-IMPL: Announcement hash matches Rust wire format");
}

// ── TEST 6: Wrong proof rejected by CLVM ─────────────────────────────

/// SMT-005: A Rust proof for one validator must NOT verify in CLVM for
/// a different root. This confirms the CLVM assert actually checks the
/// computed root.
#[test]
fn vv_req_smt_005_cross_impl_wrong_root_rejected() {
    let pubkey: [u8; 48] = [0xDD; 48];

    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pubkey);
    let correct_root = tree.root();
    let slot = compute_slot(&pubkey);
    let proof = tree.prove(slot);

    // Use a WRONG root (add another validator to change the root)
    let mut tree2 = SparseMerkleTree::new();
    tree2.insert_validator(&pubkey);
    tree2.insert_validator(&[0xEE; 48]);
    let wrong_root = tree2.root();
    assert_ne!(correct_root, wrong_root);

    // CLVM should REJECT the proof (computed root != STATE.validator_merkle_root)
    let mut a = clvmr::Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_full_query_env(
        &mut a,
        &pubkey,
        &wrong_root, // Wrong root!
        1,
        0,
        slot,
        &proof.siblings, // Proof was for correct_root
        true,
    );

    let result = common::clvm::run_puzzle(&mut a, puzzle, env);
    assert!(
        result.is_err(),
        "SMT-005: CLVM must reject proof against wrong root"
    );

    eprintln!("SMT-005 CROSS-IMPL: Wrong root correctly rejected by CLVM");
}
