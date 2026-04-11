//! REQUIREMENT: SETUP-006 — Build Configuration
//! (`docs/requirements/domains/setup/NORMATIVE.md#SETUP-006`).
//!
//! Spec: `docs/requirements/domains/setup/specs/SETUP-006.md`.
//!
//! ## Normative statement
//! The project MUST have a `[profile.release]` section with `opt-level = 3`
//! and `lto = true`, pass `cargo fmt --check`, pass `cargo clippy -- -D warnings`,
//! and have a `rust-toolchain.toml` for reproducible builds.
//!
//! ## How the tests prove the requirement
//! 1. **Release profile**: Cargo.toml has [profile.release] with opt-level=3 and lto=true.
//! 2. **Formatting**: `cargo fmt --check` passes (no unformatted code).
//! 3. **Linting**: `cargo clippy -- -D warnings` passes (no warnings).
//! 4. **Toolchain pinned**: rust-toolchain.toml exists.
//!
//! ## Completeness: HIGH
//! ## Gaps: Does not verify specific Rust version in toolchain file.

use std::path::Path;
use std::process::Command;

/// Verifies Cargo.toml has [profile.release] with opt-level=3 and lto=true
/// for maximum performance of the cryptographic operations.
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

/// Runs `cargo fmt --check` to verify all source code is properly formatted.
/// A passing result means no formatting changes are needed.
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

/// Runs `cargo clippy -- -D warnings` to verify no lint warnings exist.
/// Passing means the code follows Rust best practices.
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

/// Verifies rust-toolchain.toml exists for reproducible builds across
/// developer environments and CI.
#[test]
fn vv_req_setup_006_rust_toolchain_configured() {
    // Verify rust-toolchain.toml exists
    assert!(
        Path::new("rust-toolchain.toml").exists(),
        "SETUP-006: rust-toolchain.toml must exist for reproducible builds"
    );
}
