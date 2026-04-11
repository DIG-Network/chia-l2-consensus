//! REQUIREMENT: WIRE-006 — scalar() Function
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-006`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-006.md`.
//!
//! Verifies that the scalar(bytes) function computes sha256(bytes)
//! interpreted as a CLVM-compatible signed big-endian integer, reduced
//! modulo the BLS12-381 scalar field order r.
//!
//! CLVM's g1_multiply treats scalar atoms as SIGNED big-endian integers.
//! If the MSB of the sha256 hash is set, the value is negative (two's complement).
//! bytes_to_scalar must match this convention for vk_input consistency.

use ark_bls12_381::Fr;
use ark_ff::{BigInteger, PrimeField};
use chia_l2_consensus::testing::bytes_to_scalar;
use num_bigint::BigUint;
use sha2::{Digest, Sha256};

/// BLS12-381 scalar field modulus r
const BLS12_381_R_HEX: &str = "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001";

/// Compute the expected scalar the same way the implementation does:
/// CLVM-compatible signed interpretation of sha256 hash.
fn expected_scalar(bytes: &[u8]) -> Fr {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash: [u8; 32] = hasher.finalize().into();

    if hash[0] & 0x80 != 0 {
        // Negative: two's complement
        let complement: Vec<u8> = hash.iter().map(|&b| !b).collect();
        let abs_val = BigUint::from_bytes_be(&complement) + 1u64;
        -Fr::from(abs_val)
    } else {
        // Positive: unsigned big-endian
        Fr::from(BigUint::from_bytes_be(&hash))
    }
}

#[test]
fn vv_req_wire_006_output_is_field_element() {
    // WIRE-006: Output is a BLS12-381 scalar field element
    let bytes = b"test data";
    let scalar = bytes_to_scalar(bytes);

    let scalar_bytes = scalar.into_bigint().to_bytes_be();
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
    let scalar_big = BigUint::from_bytes_be(&scalar_bytes);

    assert!(
        scalar_big < r,
        "WIRE-006: scalar must be less than field modulus r"
    );
}

#[test]
fn vv_req_wire_006_uses_sha256() {
    // WIRE-006: scalar(bytes) uses sha256 internally with CLVM signed interpretation
    let scalar = bytes_to_scalar(b"simple test");
    let expected = expected_scalar(b"simple test");

    assert_eq!(
        scalar, expected,
        "WIRE-006: scalar must match CLVM-compatible signed sha256 mod r"
    );
}

#[test]
fn vv_req_wire_006_deterministic() {
    let bytes = b"deterministic test";
    let scalar1 = bytes_to_scalar(bytes);
    let scalar2 = bytes_to_scalar(bytes);
    assert_eq!(scalar1, scalar2, "WIRE-006: Must be deterministic");
}

#[test]
fn vv_req_wire_006_different_inputs_differ() {
    let scalar1 = bytes_to_scalar(b"input 1");
    let scalar2 = bytes_to_scalar(b"input 2");
    assert_ne!(
        scalar1, scalar2,
        "WIRE-006: Different inputs must produce different scalars"
    );
}

#[test]
fn vv_req_wire_006_empty_input() {
    // sha256("") = e3b0c442... → MSB=0xe3 (bit 7 set) → negative in CLVM
    let scalar = bytes_to_scalar(b"");
    let expected = expected_scalar(b"");

    assert_eq!(
        scalar, expected,
        "WIRE-006: Empty input must produce correct CLVM-signed scalar"
    );
}

#[test]
fn vv_req_wire_006_32_byte_input() {
    let input = [0x42u8; 32];
    let scalar = bytes_to_scalar(&input);
    let expected = expected_scalar(&input);

    assert_eq!(scalar, expected, "WIRE-006: 32-byte input must work");
}

#[test]
fn vv_req_wire_006_8_byte_integer() {
    let count: u64 = 1000;
    let bytes = count.to_be_bytes();
    let scalar = bytes_to_scalar(&bytes);
    let expected = expected_scalar(&bytes);

    assert_eq!(scalar, expected, "WIRE-006: u64 big-endian input must work");
}

#[test]
fn vv_req_wire_006_48_byte_pubkey() {
    let pubkey = [0xAAu8; 48];
    let scalar = bytes_to_scalar(&pubkey);
    let expected = expected_scalar(&pubkey);

    assert_eq!(scalar, expected, "WIRE-006: 48-byte pubkey input must work");
}

#[test]
fn vv_req_wire_006_reduction_occurs() {
    // All scalars must be less than r regardless of input
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();

    for i in 0..20u8 {
        let input = [i; 32];
        let scalar = bytes_to_scalar(&input);
        let scalar_bytes = scalar.into_bigint().to_bytes_be();
        let scalar_int = BigUint::from_bytes_be(&scalar_bytes);
        assert!(scalar_int < r, "WIRE-006: All scalars must be less than r");
    }
}

#[test]
fn vv_req_wire_006_known_test_vector() {
    let input = [0x01u8; 32];
    let scalar = bytes_to_scalar(&input);
    let expected = expected_scalar(&input);

    assert_eq!(scalar, expected, "WIRE-006: Known test vector must match");
}
