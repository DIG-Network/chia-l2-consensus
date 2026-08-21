//! Puzzle artifact integrity — the committed consensus artifacts match their
//! source-pinned expectations, and carry the format the drivers document.
//!
//! `puzzles/compiled/*.hash` and `*.hex` are tracked source files embedded with
//! `include_str!` (`src/puzzles/*.rs`), so their bytes are the shipped consensus
//! rule. Two independent things must hold, and this suite separates them
//! deliberately:
//!
//! 1. **VALUE** — each committed tree hash equals the `const` pinned in
//!    [`common::puzzle_artifacts`]. Compared in canonical form, so a `0x`
//!    prefix difference can never masquerade as a value difference.
//! 2. **FORMAT** — each committed `.hash` is a `0x`-prefixed 32-byte hex
//!    string. The `0x` prefix is added by a human commit and never by the
//!    build; `rue` emits none, and teaching the generator to add one would
//!    silence the only alarm that survived DIG-Network/dig_ecosystem#9.
//!
//! Only `registration_coin` previously had a `vv_req_*` suite covering its
//! artifacts, and it is the one puzzle that did not drift. All four are covered
//! here.
//!
//! This suite is reachable even with no `rue` installed, which is what makes it
//! independent of `build.rs`: `build.rs` compares the committed artifact against
//! a live compile and fails the BUILD on a mismatch, so a compiler-vs-source
//! divergence never reaches a test at all. These tests instead catch a committed
//! artifact edited without updating its pinned expectation.

mod common;

use common::puzzle_artifacts::EXPECTED_TREE_HASHES;
use common::puzzle_artifacts::{canonical, committed_tree_hash, expected_tree_hash};

/// Every puzzle's committed tree hash equals its source-pinned expectation.
#[test]
fn committed_tree_hashes_match_pinned_expectations() {
    for (stem, expected) in EXPECTED_TREE_HASHES {
        assert_eq!(
            committed_tree_hash(stem),
            expected,
            "puzzles/compiled/{stem}.hash does not match its pinned expectation in \
             tests/common/puzzle_artifacts.rs. If the artifact changed deliberately, \
             move the pin in the same commit and say why."
        );
    }
}

/// Every puzzle's committed `.hash` keeps the `0x`-prefixed 32-byte hex form the
/// puzzle drivers document. Independent of value: this is the assertion that a
/// build script writing raw compiler output would break.
#[test]
fn committed_tree_hashes_are_0x_prefixed_32_byte_hex() {
    for (stem, _) in EXPECTED_TREE_HASHES {
        let path = format!("puzzles/compiled/{stem}.hash");
        let raw =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
        let hash = raw.trim();
        assert!(
            hash.starts_with("0x") && hash.len() == 66,
            "{path} must be a 0x-prefixed 32-byte hex string, got: {hash}"
        );
    }
}

/// Every puzzle's committed `.hex` reveal is non-empty, un-prefixed hex. A drift
/// in the reveal necessarily drifts the tree hash, which the value test above
/// pins, so this asserts only the form the CLVM loaders rely on.
#[test]
fn committed_hex_reveals_are_bare_hex() {
    for (stem, _) in EXPECTED_TREE_HASHES {
        let path = format!("puzzles/compiled/{stem}.hex");
        let raw =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
        let hex = raw.trim();
        assert!(!hex.is_empty(), "{path} must not be empty");
        assert!(
            !hex.starts_with("0x"),
            "{path} must be bare hex with no 0x prefix — the loaders hex::decode it directly"
        );
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "{path} must contain only hex characters"
        );
        assert!(
            hex.len() % 2 == 0,
            "{path} must be a whole number of bytes, got {} chars",
            hex.len()
        );
    }
}

/// The pinned table itself is well-formed: four distinct puzzles, each a
/// canonical 32-byte hex string. A typo in the pin must not read as a drift in
/// the artifact.
#[test]
fn pinned_expectations_are_wellformed_and_distinct() {
    for (stem, expected) in EXPECTED_TREE_HASHES {
        assert_eq!(
            expected.len(),
            64,
            "pinned hash for {stem} must be 64 hex chars (no 0x prefix)"
        );
        assert_eq!(
            canonical(expected),
            expected,
            "pinned hash for {stem} must already be in canonical form"
        );
        assert_eq!(expected_tree_hash(stem), expected);
    }

    let mut hashes: Vec<&str> = EXPECTED_TREE_HASHES.iter().map(|(_, h)| *h).collect();
    hashes.sort_unstable();
    let distinct = hashes.len();
    hashes.dedup();
    assert_eq!(
        hashes.len(),
        distinct,
        "two puzzles share a pinned tree hash — almost certainly a copy-paste in the table"
    );
}
