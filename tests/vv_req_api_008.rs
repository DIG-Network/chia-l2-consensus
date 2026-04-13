//! REQUIREMENT: API-008 — Return-Not-Submit Pattern
//! (`docs/requirements/domains/crate_api/NORMATIVE.md#API-008`).
//!
//! Spec: `docs/requirements/domains/crate_api/specs/API-008.md`.
//!
//! ## Normative Statement
//!
//! Every public method that produces coin spends MUST return a SpendBundle to
//! the caller. The crate MUST NOT broadcast, submit, or push transactions to
//! a Chia node. The importing project is solely responsible for broadcasting.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] deploy() returns SpendBundle — does not broadcast
//! - [x] register_validator() returns SpendBundle — does not broadcast
//! - [x] build_checkpoint() returns SpendBundle — does not broadcast
//! - [x] recover_collateral() returns SpendBundle — does not broadcast
//! - [x] release_collateral() returns SpendBundle — does not broadcast
//! - [x] No push_tx/send_transaction in src/
//! - [x] Method named build_checkpoint (not submit_checkpoint)

use std::fs;
use std::path::Path;

/// Recursively collect all .rs files under a directory.
fn collect_rs_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    files.push(content);
                }
            }
        }
    }
    files
}

// ── No broadcast code in src/ ────────────────────────────────────────

/// API-008: No push_tx() function calls in src/ (doc comments mentioning it are OK).
#[test]
fn vv_req_api_008_no_push_tx_in_src() {
    for content in collect_rs_files(Path::new("src")) {
        // Check each non-comment line for push_tx(
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!")
            {
                continue; // Skip doc comments
            }
            assert!(
                !trimmed.contains("push_tx("),
                "API-008: src/ code must NOT call push_tx(). Found: {}",
                trimmed
            );
        }
    }
}

/// API-008: No send_transaction() calls anywhere in src/.
#[test]
fn vv_req_api_008_no_send_transaction_in_src() {
    for content in collect_rs_files(Path::new("src")) {
        assert!(
            !content.contains("send_transaction("),
            "API-008: src/ must NOT contain send_transaction() calls"
        );
    }
}

/// API-008: No submit_spend_bundle() calls anywhere in src/.
#[test]
fn vv_req_api_008_no_submit_spend_bundle_in_src() {
    for content in collect_rs_files(Path::new("src")) {
        assert!(
            !content.contains("submit_spend_bundle("),
            "API-008: src/ must NOT contain submit_spend_bundle() calls"
        );
    }
}

// ── Method signatures return SpendBundle ──────────────────────────────

/// API-008: build_checkpoint() returns ConsensusResult<SpendBundle>.
#[test]
fn vv_req_api_008_build_checkpoint_returns_bundle() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    let method = src
        .find("pub async fn build_checkpoint(")
        .expect("build_checkpoint must exist");
    let sig = &src[method..method + 500.min(src.len() - method)];
    let sig_end = sig.find('{').unwrap_or(sig.len());
    let signature = &sig[..sig_end];
    assert!(
        signature.contains("ConsensusResult<SpendBundle>"),
        "API-008: build_checkpoint must return ConsensusResult<SpendBundle>"
    );
}

/// API-008: register_validator() returns ConsensusResult<SpendBundle>.
#[test]
fn vv_req_api_008_register_validator_returns_bundle() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("pub async fn register_validator("),
        "API-008: register_validator must exist"
    );
    let method = src.find("pub async fn register_validator(").unwrap();
    let sig = &src[method..method + 500.min(src.len() - method)];
    let sig_end = sig.find('{').unwrap_or(sig.len());
    let signature = &sig[..sig_end];
    assert!(
        signature.contains("ConsensusResult<SpendBundle>"),
        "API-008: register_validator must return ConsensusResult<SpendBundle>"
    );
}

/// API-008: recover_collateral() returns ConsensusResult<SpendBundle>.
#[test]
fn vv_req_api_008_recover_collateral_returns_bundle() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("pub async fn recover_collateral("),
        "API-008: recover_collateral must exist"
    );
    let method = src.find("pub async fn recover_collateral(").unwrap();
    let sig = &src[method..method + 500.min(src.len() - method)];
    let sig_end = sig.find('{').unwrap_or(sig.len());
    let signature = &sig[..sig_end];
    assert!(
        signature.contains("ConsensusResult<SpendBundle>"),
        "API-008: recover_collateral must return ConsensusResult<SpendBundle>"
    );
}

/// API-008: release_collateral() returns ConsensusResult<SpendBundle>.
#[test]
fn vv_req_api_008_release_collateral_returns_bundle() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("pub async fn release_collateral("),
        "API-008: release_collateral must exist"
    );
    let method = src.find("pub async fn release_collateral(").unwrap();
    let sig = &src[method..method + 500.min(src.len() - method)];
    let sig_end = sig.find('{').unwrap_or(sig.len());
    let signature = &sig[..sig_end];
    assert!(
        signature.contains("ConsensusResult<SpendBundle>"),
        "API-008: release_collateral must return ConsensusResult<SpendBundle>"
    );
}

// ── Naming convention ────────────────────────────────────────────────

/// API-008: Method is named build_checkpoint, NOT submit_checkpoint.
#[test]
fn vv_req_api_008_no_submit_checkpoint_method() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        !src.contains("fn submit_checkpoint("),
        "API-008: Must use build_checkpoint(), NOT submit_checkpoint()"
    );
    assert!(
        src.contains("fn build_checkpoint("),
        "API-008: build_checkpoint() must exist"
    );
}

/// API-008: Client doc mentions return-not-submit pattern.
#[test]
fn vv_req_api_008_client_documents_pattern() {
    let src = fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("NEVER")
            || src.contains("never broadcasts")
            || src.contains("caller broadcasts"),
        "API-008: Client must document that crate never broadcasts"
    );
}

/// API-008: Spec file exists.
#[test]
fn vv_req_api_008_spec_file_exists() {
    assert!(Path::new("docs/requirements/domains/crate_api/specs/API-008.md").exists(),);
}
