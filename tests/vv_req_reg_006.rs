//! REQUIREMENT: REG-006 — Epoch Replay Protection
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-006`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-006.md`.
//!
//! Implementation: `puzzles/registration_coin.rue` (compiled to CLVM).
//!
//! Verifies that the epoch included in the announcement hash prevents
//! replay of old non-membership announcements after a validator re-registers.
//! The epoch from the solution directly affects the announcement hash,
//! so a stale epoch produces a different hash that won't match.

mod common;

use clvmr::Allocator;
use sha2::{Digest, Sha256};

use common::clvm::*;

const REG_COIN_HEX: &str = include_str!("../puzzles/compiled/registration_coin.hex");

fn get_announcement_hash(pk: &[u8], ckpt_id: &[u8], epoch: u64) -> Vec<u8> {
    let mut a = Allocator::new();
    let puzzle = load_puzzle(&mut a, REG_COIN_HEX);
    let env = build_reg_coin_env(&mut a, pk, ckpt_id, epoch, &[0xCC; 32], 1_000_000);
    let (_, output) = run_puzzle_ok(&mut a, puzzle, env);
    let conds = parse_conditions(&a, output);
    conditions_with_opcode(&conds, ASSERT_COIN_ANNOUNCEMENT)[0].args[0].clone()
}

fn expected_hash(ckpt_id: &[u8], epoch: u64, pk: &[u8]) -> [u8; 32] {
    let mut inner = Vec::new();
    inner.extend_from_slice(b"membership");
    inner.extend_from_slice(&epoch.to_be_bytes());
    inner.extend_from_slice(pk);
    inner.push(0x00);
    let inner_hash: [u8; 32] = Sha256::digest(&inner).into();
    let mut full = Vec::new();
    full.extend_from_slice(ckpt_id);
    full.extend_from_slice(&inner_hash);
    Sha256::digest(&full).into()
}

// ── Epoch changes announcement hash ────────────────────────────────

#[test]
fn vv_req_reg_006_different_epochs_different_hashes() {
    // REG-006: Each epoch produces a unique announcement hash.
    let pk = [0xAA; 48];
    let ckpt = [0xBB; 32];
    let h0 = get_announcement_hash(&pk, &ckpt, 0);
    let h1 = get_announcement_hash(&pk, &ckpt, 1);
    let h2 = get_announcement_hash(&pk, &ckpt, 2);
    let h100 = get_announcement_hash(&pk, &ckpt, 100);

    assert_ne!(h0, h1, "REG-006: Epoch 0 vs 1");
    assert_ne!(h1, h2, "REG-006: Epoch 1 vs 2");
    assert_ne!(h0, h100, "REG-006: Epoch 0 vs 100");
    assert_ne!(h2, h100, "REG-006: Epoch 2 vs 100");
}

#[test]
fn vv_req_reg_006_sequential_epochs_all_unique() {
    // REG-006: 10 sequential epochs must all produce unique hashes.
    let pk = [0x42; 48];
    let ckpt = [0x77; 32];
    let hashes: Vec<Vec<u8>> = (0..10)
        .map(|e| get_announcement_hash(&pk, &ckpt, e))
        .collect();

    for i in 0..10 {
        for j in (i + 1)..10 {
            assert_ne!(
                hashes[i], hashes[j],
                "REG-006: Epoch {} and {} must produce different hashes",
                i, j
            );
        }
    }
}

// ── Replay scenario ────────────────────────────────────────────────

#[test]
fn vv_req_reg_006_old_epoch_announcement_doesnt_match_new() {
    // REG-006: Replay scenario — validator exits at epoch 6, re-registers,
    // tries to use epoch 6 announcement at epoch 8. The hashes differ.
    let pk = [0xAA; 48];
    let ckpt = [0xBB; 32];

    let hash_epoch_6 = get_announcement_hash(&pk, &ckpt, 6);
    let hash_epoch_8 = get_announcement_hash(&pk, &ckpt, 8);

    assert_ne!(
        hash_epoch_6, hash_epoch_8,
        "REG-006: Old epoch 6 hash must not match epoch 8 — replay prevented"
    );
}

#[test]
fn vv_req_reg_006_same_epoch_same_hash() {
    // REG-006: Same epoch + same params = same hash (deterministic).
    let pk = [0xAA; 48];
    let ckpt = [0xBB; 32];

    let h1 = get_announcement_hash(&pk, &ckpt, 42);
    let h2 = get_announcement_hash(&pk, &ckpt, 42);

    assert_eq!(h1, h2, "REG-006: Same inputs must produce same hash");
}

// ── Cross-impl verification at boundary epochs ─────────────────────

#[test]
fn vv_req_reg_006_cross_impl_epoch_0() {
    let pk = [0xAA; 48];
    let ckpt = [0xBB; 32];
    let clvm = get_announcement_hash(&pk, &ckpt, 0);
    let rust = expected_hash(&ckpt, 0, &pk);
    assert_eq!(
        clvm.as_slice(),
        rust.as_slice(),
        "REG-006: Cross-impl at epoch 0"
    );
}

#[test]
fn vv_req_reg_006_cross_impl_epoch_255() {
    // Edge: epoch 255 = 0xFF = 1 byte in CLVM but 8 bytes in wire format
    let pk = [0xAA; 48];
    let ckpt = [0xBB; 32];
    let clvm = get_announcement_hash(&pk, &ckpt, 255);
    let rust = expected_hash(&ckpt, 255, &pk);
    assert_eq!(
        clvm.as_slice(),
        rust.as_slice(),
        "REG-006: Cross-impl at epoch 255"
    );
}

#[test]
fn vv_req_reg_006_cross_impl_epoch_256() {
    // Edge: epoch 256 = 0x0100 = 2 bytes in CLVM
    let pk = [0xAA; 48];
    let ckpt = [0xBB; 32];
    let clvm = get_announcement_hash(&pk, &ckpt, 256);
    let rust = expected_hash(&ckpt, 256, &pk);
    assert_eq!(
        clvm.as_slice(),
        rust.as_slice(),
        "REG-006: Cross-impl at epoch 256"
    );
}

#[test]
fn vv_req_reg_006_cross_impl_epoch_max_minus_1() {
    let pk = [0xAA; 48];
    let ckpt = [0xBB; 32];
    let clvm = get_announcement_hash(&pk, &ckpt, u64::MAX - 1);
    let rust = expected_hash(&ckpt, u64::MAX - 1, &pk);
    assert_eq!(
        clvm.as_slice(),
        rust.as_slice(),
        "REG-006: Cross-impl at near-max epoch"
    );
}

// ── Epoch encoding is exactly 8 bytes ──────────────────────────────

#[test]
fn vv_req_reg_006_epoch_encoding_consistent() {
    // REG-006: The int_to_8_bytes_be helper must produce consistent 8-byte
    // encoding. Epochs 0 and 1 differ by exactly the last byte.
    let pk = [0xAA; 48];
    let ckpt = [0xBB; 32];

    // If epoch were variable-length, small values would hash differently
    // than if they were padded to 8 bytes. Cross-impl check catches this.
    for epoch in [0u64, 1, 127, 128, 255, 256, 65535, 65536, 1_000_000] {
        let clvm = get_announcement_hash(&pk, &ckpt, epoch);
        let rust = expected_hash(&ckpt, epoch, &pk);
        assert_eq!(
            clvm.as_slice(),
            rust.as_slice(),
            "REG-006: Cross-impl must match at epoch {}",
            epoch
        );
    }
}

// ── Spec ───────────────────────────────────────────────────────────

#[test]
fn vv_req_reg_006_spec_file_exists() {
    assert!(
        std::path::Path::new("docs/requirements/domains/registration_coin/specs/REG-006.md")
            .exists()
    );
}

#[test]
fn vv_req_reg_006_puzzle_documents_epoch() {
    let src = std::fs::read_to_string("puzzles/registration_coin.rue").unwrap();
    assert!(
        src.contains("REG-006") || src.contains("epoch") || src.contains("replay"),
        "REG-006: Puzzle must reference epoch/replay protection"
    );
}
