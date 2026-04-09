//! REQUIREMENT: WIRE-006 — scalar() Function
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-006`).
//!
//! Spec: `docs/requirements/domains/wire/specs/WIRE-006.md`.
//!
//! Verifies that the scalar(bytes) function computes sha256(bytes)
//! interpreted as big-endian u256, reduced modulo the BLS12-381 scalar field order r.

use ark_ff::{BigInteger, PrimeField};
use chia_l2_consensus::bytes_to_scalar;
use num_bigint::BigUint;
use sha2::{Digest, Sha256};

/// BLS12-381 scalar field modulus r
/// r = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001
const BLS12_381_R_HEX: &str = "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001";

#[test]
fn vv_req_wire_006_output_is_field_element() {
    // WIRE-006: Output is a BLS12-381 scalar field element
    let bytes = b"test data";
    let scalar = bytes_to_scalar(bytes);

    // Verify it's less than r (it must be, as it's an Fr element)
    let scalar_bytes = scalar.into_bigint().to_bytes_be();

    // The scalar modulus
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
    let scalar_big = BigUint::from_bytes_be(&scalar_bytes);

    assert!(
        scalar_big < r,
        "WIRE-006: scalar must be less than field modulus r"
    );
}

#[test]
fn vv_req_wire_006_uses_sha256() {
    // WIRE-006: scalar(bytes) = sha256(bytes) as big-endian u256 mod r
    let bytes = b"simple test";
    let scalar = bytes_to_scalar(bytes);

    // Manual computation
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash: [u8; 32] = hasher.finalize().into();

    // Convert hash to big-endian integer
    let hash_int = BigUint::from_bytes_be(&hash);

    // Reduce mod r
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
    let expected_int = hash_int % &r;

    // Convert scalar to BigUint for comparison
    let scalar_bytes = scalar.into_bigint().to_bytes_be();
    let scalar_int = BigUint::from_bytes_be(&scalar_bytes);

    assert_eq!(
        scalar_int, expected_int,
        "WIRE-006: scalar must be sha256 mod r"
    );
}

#[test]
fn vv_req_wire_006_deterministic() {
    // WIRE-006: Same inputs always produce same output
    let bytes = b"deterministic test";

    let scalar1 = bytes_to_scalar(bytes);
    let scalar2 = bytes_to_scalar(bytes);
    let scalar3 = bytes_to_scalar(bytes);

    assert_eq!(scalar1, scalar2, "WIRE-006: Must be deterministic");
    assert_eq!(scalar2, scalar3, "WIRE-006: Must be deterministic");
}

#[test]
fn vv_req_wire_006_different_inputs_differ() {
    // WIRE-006: Different inputs produce different scalars
    let scalar1 = bytes_to_scalar(b"input 1");
    let scalar2 = bytes_to_scalar(b"input 2");

    assert_ne!(
        scalar1, scalar2,
        "WIRE-006: Different inputs must produce different scalars"
    );
}

#[test]
fn vv_req_wire_006_empty_input() {
    // WIRE-006: Empty input is valid (sha256 of empty bytes)
    let scalar = bytes_to_scalar(b"");

    // sha256 of empty = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let mut hasher = Sha256::new();
    hasher.update(b"");
    let hash: [u8; 32] = hasher.finalize().into();

    let hash_int = BigUint::from_bytes_be(&hash);
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
    let expected_int = hash_int % &r;

    let scalar_bytes = scalar.into_bigint().to_bytes_be();
    let scalar_int = BigUint::from_bytes_be(&scalar_bytes);

    assert_eq!(scalar_int, expected_int, "WIRE-006: Empty input must work");
}

#[test]
fn vv_req_wire_006_32_byte_input() {
    // WIRE-006: Test with 32-byte input (common case for roots/hashes)
    let input = [0x42u8; 32];
    let scalar = bytes_to_scalar(&input);

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let hash: [u8; 32] = hasher.finalize().into();

    let hash_int = BigUint::from_bytes_be(&hash);
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
    let expected_int = hash_int % &r;

    let scalar_bytes = scalar.into_bigint().to_bytes_be();
    let scalar_int = BigUint::from_bytes_be(&scalar_bytes);

    assert_eq!(
        scalar_int, expected_int,
        "WIRE-006: 32-byte input must work"
    );
}

#[test]
fn vv_req_wire_006_8_byte_integer() {
    // WIRE-006: Test with big-endian u64 (used for validator_count/epoch)
    let count: u64 = 1000;
    let bytes = count.to_be_bytes();
    let scalar = bytes_to_scalar(&bytes);

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash: [u8; 32] = hasher.finalize().into();

    let hash_int = BigUint::from_bytes_be(&hash);
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
    let expected_int = hash_int % &r;

    let scalar_bytes = scalar.into_bigint().to_bytes_be();
    let scalar_int = BigUint::from_bytes_be(&scalar_bytes);

    assert_eq!(
        scalar_int, expected_int,
        "WIRE-006: u64 big-endian input must work"
    );
}

#[test]
fn vv_req_wire_006_48_byte_pubkey() {
    // WIRE-006: Test with 48-byte G1 pubkey (agg_signers)
    let pubkey = [0xAAu8; 48];
    let scalar = bytes_to_scalar(&pubkey);

    let mut hasher = Sha256::new();
    hasher.update(&pubkey);
    let hash: [u8; 32] = hasher.finalize().into();

    let hash_int = BigUint::from_bytes_be(&hash);
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
    let expected_int = hash_int % &r;

    let scalar_bytes = scalar.into_bigint().to_bytes_be();
    let scalar_int = BigUint::from_bytes_be(&scalar_bytes);

    assert_eq!(
        scalar_int, expected_int,
        "WIRE-006: 48-byte pubkey input must work"
    );
}

#[test]
fn vv_req_wire_006_reduction_occurs() {
    // WIRE-006: Verify that reduction mod r actually occurs for large hash values
    // We can't easily find an input that produces a hash > r, but we can verify
    // that the output is always less than r

    // Test with various inputs
    for i in 0..20u8 {
        let input = [i; 32];
        let scalar = bytes_to_scalar(&input);

        let scalar_bytes = scalar.into_bigint().to_bytes_be();
        let scalar_int = BigUint::from_bytes_be(&scalar_bytes);

        let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
        assert!(scalar_int < r, "WIRE-006: All scalars must be less than r");
    }
}

#[test]
fn vv_req_wire_006_known_test_vector() {
    // WIRE-006: Known test vector for cross-implementation verification
    let input = [0x01u8; 32]; // 32 bytes of 0x01
    let scalar = bytes_to_scalar(&input);

    // Manual computation
    let mut hasher = Sha256::new();
    hasher.update(&input);
    let hash: [u8; 32] = hasher.finalize().into();

    let hash_int = BigUint::from_bytes_be(&hash);
    let r = BigUint::parse_bytes(BLS12_381_R_HEX.as_bytes(), 16).unwrap();
    let expected_int = hash_int % &r;

    let scalar_bytes = scalar.into_bigint().to_bytes_be();
    let scalar_int = BigUint::from_bytes_be(&scalar_bytes);

    assert_eq!(
        scalar_int, expected_int,
        "WIRE-006: Known test vector must match"
    );

    // Print for cross-impl verification
    eprintln!("WIRE-006 test vector input: [0x01; 32]");
    eprintln!("WIRE-006 sha256 hash: {:02x?}", hash);
    eprintln!(
        "WIRE-006 scalar (hex): {}",
        hex::encode(&scalar.into_bigint().to_bytes_be())
    );
}
