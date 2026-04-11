//! G1 pubkey aggregation for the Groth16 circuit (CIR-003).
//!
//! This module provides off-chain G1 elliptic curve point operations used to
//! compute the `agg_signers` public input for the Groth16 proof. The circuit
//! verifies that pk₁ + pk₂ + ... + pkₖ = agg_signers (Constraint 2).
//!
//! All G1 points use BLS12-381 compressed serialization (48 bytes, ZCash format).
//! The group operation is standard elliptic curve point addition on the G1 curve.
//!
//! Source: [spec-groth16-circuit.md Lines 277-323](../../docs/resources/spec-groth16-circuit.md)
//! (Constraint 2: Aggregate Consistency).
//! Source: [spec-wire-format.md Lines 466-544](../../docs/resources/spec-wire-format.md)
//! (Aggregate Public Key encoding).
//!
//! # Cross-references
//!
//! - Circuit constraint: [`crate::prover::circuit::ConsensusCircuit`] (CIR-003)
//! - Majority threshold: [`crate::prover::majority`] (CIR-004, determines k)
//! - Wire format: [`crate::prover::serialize`] (public input encoding)

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
/// The result is the `agg_signers` value passed as a public input to the
/// Groth16 circuit and also included in the checkpoint singleton solution
/// for BLS signature verification.
///
/// Source: [spec-groth16-circuit.md Lines 277-323](../../docs/resources/spec-groth16-circuit.md)
/// (Constraint 2: Aggregate Consistency).
/// Source: [spec-wire-format.md Lines 519-544](../../docs/resources/spec-wire-format.md)
/// (Aggregate Public Key).
///
/// # Cross-references
///
/// - Verification counterpart: [`verify_aggregate`]
/// - Circuit constraint: CIR-003 in [`crate::prover::circuit::ConsensusCircuit`]
/// - Validator signing: [`crate::validator::sign_checkpoint`]
pub fn aggregate_pubkeys(pubkeys: &[[u8; 48]]) -> Result<[u8; 48], AggregateError> {
    // Start with the identity element (point at infinity).
    // In an elliptic curve group, identity + P = P for any point P,
    // so this is the correct initial accumulator for summation.
    let mut sum = G1Projective::default();

    for (i, pk_bytes) in pubkeys.iter().enumerate() {
        // Deserialize from 48-byte compressed form (ZCash format).
        // Deserialization also validates that the point is on the G1 curve
        // and in the correct prime-order subgroup.
        let pk = G1Affine::deserialize_compressed(&pk_bytes[..])
            .map_err(|_| AggregateError::InvalidPubkey(i))?;

        // Accumulate in projective (Jacobian) coordinates. Projective addition
        // avoids a costly modular inversion per step that affine addition
        // would require. We convert back to affine only once at the end.
        sum += pk;
    }

    // Convert the projective sum back to affine form for serialization.
    // Affine form is (x, y) with z=1; projective is (X, Y, Z) where x=X/Z, y=Y/Z.
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
/// Returns `true` if pk₁ + pk₂ + ... + pkₖ = `expected_aggregate`.
///
/// Useful for pre-flight validation before proof generation: if this
/// returns `false`, the resulting Groth16 proof would fail verification.
///
/// Source: [spec-groth16-circuit.md Lines 277-323](../../docs/resources/spec-groth16-circuit.md)
/// (Constraint 2: Aggregate Consistency).
///
/// # Cross-references
///
/// - Aggregation: [`aggregate_pubkeys`]
/// - Checkpoint submission: [`crate::prover::generate_proof`]
pub fn verify_aggregate(pubkeys: &[[u8; 48]], expected_aggregate: &[u8; 48]) -> bool {
    match aggregate_pubkeys(pubkeys) {
        Ok(computed) => computed == *expected_aggregate,
        Err(_) => false,
    }
}

/// Deserialize a G1 point from compressed bytes (48 bytes, ZCash format).
///
/// Returns `None` if the bytes are not a valid, on-curve G1 point in the
/// correct prime-order subgroup. This includes subgroup membership checks
/// that reject points on the twist or in a cofactor subgroup.
///
/// # Cross-references
///
/// - Serialization counterpart: [`serialize_g1`]
/// - Wire format: [spec-wire-format.md Lines 30-70](../../docs/resources/spec-wire-format.md)
pub fn deserialize_g1(bytes: &[u8; 48]) -> Option<G1Affine> {
    G1Affine::deserialize_compressed(&bytes[..]).ok()
}

/// Serialize a G1 point to compressed bytes (48 bytes, ZCash format).
///
/// Compressed form stores only the x-coordinate plus a flag bit indicating
/// which of the two possible y-coordinates to use (the "larger" one).
///
/// # Cross-references
///
/// - Deserialization counterpart: [`deserialize_g1`]
pub fn serialize_g1(point: &G1Affine) -> [u8; 48] {
    let mut bytes = Vec::with_capacity(G1_COMPRESSED_SIZE);
    point
        .serialize_compressed(&mut bytes)
        .expect("G1 serialization should not fail");
    bytes.try_into().expect("G1 should serialize to 48 bytes")
}

/// Get the G1 identity point (point at infinity) as bytes.
///
/// The identity element is the neutral element of the elliptic curve group:
/// for any point P, identity + P = P. This is used for padding when the
/// actual signer count k < MAX_SIGNERS, so unused signer slots contribute
/// nothing to the aggregate sum.
///
/// Source: [spec-groth16-circuit.md Lines 277-323](../../docs/resources/spec-groth16-circuit.md)
/// (inactive signers mapped to identity).
pub fn g1_identity() -> [u8; 48] {
    serialize_g1(&G1Affine::zero())
}

/// Add two G1 points given as compressed bytes.
///
/// Computes the elliptic curve group operation: result = a + b.
/// Both inputs are deserialized, added in projective coordinates
/// for efficiency, then converted back to compressed affine form.
///
/// Returns `None` if either input is not a valid G1 point.
///
/// # Cross-references
///
/// - Batch version: [`aggregate_pubkeys`]
/// - Inverse operation: [`negate_g1`]
pub fn add_g1(a: &[u8; 48], b: &[u8; 48]) -> Option<[u8; 48]> {
    let a_point = deserialize_g1(a)?;
    let b_point = deserialize_g1(b)?;
    // Add in projective coordinates (avoids per-addition field inversion)
    let sum = G1Projective::from(a_point) + G1Projective::from(b_point);
    Some(serialize_g1(&G1Affine::from(sum)))
}

/// Negate a G1 point given as compressed bytes.
///
/// Returns -pk such that pk + (-pk) = identity (point at infinity).
/// On BLS12-381 G1, negation flips the y-coordinate: -(x, y) = (x, -y).
/// Returns `None` if the input is not a valid G1 point.
///
/// This is useful for removing a validator's contribution from an
/// aggregate key without recomputing the full sum from scratch.
///
/// # Cross-references
///
/// - Addition: [`add_g1`]
/// - Identity: [`g1_identity`]
pub fn negate_g1(pk: &[u8; 48]) -> Option<[u8; 48]> {
    let point = deserialize_g1(pk)?;
    // Negation on an elliptic curve: negate the y-coordinate mod p
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
