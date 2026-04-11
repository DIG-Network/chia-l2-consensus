//! REQUIREMENT: NET-001 — Network Coin Singleton Identity
//! (`docs/requirements/domains/network_coin/NORMATIVE.md#NET-001`).
//!
//! Spec: `docs/requirements/domains/network_coin/specs/NET-001.md`.
//!
//! ## Normative Statement
//!
//! The network coin MUST be implemented as a Chia singleton to guarantee
//! exactly one instance exists with a given launcher ID. The singleton wrapper
//! (`singleton_top_layer_v1_1`) enforces uniqueness, lineage, and odd-amount
//! recreation. Without this, attackers could deploy fake network coins.
//!
//! ## How These Tests Prove the Requirement
//!
//! Tests verify: the Rue puzzle compiles successfully, produces valid CLVM
//! output, compilation is deterministic, curried parameters (registration_coin_-
//! mod_hash, collateral_amount, checkpoint_singleton_id) are declared, solution
//! parameters (new_validator_pubkey, conditions) are present, the puzzle returns
//! List<Condition>, singleton usage is documented, and the inner-puzzle pattern
//! is followed (fn main, wrapper documentation).
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Network coin puzzle compiles (rue build succeeds)
//! - [x] Puzzle has correct curried parameters (3 parameters verified)
//! - [x] Puzzle has correct solution parameters (2 parameters verified)
//! - [x] Puzzle returns List<Condition>
//! - [x] Singleton wrapper documented in puzzle source
//! - [x] Inner puzzle pattern followed (fn main, wrapper reference)
//! - [ ] Singleton wrapper actually works (tested in NET-006 simulator test)
//! - [ ] Only one network coin can exist at any time (requires simulator)
//! - [ ] Duplicate network coin creation fails (requires simulator)
//! - [ ] Singleton lineage verifiable (requires simulator)
//!
//! ## Gaps
//!
//! These tests verify the puzzle source structure via string inspection and
//! compilation. They do NOT execute the CLVM or wrap with the singleton layer.
//! Full singleton behavior is tested in `vv_req_net_006.rs` (simulator test).

use std::process::Command;

/// Helper to run rue build and check for successful compilation
fn puzzle_compiles(puzzle_path: &str) -> bool {
    let output = Command::new("rue")
        .args(["build", puzzle_path])
        .output()
        .expect("Failed to execute rue build");

    output.status.success()
}

/// Helper to get puzzle output (CLVM bytecode)
fn get_puzzle_clvm(puzzle_path: &str) -> Option<String> {
    let output = Command::new("rue")
        .args(["build", puzzle_path])
        .output()
        .expect("Failed to execute rue build");

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

// Verifies that `rue build puzzles/network_coin_inner.rue` succeeds. If the
// puzzle has syntax errors or type mismatches, it cannot be deployed. This is
// the most basic correctness gate.
#[test]
fn vv_req_net_001_puzzle_compiles() {
    // NET-001: Network coin inner puzzle must compile
    assert!(
        puzzle_compiles("puzzles/network_coin_inner.rue"),
        "NET-001: network_coin_inner.rue must compile with rue build"
    );
}

// Verifies the compiler produces non-empty CLVM output starting with '('.
// This confirms the puzzle compiles to a valid s-expression, not an error
// message or empty output.
#[test]
fn vv_req_net_001_puzzle_produces_clvm() {
    // NET-001: Compiled puzzle produces valid CLVM output
    let clvm = get_puzzle_clvm("puzzles/network_coin_inner.rue");
    assert!(clvm.is_some(), "NET-001: Puzzle must produce CLVM output");

    let clvm = clvm.unwrap();
    // CLVM starts with (a or (q or similar
    assert!(
        clvm.starts_with('('),
        "NET-001: CLVM output must be valid s-expression"
    );
}

// Verifies that two compilations of the same source produce identical CLVM.
// Non-deterministic compilation would make puzzle hash unpredictable,
// breaking the singleton identity.
#[test]
fn vv_req_net_001_puzzle_is_deterministic() {
    // NET-001: Puzzle compilation is deterministic
    let clvm1 = get_puzzle_clvm("puzzles/network_coin_inner.rue");
    let clvm2 = get_puzzle_clvm("puzzles/network_coin_inner.rue");

    assert!(clvm1.is_some() && clvm2.is_some());
    assert_eq!(
        clvm1.unwrap(),
        clvm2.unwrap(),
        "NET-001: Puzzle compilation must be deterministic"
    );
}

// Verifies the three curried parameters are declared in the puzzle source:
// registration_coin_mod_hash, collateral_amount, checkpoint_singleton_id.
// These are fixed at deployment and define the network's registration rules.
#[test]
fn vv_req_net_001_puzzle_has_curried_params() {
    // NET-001: Network coin inner puzzle accepts curried parameters:
    // - registration_coin_mod_hash: Bytes32
    // - collateral_amount: Int
    // - checkpoint_singleton_id: Bytes32

    // Read the puzzle source to verify parameters
    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Verify curried parameters are declared
    assert!(
        puzzle_source.contains("registration_coin_mod_hash: Bytes32"),
        "NET-001: Puzzle must have registration_coin_mod_hash parameter"
    );
    assert!(
        puzzle_source.contains("collateral_amount: Int"),
        "NET-001: Puzzle must have collateral_amount parameter"
    );
    assert!(
        puzzle_source.contains("checkpoint_singleton_id: Bytes32"),
        "NET-001: Puzzle must have checkpoint_singleton_id parameter"
    );
}

// Verifies the two solution parameters are declared: new_validator_pubkey
// and conditions. These are provided per-spend and define the registration.
#[test]
fn vv_req_net_001_puzzle_has_solution_params() {
    // NET-001: Network coin inner puzzle accepts solution parameters:
    // - new_validator_pubkey: PublicKey
    // - conditions: List<Condition>

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("new_validator_pubkey: PublicKey"),
        "NET-001: Puzzle must have new_validator_pubkey parameter"
    );
    assert!(
        puzzle_source.contains("conditions: List<Condition>"),
        "NET-001: Puzzle must have conditions parameter"
    );
}

// Verifies the puzzle declares `-> List<Condition>` return type. This is
// required by the singleton wrapper to process inner puzzle output.
#[test]
fn vv_req_net_001_puzzle_returns_conditions() {
    // NET-001: Network coin inner puzzle returns List<Condition>

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("-> List<Condition>"),
        "NET-001: Puzzle must return List<Condition>"
    );
}

// Verifies the puzzle source mentions "singleton", documenting that it is
// designed to be wrapped by singleton_top_layer_v1_1.
#[test]
fn vv_req_net_001_singleton_wrapper_documented() {
    // NET-001: Network coin uses singleton_top_layer_v1_1 wrapper
    // This is documented in the puzzle comments

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("singleton") || puzzle_source.contains("Singleton"),
        "NET-001: Puzzle must document singleton usage"
    );
}

// Verifies the inner puzzle pattern: has fn main, and references the
// singleton wrapper or NET-004 recreation. This confirms the puzzle is
// designed as an inner puzzle, not a standalone puzzle.
#[test]
fn vv_req_net_001_inner_puzzle_pattern() {
    // NET-001: Network coin follows inner puzzle pattern
    // Inner puzzles are wrapped by the singleton layer which handles:
    // - Lineage verification
    // - Amount oddness enforcement
    // - Self-recreation

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Inner puzzle should have main function
    assert!(
        puzzle_source.contains("fn main("),
        "NET-001: Inner puzzle must have main function"
    );

    // Should document that singleton wrapper handles recreation
    assert!(
        puzzle_source.contains("NET-004")
            || puzzle_source.contains("recreation")
            || puzzle_source.contains("wrapper"),
        "NET-001: Puzzle should document singleton wrapper handles recreation"
    );
}
