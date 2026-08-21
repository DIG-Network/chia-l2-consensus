//! Build script: VERIFY the committed Rue puzzle artifacts. Never rewrite them.
//!
//! `puzzles/compiled/*.hex` and `*.hash` are TRACKED source files, embedded into
//! the library with `include_str!` (`src/puzzles/*.rs`), so their bytes *are* the
//! shipped consensus rule. This script therefore compiles each `puzzles/*.rue`
//! into `OUT_DIR` and compares the result against the committed artifact,
//! panicking with both values on any mismatch.
//!
//! It deliberately does NOT write into the source tree. An earlier revision
//! `fs::write`-d the compiler's output straight over the committed files, which
//! made a divergence between the source and the compiler invisible at build time
//! and let a regenerated artifact reach two immutable crates.io releases
//! (DIG-Network/dig_ecosystem#9).
//!
//! ## Comparison is on the canonical form, and the `0x` prefix is a human's job
//!
//! The committed `.hash` files carry a `0x` prefix; `rue` emits none. Both forms
//! are compared after normalisation (see [`canonical`]) so a prefix difference is
//! not reported as a value difference. This script never *adds* the prefix:
//! teaching the generator to match the committed form would silence the only
//! alarm that survived the incident above. The prefix stays a human commit.
//!
//! ## Without `rue`, verification is SKIPPED, not satisfied
//!
//! A plain `cargo build` on a machine with no `rue` still works — the committed
//! artifacts are all the library needs. The skip is announced as a
//! `cargo:warning`, and CI installs the pinned compiler (`puzzles/RUE_VERSION`)
//! so the check is never merely skipped there. A silent skip is how the
//! committed artifacts went unverified for four months.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One compiled form of a puzzle: the `rue build` flag that emits it, and the
/// extension of the committed file that must equal it.
struct Form {
    flag: &'static str,
    ext: &'static str,
}

const FORMS: [Form; 2] = [
    Form {
        flag: "--hex",
        ext: "hex",
    },
    Form {
        flag: "--hash",
        ext: "hash",
    },
];

/// Printed when a committed artifact and the compiler disagree. Kept as one
/// flush-left line so the build log carries no stray indentation.
const MISMATCH_REMEDY: &str = "A committed puzzle artifact does not match the compiler. It is a tracked source file embedded via include_str!, so these bytes are the shipped consensus rule, and this build script will not silently update it. Decide which value is authoritative, then commit the artifact deliberately.";

fn main() {
    let puzzles_dir = Path::new("puzzles");
    let compiled_dir = puzzles_dir.join("compiled");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo always sets OUT_DIR"));

    println!("cargo:rerun-if-changed=puzzles");

    if let Err(e) = Command::new("rue").arg("--help").output() {
        println!(
            "cargo:warning=rue not found ({e}); SKIPPING puzzle artifact verification. \
             CI installs the pinned compiler from puzzles/RUE_VERSION -- see README."
        );
        return;
    }

    let mut mismatches: Vec<String> = Vec::new();

    for rue_path in rue_sources(puzzles_dir) {
        println!("cargo:rerun-if-changed={}", rue_path.display());
        let stem = rue_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| panic!("non-UTF-8 puzzle filename: {}", rue_path.display()))
            .to_owned();

        for form in &FORMS {
            let compiled = compile(&rue_path, form.flag);

            // OUT_DIR is the only place this script ever writes. Keeping the fresh
            // output on disk makes a mismatch diffable without re-running `rue`.
            let fresh_path = out_dir.join(format!("{stem}.{ext}", ext = form.ext));
            fs::write(&fresh_path, &compiled)
                .unwrap_or_else(|e| panic!("failed to write {}: {e}", fresh_path.display()));

            let committed_path = compiled_dir.join(format!("{stem}.{ext}", ext = form.ext));
            let committed = fs::read_to_string(&committed_path).unwrap_or_else(|e| {
                panic!(
                    "{} is missing or unreadable ({e}). It is a tracked source file \
                     embedded via include_str!; regenerate it deliberately and commit it.",
                    committed_path.display()
                )
            });

            if canonical(&committed) != canonical(&compiled) {
                mismatches.push(format!(
                    "  {committed_file}\n    committed: {committed_value}\n    \
                     compiled : {compiled_value}\n    \
                     full compiled output: {fresh_file}",
                    committed_file = committed_path.display(),
                    committed_value = abbreviate(&canonical(&committed)),
                    compiled_value = abbreviate(&canonical(&compiled)),
                    fresh_file = fresh_path.display(),
                ));
            }
        }
    }

    // Every artifact is checked before failing: one build should reveal the whole
    // divergence, not just the alphabetically-first file of it.
    if !mismatches.is_empty() {
        panic!("{MISMATCH_REMEDY}\n{}", mismatches.join("\n"));
    }
}

/// A value short enough to read in a build log. A 32-byte tree hash is printed
/// whole; a multi-kilobyte CLVM reveal is elided in the middle, with the full
/// text left in `OUT_DIR` for anyone who needs to diff it.
fn abbreviate(value: &str) -> String {
    const KEEP: usize = 32;
    if value.len() <= KEEP * 2 {
        return value.to_owned();
    }
    format!(
        "{head}...{tail} ({len} chars)",
        head = &value[..KEEP],
        tail = &value[value.len() - KEEP..],
        len = value.len(),
    )
}

/// The `puzzles/*.rue` sources, in a stable order so build output is reproducible.
fn rue_sources(puzzles_dir: &Path) -> Vec<PathBuf> {
    let mut sources: Vec<PathBuf> = fs::read_dir(puzzles_dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", puzzles_dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rue"))
        .collect();
    sources.sort();
    sources
}

/// Compile one puzzle to one form. Panics on a compiler error: `rue` is known to
/// be present by this point, so a failure here is a real defect in the puzzle.
fn compile(rue_path: &Path, flag: &str) -> String {
    let output = Command::new("rue")
        .args(["build", &rue_path.to_string_lossy(), flag])
        .output()
        .unwrap_or_else(|e| panic!("failed to run rue build {flag}: {e}"));

    if !output.status.success() {
        panic!(
            "rue build {flag} failed for {}: {}",
            rue_path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The comparable form of an artifact: trimmed, un-prefixed, lowercase. Lets a
/// committed `0x`-prefixed hash be compared with the compiler's bare output
/// without either side being rewritten to match the other.
fn canonical(artifact: &str) -> String {
    artifact
        .trim()
        .trim_start_matches("0x")
        .to_ascii_lowercase()
}
