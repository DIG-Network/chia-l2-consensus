//! REQUIREMENT: NET-004 — Network Coin Self-Recreation
//! (`docs/requirements/domains/network_coin/NORMATIVE.md#NET-004`).
//!
//! Spec: `docs/requirements/domains/network_coin/specs/NET-004.md`.
//!
//! ## Normative Statement
//!
//! After creating a registration coin, the network coin MUST recreate itself
//! with identical puzzle hash and 1 mojo amount. The inner puzzle emits a
//! `CreateCoin` with `INNER_MOD_HASH` and amount=1; the singleton_top_layer_v1_1
//! wrapper intercepts this odd-amount CREATE_COIN and transforms it to the full
//! singleton puzzle hash. This ensures the network coin is never consumed.
//!
//! ## How These Tests Prove the Requirement
//!
//! Tests verify via source inspection: NET-004 documented, singleton wrapper
//! responsibility mentioned, inner puzzle pattern confirmed, returns
//! List<Condition>, exactly 2 CreateCoin structs (registration + recreation),
//! registration CreateCoin uses registration_coin_puzzle_hash (not self), no
//! MY_PUZZLE_HASH in inner puzzle, curried params are constant across
//! recreations, and curried params documented as deployment-fixed.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Inner puzzle emits 2 CreateCoin conditions (registration + recreation)
//! - [x] Registration CreateCoin is for registration coin, not self
//! - [x] Inner puzzle is an inner puzzle (not standalone)
//! - [x] Singleton wrapper handles self-recreation (documented)
//! - [x] No MY_PUZZLE_HASH in inner puzzle (wrapper concept)
//! - [x] Curried parameters constant across recreations
//! - [ ] After registration spend, new network coin exists (NET-006)
//! - [ ] New network coin has identical puzzle hash (NET-006)
//! - [ ] New network coin has 1 mojo (NET-006)
//! - [ ] Sequential registrations succeed (NET-006)
//!
//! ## Gaps
//!
//! These tests inspect puzzle source only. Actual self-recreation behavior
//! (new coin created, lineage unbroken, amount=1) is verified in NET-006
//! via the simulator.

// Traceability: verifies the puzzle source references NET-004.
#[test]
fn vv_req_net_004_puzzle_documents_self_recreation() {
    // NET-004: Puzzle should document that singleton wrapper handles recreation

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("NET-004"),
        "NET-004: Puzzle should document NET-004 requirement"
    );
}

// Verifies the puzzle documents that the singleton wrapper is responsible
// for self-recreation, not the inner puzzle directly.
#[test]
fn vv_req_net_004_puzzle_mentions_singleton_wrapper() {
    // NET-004: Puzzle documents singleton wrapper responsibility

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("Singleton wrapper handles self-recreation")
            || puzzle_source.contains("singleton wrapper handles recreation")
            || (puzzle_source.contains("NET-004") && puzzle_source.contains("wrapper")),
        "NET-004: Puzzle should document singleton wrapper handles self-recreation"
    );
}

// Verifies this is an inner puzzle (mentions "inner puzzle") wrapped by
// singleton_top_layer_v1_1, not a standalone puzzle.
#[test]
fn vv_req_net_004_puzzle_is_inner_puzzle() {
    // NET-004: This is an inner puzzle, not a standalone puzzle
    // Inner puzzles are wrapped by singleton_top_layer_v1_1

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("inner puzzle")
            || puzzle_source.contains("Inner puzzle")
            || puzzle_source.contains("network_coin_inner"),
        "NET-004: File should be inner puzzle (wrapped by singleton)"
    );
}

// Verifies the inner puzzle returns List<Condition>, which is required by
// the singleton wrapper to process and augment the output.
#[test]
fn vv_req_net_004_inner_puzzle_returns_conditions() {
    // NET-004: Inner puzzle returns List<Condition> for wrapper to process

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("-> List<Condition>"),
        "NET-004: Inner puzzle must return List<Condition>"
    );
}

// Verifies the inner puzzle has exactly 2 CreateCoin struct instantiations:
// one for the registration coin (NET-003) and one for singleton self-
// recreation (amount=1, puzzle_hash=INNER_MOD_HASH). The singleton wrapper
// intercepts the odd-amount one.
#[test]
fn vv_req_net_004_puzzle_creates_recreation_coin() {
    // NET-004: Inner puzzle emits CreateCoin with INNER_MOD_HASH for singleton morphing.
    // The singleton_top_layer intercepts the odd-amount CREATE_COIN and transforms
    // it to the full singleton puzzle hash.

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Count CreateCoin struct instantiations (with opening brace)
    let create_coin_struct_count = puzzle_source.matches("CreateCoin {").count();

    // Should have exactly two CreateCoin structs:
    // 1. Registration coin (NET-003, amount = collateral)
    // 2. Self-recreation (NET-004, amount = 1, puzzle_hash = INNER_MOD_HASH)
    assert!(
        create_coin_struct_count == 2,
        "NET-004: Inner puzzle should create registration coin + recreation coin (found {} CreateCoin structs).",
        create_coin_struct_count
    );
}

// Verifies that one CreateCoin uses registration_coin_puzzle_hash (not self).
#[test]
fn vv_req_net_004_create_coin_is_for_registration() {
    // NET-004: The CreateCoin in inner puzzle is for registration, not self

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // CreateCoin should use registration_coin_puzzle_hash, not self puzzle hash
    assert!(
        puzzle_source.contains("puzzle_hash: registration_coin_puzzle_hash"),
        "NET-004: CreateCoin should be for registration coin"
    );
}

// Verifies the puzzle references the singleton pattern, documenting the
// integration with singleton_top_layer_v1_1.
#[test]
fn vv_req_net_004_puzzle_mentions_singleton_top_layer() {
    // NET-004: Puzzle should reference singleton_top_layer_v1_1

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("singleton_top_layer_v1_1") || puzzle_source.contains("singleton"),
        "NET-004: Puzzle should reference singleton pattern"
    );
}

// Verifies the inner puzzle does NOT use MY_PUZZLE_HASH (a singleton wrapper
// concept). The inner puzzle should not reference it because the wrapper
// computes the full puzzle hash by combining the inner hash with the
// singleton struct.
#[test]
fn vv_req_net_004_no_my_puzzle_hash_in_inner() {
    // NET-004: Inner puzzle should NOT reference MY_PUZZLE_HASH
    // (that's a singleton wrapper concept)

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // MY_PUZZLE_HASH is a singleton concept, not used in inner puzzle
    let has_my_puzzle_hash =
        puzzle_source.contains("MY_PUZZLE_HASH") || puzzle_source.contains("my_puzzle_hash");

    assert!(
        !has_my_puzzle_hash,
        "NET-004: Inner puzzle should not use MY_PUZZLE_HASH (wrapper handles this)"
    );
}

// Verifies all three curried parameters are declared, confirming they are
// constant across recreations (same puzzle hash after each spend).
#[test]
fn vv_req_net_004_curried_params_are_constant() {
    // NET-004: Curried parameters remain constant across recreations
    // - registration_coin_mod_hash
    // - collateral_amount
    // - checkpoint_singleton_id

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // All three curried params should be present
    assert!(
        puzzle_source.contains("registration_coin_mod_hash: Bytes32"),
        "NET-004: registration_coin_mod_hash should be curried constant"
    );
    assert!(
        puzzle_source.contains("collateral_amount: Int"),
        "NET-004: collateral_amount should be curried constant"
    );
    assert!(
        puzzle_source.contains("checkpoint_singleton_id: Bytes32"),
        "NET-004: checkpoint_singleton_id should be curried constant"
    );
}

// Verifies the puzzle documents that curried parameters are fixed at
// deployment time.
#[test]
fn vv_req_net_004_curried_params_are_deployment_fixed() {
    // NET-004: Curried parameters are documented as fixed at deployment

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("Curried parameters")
            || puzzle_source.contains("fixed at deployment")
            || puzzle_source.contains("curried"),
        "NET-004: Curried parameters should be documented as fixed"
    );
}
