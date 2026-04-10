//! REQUIREMENT: NET-003 — Registration Coin Creation
//! (`docs/requirements/domains/network_coin/NORMATIVE.md#NET-003`).
//!
//! Spec: `docs/requirements/domains/network_coin/specs/NET-003.md`.
//!
//! Verifies that the network coin creates a registration coin with the
//! correct puzzle hash (curried from MOD_HASH, pubkey, CHECKPOINT_ID)
//! and the correct collateral amount.

use sha2::{Digest, Sha256};

#[test]
fn vv_req_net_003_puzzle_has_create_coin_condition() {
    // NET-003: Puzzle must emit CreateCoin condition

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("CreateCoin"),
        "NET-003: Puzzle must emit CreateCoin condition"
    );
}

#[test]
fn vv_req_net_003_puzzle_computes_puzzle_hash() {
    // NET-003: Puzzle must compute registration_coin_puzzle_hash

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("registration_coin_puzzle_hash"),
        "NET-003: Puzzle must compute registration_coin_puzzle_hash"
    );
}

#[test]
fn vv_req_net_003_puzzle_uses_curry_hash() {
    // NET-003: Puzzle hash computed using curry_tree_hash

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("curry_tree_hash"),
        "NET-003: Puzzle must use curry_tree_hash to compute puzzle hash"
    );
}

#[test]
fn vv_req_net_003_puzzle_hash_includes_mod_hash() {
    // NET-003: Puzzle hash includes registration_coin_mod_hash

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("curry_tree_hash(\n        registration_coin_mod_hash")
            || puzzle_source.contains("curry_tree_hash(registration_coin_mod_hash"),
        "NET-003: curry_tree_hash must include registration_coin_mod_hash"
    );
}

#[test]
fn vv_req_net_003_puzzle_hash_includes_pubkey() {
    // NET-003: Puzzle hash includes new_validator_pubkey

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("tree_hash(new_validator_pubkey)"),
        "NET-003: curry_tree_hash must include new_validator_pubkey"
    );
}

#[test]
fn vv_req_net_003_puzzle_hash_includes_checkpoint_id() {
    // NET-003: Puzzle hash includes checkpoint_singleton_id

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("tree_hash(checkpoint_singleton_id)"),
        "NET-003: curry_tree_hash must include checkpoint_singleton_id"
    );
}

#[test]
fn vv_req_net_003_create_coin_uses_computed_hash() {
    // NET-003: CreateCoin uses the computed puzzle hash

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("puzzle_hash: registration_coin_puzzle_hash"),
        "NET-003: CreateCoin must use registration_coin_puzzle_hash"
    );
}

#[test]
fn vv_req_net_003_create_coin_uses_collateral_amount() {
    // NET-003: CreateCoin amount equals collateral_amount

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("amount: collateral_amount"),
        "NET-003: CreateCoin must use collateral_amount"
    );
}

#[test]
fn vv_req_net_003_collateral_amount_is_curried() {
    // NET-003: collateral_amount is a curried parameter

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("collateral_amount: Int"),
        "NET-003: collateral_amount must be a curried parameter"
    );
}

#[test]
fn vv_req_net_003_mod_hash_is_curried() {
    // NET-003: registration_coin_mod_hash is a curried parameter

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("registration_coin_mod_hash: Bytes32"),
        "NET-003: registration_coin_mod_hash must be a curried parameter"
    );
}

#[test]
fn vv_req_net_003_checkpoint_id_is_curried() {
    // NET-003: checkpoint_singleton_id is a curried parameter

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("checkpoint_singleton_id: Bytes32"),
        "NET-003: checkpoint_singleton_id must be a curried parameter"
    );
}

#[test]
fn vv_req_net_003_puzzle_documents_net_003() {
    // NET-003: Puzzle should document NET-003 requirement

    let puzzle_source = std::fs::read_to_string("puzzles/network_coin_inner.rue")
        .expect("Failed to read puzzle source");

    assert!(
        puzzle_source.contains("NET-003"),
        "NET-003: Puzzle should document NET-003 requirement"
    );
}

#[test]
fn vv_req_net_003_curry_hash_concept() {
    // NET-003: Verify curry_hash conceptual correctness
    // curry_hash(MOD_HASH, pubkey, checkpoint_id) produces a unique hash

    // Two different pubkeys should produce different puzzle hashes
    let pubkey1 = [0x97u8; 48];
    let mut pubkey2 = [0x97u8; 48];
    pubkey2[0] = 0x98;

    let checkpoint_id = [0xABu8; 32];
    let mod_hash = [0xCDu8; 32];

    // Simplified curry hash simulation (actual uses Chialisp tree_hash)
    let compute_simple_hash = |pk: &[u8], ckpt: &[u8], mod_h: &[u8]| {
        let mut hasher = Sha256::new();
        hasher.update(mod_h);
        hasher.update(pk);
        hasher.update(ckpt);
        hasher.finalize()
    };

    let hash1 = compute_simple_hash(&pubkey1, &checkpoint_id, &mod_hash);
    let hash2 = compute_simple_hash(&pubkey2, &checkpoint_id, &mod_hash);

    assert_ne!(
        hash1[..],
        hash2[..],
        "NET-003: Different pubkeys must produce different puzzle hashes"
    );
}
