//! REQUIREMENT: REG-001 — Registration Coin Puzzle Structure
//! (`docs/requirements/domains/registration_coin/NORMATIVE.md#REG-001`).
//!
//! Spec: `docs/requirements/domains/registration_coin/specs/REG-001.md`.
//!
//! Verifies that the registration coin puzzle is curried with exactly two
//! parameters: VALIDATOR_PUBKEY (48-byte BLS G1 point) and
//! CHECKPOINT_SINGLETON_ID (32-byte coin ID of the checkpoint singleton).

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

/// Helper to read the puzzle source
fn puzzle_source() -> String {
    std::fs::read_to_string("puzzles/registration_coin.rue")
        .expect("Failed to read puzzles/registration_coin.rue")
}

// ── Compilation ──────────────────────────────────────────────────────

#[test]
fn vv_req_reg_001_puzzle_compiles() {
    // REG-001: Registration coin puzzle must compile with rue build
    assert!(
        puzzle_compiles("puzzles/registration_coin.rue"),
        "REG-001: registration_coin.rue must compile with rue build"
    );
}

#[test]
fn vv_req_reg_001_puzzle_produces_clvm() {
    // REG-001: Compiled puzzle produces valid CLVM output
    let clvm = get_puzzle_clvm("puzzles/registration_coin.rue");
    assert!(clvm.is_some(), "REG-001: Puzzle must produce CLVM output");

    let clvm = clvm.unwrap();
    assert!(
        clvm.starts_with('('),
        "REG-001: CLVM output must be valid s-expression"
    );
}

#[test]
fn vv_req_reg_001_puzzle_is_deterministic() {
    // REG-001: Puzzle compilation is deterministic — same source always
    // produces identical CLVM bytecode
    let clvm1 = get_puzzle_clvm("puzzles/registration_coin.rue");
    let clvm2 = get_puzzle_clvm("puzzles/registration_coin.rue");

    assert!(clvm1.is_some() && clvm2.is_some());
    assert_eq!(
        clvm1.unwrap(),
        clvm2.unwrap(),
        "REG-001: Puzzle compilation must be deterministic"
    );
}

// ── Curried parameters (exactly 2) ──────────────────────────────────

#[test]
fn vv_req_reg_001_has_validator_pubkey_curried() {
    // REG-001: First curried parameter is VALIDATOR_PUBKEY: PublicKey
    // (48-byte BLS12-381 G1 point, compressed ZCash format)
    let src = puzzle_source();

    assert!(
        src.contains("VALIDATOR_PUBKEY: PublicKey"),
        "REG-001: Puzzle must have VALIDATOR_PUBKEY: PublicKey curried parameter"
    );
}

#[test]
fn vv_req_reg_001_has_checkpoint_singleton_id_curried() {
    // REG-001: Second curried parameter is CHECKPOINT_SINGLETON_ID: Bytes32
    // This is the coin ID (not launcher ID) per spec-wire-format — Common Mistakes
    let src = puzzle_source();

    assert!(
        src.contains("CHECKPOINT_SINGLETON_ID: Bytes32"),
        "REG-001: Puzzle must have CHECKPOINT_SINGLETON_ID: Bytes32 curried parameter"
    );
}

#[test]
fn vv_req_reg_001_exactly_two_curried_params() {
    // REG-001: The puzzle must have exactly 2 curried parameters.
    // In Rue convention, UPPERCASE parameters before the blank line
    // separator are curried; lowercase are solution params.
    let src = puzzle_source();

    // Extract the fn main signature
    let main_start = src.find("fn main(").expect("Must have fn main");
    let sig_end = src[main_start..]
        .find(") ->")
        .expect("Must have return type");
    let signature = &src[main_start..main_start + sig_end];

    // Count UPPERCASE parameter names (curried convention)
    let curried_count = signature
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Curried params: start with UPPERCASE letter followed by _
            // e.g., VALIDATOR_PUBKEY: PublicKey
            trimmed.starts_with(|c: char| c.is_ascii_uppercase()) && trimmed.contains(':')
        })
        .count();

    assert_eq!(
        curried_count, 2,
        "REG-001: Puzzle must have exactly 2 curried parameters, found {}",
        curried_count
    );
}

// ── Solution parameters ─────────────────────────────────────────────

#[test]
fn vv_req_reg_001_has_epoch_solution_param() {
    // REG-001: Solution includes epoch for announcement construction
    let src = puzzle_source();

    assert!(
        src.contains("epoch: Int"),
        "REG-001: Puzzle must have epoch: Int solution parameter"
    );
}

#[test]
fn vv_req_reg_001_has_collateral_destination_solution_param() {
    // REG-001: Solution includes collateral_destination for fund recovery
    let src = puzzle_source();

    assert!(
        src.contains("collateral_destination: Bytes32"),
        "REG-001: Puzzle must have collateral_destination: Bytes32 solution parameter"
    );
}

#[test]
fn vv_req_reg_001_has_collateral_amount_solution_param() {
    // REG-001: Solution includes collateral_amount for fund recovery
    let src = puzzle_source();

    assert!(
        src.contains("collateral_amount: Int"),
        "REG-001: Puzzle must have collateral_amount: Int solution parameter"
    );
}

#[test]
fn vv_req_reg_001_no_conditions_passthrough() {
    // SEC-008: conditions parameter REMOVED to prevent injection attacks.
    // The puzzle outputs only [assert_announcement, create_collateral].
    let src = puzzle_source();

    assert!(
        !src.contains("conditions: List<Condition>"),
        "REG-001/SEC-008: Puzzle must NOT have conditions passthrough (injection prevention)"
    );
}

// ── Return type ─────────────────────────────────────────────────────

#[test]
fn vv_req_reg_001_returns_list_condition() {
    // REG-001: Puzzle must return List<Condition>
    let src = puzzle_source();

    assert!(
        src.contains("-> List<Condition>"),
        "REG-001: Puzzle must return List<Condition>"
    );
}

// ── Coin ID vs Launcher ID ─────────────────────────────────────────

#[test]
fn vv_req_reg_001_documents_coin_id_not_launcher_id() {
    // REG-001: The CHECKPOINT_SINGLETON_ID parameter is the coin ID,
    // not the launcher ID. The puzzle must document this distinction
    // because using launcher ID is a common mistake per spec-wire-format.
    let src = puzzle_source();

    assert!(
        src.contains("coin ID")
            || src.contains("coin_id")
            || src.contains("not the launcher")
            || src.contains("not launcher"),
        "REG-001: Puzzle must document that CHECKPOINT_SINGLETON_ID is coin ID, not launcher ID"
    );
}

// ── Puzzle identity ─────────────────────────────────────────────────

#[test]
fn vv_req_reg_001_is_standalone_puzzle() {
    // REG-001: Registration coin is NOT a singleton inner puzzle.
    // Unlike network_coin_inner.rue, registration_coin.rue is a
    // standalone puzzle (no singleton wrapper). It's created by the
    // network coin and spent for collateral recovery.
    let src = puzzle_source();

    // File name should not contain "inner" — it's the full puzzle
    let path = "puzzles/registration_coin.rue";
    assert!(
        !path.contains("inner"),
        "REG-001: Registration coin is a standalone puzzle, not an inner puzzle"
    );

    // Should have fn main (direct puzzle, not wrapped)
    assert!(
        src.contains("fn main("),
        "REG-001: Puzzle must have fn main (standalone puzzle)"
    );
}

#[test]
fn vv_req_reg_001_puzzle_documents_reg_001() {
    // REG-001: The puzzle or its spec reference should be traceable
    let src = puzzle_source();

    // Must reference the spec or contain registration-coin identifying comments
    assert!(
        src.contains("registration_coin")
            || src.contains("Registration")
            || src.contains("spec-registration-coin"),
        "REG-001: Puzzle must reference registration coin specification"
    );
}

// ── Compiled artifacts (.hex and .hash) ────────────────────────────

#[test]
fn vv_req_reg_001_compiled_hex_exists() {
    // REG-001: Compiled hex artifact must exist in puzzles/compiled/
    let hex_path = "puzzles/compiled/registration_coin.hex";
    assert!(
        std::path::Path::new(hex_path).exists(),
        "REG-001: {} must exist — run `rue build -x` to generate",
        hex_path
    );

    let hex = std::fs::read_to_string(hex_path).expect("Failed to read compiled hex");
    assert!(
        !hex.trim().is_empty(),
        "REG-001: Compiled hex must not be empty"
    );
    // Hex output should be valid hex characters
    assert!(
        hex.trim().chars().all(|c| c.is_ascii_hexdigit()),
        "REG-001: Compiled hex must contain only hex characters"
    );
}

#[test]
fn vv_req_reg_001_compiled_hash_exists() {
    // REG-001: Compiled hash artifact must exist in puzzles/compiled/
    let hash_path = "puzzles/compiled/registration_coin.hash";
    assert!(
        std::path::Path::new(hash_path).exists(),
        "REG-001: {} must exist — run `rue build --hash` to generate",
        hash_path
    );

    let hash = std::fs::read_to_string(hash_path).expect("Failed to read compiled hash");
    let hash = hash.trim();
    // Hash should be 0x-prefixed 32-byte hex (66 chars total)
    assert!(
        hash.starts_with("0x") && hash.len() == 66,
        "REG-001: Compiled hash must be 0x-prefixed 32-byte hex, got: {}",
        hash
    );
}

#[test]
fn vv_req_reg_001_compiled_hex_matches_live_build() {
    // REG-001: The stored .hex must match a fresh `rue build -x`
    let output = Command::new("rue")
        .args(["build", "-x", "puzzles/registration_coin.rue"])
        .output()
        .expect("Failed to run rue build -x");
    assert!(
        output.status.success(),
        "REG-001: rue build -x must succeed"
    );

    let fresh_hex = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stored_hex = std::fs::read_to_string("puzzles/compiled/registration_coin.hex")
        .expect("Failed to read stored hex")
        .trim()
        .to_string();

    assert_eq!(
        fresh_hex, stored_hex,
        "REG-001: Stored .hex must match fresh rue build -x output"
    );
}

#[test]
fn vv_req_reg_001_compiled_hash_matches_live_build() {
    // REG-001: The stored .hash must match a fresh `rue build --hash`
    let output = Command::new("rue")
        .args(["build", "--hash", "puzzles/registration_coin.rue"])
        .output()
        .expect("Failed to run rue build --hash");
    assert!(
        output.status.success(),
        "REG-001: rue build --hash must succeed"
    );

    let fresh_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stored_hash = std::fs::read_to_string("puzzles/compiled/registration_coin.hash")
        .expect("Failed to read stored hash")
        .trim()
        .to_string();

    assert_eq!(
        fresh_hash, stored_hash,
        "REG-001: Stored .hash must match fresh rue build --hash output"
    );
}
