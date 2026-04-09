//! REQUIREMENT: WIRE-003 — Groth16 Proof Format
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-003`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-003.md`.
//!
//! Verifies that the Groth16 proof is exactly 192 bytes:
//! A (G1, 48 bytes) + B (G2, 96 bytes) + C (G1, 48 bytes).

use ark_bls12_381::{Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use ark_ff::UniformRand;
use ark_serialize::CanonicalSerialize;
use chia_l2_consensus::{G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE, GROTH16_PROOF_SIZE};
use rand::thread_rng;

#[test]
fn vv_req_wire_003_proof_size_is_192_bytes() {
    // WIRE-003: Total proof size is 48 + 96 + 48 = 192 bytes
    assert_eq!(
        GROTH16_PROOF_SIZE, 192,
        "WIRE-003: Groth16 proof must be exactly 192 bytes"
    );
}

#[test]
fn vv_req_wire_003_proof_a_is_48_bytes() {
    // WIRE-003: proof.a is G1 compressed (48 bytes)
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);
    let a = G1Affine::from(G1Affine::generator() * scalar);

    let mut bytes = Vec::new();
    a.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G1_COMPRESSED_SIZE,
        "WIRE-003: proof.a must be 48 bytes (G1 compressed)"
    );
}

#[test]
fn vv_req_wire_003_proof_b_is_96_bytes() {
    // WIRE-003: proof.b is G2 compressed (96 bytes)
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);
    let b = G2Affine::from(G2Affine::generator() * scalar);

    let mut bytes = Vec::new();
    b.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G2_COMPRESSED_SIZE,
        "WIRE-003: proof.b must be 96 bytes (G2 compressed)"
    );
}

#[test]
fn vv_req_wire_003_proof_c_is_48_bytes() {
    // WIRE-003: proof.c is G1 compressed (48 bytes)
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);
    let c = G1Affine::from(G1Affine::generator() * scalar);

    let mut bytes = Vec::new();
    c.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G1_COMPRESSED_SIZE,
        "WIRE-003: proof.c must be 48 bytes (G1 compressed)"
    );
}

#[test]
fn vv_req_wire_003_concatenated_abc_is_192() {
    // WIRE-003: Concatenating A || B || C gives exactly 192 bytes
    let mut rng = thread_rng();

    let scalar_a = Fr::rand(&mut rng);
    let scalar_b = Fr::rand(&mut rng);
    let scalar_c = Fr::rand(&mut rng);

    let a = G1Affine::from(G1Affine::generator() * scalar_a);
    let b = G2Affine::from(G2Affine::generator() * scalar_b);
    let c = G1Affine::from(G1Affine::generator() * scalar_c);

    let mut a_bytes = Vec::new();
    let mut b_bytes = Vec::new();
    let mut c_bytes = Vec::new();

    a.serialize_compressed(&mut a_bytes).unwrap();
    b.serialize_compressed(&mut b_bytes).unwrap();
    c.serialize_compressed(&mut c_bytes).unwrap();

    let total = a_bytes.len() + b_bytes.len() + c_bytes.len();

    assert_eq!(
        total, GROTH16_PROOF_SIZE,
        "WIRE-003: A || B || C must be exactly 192 bytes"
    );
}

#[test]
fn vv_req_wire_003_field_order_is_abc() {
    // WIRE-003: Proof MUST be serialized in order A, B, C
    // This test documents the expected order for concatenation

    let mut rng = thread_rng();

    // Create distinct points so we can verify ordering
    let a = G1Affine::from(G1Affine::generator() * Fr::rand(&mut rng));
    let b = G2Affine::from(G2Affine::generator() * Fr::rand(&mut rng));
    let c = G1Affine::from(G1Affine::generator() * Fr::rand(&mut rng));

    let mut a_bytes = Vec::new();
    let mut b_bytes = Vec::new();
    let mut c_bytes = Vec::new();

    a.serialize_compressed(&mut a_bytes).unwrap();
    b.serialize_compressed(&mut b_bytes).unwrap();
    c.serialize_compressed(&mut c_bytes).unwrap();

    // Concatenate in order A || B || C
    let mut proof_bytes = Vec::with_capacity(GROTH16_PROOF_SIZE);
    proof_bytes.extend_from_slice(&a_bytes);
    proof_bytes.extend_from_slice(&b_bytes);
    proof_bytes.extend_from_slice(&c_bytes);

    // Verify structure
    assert_eq!(proof_bytes.len(), 192);
    assert_eq!(&proof_bytes[0..48], &a_bytes[..], "First 48 bytes are A");
    assert_eq!(&proof_bytes[48..144], &b_bytes[..], "Next 96 bytes are B");
    assert_eq!(&proof_bytes[144..192], &c_bytes[..], "Final 48 bytes are C");
}

#[test]
fn vv_req_wire_003_separate_atoms_for_clvm() {
    // WIRE-003: CLVM receives three separate atoms (not concatenated)
    // This is how the proof is passed to the checkpoint singleton

    let mut rng = thread_rng();

    let a = G1Affine::from(G1Affine::generator() * Fr::rand(&mut rng));
    let b = G2Affine::from(G2Affine::generator() * Fr::rand(&mut rng));
    let c = G1Affine::from(G1Affine::generator() * Fr::rand(&mut rng));

    let mut a_bytes = Vec::new();
    let mut b_bytes = Vec::new();
    let mut c_bytes = Vec::new();

    a.serialize_compressed(&mut a_bytes).unwrap();
    b.serialize_compressed(&mut b_bytes).unwrap();
    c.serialize_compressed(&mut c_bytes).unwrap();

    // Simulate CLVM representation: (proof_a proof_b proof_c ...)
    // Each atom has the correct size for its curve group
    assert_eq!(
        a_bytes.len(),
        48,
        "WIRE-003: proof_a atom is 48 bytes for G1"
    );
    assert_eq!(
        b_bytes.len(),
        96,
        "WIRE-003: proof_b atom is 96 bytes for G2"
    );
    assert_eq!(
        c_bytes.len(),
        48,
        "WIRE-003: proof_c atom is 48 bytes for G1"
    );
}

#[test]
fn vv_req_wire_003_identity_points() {
    // WIRE-003: Identity/infinity points still produce correct sizes
    let a = G1Affine::zero();
    let b = G2Affine::zero();
    let c = G1Affine::zero();

    let mut a_bytes = Vec::new();
    let mut b_bytes = Vec::new();
    let mut c_bytes = Vec::new();

    a.serialize_compressed(&mut a_bytes).unwrap();
    b.serialize_compressed(&mut b_bytes).unwrap();
    c.serialize_compressed(&mut c_bytes).unwrap();

    let total = a_bytes.len() + b_bytes.len() + c_bytes.len();
    assert_eq!(
        total, 192,
        "WIRE-003: Identity points still produce 192-byte proof"
    );
}
