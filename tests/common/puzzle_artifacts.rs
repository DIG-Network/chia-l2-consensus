//! The expected tree hash of every Rue puzzle, pinned in Rust source.
//!
//! ## Why the expectation lives here and not in a generated file
//!
//! `puzzles/compiled/*.hash` are tracked source files embedded with
//! `include_str!`, so their bytes are the shipped consensus rule. Until
//! DIG-Network/dig_ecosystem#9, `build.rs` regenerated those files on every
//! build — which meant the two tests that guarded them
//! (`..._compiled_hex_matches_live_build` and `..._compiled_hash_matches_live_build`)
//! compared a freshly-built artifact against a file the same compiler had
//! rewritten seconds earlier. **The gate and the thing it guarded shared a
//! producer, so neither could ever fail.**
//!
//! The table below breaks that loop: the expectation is a `const` in Rust
//! source, which no build step writes. Changing a committed artifact without
//! also changing this table is now a test failure.
//!
//! ## Updating the table
//!
//! Only ever by hand, in the same commit as the artifact it describes, with the
//! reason in the commit message. A regenerated artifact and a regenerated
//! expectation together prove nothing.
//!
//! The values below are the artifacts committed on `main` as of 0.4.0. Three of
//! them differ from what the pinned compiler produces (see
//! DIG-Network/dig_ecosystem#9); reconciling that divergence is a separate,
//! deliberate step, and this table moves when it does.

/// Every puzzle: source stem, and the tree hash its committed `.hash` must hold.
///
/// Canonical form — lowercase, no `0x` prefix. The committed files carry the
/// prefix; [`canonical`] normalises it away so a formatting difference is never
/// reported as a value difference.
pub const EXPECTED_TREE_HASHES: [(&str, &str); 4] = [
    (
        "registration_coin",
        "8e6a277b006c878ff0460b50621decb59b0b372966bd686fc77001ed0f4154c6",
    ),
    (
        "checkpoint_inner",
        "a5ab9c0f24fb913cc2affe69e26df3991e816c733ee0a5e52e811e931fcc3d16",
    ),
    (
        "network_coin_inner",
        "81865e9faae271365aa4eeac8a05fe0a37fead6a96507c751cffe2c45a95dd20",
    ),
    (
        "withdraw_delay_coin",
        "5f5ab1bd89b7592c57a5c770d42d060d03dab21b50e02a94ee686aa075a50f19",
    ),
];

/// The pinned tree hash for one puzzle stem.
///
/// # Panics
/// If `stem` is not a known puzzle — a typo in a test must fail loudly rather
/// than silently skip the comparison it was written to make.
pub fn expected_tree_hash(stem: &str) -> &'static str {
    EXPECTED_TREE_HASHES
        .iter()
        .find(|(name, _)| *name == stem)
        .map(|(_, hash)| *hash)
        .unwrap_or_else(|| panic!("no pinned tree hash for puzzle {stem}"))
}

/// Reads the committed `.hash` for one puzzle in [`canonical`] form.
pub fn committed_tree_hash(stem: &str) -> String {
    let path = format!("puzzles/compiled/{stem}.hash");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read committed artifact {path}: {e}"));
    canonical(&raw)
}

/// The comparable form of an artifact: trimmed, un-prefixed, lowercase. Mirrors
/// `build.rs`'s own normalisation so the two agree on what "equal" means.
pub fn canonical(artifact: &str) -> String {
    artifact
        .trim()
        .trim_start_matches("0x")
        .to_ascii_lowercase()
}
