//! REQUIREMENT: NET-001 — Network Coin Singleton Identity
//! (`docs/requirements/domains/network_coin/NORMATIVE.md#NET-001`).
//!
//! Spec: `docs/requirements/domains/network_coin/specs/NET-001.md`.
//!
//! Verifies that the network coin is implemented as a Chia singleton,
//! guaranteeing exactly one instance exists with a given launcher ID.
//!
//! Note: Full singleton behavior requires on-chain testing. These tests
//! verify the puzzle structure and compilation.

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

#[test]
fn vv_req_net_001_puzzle_compiles() {
    // NET-001: Network coin inner puzzle must compile
    assert!(
        puzzle_compiles("puzzles/network_coin_inner.rue"),
        "NET-001: network_coin_inner.rue must compile with rue build"
    );
}

#[test]
fn vv_req_net_001_puzzle_produces_clvm() {
    // NET-001: Compiled puzzle produces valid CLVM output
    let clvm = get_puzzle_clvm("puzzles/network_coin_inner.rue");
    assert!(
        clvm.is_some(),
        "NET-001: Puzzle must produce CLVM output"
    );

    let clvm = clvm.unwrap();
    // CLVM starts with (a or (q or similar
    assert!(
        clvm.starts_with('('),
        "NET-001: CLVM output must be valid s-expression"
    );
}

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
        puzzle_source.contains("NET-004") || puzzle_source.contains("recreation")
            || puzzle_source.contains("wrapper"),
        "NET-001: Puzzle should document singleton wrapper handles recreation"
    );
}
