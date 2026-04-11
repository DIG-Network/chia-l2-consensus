//! REQUIREMENT: IDX-003 — Merkle Consistency
//! (`docs/requirements/domains/indexer/NORMATIVE.md#IDX-003`).
//!
//! Spec: `docs/requirements/domains/indexer/specs/IDX-003.md`.
//!
//! Implementation: `src/indexer/validator_set.rs`.
//!
//! After every sync, the indexer rebuilds the sparse Merkle tree from
//! registration coins and verifies the computed root matches the on-chain
//! `validator_merkle_root`. Mismatches return `StateMismatch` error.

use chia_protocol::Bytes32;

// ── IDX-003: Consistency check with matching root ─────────────────────

#[test]
fn vv_req_idx_003_matching_root_succeeds() {
    // Build a tree from validators, then verify against the same root.
    use chia_l2_consensus::testing::verify_merkle_consistency;
    use chia_l2_consensus::testing::SparseMerkleTree;

    let pk1 = [0xAAu8; 48];
    let pk2 = [0xBBu8; 48];

    // Build expected tree
    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pk1);
    tree.insert_validator(&pk2);
    let expected_root: Bytes32 = tree.root().into();

    // verify_merkle_consistency takes pubkeys + on-chain root → Ok(tree) or Err
    let result = verify_merkle_consistency(&[pk1, pk2], expected_root);
    assert!(result.is_ok(), "IDX-003: Matching root must succeed");
    let verified_tree = result.unwrap();
    assert_eq!(
        verified_tree.root(),
        tree.root(),
        "IDX-003: Returned tree must have same root"
    );
    assert_eq!(verified_tree.len(), 2);
}

// ── IDX-003: Mismatch detected ────────────────────────────────────────

#[test]
fn vv_req_idx_003_mismatch_detected() {
    // Build a tree from validators, but provide a DIFFERENT on-chain root.
    use chia_l2_consensus::testing::verify_merkle_consistency;

    let pk1 = [0xAAu8; 48];
    let pk2 = [0xBBu8; 48];

    let wrong_root = Bytes32::from([0xFF; 32]); // does not match

    let result = verify_merkle_consistency(&[pk1, pk2], wrong_root);
    assert!(
        result.is_err(),
        "IDX-003: Mismatched root must return error"
    );
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("mismatch") || msg.contains("Mismatch") || msg.contains("state"),
        "IDX-003: Error must indicate state mismatch, got: {}",
        msg
    );
}

// ── IDX-003: Empty validator set ──────────────────────────────────────

#[test]
fn vv_req_idx_003_empty_validator_set() {
    // Empty set should match the empty tree root.
    use chia_l2_consensus::testing::verify_merkle_consistency;
    use chia_l2_consensus::testing::SparseMerkleTree;

    let empty_tree = SparseMerkleTree::new();
    let empty_root: Bytes32 = empty_tree.root().into();

    let result = verify_merkle_consistency(&[], empty_root);
    assert!(
        result.is_ok(),
        "IDX-003: Empty set with empty root must succeed"
    );
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn vv_req_idx_003_empty_set_wrong_root() {
    // Empty set with a non-empty root must fail.
    use chia_l2_consensus::testing::verify_merkle_consistency;

    let wrong_root = Bytes32::from([0x11; 32]);
    let result = verify_merkle_consistency(&[], wrong_root);
    assert!(
        result.is_err(),
        "IDX-003: Empty set with non-empty root must fail"
    );
}

// ── IDX-003: Order independence ───────────────────────────────────────

#[test]
fn vv_req_idx_003_insertion_order_independent() {
    // The tree root must be the same regardless of insertion order.
    use chia_l2_consensus::testing::verify_merkle_consistency;
    use chia_l2_consensus::testing::SparseMerkleTree;

    let pk1 = [0xAAu8; 48];
    let pk2 = [0xBBu8; 48];
    let pk3 = [0xCCu8; 48];

    // Build reference tree in one order
    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pk1);
    tree.insert_validator(&pk2);
    tree.insert_validator(&pk3);
    let root: Bytes32 = tree.root().into();

    // verify_merkle_consistency should produce the same root in any order
    let result_forward = verify_merkle_consistency(&[pk1, pk2, pk3], root);
    assert!(result_forward.is_ok(), "IDX-003: Forward order must match");

    let result_reverse = verify_merkle_consistency(&[pk3, pk2, pk1], root);
    assert!(result_reverse.is_ok(), "IDX-003: Reverse order must match");

    let result_shuffled = verify_merkle_consistency(&[pk2, pk3, pk1], root);
    assert!(
        result_shuffled.is_ok(),
        "IDX-003: Shuffled order must match"
    );
}

// ── IDX-003: Missing validator triggers mismatch ──────────────────────

#[test]
fn vv_req_idx_003_missing_validator_mismatch() {
    // If the indexer missed one registration, the root won't match.
    use chia_l2_consensus::testing::verify_merkle_consistency;
    use chia_l2_consensus::testing::SparseMerkleTree;

    let pk1 = [0xAAu8; 48];
    let pk2 = [0xBBu8; 48];

    // On-chain root includes both validators
    let mut tree = SparseMerkleTree::new();
    tree.insert_validator(&pk1);
    tree.insert_validator(&pk2);
    let on_chain_root: Bytes32 = tree.root().into();

    // But indexer only has pk1 (missed pk2)
    let result = verify_merkle_consistency(&[pk1], on_chain_root);
    assert!(
        result.is_err(),
        "IDX-003: Missing validator must cause mismatch"
    );
}

// ── Spec ───────────────────────────────────────────────────────────

#[test]
fn vv_req_idx_003_spec_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/indexer/specs/IDX-003.md").exists());
}
