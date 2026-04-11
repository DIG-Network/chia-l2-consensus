//! REQUIREMENTS: CHK-001 through CHK-007 — Checkpoint Singleton
//!
//! This file covers seven checkpoint singleton requirements in a single
//! compilation unit, sharing helper infrastructure.
//!
//! - **CHK-001**: Singleton identity -- inner puzzle compiles, hex/hash
//!   artifacts exist, puzzle loads into CLVM allocator.
//! - **CHK-002**: State tracking -- curried parameters include state_root,
//!   epoch, validator_merkle_root, validator_count, TREE_DEPTH.
//! - **CHK-003**: Groth16+BLS verification -- checkpoint spend path selector
//!   exists (detailed CLVM execution tested in `vv_req_chk_003.rs`).
//! - **CHK-004**: State update -- checkpoint path emits CreateCoinAnnouncement
//!   and increments epoch by 1 (CLVM execution in `vv_req_chk_003.rs`).
//! - **CHK-005**: Membership query -- Merkle verify + announcement at depth=0
//!   (depth>0 deferred to CHK-008 simulator test due to Rue position bug).
//! - **CHK-006**: Permissionless -- no AGG_SIG_ME/UNSAFE in membership query.
//! - **CHK-007**: VK immutability -- VK is curried, included in recreation hash.
//!
//! Implementation: `puzzles/checkpoint_inner.rue`.
//!
//! ## How the tests prove the requirements
//! - CHK-001: Compilation via `rue build`, artifact existence, CLVM load.
//! - CHK-002: Source-level assertions for typed parameter declarations.
//! - CHK-003/004: Source-level structural checks; CLVM execution delegated.
//! - CHK-005: Runs compiled puzzle at depth=0 for membership + non-membership,
//!   cross-impl verifies announcement hash against Rust wire format.
//! - CHK-006: Asserts no AGG_SIG_ME (50) or AGG_SIG_UNSAFE (49) in output.
//! - CHK-007: Source-level check that VK is curried and in curry_tree_hash.
//!
//! ## Completeness: MODERATE
//! Depth=0 query path is fully CLVM-tested. Depth>0 and checkpoint path
//! require simulator E2E (CHK-008, vv_req_chk_003.rs).
//!
//! ## Gaps
//! - Depth>0 Merkle verification not tested here (Rue recursive helper bug).
//! - CHK-003/004 structural only -- CLVM execution in companion file.

mod common;

use std::process::Command;

use clvmr::Allocator;
use sha2::{Digest, Sha256};

use common::clvm::*;

const CHK_HEX: &str = include_str!("../puzzles/compiled/checkpoint_inner.hex");

// ── Helpers ────────────────────────────────────────────────────────

fn compute_empty_leaf_hash() -> [u8; 32] {
    Sha256::digest(&[0u8; 48]).into()
}

/// Build a trivial 1-leaf Merkle tree and return (root, siblings).
/// For tree_depth=1, one leaf is active, one empty.
fn build_single_leaf_tree(pubkey: &[u8], depth: usize) -> ([u8; 32], Vec<[u8; 32]>) {
    let empty_leaf: [u8; 32] = compute_empty_leaf_hash();
    let active_leaf: [u8; 32] = Sha256::digest(pubkey).into();

    // For depth=1: tree has 2 leaves. Put active at index 0, empty at index 1.
    // root = sha256(active_leaf || empty_leaf)
    // Sibling of index 0 = empty_leaf
    if depth == 1 {
        let mut root_pre = Vec::new();
        root_pre.extend_from_slice(&active_leaf);
        root_pre.extend_from_slice(&empty_leaf);
        let root: [u8; 32] = Sha256::digest(&root_pre).into();
        return (root, vec![empty_leaf]);
    }

    // For deeper trees, build bottom-up with empty siblings
    let mut empty_hashes = vec![[0u8; 32]; depth + 1];
    empty_hashes[0] = empty_leaf;
    for i in 1..=depth {
        let mut pre = Vec::new();
        pre.extend_from_slice(&empty_hashes[i - 1]);
        pre.extend_from_slice(&empty_hashes[i - 1]);
        empty_hashes[i] = Sha256::digest(&pre).into();
    }

    // Place active leaf at index 0, all siblings are empty hashes
    let mut siblings = Vec::with_capacity(depth);
    let mut current = active_leaf;
    for level in 0..depth {
        siblings.push(empty_hashes[level]);
        let mut pre = Vec::new();
        pre.extend_from_slice(&current);
        pre.extend_from_slice(&empty_hashes[level]);
        current = Sha256::digest(&pre).into();
    }
    (current, siblings)
}

// ── CHK-001: Puzzle compiles and loads ─────────────────────────────

/// CHK-001: Verifies the checkpoint_inner.rue source compiles successfully
/// via the Rue compiler CLI. Passing proves the puzzle is syntactically
/// valid and produces CLVM output.
#[test]
fn vv_req_chk_001_puzzle_compiles() {
    let output = Command::new("rue")
        .args(["build", "puzzles/checkpoint_inner.rue"])
        .output()
        .expect("Failed to run rue");
    assert!(
        output.status.success(),
        "CHK-001: checkpoint_inner.rue must compile"
    );
}

/// CHK-001: Compiled hex artifact must exist on disk for CLVM execution.
#[test]
fn vv_req_chk_001_hex_artifact_exists() {
    assert!(std::path::Path::new("puzzles/compiled/checkpoint_inner.hex").exists());
}

/// CHK-001: Compiled hash artifact must exist for curry_tree_hash usage.
#[test]
fn vv_req_chk_001_hash_artifact_exists() {
    assert!(std::path::Path::new("puzzles/compiled/checkpoint_inner.hash").exists());
}

/// CHK-001: The hex artifact deserializes into a non-nil CLVM node,
/// proving it is valid serialized CLVM.
#[test]
fn vv_req_chk_001_puzzle_loads() {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    assert_ne!(puzzle, a.nil(), "CHK-001: Puzzle must load from hex");
}

/// CHK-001: Source contains fn main() and references inner/singleton role,
/// confirming it is designed as a singleton inner puzzle.
#[test]
fn vv_req_chk_001_is_inner_puzzle() {
    let src = std::fs::read_to_string("puzzles/checkpoint_inner.rue").unwrap();
    assert!(src.contains("fn main("), "CHK-001: Must have fn main");
    assert!(
        src.contains("inner") || src.contains("singleton"),
        "CHK-001: Must document inner puzzle / singleton role"
    );
}

// ── CHK-002: State tracking ────────────────────────────────────────

/// CHK-002: Verifies the source declares typed parameters for all state
/// fields: state_root, epoch, validator_merkle_root, validator_count.
/// These are curried into the puzzle and track checkpoint state.
#[test]
fn vv_req_chk_002_has_state_params() {
    let src = std::fs::read_to_string("puzzles/checkpoint_inner.rue").unwrap();
    assert!(src.contains("state_root: Bytes32"));
    assert!(src.contains("epoch: Int"));
    assert!(src.contains("validator_merkle_root: Bytes32"));
    assert!(src.contains("validator_count: Int"));
}

/// CHK-002: TREE_DEPTH is a curried Int parameter controlling Merkle tree
/// depth for validator set membership proofs.
#[test]
fn vv_req_chk_002_has_tree_depth_param() {
    let src = std::fs::read_to_string("puzzles/checkpoint_inner.rue").unwrap();
    assert!(src.contains("TREE_DEPTH: Int"));
}

// ── CHK-003: Groth16+BLS (structural only — verification TODO) ────

/// CHK-003: Structural check -- source has is_checkpoint spend path selector
/// and documents Groth16 verification status. Detailed CLVM execution
/// tested in vv_req_chk_003.rs.
#[test]
fn vv_req_chk_003_checkpoint_path_exists() {
    let src = std::fs::read_to_string("puzzles/checkpoint_inner.rue").unwrap();
    assert!(
        src.contains("is_checkpoint"),
        "CHK-003: Must have checkpoint spend path selector"
    );
    assert!(
        src.contains("CHK-003") || src.contains("Groth16") || src.contains("TODO"),
        "CHK-003: Must document Groth16 verification status"
    );
}

// ── CHK-004: State update — CLVM execution tested in vv_req_chk_003.rs
//    (vv_req_chk_003_checkpoint_path_executes covers this requirement)

/// CHK-004: Structural check -- source emits CreateCoinAnnouncement with
/// "checkpoint" keyword, and computes new_epoch = epoch + 1. The CLVM
/// execution that verifies this produces correct output is in
/// vv_req_chk_003.rs (checkpoint_path_executes).
#[test]
fn vv_req_chk_004_has_checkpoint_announcement() {
    let src = std::fs::read_to_string("puzzles/checkpoint_inner.rue").unwrap();
    assert!(
        src.contains("CreateCoinAnnouncement") && src.contains("checkpoint"),
        "CHK-004: Must emit checkpoint state announcement"
    );
    assert!(
        src.contains("new_epoch") && src.contains("epoch + 1"),
        "CHK-004: Epoch must increment by 1"
    );
}

// CHK-004 CLVM execution: covered by vv_req_chk_003_checkpoint_path_executes

// ── CHK-005: Membership query ──────────────────────────────────────

/// Build checkpoint inner puzzle flat env for membership query.
///
/// Matches fn main() param order exactly. All 19 params as right-linked list.
/// Struct params encoded as Rue structs: (f1 . (f2 . (f3 . f4))) — last NOT wrapped.
/// Empty conditions = (nil . nil) per Rue spread param convention.
fn build_query_env(
    a: &mut Allocator,
    pubkey: &[u8],
    depth: u64,
    root: &[u8; 32],
    vc: u64,
    epoch: u64,
    leaf_index: u64,
    siblings: &[[u8; 32]],
    is_member: bool,
) -> clvmr::NodePtr {
    let nil = a.nil();
    let empty_leaf = compute_empty_leaf_hash();

    // Build right-to-left (param 19 first, param 1 last)

    // 19. conditions: List<Condition> — spread param, (nil . nil) for empty
    let conds = a.new_pair(nil, nil).unwrap();

    // 18. is_member: Bool
    let is_mem = if is_member {
        a.new_atom(&[1]).unwrap()
    } else {
        nil
    };
    let t = a.new_pair(is_mem, conds).unwrap();

    // 17. siblings: HashCons — cons list of Bytes32
    let mut sib_list = nil;
    for s in siblings.iter().rev() {
        let sn = a.new_atom(s).unwrap();
        sib_list = a.new_pair(sn, sib_list).unwrap();
    }
    let t = a.new_pair(sib_list, t).unwrap();

    // 16. leaf_index: Int
    let li = u64_to_clvm(a, leaf_index);
    let t = a.new_pair(li, t).unwrap();

    // 15. query_pubkey: PublicKey (48 bytes)
    let pk = a.new_atom(pubkey).unwrap();
    let t = a.new_pair(pk, t).unwrap();

    // 14. scalars: Scalars { s1..s6 } — 6-field struct (nil-terminated proper list)
    let scalars = {
        let s6 = a.new_atom(&[0u8; 48]).unwrap();
        let s5 = a.new_atom(&[0u8; 48]).unwrap();
        let s4 = a.new_atom(&[0u8; 48]).unwrap();
        let s3 = a.new_atom(&[0u8; 48]).unwrap();
        let s2 = a.new_atom(&[0u8; 48]).unwrap();
        let s1 = a.new_atom(&[0u8; 48]).unwrap();
        let t = a.new_pair(s6, nil).unwrap();
        let t = a.new_pair(s5, t).unwrap();
        let t = a.new_pair(s4, t).unwrap();
        let t = a.new_pair(s3, t).unwrap();
        let t = a.new_pair(s2, t).unwrap();
        a.new_pair(s1, t).unwrap()
    };
    let t = a.new_pair(scalars, t).unwrap();

    // 13. agg_sig: Signature (96 bytes, dummy)
    let agg_sig = a.new_atom(&[0u8; 96]).unwrap();
    let t = a.new_pair(agg_sig, t).unwrap();

    // 12. agg_signers: PublicKey (48 bytes, dummy)
    let agg_signers = a.new_atom(&[0u8; 48]).unwrap();
    let t = a.new_pair(agg_signers, t).unwrap();

    // 11. new_validator_count: Int (dummy 0)
    let nvc = nil; // 0
    let t = a.new_pair(nvc, t).unwrap();

    // 10. new_validator_merkle_root: Bytes32 (dummy)
    let nvmr = a.new_atom(&[0u8; 32]).unwrap();
    let t = a.new_pair(nvmr, t).unwrap();

    // 9. new_state_root: Bytes32 (dummy)
    let nsr = a.new_atom(&[0u8; 32]).unwrap();
    let t = a.new_pair(nsr, t).unwrap();

    // 8. proof: Proof { a: PK, b: Sig, c: PK } — 3-field struct (nil-terminated)
    let proof = {
        let pc = a.new_atom(&[0u8; 48]).unwrap();
        let pb = a.new_atom(&[0u8; 96]).unwrap();
        let pa = a.new_atom(&[0u8; 48]).unwrap();
        let t = a.new_pair(pc, nil).unwrap();
        let t = a.new_pair(pb, t).unwrap();
        a.new_pair(pa, t).unwrap()
    };
    let t = a.new_pair(proof, t).unwrap();

    // 7. is_checkpoint: Bool — false for membership query
    let t = a.new_pair(nil, t).unwrap();

    // 6. STATE: State { state_root, epoch, validator_merkle_root, validator_count }
    let state = {
        let sr = a.new_atom(&[0xAA; 32]).unwrap();
        let ep = u64_to_clvm(a, epoch);
        let vmr = a.new_atom(root).unwrap();
        let vc_n = u64_to_clvm(a, vc);
        // (sr . (ep . (vmr . (vc . nil)))) — nil-terminated proper list
        let t = a.new_pair(vc_n, nil).unwrap();
        let t = a.new_pair(vmr, t).unwrap();
        let t = a.new_pair(ep, t).unwrap();
        a.new_pair(sr, t).unwrap()
    };
    let t = a.new_pair(state, t).unwrap();

    // 6. NETWORK_COIN_LAUNCHER_ID (CHK-012)
    let ncli = a.new_atom(&[0x00u8; 32]).unwrap();
    let t = a.new_pair(ncli, t).unwrap();

    // 5. EMPTY_LEAF_HASH: Bytes32
    let elh = a.new_atom(&empty_leaf).unwrap();
    let t = a.new_pair(elh, t).unwrap();

    // 4. TREE_DEPTH: Int
    let td = u64_to_clvm(a, depth);
    let t = a.new_pair(td, t).unwrap();

    // 3. IC: IC { ic0..ic6 } — 7-field struct (nil-terminated proper list)
    let ic = {
        let ic6 = a.new_atom(&[0x01; 48]).unwrap();
        let ic5 = a.new_atom(&[0x01; 48]).unwrap();
        let ic4 = a.new_atom(&[0x01; 48]).unwrap();
        let ic3 = a.new_atom(&[0x01; 48]).unwrap();
        let ic2 = a.new_atom(&[0x01; 48]).unwrap();
        let ic1 = a.new_atom(&[0x01; 48]).unwrap();
        let ic0 = a.new_atom(&[0x01; 48]).unwrap();
        let t = a.new_pair(ic6, nil).unwrap();
        let t = a.new_pair(ic5, t).unwrap();
        let t = a.new_pair(ic4, t).unwrap();
        let t = a.new_pair(ic3, t).unwrap();
        let t = a.new_pair(ic2, t).unwrap();
        let t = a.new_pair(ic1, t).unwrap();
        a.new_pair(ic0, t).unwrap()
    };
    let t = a.new_pair(ic, t).unwrap();

    // 2. VK: VK { alpha: PK, beta: Sig, gamma: Sig, delta: Sig } — (nil-terminated)
    let vk = {
        let delta = a.new_atom(&[0x01; 96]).unwrap();
        let gamma = a.new_atom(&[0x01; 96]).unwrap();
        let beta = a.new_atom(&[0x01; 96]).unwrap();
        let alpha = a.new_atom(&[0x01; 48]).unwrap();
        let t = a.new_pair(delta, nil).unwrap();
        let t = a.new_pair(gamma, t).unwrap();
        let t = a.new_pair(beta, t).unwrap();
        a.new_pair(alpha, t).unwrap()
    };
    let t = a.new_pair(vk, t).unwrap();

    // 1. INNER_MOD_HASH: Bytes32
    let imh = a.new_atom(&[0x11; 32]).unwrap();
    a.new_pair(imh, t).unwrap()
}

/// CHK-005: CLVM execution test for membership query at depth=0 (trivial
/// 1-leaf Merkle tree). Verifies the puzzle runs, costs > 0, emits
/// CREATE_COIN (singleton recreation) and CREATE_COIN_ANNOUNCEMENT
/// (membership announcement). Depth>0 deferred to CHK-008 due to Rue
/// compiler position mapping bug in the recursive Merkle helper.
#[test]
fn vv_req_chk_005_membership_query_runs() {
    // Depth=0: root = leaf = sha256(pubkey), no siblings needed.
    let pubkey = [0xAA; 48];
    let root: [u8; 32] = Sha256::digest(&pubkey).into();

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_query_env(&mut a, &pubkey, 0, &root, 1, 5, 0, &[], true);

    let (cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    assert!(cost > 0);
    let conditions = parse_conditions(&a, output);

    assert!(
        has_opcode(&conditions, CREATE_COIN),
        "CHK-005: Membership query must emit CREATE_COIN (recreation)"
    );
    assert!(
        has_opcode(&conditions, CREATE_COIN_ANNOUNCEMENT),
        "CHK-005: Membership query must emit CREATE_COIN_ANNOUNCEMENT"
    );
}

/// CHK-005: Cross-implementation test for the membership announcement hash.
/// Computes the expected hash in Rust using the wire format
/// (sha256("membership" + epoch_be8 + pubkey + 0x01)) and compares
/// against the CLVM CREATE_COIN_ANNOUNCEMENT output. Passing proves the
/// on-chain puzzle and Rust code produce identical announcement hashes.
#[test]
fn vv_req_chk_005_membership_announcement_cross_impl() {
    let pubkey = [0xAA; 48];
    let epoch: u64 = 7;
    let root: [u8; 32] = Sha256::digest(&pubkey).into();

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_query_env(&mut a, &pubkey, 0, &root, 1, epoch, 0, &[], true);

    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let anns = conditions_with_opcode(&conditions, CREATE_COIN_ANNOUNCEMENT);
    assert_eq!(anns.len(), 1);

    let mut inner = Vec::new();
    inner.extend_from_slice(b"membership");
    inner.extend_from_slice(&epoch.to_be_bytes());
    inner.extend_from_slice(&pubkey);
    inner.push(0x01);
    let expected: [u8; 32] = Sha256::digest(&inner).into();

    assert_eq!(
        anns[0].args[0].as_slice(),
        expected.as_slice(),
        "CHK-005: Membership announcement must match Rust wire format"
    );
}

/// CHK-005: Non-membership announcement test at depth=0. When is_member=false
/// and the root is EMPTY_LEAF_HASH, the puzzle emits an announcement with
/// is_member byte = 0x00 (verified via cross-impl comparison). Passing
/// proves the non-membership path produces the correct announcement that
/// registration coins can assert against.
#[test]
fn vv_req_chk_005_non_membership_announcement() {
    let pubkey = [0xBB; 48];
    let empty_leaf = compute_empty_leaf_hash();
    // For depth=0 non-membership: root = EMPTY_LEAF_HASH (since leaf = EMPTY_LEAF for non-member)
    let root = empty_leaf;

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_query_env(&mut a, &pubkey, 0, &root, 1, 3, 0, &[], false);

    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    let anns = conditions_with_opcode(&conditions, CREATE_COIN_ANNOUNCEMENT);
    assert_eq!(anns.len(), 1);

    let mut inner = Vec::new();
    inner.extend_from_slice(b"membership");
    inner.extend_from_slice(&3u64.to_be_bytes());
    inner.extend_from_slice(&pubkey);
    inner.push(0x00);
    let expected: [u8; 32] = Sha256::digest(&inner).into();

    assert_eq!(
        anns[0].args[0].as_slice(),
        expected.as_slice(),
        "CHK-005: Non-membership announcement must have is_member=0x00"
    );
}

// ── CHK-006: Permissionless ────────────────────────────────────────

/// CHK-006: Verifies the membership query path produces no signature
/// conditions (AGG_SIG_ME=50 or AGG_SIG_UNSAFE=49). This proves the
/// query is permissionless -- anyone can query membership status without
/// holding a private key, which is essential for collateral recovery.
#[test]
fn vv_req_chk_006_no_agg_sig_in_membership_query() {
    let pubkey = [0xAA; 48];
    let root: [u8; 32] = Sha256::digest(&pubkey).into();

    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, CHK_HEX);
    let env = build_query_env(&mut a, &pubkey, 0, &root, 1, 5, 0, &[], true);

    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);

    assert!(
        !has_opcode(&conditions, AGG_SIG_ME),
        "CHK-006: Membership query must NOT have AGG_SIG_ME"
    );
    assert!(
        !has_opcode(&conditions, 49),
        "CHK-006: Membership query must NOT have AGG_SIG_UNSAFE"
    );
}

// ── CHK-007: VK immutability ───────────────────────────────────────

/// CHK-007: Verifies the verification key (VK) is a curried parameter, not
/// a solution value. Curried parameters are immutable after deployment,
/// ensuring the VK cannot be swapped by an attacker.
#[test]
fn vv_req_chk_007_vk_is_curried() {
    let src = std::fs::read_to_string("puzzles/checkpoint_inner.rue").unwrap();
    assert!(
        src.contains("VK: VK") || src.contains("VK_ALPHA: PublicKey"),
        "CHK-007: VK must be a curried parameter"
    );
}

/// CHK-007: Verifies the VK is included in the curry_tree_hash computation
/// for singleton recreation. This ensures the VK is carried forward
/// identically when the checkpoint singleton recreates itself, making
/// it truly immutable across all epochs.
#[test]
fn vv_req_chk_007_vk_in_recreation() {
    // CHK-007: VK must be included when computing recreated puzzle hash
    let src = std::fs::read_to_string("puzzles/checkpoint_inner.rue").unwrap();
    assert!(
        src.contains("tree_hash(VK)") || src.contains("tree_hash(VK_HASH)"),
        "CHK-007: VK must be in curry_tree_hash for recreation"
    );
}

// ── Spec files exist ───────────────────────────────────────────────

/// Traceability: confirms spec files for CHK-001 through CHK-007 all exist.
#[test]
fn vv_req_chk_specs_exist() {
    for id in [
        "CHK-001", "CHK-002", "CHK-003", "CHK-004", "CHK-005", "CHK-006", "CHK-007",
    ] {
        let path = format!("docs/requirements/domains/checkpoint/specs/{}.md", id);
        assert!(
            std::path::Path::new(&path).exists(),
            "Spec {} must exist",
            id
        );
    }
}
