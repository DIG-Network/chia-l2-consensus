//! REQUIREMENT: SETUP-006 — Build configuration
//! (`docs/requirements/domains/setup/NORMATIVE.md#SETUP-006`).
//!
//! Spec: `docs/requirements/domains/setup/specs/SETUP-006.md`.
//!
//! Verifies build configuration: release profile, clippy, fmt.

use std::path::Path;
use std::process::Command;

#[test]
fn vv_req_setup_006_release_profile_configured() {
    // Verify Cargo.toml contains release profile settings
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Should read Cargo.toml");

    assert!(
        cargo_toml.contains("[profile.release]"),
        "SETUP-006: Cargo.toml must have [profile.release] section"
    );
    assert!(
        cargo_toml.contains("opt-level = 3") || cargo_toml.contains("opt-level=3"),
        "SETUP-006: Release profile must have opt-level = 3"
    );
    assert!(
        cargo_toml.contains("lto = true") || cargo_toml.contains("lto=true"),
        "SETUP-006: Release profile must have lto = true"
    );
}

#[test]
fn vv_req_setup_006_cargo_fmt_check_passes() {
    // Run cargo fmt --check to verify formatting
    let output = Command::new("cargo")
        .args(["fmt", "--check"])
        .output()
        .expect("Failed to run cargo fmt");

    assert!(
        output.status.success(),
        "SETUP-006: cargo fmt --check must pass. Run `cargo fmt` to fix.\nStderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn vv_req_setup_006_cargo_clippy_passes() {
    // Run cargo clippy with warnings as errors
    let output = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .output()
        .expect("Failed to run cargo clippy");

    assert!(
        output.status.success(),
        "SETUP-006: cargo clippy -- -D warnings must pass.\nStderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn vv_req_setup_006_rust_toolchain_configured() {
    // Verify rust-toolchain.toml exists
    assert!(
        Path::new("rust-toolchain.toml").exists(),
        "SETUP-006: rust-toolchain.toml must exist for reproducible builds"
    );
}
