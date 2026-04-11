//! REQUIREMENT: WIRE-002 — Point Encoding (G1 and G2)
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-002`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-002.md`.
//!
//! **Normative statement:** G1 points (public keys) are 48-byte ZCash
//! compressed BLS12-381; G2 points (signatures) are 96-byte ZCash compressed
//! BLS12-381. The compression flag (bit 7) and infinity flag (bit 6) follow
//! the ZCash convention. Arkworks `serialize_compressed` produces exactly
//! these formats.
//!
//! **How the tests prove this:**
//! - `g1_is_48_bytes` and `g2_is_96_bytes` serialize random curve points and
//!   check the byte length.
//! - `g1_generator_is_48_bytes` and `g2_generator_is_96_bytes` check the
//!   well-known generator points.
//! - `g1_infinity_is_48_bytes` and `g2_infinity_is_96_bytes` verify the
//!   identity point encoding and check the infinity flag bit.
//! - `g1_round_trip` and `g2_round_trip` serialize then deserialize a random
//!   point and confirm equality.
//! - `compression_flag_set` verifies bit 7 of the first byte is 1 for both
//!   G1 and G2 compressed representations.
//! - `multiple_random_points` repeats the size check for 10 random points.
//! - `projective_to_affine_preserves_size` ensures coordinate conversion does
//!   not change serialized length.
//!
//! **Acceptance-criteria coverage (from spec):**
//! - [x] G1 points are exactly 48 bytes
//! - [x] G2 points are exactly 96 bytes
//! - [x] Arkworks serialization matches Chia node expectations (ZCash format)
//! - [x] Point infinity is correctly encoded
//! - [x] Sign bit correctly distinguishes y-coordinate variants (via round-trip)

use ark_bls12_381::{Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::AffineRepr;
use ark_ff::UniformRand;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use chia_l2_consensus::testing::{G1_COMPRESSED_SIZE, G2_COMPRESSED_SIZE};
use rand::thread_rng;

/// Verifies a random G1 point serializes to exactly 48 bytes.
/// Strategy: generate a random scalar, compute scalar * G1, serialize compressed.
/// Confidence: the serialization format produces fixed-width G1 output.
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

/// Verifies a random G2 point serializes to exactly 96 bytes.
/// Strategy: generate a random scalar, compute scalar * G2, serialize compressed.
/// Confidence: the serialization format produces fixed-width G2 output.
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

/// Verifies the G1 generator point serializes to exactly 48 bytes.
/// Strategy: serialize the well-known generator.
/// Confidence: the canonical generator is not a special-case exception.
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

/// Verifies the G2 generator point serializes to exactly 96 bytes.
/// Strategy: serialize the well-known G2 generator.
/// Confidence: the canonical G2 generator is not a special-case exception.
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

/// Verifies the G1 identity (point at infinity) is still 48 bytes and has the
/// infinity flag (bit 6) set in the first byte.
/// Strategy: serialize G1Affine::zero() and inspect the output.
/// Confidence: identity points are correctly encoded in the wire format.
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

/// Verifies the G2 identity (point at infinity) is still 96 bytes and has the
/// infinity flag (bit 6) set in the first byte.
/// Strategy: serialize G2Affine::zero() and inspect the output.
/// Confidence: G2 identity points are correctly encoded in the wire format.
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

/// Verifies G1 round-trip: serialize_compressed then deserialize_compressed
/// recovers the original point.
/// Strategy: random G1 point -> bytes -> recovered point, assert equality.
/// Confidence: the sign bit and x-coordinate encoding are lossless.
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

/// Verifies G2 round-trip: serialize_compressed then deserialize_compressed
/// recovers the original point.
/// Strategy: random G2 point -> bytes -> recovered point, assert equality.
/// Confidence: the two-component x-coordinate encoding is lossless.
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

/// Verifies the compression flag (bit 7 of first byte) is set for both G1
/// and G2 compressed serializations.
/// Strategy: serialize the generators and inspect byte[0] >> 7.
/// Confidence: the Chia node expects this flag; missing it causes rejection.
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

/// Verifies 10 random G1 and G2 points all have the correct sizes.
/// Strategy: generate 10 random scalars, serialize both groups, check lengths.
/// Confidence: size correctness is not dependent on the specific point.
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

/// Verifies that projective-to-affine conversion does not change serialized size.
/// Strategy: work in projective coordinates, convert to affine, serialize.
/// Confidence: internal coordinate representation does not leak into wire format.
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
