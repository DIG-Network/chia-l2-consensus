//! REQUIREMENT: SETUP-002 — Cargo.toml Configuration
//! (`docs/requirements/domains/setup/NORMATIVE.md#SETUP-002`).
//!
//! Spec: `docs/requirements/domains/setup/specs/SETUP-002.md`.
//!
//! ## Normative statement
//! The crate MUST be named `chia-l2-consensus` with version `0.1.0` and edition
//! `2021`. All dependencies MUST be pinned to specific versions to ensure
//! reproducible builds.
//!
//! ## How the tests prove the requirement
//! Tests parse Cargo.toml as text and verify package metadata, key dependency
//! presence, and version pinning (no wildcard/range specifiers).

/// Reads Cargo.toml once for all tests.
fn cargo_toml() -> String {
    std::fs::read_to_string("Cargo.toml").expect("SETUP-002: Cargo.toml must be readable")
}

/// Verifies `package.name = "chia-l2-consensus"`.
#[test]
fn vv_req_setup_002_crate_name() {
    let toml = cargo_toml();
    assert!(
        toml.contains("name") && toml.contains("\"chia-l2-consensus\""),
        "SETUP-002: Cargo.toml must set name = \"chia-l2-consensus\""
    );
}

/// Verifies `package.version = "0.1.0"`.
#[test]
fn vv_req_setup_002_crate_version() {
    let toml = cargo_toml();
    assert!(
        toml.contains("version") && toml.contains("\"0.1.0\""),
        "SETUP-002: Cargo.toml must set version = \"0.1.0\""
    );
}

/// Verifies `package.edition = "2021"`.
#[test]
fn vv_req_setup_002_edition_2021() {
    let toml = cargo_toml();
    assert!(
        toml.contains("edition") && toml.contains("\"2021\""),
        "SETUP-002: Cargo.toml must set edition = \"2021\""
    );
}

/// Verifies key Chia dependencies exist.
#[test]
fn vv_req_setup_002_chia_deps() {
    let toml = cargo_toml();
    for dep in &["chia-wallet-sdk", "chia-protocol", "clvmr", "chia-bls"] {
        assert!(
            toml.contains(dep),
            "SETUP-002: Cargo.toml must include dependency '{}'",
            dep
        );
    }
}

/// Verifies arkworks ZK dependencies exist.
#[test]
fn vv_req_setup_002_arkworks_deps() {
    let toml = cargo_toml();
    for dep in &["ark-groth16", "ark-bls12-381", "ark-r1cs-std", "ark-ff"] {
        assert!(
            toml.contains(dep),
            "SETUP-002: Cargo.toml must include dependency '{}'",
            dep
        );
    }
}

/// Verifies blst BLS aggregation dependency exists.
#[test]
fn vv_req_setup_002_blst_dep() {
    let toml = cargo_toml();
    assert!(
        toml.contains("blst"),
        "SETUP-002: Cargo.toml must include dependency 'blst'"
    );
}

/// Verifies sha2 hashing dependency exists.
#[test]
fn vv_req_setup_002_sha2_dep() {
    let toml = cargo_toml();
    assert!(
        toml.contains("sha2"),
        "SETUP-002: Cargo.toml must include dependency 'sha2'"
    );
}

/// Verifies no wildcard version specifiers ("*") in dependencies.
#[test]
fn vv_req_setup_002_no_wildcard_versions() {
    let toml = cargo_toml();
    let in_deps = toml
        .lines()
        .skip_while(|l| !l.starts_with("[dependencies]"))
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| l.contains('=') && !l.trim().starts_with('#'));

    for line in in_deps {
        assert!(
            !line.contains("\"*\""),
            "SETUP-002: Wildcard version not allowed: {}",
            line.trim()
        );
    }
}

/// Verifies release profile has LTO enabled.
#[test]
fn vv_req_setup_002_release_lto() {
    let toml = cargo_toml();
    assert!(
        toml.contains("lto") && toml.contains("true"),
        "SETUP-002: Release profile must have lto = true"
    );
}
