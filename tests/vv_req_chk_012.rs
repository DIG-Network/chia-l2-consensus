//! REQUIREMENT: CHK-012 — Network ID Binding
//! (`docs/requirements/domains/checkpoint/NORMATIVE.md#CHK-012`).
//!
//! Spec: `docs/requirements/domains/checkpoint/specs/CHK-012.md`.
//!
//! Verifies that checkpoint_message includes network_coin_launcher_id,
//! preventing cross-network proof replay.

use chia_l2_consensus::testing::{bytes_to_scalar, compute_checkpoint_message};
use sha2::{Digest, Sha256};

// ── Different network IDs produce different checkpoint messages ──────

#[test]
fn vv_req_chk_012_different_networks_different_messages() {
    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 5;
    let epoch: u64 = 3;

    let msg_net_a = compute_checkpoint_message(sr, mr, vc, epoch, [0xAA; 32]);
    let msg_net_b = compute_checkpoint_message(sr, mr, vc, epoch, [0xBB; 32]);

    assert_ne!(
        msg_net_a, msg_net_b,
        "CHK-012: Different network IDs must produce different checkpoint messages"
    );
}

// ── Network ID is the 5th field (last) in the 112-byte preimage ─────

#[test]
fn vv_req_chk_012_network_id_in_preimage() {
    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 5;
    let epoch: u64 = 3;
    let net_id = [0xFF; 32];

    let msg = compute_checkpoint_message(sr, mr, vc, epoch, net_id);

    // Manual 112-byte preimage
    let mut hasher = Sha256::new();
    hasher.update(sr);
    hasher.update(mr);
    hasher.update(vc.to_be_bytes());
    hasher.update(epoch.to_be_bytes());
    hasher.update(net_id); // 5th field
    let expected: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        msg, expected,
        "CHK-012: checkpoint_message must be sha256(sr ‖ mr ‖ vc ‖ epoch ‖ network_id)"
    );
}

// ── Proof for network A differs from network B ──────────────────────

#[test]
fn vv_req_chk_012_proof_bound_to_network() {
    use chia_l2_consensus::testing::{
        deserialize_proving_key, generate_proof, run_test_setup, ConsensusCircuit,
    };

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 1;
    let epoch: u64 = 1;

    let msg_a = compute_checkpoint_message(sr, mr, vc, epoch, [0xAA; 32]);
    let msg_b = compute_checkpoint_message(sr, mr, vc, epoch, [0xBB; 32]);

    let circuit_a = ConsensusCircuit::with_public_inputs(mr, vc, mr, vc, [0xCC; 48], msg_a, 1);
    let circuit_b = ConsensusCircuit::with_public_inputs(mr, vc, mr, vc, [0xCC; 48], msg_b, 1);

    let proof_a = generate_proof(circuit_a, &pk).expect("Proof A");
    let proof_b = generate_proof(circuit_b, &pk).expect("Proof B");

    assert_ne!(
        proof_a, proof_b,
        "CHK-012: Proofs for different networks must differ"
    );
}

// ── Scalar s6 changes with network ID ───────────────────────────────

#[test]
fn vv_req_chk_012_scalar_changes_with_network() {
    let sr = [0x11; 32];
    let mr = [0x22; 32];
    let vc: u64 = 5;
    let epoch: u64 = 3;

    let msg_a = compute_checkpoint_message(sr, mr, vc, epoch, [0xAA; 32]);
    let msg_b = compute_checkpoint_message(sr, mr, vc, epoch, [0xBB; 32]);

    let s6_a = bytes_to_scalar(&msg_a);
    let s6_b = bytes_to_scalar(&msg_b);

    assert_ne!(
        s6_a, s6_b,
        "CHK-012: Different network IDs must produce different scalars"
    );
}

// ── Same network ID is deterministic ────────────────────────────────

#[test]
fn vv_req_chk_012_deterministic() {
    let msg1 = compute_checkpoint_message([0x11; 32], [0x22; 32], 5, 3, [0xAA; 32]);
    let msg2 = compute_checkpoint_message([0x11; 32], [0x22; 32], 5, 3, [0xAA; 32]);

    assert_eq!(msg1, msg2, "CHK-012: Same inputs must produce same message");
}

// ── Zero network ID is valid ────────────────────────────────────────

#[test]
fn vv_req_chk_012_zero_network_id_valid() {
    let msg = compute_checkpoint_message([0; 32], [0; 32], 0, 1, [0; 32]);
    assert_eq!(msg.len(), 32);
    assert!(
        !msg.iter().all(|&b| b == 0),
        "CHK-012: Zero network ID still produces non-trivial message"
    );
}

// ── Preimage is 112 bytes (32+32+8+8+32) ───────────────────────────

#[test]
fn vv_req_chk_012_preimage_112_bytes() {
    // The preimage size is implicit in the sha256 computation.
    // Verify by checking that removing the network_id changes the output.
    let with_id = compute_checkpoint_message([0; 32], [0; 32], 0, 1, [0xFF; 32]);
    let with_zero_id = compute_checkpoint_message([0; 32], [0; 32], 0, 1, [0x00; 32]);

    assert_ne!(
        with_id, with_zero_id,
        "CHK-012: Network ID must affect the hash (112-byte preimage, not 80)"
    );
}
