//! REQUIREMENT: SETUP-005 — Rue Tooling
//! (`docs/requirements/domains/setup/NORMATIVE.md#SETUP-005`).
//!
//! Spec: `docs/requirements/domains/setup/specs/SETUP-005.md`.
//!
//! Verifies that Rue tooling is available for compiling puzzles.

use std::path::Path;
use std::process::Command;

#[test]
fn vv_req_setup_005_rue_compiler_available() {
    // Check if rue command is available
    let output = Command::new("rue")
        .arg("--help")
        .output();

    match output {
        Ok(result) => {
            // rue --help should succeed (exit code 0)
            assert!(
                result.status.success(),
                "SETUP-005: rue --help should succeed. Got: {}",
                String::from_utf8_lossy(&result.stderr)
            );

            // Should contain usage info
            let stdout = String::from_utf8_lossy(&result.stdout);
            assert!(
                stdout.contains("build") || stdout.contains("COMMAND"),
                "SETUP-005: rue --help should show build command"
            );
        }
        Err(e) => {
            panic!(
                "SETUP-005: Rue compiler not found. Install from https://rue-lang.dev/\nError: {}",
                e
            );
        }
    }
}

#[test]
fn vv_req_setup_005_puzzle_source_files_exist() {
    // Verify Rue source files exist
    assert!(
        Path::new("puzzles/network_coin_inner.rue").exists(),
        "SETUP-005: puzzles/network_coin_inner.rue must exist"
    );
    assert!(
        Path::new("puzzles/registration_coin.rue").exists(),
        "SETUP-005: puzzles/registration_coin.rue must exist"
    );
    assert!(
        Path::new("puzzles/checkpoint_inner.rue").exists(),
        "SETUP-005: puzzles/checkpoint_inner.rue must exist"
    );
}

#[test]
fn vv_req_setup_005_compiled_directory_exists() {
    // Verify compiled output directory exists
    assert!(
        Path::new("puzzles/compiled").exists(),
        "SETUP-005: puzzles/compiled/ directory must exist"
    );
    assert!(
        Path::new("puzzles/compiled").is_dir(),
        "SETUP-005: puzzles/compiled must be a directory"
    );
}
