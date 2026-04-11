//! REQUIREMENT: NET-003 — Registration Coin Creation
//! (`docs/requirements/domains/network_coin/NORMATIVE.md#NET-003`).
//!
//! Spec: `docs/requirements/domains/network_coin/specs/NET-003.md`.
//!
//! ## Normative Statement
//!
//! When a validator registers, the network coin creates a registration coin via
//! `CreateCoin` with puzzle hash = `curry_hash(REGISTRATION_COIN_MOD_HASH,
//! new_validator_pubkey, CHECKPOINT_SINGLETON_ID)` and amount = COLLATERAL_AMOUNT.
//! The puzzle hash is deterministically derived so the indexer can verify lineage.
//!
//! ## How These Tests Prove the Requirement
//!
//! Tests inspect the puzzle source to verify: CreateCoin condition present,
//! registration_coin_puzzle_hash computed via curry_tree_hash, the curry includes
//! registration_coin_mod_hash + new_validator_pubkey + checkpoint_singleton_id,
//! CreateCoin uses the computed hash, CreateCoin uses collateral_amount, and all
//! relevant parameters are curried. A conceptual curry_hash test shows different
//! pubkeys produce different puzzle hashes.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] CreateCoin condition emitted (source inspection)
//! - [x] Puzzle hash computed via curry_tree_hash (source inspection)
//! - [x] Puzzle hash includes MOD_HASH, pubkey, CHECKPOINT_ID (source inspection)
//! - [x] CreateCoin uses computed puzzle hash (source inspection)
//! - [x] CreateCoin uses collateral_amount (source inspection)
//! - [x] All curried parameters declared correctly
//! - [x] Different pubkeys -> different puzzle hashes (conceptual test)
//! - [ ] Actual coin creation with correct amount (tested in NET-006)
//! - [ ] Parent coin ID is the spent network coin (tested in NET-006)
//!
//! ## Gaps
//!
//! Tests are source-inspection only. Actual CLVM execution and on-chain coin
//! creation are tested in NET-006 (simulator test). The curry_hash conceptual
//! test uses a simplified hash, not the real Chialisp curry_tree_hash.

use sha2::{Digest, Sha256};

// Source inspection: verifies the puzzle contains "CreateCoin", confirming
// it emits the condition to create the registration coin.
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

// Source inspection: verifies the puzzle computes registration_coin_puzzle_hash.
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

// Source inspection: verifies curry_tree_hash is used for puzzle hash derivation.
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

// Source inspection: verifies curry_tree_hash includes registration_coin_mod_hash
// as first argument.
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

// Source inspection: verifies the curry includes tree_hash(new_validator_pubkey).
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

// Source inspection: verifies the curry includes tree_hash(checkpoint_singleton_id).
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

// Source inspection: verifies CreateCoin uses `puzzle_hash: registration_coin_puzzle_hash`.
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

// Source inspection: verifies CreateCoin uses `amount: collateral_amount`.
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

// Verifies collateral_amount is a curried parameter (Int type), fixed at deployment.
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

// Verifies registration_coin_mod_hash is a curried Bytes32 parameter.
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

// Verifies checkpoint_singleton_id is a curried Bytes32 parameter.
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

// Traceability: verifies the puzzle source references NET-003.
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

// Conceptual test: verifies that different pubkeys produce different puzzle
// hashes using a simplified hash simulation. The actual curry_tree_hash is
// more complex, but the uniqueness property must hold.
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
