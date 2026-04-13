//! REQUIREMENT: SETUP-001 — Rust Toolchain
//! (`docs/requirements/domains/setup/NORMATIVE.md#SETUP-001`).
//!
//! Spec: `docs/requirements/domains/setup/specs/SETUP-001.md`.
//!
//! ## Normative statement
//! The project MUST use Rust 2021 edition with stable toolchain. Developers
//! MUST have `rustfmt` and `clippy` installed for code formatting and linting.
//!
//! ## How the tests prove the requirement
//! Tests invoke each required tool (`rustc`, `cargo`, `rustfmt`, `clippy`) and
//! verify they execute successfully, then parse `Cargo.toml` to confirm edition
//! is 2021.

use std::process::Command;

/// Verifies `rustc --version` succeeds and returns a version string.
#[test]
fn vv_req_setup_001_rustc_available() {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("SETUP-001: rustc must be available in PATH");

    assert!(
        output.status.success(),
        "SETUP-001: rustc --version must succeed"
    );

    let version = String::from_utf8_lossy(&output.stdout);
    assert!(
        version.starts_with("rustc "),
        "SETUP-001: rustc version must start with 'rustc ', got: {}",
        version.trim()
    );
}

/// Verifies `cargo --version` succeeds.
#[test]
fn vv_req_setup_001_cargo_available() {
    let output = Command::new("cargo")
        .arg("--version")
        .output()
        .expect("SETUP-001: cargo must be available in PATH");

    assert!(
        output.status.success(),
        "SETUP-001: cargo --version must succeed"
    );

    let version = String::from_utf8_lossy(&output.stdout);
    assert!(
        version.starts_with("cargo "),
        "SETUP-001: cargo version must start with 'cargo ', got: {}",
        version.trim()
    );
}

/// Verifies `rustfmt --version` succeeds (required for code formatting).
#[test]
fn vv_req_setup_001_rustfmt_available() {
    let output = Command::new("rustfmt")
        .arg("--version")
        .output()
        .expect("SETUP-001: rustfmt must be available in PATH");

    assert!(
        output.status.success(),
        "SETUP-001: rustfmt --version must succeed"
    );

    let version = String::from_utf8_lossy(&output.stdout);
    assert!(
        version.contains("rustfmt"),
        "SETUP-001: rustfmt version must contain 'rustfmt', got: {}",
        version.trim()
    );
}

/// Verifies `cargo clippy --version` succeeds (required for linting).
#[test]
fn vv_req_setup_001_clippy_available() {
    let output = Command::new("cargo")
        .args(["clippy", "--version"])
        .output()
        .expect("SETUP-001: cargo clippy must be available");

    assert!(
        output.status.success(),
        "SETUP-001: cargo clippy --version must succeed"
    );

    let version = String::from_utf8_lossy(&output.stdout);
    assert!(
        version.contains("clippy"),
        "SETUP-001: clippy version must contain 'clippy', got: {}",
        version.trim()
    );
}

/// Verifies the project uses Rust 2021 edition by parsing Cargo.toml.
#[test]
fn vv_req_setup_001_edition_2021() {
    let cargo_toml =
        std::fs::read_to_string("Cargo.toml").expect("SETUP-001: Cargo.toml must be readable");

    assert!(
        cargo_toml.contains("edition") && cargo_toml.contains("\"2021\""),
        "SETUP-001: Cargo.toml must specify edition = \"2021\""
    );
}
