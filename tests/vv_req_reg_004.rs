//! REQUIREMENT: REG-004 — Announcement Assertion
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-004`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-004.md`.
//!
//! Implementation: `puzzles/registration_coin.rue` (compiled to CLVM).
//!
//! Verifies the exact byte-level format of the announcement hash:
//! - Inner: sha256("membership" + epoch_be8 + pubkey + 0x00) = 67-byte preimage
//! - Full: sha256(checkpoint_singleton_id + inner_hash) = 64-byte preimage
//!
//! CLVM execution tests run the compiled puzzle and compare output to
//! hand-computed test vectors.

mod common;

use clvmr::Allocator;
use sha2::{Digest, Sha256};

use common::clvm::*;

const REG_COIN_HEX: &str = include_str!("../puzzles/compiled/registration_coin.hex");

/// Build registration coin env as flat list.
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

/// Run the registration coin CLVM and extract the announcement hash.
fn get_clvm_announcement_hash(pk: &[u8], ckpt_id: &[u8], epoch: u64) -> Vec<u8> {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_env(&mut a, pk, ckpt_id, epoch, &[0xCC; 32], 1_000_000);
    let (_cost, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conditions = parse_conditions(&a, output);
    conditions_with_opcode(&conditions, ASSERT_COIN_ANNOUNCEMENT)[0].args[0].clone()
}

/// Compute inner announcement preimage (67 bytes).
fn inner_preimage(epoch: u64, pk: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(67);
    buf.extend_from_slice(b"membership"); // 10 bytes
    buf.extend_from_slice(&epoch.to_be_bytes()); // 8 bytes
    buf.extend_from_slice(pk); // 48 bytes
    buf.push(0x00); // 1 byte
    buf
}

/// Compute full announcement hash from components.
fn full_hash(ckpt_id: &[u8], epoch: u64, pk: &[u8]) -> [u8; 32] {
    let inner: [u8; 32] = Sha256::digest(&inner_preimage(epoch, pk)).into();
    let mut full = Vec::with_capacity(64);
    full.extend_from_slice(ckpt_id); // 32 bytes
    full.extend_from_slice(&inner); // 32 bytes
    Sha256::digest(&full).into()
}

// ── Inner preimage format (67 bytes) ───────────────────────────────

#[test]
fn vv_req_reg_004_inner_preimage_is_67_bytes() {
    // REG-004: Inner announcement preimage = "membership"(10) + epoch(8) + pubkey(48) + is_member(1) = 67 bytes
    let preimage = inner_preimage(5, &[0xAA; 48]);
    assert_eq!(
        preimage.len(),
        67,
        "REG-004: Inner preimage must be exactly 67 bytes"
    );
}

#[test]
fn vv_req_reg_004_prefix_is_membership_10_bytes() {
    // REG-004: First 10 bytes of inner preimage = "membership" (UTF-8, no null)
    let preimage = inner_preimage(0, &[0x00; 48]);
    assert_eq!(
        &preimage[0..10],
        b"membership",
        "REG-004: Prefix must be 'membership'"
    );
    assert_eq!(
        &preimage[0..10],
        &[0x6d, 0x65, 0x6d, 0x62, 0x65, 0x72, 0x73, 0x68, 0x69, 0x70],
        "REG-004: 'membership' must be these exact bytes"
    );
}

#[test]
fn vv_req_reg_004_epoch_is_8_bytes_big_endian() {
    // REG-004: Bytes 10-17 = epoch as 8-byte big-endian u64
    let epoch: u64 = 256; // 0x0000000000000100
    let preimage = inner_preimage(epoch, &[0x00; 48]);
    assert_eq!(
        &preimage[10..18],
        &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00],
        "REG-004: Epoch 256 must be 0x0000000000000100 in 8-byte BE"
    );
}

#[test]
fn vv_req_reg_004_epoch_zero_is_8_zero_bytes() {
    // REG-004: Epoch 0 = 8 zero bytes, NOT empty atom
    let preimage = inner_preimage(0, &[0x00; 48]);
    assert_eq!(
        &preimage[10..18],
        &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        "REG-004: Epoch 0 must be 8 zero bytes"
    );
}

#[test]
fn vv_req_reg_004_pubkey_is_48_bytes_at_offset_18() {
    // REG-004: Bytes 18-65 = validator pubkey (48 bytes compressed G1)
    let pk = [0x42; 48];
    let preimage = inner_preimage(0, &pk);
    assert_eq!(
        &preimage[18..66],
        &pk,
        "REG-004: Pubkey must be bytes 18-65 of inner preimage"
    );
}

#[test]
fn vv_req_reg_004_is_member_byte_is_0x00() {
    // REG-004: Last byte (offset 66) = 0x00 (non-member)
    let preimage = inner_preimage(0, &[0x00; 48]);
    assert_eq!(
        preimage[66], 0x00,
        "REG-004: is_member byte must be 0x00 (non-member)"
    );
}

// ── Full announcement hash format (64-byte preimage) ───────────────

#[test]
fn vv_req_reg_004_full_hash_includes_checkpoint_id() {
    // REG-004: Full hash = sha256(checkpoint_singleton_id + inner_hash)
    // The checkpoint_singleton_id is the first 32 bytes of the preimage.
    let ckpt_id = [0xBB; 32];
    let pk = [0xAA; 48];
    let epoch: u64 = 5;

    let inner: [u8; 32] = Sha256::digest(&inner_preimage(epoch, &pk)).into();

    // Verify full hash manually
    let mut full_preimage = Vec::with_capacity(64);
    full_preimage.extend_from_slice(&ckpt_id);
    full_preimage.extend_from_slice(&inner);
    assert_eq!(
        full_preimage.len(),
        64,
        "REG-004: Full preimage must be 64 bytes"
    );

    let expected: [u8; 32] = Sha256::digest(&full_preimage).into();
    let computed = full_hash(&ckpt_id, epoch, &pk);
    assert_eq!(expected, computed);
}

// ── CLVM execution: format correctness ─────────────────────────────

#[test]
fn vv_req_reg_004_clvm_matches_spec_format() {
    // REG-004: CLVM output must match the spec-defined format exactly.
    // This is the canonical cross-impl test.
    let pk = [0xAA; 48];
    let ckpt_id = [0xBB; 32];
    let epoch: u64 = 5;

    let clvm_hash = get_clvm_announcement_hash(&pk, &ckpt_id, epoch);
    let expected = full_hash(&ckpt_id, epoch, &pk);

    assert_eq!(
        clvm_hash.as_slice(),
        expected.as_slice(),
        "REG-004: CLVM announcement hash must match spec format"
    );
}

// ── Known test vectors ─────────────────────────────────────────────

#[test]
fn vv_req_reg_004_test_vector_all_zeros() {
    // REG-004: Test vector — all-zero inputs
    let pk = [0x00; 48];
    let ckpt_id = [0x00; 32];
    let epoch: u64 = 0;

    let clvm_hash = get_clvm_announcement_hash(&pk, &ckpt_id, epoch);
    let expected = full_hash(&ckpt_id, epoch, &pk);

    assert_eq!(clvm_hash.as_slice(), expected.as_slice());
    // Verify it's a 32-byte hash
    assert_eq!(clvm_hash.len(), 32, "REG-004: Hash must be 32 bytes");
}

#[test]
fn vv_req_reg_004_test_vector_all_ff() {
    // REG-004: Test vector — all-0xFF inputs
    let pk = [0xFF; 48];
    let ckpt_id = [0xFF; 32];
    let epoch: u64 = u64::MAX;

    let clvm_hash = get_clvm_announcement_hash(&pk, &ckpt_id, epoch);
    let expected = full_hash(&ckpt_id, epoch, &pk);

    assert_eq!(
        clvm_hash.as_slice(),
        expected.as_slice(),
        "REG-004: All-0xFF vector must match"
    );
}

#[test]
fn vv_req_reg_004_test_vector_epoch_1() {
    // REG-004: Test vector — epoch 1 (common first checkpoint)
    let pk = [0x01; 48];
    let ckpt_id = [0x01; 32];
    let epoch: u64 = 1;

    let clvm_hash = get_clvm_announcement_hash(&pk, &ckpt_id, epoch);
    let expected = full_hash(&ckpt_id, epoch, &pk);

    assert_eq!(clvm_hash.as_slice(), expected.as_slice());
}

#[test]
fn vv_req_reg_004_test_vector_realistic() {
    // REG-004: Realistic test vector with plausible values
    let pk =
        hex::decode("a572cbea904d67468808c8eb50a9450c9721db309128012543902d0ac358a62a").unwrap();
    // Pad to 48 bytes (this isn't a valid BLS point but tests format)
    let mut pk48 = [0u8; 48];
    pk48[..pk.len().min(48)].copy_from_slice(&pk[..pk.len().min(48)]);

    let ckpt_id =
        hex::decode("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap();
    let epoch: u64 = 42;

    let clvm_hash = get_clvm_announcement_hash(&pk48, &ckpt_id, epoch);
    let expected = full_hash(&ckpt_id, epoch, &pk48);

    assert_eq!(clvm_hash.as_slice(), expected.as_slice());
}

// ── Permutation: each field changes the hash ───────────────────────

#[test]
fn vv_req_reg_004_changing_one_pubkey_byte_changes_hash() {
    // REG-004: Flipping a single pubkey byte produces a different hash.
    let mut pk1 = [0xAA; 48];
    let mut pk2 = [0xAA; 48];
    pk2[23] = 0xAB; // flip one byte

    let h1 = get_clvm_announcement_hash(&pk1, &[0xBB; 32], 5);
    let h2 = get_clvm_announcement_hash(&pk2, &[0xBB; 32], 5);

    assert_ne!(
        h1, h2,
        "REG-004: Single byte change in pubkey must change hash"
    );
}

#[test]
fn vv_req_reg_004_changing_one_ckpt_byte_changes_hash() {
    // REG-004: Flipping a single checkpoint ID byte produces a different hash.
    let mut ckpt1 = [0xBB; 32];
    let mut ckpt2 = [0xBB; 32];
    ckpt2[0] = 0xBC;

    let h1 = get_clvm_announcement_hash(&[0xAA; 48], &ckpt1, 5);
    let h2 = get_clvm_announcement_hash(&[0xAA; 48], &ckpt2, 5);

    assert_ne!(
        h1, h2,
        "REG-004: Single byte change in checkpoint ID must change hash"
    );
}

#[test]
fn vv_req_reg_004_adjacent_epochs_different_hash() {
    // REG-004: Epoch N and N+1 produce different hashes.
    let h1 = get_clvm_announcement_hash(&[0xAA; 48], &[0xBB; 32], 100);
    let h2 = get_clvm_announcement_hash(&[0xAA; 48], &[0xBB; 32], 101);

    assert_ne!(
        h1, h2,
        "REG-004: Adjacent epochs must produce different hashes"
    );
}

// ── Announcement does not depend on solution-only fields ───────────

#[test]
fn vv_req_reg_004_hash_independent_of_destination() {
    // REG-004: The announcement hash depends on curried params + epoch only,
    // NOT on collateral_destination or collateral_amount.
    let pk = [0xAA; 48];
    let ckpt_id = [0xBB; 32];
    let epoch: u64 = 5;

    let mut a1 = Allocator::new();
    let p1 = load_puzzle(&mut a1, REG_COIN_HEX);
    let e1 = build_env(&mut a1, &pk, &ckpt_id, epoch, &[0xCC; 32], 1_000_000);
    let (_, o1) = run_puzzle_ok(&mut a1, p1, e1);
    let h1 = conditions_with_opcode(&parse_conditions(&a1, o1), ASSERT_COIN_ANNOUNCEMENT)[0].args
        [0]
    .clone();

    let mut a2 = Allocator::new();
    let p2 = load_puzzle(&mut a2, REG_COIN_HEX);
    let e2 = build_env(&mut a2, &pk, &ckpt_id, epoch, &[0xDD; 32], 999_999);
    let (_, o2) = run_puzzle_ok(&mut a2, p2, e2);
    let h2 = conditions_with_opcode(&parse_conditions(&a2, o2), ASSERT_COIN_ANNOUNCEMENT)[0].args
        [0]
    .clone();

    assert_eq!(
        h1, h2,
        "REG-004: Announcement hash must NOT depend on destination or amount"
    );
}

// ── Spec and documentation ─────────────────────────────────────────

#[test]
fn vv_req_reg_004_puzzle_documents_announcement_format() {
    let src = std::fs::read_to_string("puzzles/registration_coin.rue")
        .expect("Failed to read puzzle source");

    assert!(
        src.contains("membership") && src.contains("0x00"),
        "REG-004: Puzzle must document the announcement format"
    );
}

#[test]
fn vv_req_reg_004_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/registration_coin/specs/REG-004.md")
            .exists(),
        "REG-004: Spec file must exist"
    );
}
