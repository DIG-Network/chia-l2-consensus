//! G1 pubkey aggregation for the Groth16 circuit.
//!
//! See [spec-groth16-circuit.md](../../docs/resources/spec-groth16-circuit.md).
//!
//! Implements CIR-003: Aggregate key constraint.
//! The circuit verifies that pk₁ + pk₂ + ... + pkₖ = agg_signers.

use ark_bls12_381::{G1Affine, G1Projective};
use ark_ec::{AffineRepr, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use super::G1_COMPRESSED_SIZE;

/// Aggregate multiple G1 pubkeys by elliptic curve addition.
///
/// Returns the sum of all pubkeys as a G1 point. If the input is empty,
/// returns the identity point (point at infinity).
///
/// This function computes: pk₁ + pk₂ + ... + pkₖ
///
/// Source: spec-groth16-circuit.md Lines 277-323 (Constraint 2: Aggregate consistency)
pub fn aggregate_pubkeys(pubkeys: &[[u8; 48]]) -> Result<[u8; 48], AggregateError> {
    // Start with identity (point at infinity)
    let mut sum = G1Projective::default(); // identity

    for (i, pk_bytes) in pubkeys.iter().enumerate() {
        // Deserialize the pubkey
        let pk = G1Affine::deserialize_compressed(&pk_bytes[..])
            .map_err(|_| AggregateError::InvalidPubkey(i))?;

        // Add to sum (in projective coordinates for efficiency)
        sum += pk;
    }

    // Convert to affine and serialize
    let result = G1Affine::from(sum);
    let mut bytes = Vec::with_capacity(G1_COMPRESSED_SIZE);
    result
        .serialize_compressed(&mut bytes)
        .map_err(|_| AggregateError::SerializationError)?;

    bytes
        .try_into()
        .map_err(|_| AggregateError::SerializationError)
}

/// Verify that the sum of pubkeys equals the expected aggregate.
///
/// This is the off-chain equivalent of the CIR-003 circuit constraint.
/// Returns true if pk₁ + pk₂ + ... + pkₖ = expected_aggregate.
///
/// Source: spec-groth16-circuit.md Lines 277-323
pub fn verify_aggregate(pubkeys: &[[u8; 48]], expected_aggregate: &[u8; 48]) -> bool {
    match aggregate_pubkeys(pubkeys) {
        Ok(computed) => computed == *expected_aggregate,
        Err(_) => false,
    }
}

/// Deserialize a G1 point from compressed bytes.
///
/// Returns None if the bytes are not a valid G1 point.
pub fn deserialize_g1(bytes: &[u8; 48]) -> Option<G1Affine> {
    G1Affine::deserialize_compressed(&bytes[..]).ok()
}

/// Serialize a G1 point to compressed bytes.
pub fn serialize_g1(point: &G1Affine) -> [u8; 48] {
    let mut bytes = Vec::with_capacity(G1_COMPRESSED_SIZE);
    point
        .serialize_compressed(&mut bytes)
        .expect("G1 serialization should not fail");
    bytes.try_into().expect("G1 should serialize to 48 bytes")
}

/// Get the G1 identity point (point at infinity) as bytes.
///
/// This is used for padding when k < MAX_SIGNERS.
/// identity + pk = pk, so adding identity doesn't change the sum.
pub fn g1_identity() -> [u8; 48] {
    serialize_g1(&G1Affine::zero())
}

/// Add two G1 points given as compressed bytes.
///
/// Returns None if either input is invalid.
pub fn add_g1(a: &[u8; 48], b: &[u8; 48]) -> Option<[u8; 48]> {
    let a_point = deserialize_g1(a)?;
    let b_point = deserialize_g1(b)?;
    let sum = G1Projective::from(a_point) + G1Projective::from(b_point);
    Some(serialize_g1(&G1Affine::from(sum)))
}

/// Negate a G1 point given as compressed bytes.
///
/// Returns -pk such that pk + (-pk) = identity.
/// Returns None if the input is invalid.
pub fn negate_g1(pk: &[u8; 48]) -> Option<[u8; 48]> {
    let point = deserialize_g1(pk)?;
    let negated = -point;
    Some(serialize_g1(&negated))
}

/// Error type for aggregation operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateError {
    /// Invalid pubkey at the given index.
    InvalidPubkey(usize),
    /// Serialization error.
    SerializationError,
}

impl std::fmt::Display for AggregateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregateError::InvalidPubkey(i) => write!(f, "Invalid pubkey at index {}", i),
            AggregateError::SerializationError => write!(f, "Serialization error"),
        }
    }
}

impl std::error::Error for AggregateError {}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ec::AffineRepr;
    use ark_ff::UniformRand;
    use rand::thread_rng;

    #[test]
    fn test_empty_aggregate_is_identity() {
        let result = aggregate_pubkeys(&[]).unwrap();
        assert_eq!(result, g1_identity());
    }

    #[test]
    fn test_single_pubkey_is_itself() {
        let mut rng = thread_rng();
        let scalar = ark_bls12_381::Fr::rand(&mut rng);
        let pk = G1Affine::from(G1Affine::generator() * scalar);
        let pk_bytes = serialize_g1(&pk);

        let result = aggregate_pubkeys(&[pk_bytes]).unwrap();
        assert_eq!(result, pk_bytes);
    }

    #[test]
    fn test_identity_round_trip() {
        let identity = g1_identity();
        let point = deserialize_g1(&identity).unwrap();
        assert!(point.is_zero());
    }

    #[test]
    fn test_negate_then_add_is_identity() {
        let mut rng = thread_rng();
        let scalar = ark_bls12_381::Fr::rand(&mut rng);
        let pk = G1Affine::from(G1Affine::generator() * scalar);
        let pk_bytes = serialize_g1(&pk);

        let neg_pk = negate_g1(&pk_bytes).unwrap();
        let sum = add_g1(&pk_bytes, &neg_pk).unwrap();

        assert_eq!(sum, g1_identity());
    }
}
