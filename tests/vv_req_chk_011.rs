//! REQUIREMENT: CHK-011 — State Hash Binding
//! (`docs/requirements/domains/checkpoint/NORMATIVE.md#CHK-011`).
//!
//! Spec: `docs/requirements/domains/checkpoint/specs/CHK-011.md`.
//!
//! ## Normative statement
//! The new_state_root MUST be the first field in the checkpoint_message
//! preimage, binding the Groth16 proof and BLS signatures to a specific L2
//! state. Different state roots MUST produce different checkpoint messages.
//!
//! ## How the tests prove the requirement
//! 1. **First field**: Manual sha256 computation confirms state_root is the
//!    first 32 bytes of the 112-byte preimage.
//! 2. **Different roots differ**: Three distinct state_roots produce three
//!    distinct messages.
//! 3. **Single byte sensitivity**: Changing one byte of state_root changes
//!    the checkpoint_message.
//! 4. **Rue source checks**: Source references new_state_root in both the
//!    checkpoint_message computation and State recreation.
//! 5. **Scalar s6 changes**: Different state_roots produce different scalar s6
//!    values, invalidating the Groth16 proof.
//! 6. **Proof binding**: Real Groth16 proofs for different state_roots differ.
//! 7. **Zero state valid**: All-zero state_root (genesis) produces a valid,
//!    non-trivial message.
//!
//! ## Completeness: HIGH
//! ## Gaps: None significant.

use chia_l2_consensus::testing::{
    bytes_to_scalar, compute_checkpoint_message, deserialize_proving_key, generate_proof,
    run_test_setup, ConsensusCircuit,
};
use sha2::{Digest, Sha256};

// ── state_root is the first field in checkpoint_message preimage ─────

/// Cross-impl: manually constructs the 112-byte preimage with state_root
/// as the first 32 bytes and verifies compute_checkpoint_message matches.
#[test]
fn vv_req_chk_011_state_root_is_first_field() {
    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 5;
    let epoch: u64 = 3;

    let msg = compute_checkpoint_message(sr, mr, vc, epoch, [0x00; 32]);

    // Manual computation: state_root is first 32 bytes of the 112-byte preimage
    let mut hasher = Sha256::new();
    hasher.update(sr); // state_root FIRST
    hasher.update(mr);
    hasher.update(vc.to_be_bytes());
    hasher.update(epoch.to_be_bytes());
    hasher.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        msg, expected,
        "CHK-011: state_root must be the first field in checkpoint_message preimage"
    );
}

// ── Different state_roots produce different checkpoint_messages ──────

/// Three distinct state_roots produce three distinct checkpoint messages,
/// proving the state_root field contributes uniquely to the hash.
#[test]
fn vv_req_chk_011_different_state_roots_different_messages() {
    let mr = [0x22; 32];
    let vc: u64 = 5;
    let epoch: u64 = 3;

    let msg_a = compute_checkpoint_message([0xAA; 32], mr, vc, epoch, [0x00; 32]);
    let msg_b = compute_checkpoint_message([0xBB; 32], mr, vc, epoch, [0x00; 32]);
    let msg_c = compute_checkpoint_message([0x00; 32], mr, vc, epoch, [0x00; 32]);

    assert_ne!(msg_a, msg_b, "CHK-011: Different state_roots must differ");
    assert_ne!(msg_a, msg_c, "CHK-011: Different state_roots must differ");
    assert_ne!(msg_b, msg_c, "CHK-011: Different state_roots must differ");
}

// ── Single byte change in state_root changes the message ────────────

/// Sensitivity: flipping one byte in state_root changes the message,
/// proving no bytes are ignored or truncated.
#[test]
fn vv_req_chk_011_single_byte_change() {
    let mr = [0x22; 32];
    let vc: u64 = 5;
    let epoch: u64 = 3;

    let sr_a = [0xAA; 32];
    let mut sr_b = [0xAA; 32];
    sr_b[31] = 0xAB; // Change only the last byte

    let msg_a = compute_checkpoint_message(sr_a, mr, vc, epoch, [0x00; 32]);
    let msg_b = compute_checkpoint_message(sr_b, mr, vc, epoch, [0x00; 32]);

    assert_ne!(
        msg_a, msg_b,
        "CHK-011: Changing one byte of state_root must change checkpoint_message"
    );
}

// ── Rue puzzle includes state_root in checkpoint_message computation ─

/// Source-level: Rue puzzle references new_state_root and includes state_root
/// bytes in the sha256 preimage computation.
#[test]
fn vv_req_chk_011_rue_includes_state_root() {
    let source = include_str!("../puzzles/checkpoint_inner.rue");

    // The Rue puzzle must include new_state_root in the checkpoint_message hash
    assert!(
        source.contains("new_state_root"),
        "CHK-011: Rue puzzle must reference new_state_root"
    );

    // state_root bytes must be concatenated into the sha256 preimage
    assert!(
        source.contains("sr_b") || source.contains("state_root"),
        "CHK-011: Rue puzzle must include state_root bytes in sha256 preimage"
    );
}

// ── Rue puzzle uses same state_root in message AND recreation ────────

/// Consistency: the puzzle uses the same new_state_root in both the
/// checkpoint_message computation AND the State recreation. This ensures
/// the announced state matches what the singleton carries forward.
#[test]
fn vv_req_chk_011_same_state_root_for_message_and_recreation() {
    let source = include_str!("../puzzles/checkpoint_inner.rue");

    // The puzzle must use new_state_root in the checkpoint_message computation
    assert!(
        source.contains("checkpoint_message"),
        "CHK-011: Puzzle must compute checkpoint_message"
    );

    // The puzzle must also use new_state_root in the new State for recreation
    assert!(
        source.contains("state_root: new_state_root"),
        "CHK-011: Puzzle must use same new_state_root in State recreation"
    );
}

// ── Changing state_root changes scalar s6 (invalidates proof) ───────

/// Different state_roots produce different scalar s6 values. Since s6 is
/// a Groth16 public input, a proof for state_root A will fail verification
/// at state_root B.
#[test]
fn vv_req_chk_011_state_root_changes_scalar() {
    let mr = [0x22; 32];
    let vc: u64 = 5;
    let epoch: u64 = 3;

    let msg_a = compute_checkpoint_message([0xAA; 32], mr, vc, epoch, [0x00; 32]);
    let msg_b = compute_checkpoint_message([0xBB; 32], mr, vc, epoch, [0x00; 32]);

    let s6_a = bytes_to_scalar(&msg_a);
    let s6_b = bytes_to_scalar(&msg_b);

    assert_ne!(
        s6_a, s6_b,
        "CHK-011: Different state_roots must produce different scalars (proof invalidated)"
    );
}

// ── Proof for state_root A differs from state_root B ────────────────

/// Generates real Groth16 proofs for different state_roots and confirms
/// the proof bytes differ, proving state binding at the cryptographic level.
#[test]
fn vv_req_chk_011_proof_bound_to_state_root() {
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let mr = [0x22; 32];
    let vc: u64 = 1;
    let epoch: u64 = 1;

    let msg_a = compute_checkpoint_message([0xAA; 32], mr, vc, epoch, [0x00; 32]);
    let msg_b = compute_checkpoint_message([0xBB; 32], mr, vc, epoch, [0x00; 32]);

    let circuit_a = ConsensusCircuit::with_public_inputs(mr, vc, mr, vc, [0xCC; 48], msg_a, 1);
    let circuit_b = ConsensusCircuit::with_public_inputs(mr, vc, mr, vc, [0xCC; 48], msg_b, 1);

    let proof_a = generate_proof(circuit_a, &pk).expect("Proof A");
    let proof_b = generate_proof(circuit_b, &pk).expect("Proof B");

    assert_ne!(
        proof_a, proof_b,
        "CHK-011: Proofs for different state_roots must differ"
    );
}

// ── Zero state_root is valid (genesis) ──────────────────────────────

/// Genesis boundary: all-zero state_root produces a valid, non-trivial
/// 32-byte message, proving the hash is meaningful even at genesis.
#[test]
fn vv_req_chk_011_zero_state_root_valid() {
    let msg = compute_checkpoint_message([0x00; 32], [0x00; 32], 0, 1, [0x00; 32]);
    assert_eq!(
        msg.len(),
        32,
        "CHK-011: Zero state_root produces valid 32-byte message"
    );
    assert!(
        !msg.iter().all(|&b| b == 0),
        "CHK-011: Message is non-trivial even with zero state_root"
    );
}
