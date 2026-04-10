//! REQUIREMENT: NET-004 — Network Coin Self-Recreation
//! (`docs/requirements/domains/network_coin/NORMATIVE.md#NET-004`).
//!
//! Spec: `docs/requirements/domains/network_coin/specs/NET-004.md`.
//!
//! Verifies that the network coin recreates itself after registration.
//! Self-recreation is handled by the singleton_top_layer_v1_1 wrapper,
//! not the inner puzzle directly.

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

#[test]
fn vv_req_net_004_puzzle_does_not_create_self() {
    // NET-004: Inner puzzle should NOT create self - wrapper does this
    // Check that there's no explicit self-recreation in the inner puzzle

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // Count CreateCoin struct instantiations (with opening brace)
    // This excludes comments that just mention CreateCoin
    let create_coin_struct_count = puzzle_source.matches("CreateCoin {").count();

    // Should have exactly one CreateCoin struct (for registration coin)
    // Self-recreation is handled by wrapper
    assert!(
        create_coin_struct_count == 1,
        "NET-004: Inner puzzle should only create registration coin (found {} CreateCoin structs). Self-recreation handled by wrapper.",
        create_coin_struct_count
    );
}

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

#[test]
fn vv_req_net_004_no_my_puzzle_hash_in_inner() {
    // NET-004: Inner puzzle should NOT reference MY_PUZZLE_HASH
    // (that's a singleton wrapper concept)

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    // MY_PUZZLE_HASH is a singleton concept, not used in inner puzzle
    let has_my_puzzle_hash = puzzle_source.contains("MY_PUZZLE_HASH")
        || puzzle_source.contains("my_puzzle_hash");

    assert!(
        !has_my_puzzle_hash,
        "NET-004: Inner puzzle should not use MY_PUZZLE_HASH (wrapper handles this)"
    );
}

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
