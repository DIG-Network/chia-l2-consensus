//! REQUIREMENT: WIRE-002 — Point Encoding (G1 and G2)
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-002`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-002.md`.
//!
//! Verifies that G1 points are 48 bytes and G2 points are 96 bytes
//! using ZCash compressed BLS12-381 format.

use ark_bls12_381::{Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::AffineRepr;
use ark_ff::UniformRand;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use chia_l2_consensus::{G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE};
use rand::thread_rng;

#[test]
fn vv_req_wire_002_g1_is_48_bytes() {
    // WIRE-002: G1 points (pubkeys) MUST be 48 bytes
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);
    let point = G1Affine::generator() * scalar;
    let g1 = G1Affine::from(point);

    let mut bytes = Vec::new();
    g1.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G1_COMPRESSED_SIZE,
        "WIRE-002: G1 points must be exactly 48 bytes"
    );
}

#[test]
fn vv_req_wire_002_g2_is_96_bytes() {
    // WIRE-002: G2 points (signatures) MUST be 96 bytes
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);
    let point = G2Affine::generator() * scalar;
    let g2 = G2Affine::from(point);

    let mut bytes = Vec::new();
    g2.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G2_COMPRESSED_SIZE,
        "WIRE-002: G2 points must be exactly 96 bytes"
    );
}

#[test]
fn vv_req_wire_002_g1_generator_is_48_bytes() {
    // WIRE-002: The G1 generator must also be 48 bytes
    let g1 = G1Affine::generator();

    let mut bytes = Vec::new();
    g1.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G1_COMPRESSED_SIZE,
        "WIRE-002: G1 generator must be exactly 48 bytes"
    );
}

#[test]
fn vv_req_wire_002_g2_generator_is_96_bytes() {
    // WIRE-002: The G2 generator must also be 96 bytes
    let g2 = G2Affine::generator();

    let mut bytes = Vec::new();
    g2.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G2_COMPRESSED_SIZE,
        "WIRE-002: G2 generator must be exactly 96 bytes"
    );
}

#[test]
fn vv_req_wire_002_g1_infinity_is_48_bytes() {
    // WIRE-002: Point at infinity is still 48 bytes for G1
    let g1 = G1Affine::zero();

    let mut bytes = Vec::new();
    g1.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G1_COMPRESSED_SIZE,
        "WIRE-002: G1 identity/infinity must be exactly 48 bytes"
    );

    // Verify infinity flag is set (bit 6 of first byte)
    // In ZCash format: byte[0] has compression flag (bit 7) and infinity flag (bit 6)
    let first_byte = bytes[0];
    let infinity_flag = (first_byte >> 6) & 1;
    assert_eq!(infinity_flag, 1, "WIRE-002: G1 infinity flag must be set");
}

#[test]
fn vv_req_wire_002_g2_infinity_is_96_bytes() {
    // WIRE-002: Point at infinity is still 96 bytes for G2
    let g2 = G2Affine::zero();

    let mut bytes = Vec::new();
    g2.serialize_compressed(&mut bytes).unwrap();

    assert_eq!(
        bytes.len(),
        G2_COMPRESSED_SIZE,
        "WIRE-002: G2 identity/infinity must be exactly 96 bytes"
    );

    // Verify infinity flag is set
    let first_byte = bytes[0];
    let infinity_flag = (first_byte >> 6) & 1;
    assert_eq!(infinity_flag, 1, "WIRE-002: G2 infinity flag must be set");
}

#[test]
fn vv_req_wire_002_g1_round_trip() {
    // WIRE-002: G1 serialize then deserialize must produce same point
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);
    let original = G1Affine::from(G1Affine::generator() * scalar);

    let mut bytes = Vec::new();
    original.serialize_compressed(&mut bytes).unwrap();

    let recovered = G1Affine::deserialize_compressed(&bytes[..]).unwrap();

    assert_eq!(
        original, recovered,
        "WIRE-002: G1 round-trip must preserve point"
    );
}

#[test]
fn vv_req_wire_002_g2_round_trip() {
    // WIRE-002: G2 serialize then deserialize must produce same point
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);
    let original = G2Affine::from(G2Affine::generator() * scalar);

    let mut bytes = Vec::new();
    original.serialize_compressed(&mut bytes).unwrap();

    let recovered = G2Affine::deserialize_compressed(&bytes[..]).unwrap();

    assert_eq!(
        original, recovered,
        "WIRE-002: G2 round-trip must preserve point"
    );
}

#[test]
fn vv_req_wire_002_compression_flag_set() {
    // WIRE-002: Compression flag (bit 7) must be set for compressed format
    let g1 = G1Affine::generator();
    let g2 = G2Affine::generator();

    let mut g1_bytes = Vec::new();
    let mut g2_bytes = Vec::new();
    g1.serialize_compressed(&mut g1_bytes).unwrap();
    g2.serialize_compressed(&mut g2_bytes).unwrap();

    // Compression flag is bit 7 (MSB) of first byte
    let g1_compression_flag = (g1_bytes[0] >> 7) & 1;
    let g2_compression_flag = (g2_bytes[0] >> 7) & 1;

    assert_eq!(
        g1_compression_flag, 1,
        "WIRE-002: G1 compression flag must be set"
    );
    assert_eq!(
        g2_compression_flag, 1,
        "WIRE-002: G2 compression flag must be set"
    );
}

#[test]
fn vv_req_wire_002_multiple_random_points() {
    // WIRE-002: Multiple random points must all have correct sizes
    let mut rng = thread_rng();

    for _ in 0..10 {
        let scalar = Fr::rand(&mut rng);
        let g1 = G1Affine::from(G1Affine::generator() * scalar);
        let g2 = G2Affine::from(G2Affine::generator() * scalar);

        let mut g1_bytes = Vec::new();
        let mut g2_bytes = Vec::new();
        g1.serialize_compressed(&mut g1_bytes).unwrap();
        g2.serialize_compressed(&mut g2_bytes).unwrap();

        assert_eq!(g1_bytes.len(), G1_COMPRESSED_SIZE);
        assert_eq!(g2_bytes.len(), G2_COMPRESSED_SIZE);
    }
}

#[test]
fn vv_req_wire_002_projective_to_affine_preserves_size() {
    // WIRE-002: Converting from projective to affine preserves serialization size
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);

    // Work in projective coordinates
    let g1_proj: G1Projective = G1Affine::generator().into();
    let g1_proj = g1_proj * scalar;
    let g1_affine = G1Affine::from(g1_proj);

    let g2_proj: G2Projective = G2Affine::generator().into();
    let g2_proj = g2_proj * scalar;
    let g2_affine = G2Affine::from(g2_proj);

    let mut g1_bytes = Vec::new();
    let mut g2_bytes = Vec::new();
    g1_affine.serialize_compressed(&mut g1_bytes).unwrap();
    g2_affine.serialize_compressed(&mut g2_bytes).unwrap();

    assert_eq!(
        g1_bytes.len(),
        G1_COMPRESSED_SIZE,
        "WIRE-002: G1 from projective must be 48 bytes"
    );
    assert_eq!(
        g2_bytes.len(),
        G2_COMPRESSED_SIZE,
        "WIRE-002: G2 from projective must be 96 bytes"
    );
}
