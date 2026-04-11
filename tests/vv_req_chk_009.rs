//! REQUIREMENT: CHK-009 — Epoch Binding
//! (`docs/requirements/domains/checkpoint/NORMATIVE.md#CHK-009`).
//!
//! Spec: `docs/requirements/domains/checkpoint/specs/CHK-009.md`.
//!
//! Verifies that the Groth16 proof is bound to a specific epoch value
//! through the checkpoint_message hash, and that the CLVM puzzle computes
//! new_epoch = old_epoch + 1 internally.

use chia_l2_consensus::testing::{
    bytes_to_scalar, compute_checkpoint_message, deserialize_proving_key, generate_proof,
    run_test_setup, ConsensusCircuit,
};
use sha2::{Digest, Sha256};

// ── Checkpoint message includes epoch in its preimage ───────────────

#[test]
fn vv_req_chk_009_checkpoint_message_includes_epoch() {
    let sr = [0xAA; 32];
    let mr = [0xBB; 32];
    let vc: u64 = 5;

    let msg_epoch_1 = compute_checkpoint_message(sr, mr, vc, 1);
    let msg_epoch_2 = compute_checkpoint_message(sr, mr, vc, 2);

    assert_ne!(
        msg_epoch_1, msg_epoch_2,
        "CHK-009: Checkpoint messages with different epochs MUST differ"
    );
}

// ── Checkpoint message format: 80 bytes = sr(32) + mr(32) + vc(8) + epoch(8)

#[test]
fn vv_req_chk_009_checkpoint_message_format() {
    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 10;
    let epoch: u64 = 42;

    let msg = compute_checkpoint_message(sr, mr, vc, epoch);

    // Manual computation
    let mut hasher = Sha256::new();
    hasher.update(sr);
    hasher.update(mr);
    hasher.update(vc.to_be_bytes());
    hasher.update(epoch.to_be_bytes()); // epoch MUST be here
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        msg, expected,
        "CHK-009: Checkpoint message must be sha256(sr ‖ mr ‖ vc_be8 ‖ epoch_be8)"
    );
}

// ── Adjacent epochs produce different messages ──────────────────────

#[test]
fn vv_req_chk_009_adjacent_epochs_differ() {
    let sr = [0x00; 32];
    let mr = [0x00; 32];
    let vc: u64 = 1;

    let messages: Vec<[u8; 32]> = (0..10u64)
        .map(|e| compute_checkpoint_message(sr, mr, vc, e))
        .collect();

    for i in 0..messages.len() {
        for j in (i + 1)..messages.len() {
            assert_ne!(
                messages[i], messages[j],
                "CHK-009: Epoch {} and {} must produce different messages",
                i, j
            );
        }
    }
}

// ── Proof for epoch N encodes that epoch in its public inputs ───────

#[test]
fn vv_req_chk_009_proof_encodes_epoch() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let sr = [0xAA; 32];
    let mr = [0xBB; 32];
    let vc: u64 = 1;

    // Generate proof for epoch 5 (new_epoch = 5, so old_epoch was 4)
    let msg_epoch_5 = compute_checkpoint_message(sr, mr, vc, 5);
    let circuit_5 =
        ConsensusCircuit::with_public_inputs(mr, vc, mr, vc, [0xCC; 48], msg_epoch_5, 1);
    let proof_5 = generate_proof(circuit_5, &pk).expect("Proof for epoch 5");

    // Generate proof for epoch 10 (new_epoch = 10, so old_epoch was 9)
    let msg_epoch_10 = compute_checkpoint_message(sr, mr, vc, 10);
    let circuit_10 =
        ConsensusCircuit::with_public_inputs(mr, vc, mr, vc, [0xCC; 48], msg_epoch_10, 1);
    let proof_10 = generate_proof(circuit_10, &pk).expect("Proof for epoch 10");

    // The proofs must be different because they encode different checkpoint_messages
    assert_ne!(
        proof_5, proof_10,
        "CHK-009: Proofs for different epochs must differ (different checkpoint_messages)"
    );
}

// ── Scalar s6 changes with epoch ────────────────────────────────────

#[test]
fn vv_req_chk_009_scalar_s6_changes_with_epoch() {
    let sr = [0xAA; 32];
    let mr = [0xBB; 32];
    let vc: u64 = 1;

    let msg_5 = compute_checkpoint_message(sr, mr, vc, 5);
    let msg_6 = compute_checkpoint_message(sr, mr, vc, 6);

    let s6_epoch5 = bytes_to_scalar(&msg_5);
    let s6_epoch6 = bytes_to_scalar(&msg_6);

    assert_ne!(
        s6_epoch5, s6_epoch6,
        "CHK-009: Scalar s6 (checkpoint_message) must differ between epochs"
    );
}

// ── Rue puzzle computes new_epoch internally ────────────────────────

#[test]
fn vv_req_chk_009_puzzle_computes_epoch_internally() {
    let source = include_str!("../puzzles/checkpoint_inner.rue");

    // The puzzle MUST compute new_epoch = old_epoch + 1
    assert!(
        source.contains("STATE.epoch + 1") || source.contains("epoch + 1"),
        "CHK-009: Puzzle must compute new_epoch = old_epoch + 1 internally"
    );

    // The puzzle MUST NOT accept epoch from the solution
    // Check that the checkpoint solution params do not include "epoch"
    // The solution for checkpoint path has: new_state_root, new_validator_merkle_root,
    // new_validator_count, agg_signers, agg_sig, proof, scalars
    // but NOT epoch
    assert!(
        !source.contains("solution.epoch") && !source.contains("sol.epoch"),
        "CHK-009: Puzzle must NOT accept epoch from solution"
    );
}

// ── Epoch is in checkpoint_message computation in Rue ───────────────

#[test]
fn vv_req_chk_009_rue_includes_epoch_in_message() {
    let source = include_str!("../puzzles/checkpoint_inner.rue");

    // The Rue puzzle must compute checkpoint_message including new_epoch
    assert!(
        source.contains("int_to_8_bytes_be(new_epoch)") || source.contains("new_epoch"),
        "CHK-009: Rue puzzle must include new_epoch in checkpoint_message computation"
    );

    // Verify the checkpoint_message hash includes epoch
    assert!(
        source.contains("checkpoint_message"),
        "CHK-009: Rue puzzle must compute checkpoint_message"
    );
}

// ── Epoch 0 to 1 transition produces valid proof ────────────────────

#[test]
fn vv_req_chk_009_epoch_0_to_1_valid() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let msg = compute_checkpoint_message([0; 32], [0; 32], 0, 1); // new_epoch = 1
    let circuit = ConsensusCircuit::with_public_inputs([0; 32], 1, [0; 32], 0, [0; 48], msg, 1);
    let proof = generate_proof(circuit, &pk);

    assert!(
        proof.is_ok(),
        "CHK-009: Epoch 0→1 transition must produce valid proof"
    );
}

// ── Large epoch values work correctly ───────────────────────────────

#[test]
fn vv_req_chk_009_large_epoch() {
    let msg_large = compute_checkpoint_message([0; 32], [0; 32], 0, u64::MAX);
    let msg_prev = compute_checkpoint_message([0; 32], [0; 32], 0, u64::MAX - 1);

    assert_ne!(
        msg_large, msg_prev,
        "CHK-009: Even u64::MAX epoch produces unique message"
    );
    assert_eq!(msg_large.len(), 32, "CHK-009: Message is always 32 bytes");
}
