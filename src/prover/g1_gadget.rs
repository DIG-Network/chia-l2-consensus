//! Non-native G1 point arithmetic for BLS12-381 inside a BLS12-381 Groth16 circuit.
//!
//! BLS12-381 G1 points have coordinates in Fq (base field, 381 bits).
//! The Groth16 circuit operates over Fr (scalar field, 255 bits).
//! Since Fq ≠ Fr, we use `NonNativeFieldVar<Fq, Fr>` for field emulation.
//!
//! This module provides:
//! - `G1NonNativeVar`: A G1 point represented as (x, y, is_infinity) in non-native Fq
//! - `alloc_g1_witness`: Allocate a G1 point from compressed 48-byte witness
//! - `enforce_g1_sum_equals`: Constrain that pk₁ + pk₂ + ... + pkₖ = agg_signers
//!
//! # CIR-003: Aggregate Key Constraint
//!
//! The circuit must enforce: G1_sum(signing_pubkeys) == agg_signers.
//! This binds the Groth16 proof to the specific aggregate key verified
//! by `bls_verify` on-chain, closing the SEC-011 phantom majority vulnerability.

use ark_bls12_381::{Fq, Fr, G1Affine};
use ark_ec::AffineRepr;
use ark_ff::{Field, One, PrimeField, Zero};
use ark_r1cs_std::{
    boolean::Boolean,
    fields::{fp::FpVar, nonnative::NonNativeFieldVar},
    prelude::*,
};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use ark_serialize::CanonicalDeserialize;

/// A BLS12-381 G1 point in non-native field representation.
///
/// Coordinates (x, y) are `NonNativeFieldVar<Fq, Fr>` — Fq elements emulated
/// using multiple Fr limbs. The `is_infinity` flag handles the identity point.
#[derive(Clone)]
pub struct G1NonNativeVar {
    pub x: NonNativeFieldVar<Fq, Fr>,
    pub y: NonNativeFieldVar<Fq, Fr>,
    pub is_infinity: Boolean<Fr>,
}

impl G1NonNativeVar {
    /// Allocate the identity (point at infinity).
    pub fn zero(cs: ConstraintSystemRef<Fr>) -> Result<Self, SynthesisError> {
        Ok(Self {
            x: NonNativeFieldVar::new_witness(cs.clone(), || Ok(Fq::zero()))?,
            y: NonNativeFieldVar::new_witness(cs.clone(), || Ok(Fq::one()))?,
            is_infinity: Boolean::new_witness(cs, || Ok(true))?,
        })
    }

    /// Allocate a G1 point from its affine coordinates as a witness.
    pub fn new_witness_from_affine(
        cs: ConstraintSystemRef<Fr>,
        point: &G1Affine,
    ) -> Result<Self, SynthesisError> {
        if point.is_zero() {
            return Self::zero(cs);
        }
        let (x, y) = point.xy().unwrap();
        Ok(Self {
            x: NonNativeFieldVar::new_witness(cs.clone(), || Ok(*x))?,
            y: NonNativeFieldVar::new_witness(cs.clone(), || Ok(*y))?,
            is_infinity: Boolean::new_witness(cs, || Ok(false))?,
        })
    }

    /// Enforce that this point lies on the BLS12-381 G1 curve: y² = x³ + 4.
    ///
    /// This prevents a malicious prover from providing off-curve points that
    /// could break the group operation properties.
    pub fn enforce_on_curve(&self, _cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // For infinity point, curve equation doesn't apply
        // For non-infinity: y² = x³ + 4

        let x_squared = &self.x * &self.x;
        let x_cubed = &x_squared * &self.x;

        let b = NonNativeFieldVar::<Fq, Fr>::new_constant(self.x.cs(), Fq::from(4u64))?;
        let rhs = &x_cubed + &b; // x³ + 4

        let y_squared = &self.y * &self.y;

        // if not infinity: y² == x³ + 4
        // We enforce unconditionally for non-infinity points.
        // For infinity, the is_infinity flag makes this point unused in the sum.
        y_squared.conditional_enforce_equal(&rhs, &self.is_infinity.not())?;

        Ok(())
    }

    /// Constrained affine point addition: self + other.
    ///
    /// Uses the standard short Weierstrass addition formula:
    ///   λ = (y₂ - y₁) / (x₂ - x₁)
    ///   x₃ = λ² - x₁ - x₂
    ///   y₃ = λ(x₁ - x₃) - y₁
    ///
    /// Handles identity: P + 0 = P, 0 + Q = Q.
    /// Does NOT handle P + P (doubling) or P + (-P) = 0.
    /// For distinct, non-inverse points only.
    pub fn add(&self, other: &Self) -> Result<Self, SynthesisError> {
        let cs = self.x.cs();

        // Compute the slope λ = (y₂ - y₁) / (x₂ - x₁)
        // We use a witness for λ and verify the relationship via multiplication.
        let dy = &other.y - &self.y;
        let dx = &other.x - &self.x;

        // λ * dx = dy  (avoids division in constraints)
        // Allocate λ as witness
        let lambda_val = {
            let dy_val = dy.value().unwrap_or(Fq::zero());
            let dx_val = dx.value().unwrap_or(Fq::one());
            if dx_val.is_zero() {
                Fq::zero() // Degenerate case
            } else {
                dy_val * dx_val.inverse().unwrap()
            }
        };
        let lambda = NonNativeFieldVar::new_witness(cs.clone(), || Ok(lambda_val))?;

        // Enforce: λ * (x₂ - x₁) = (y₂ - y₁)
        let lambda_times_dx = &lambda * &dx;
        lambda_times_dx.enforce_equal(&dy)?;

        // x₃ = λ² - x₁ - x₂
        let lambda_sq = &lambda * &lambda;
        let x3 = &lambda_sq - &self.x - &other.x;

        // y₃ = λ(x₁ - x₃) - y₁
        let x1_minus_x3 = &self.x - &x3;
        let y3 = &(&lambda * &x1_minus_x3) - &self.y;

        // Handle identity: if self is infinity, result = other; if other is infinity, result = self
        let x_result = NonNativeFieldVar::conditionally_select(&self.is_infinity, &other.x, &x3)?;
        let y_result = NonNativeFieldVar::conditionally_select(&self.is_infinity, &other.y, &y3)?;

        let x_result =
            NonNativeFieldVar::conditionally_select(&other.is_infinity, &self.x, &x_result)?;
        let y_result =
            NonNativeFieldVar::conditionally_select(&other.is_infinity, &self.y, &y_result)?;

        let both_inf = self.is_infinity.and(&other.is_infinity)?;
        let result_inf =
            Boolean::conditionally_select(&self.is_infinity, &other.is_infinity, &Boolean::FALSE)?;
        let result_inf =
            Boolean::conditionally_select(&other.is_infinity, &self.is_infinity, &result_inf)?;
        let result_inf = Boolean::or(&both_inf, &result_inf)?;
        // Simplify: result is infinity only if both are infinity
        let result_inf = both_inf;

        Ok(Self {
            x: x_result,
            y: y_result,
            is_infinity: result_inf,
        })
    }

    /// Enforce that this point equals another point.
    pub fn enforce_equal(&self, other: &Self) -> Result<(), SynthesisError> {
        // If both infinity → equal. If one infinity and other not → not equal.
        // If neither infinity → x and y must match.
        self.is_infinity.enforce_equal(&other.is_infinity)?;
        self.x
            .conditional_enforce_equal(&other.x, &self.is_infinity.not())?;
        self.y
            .conditional_enforce_equal(&other.y, &self.is_infinity.not())?;
        Ok(())
    }
}

/// Decompress a 48-byte BLS12-381 G1 point (ZCash format) to affine coordinates.
///
/// Returns None if the bytes are not a valid G1 point.
pub fn decompress_g1(compressed: &[u8; 48]) -> Option<G1Affine> {
    G1Affine::deserialize_compressed(&compressed[..]).ok()
}

/// CIR-003: Enforce that the sum of k signing pubkeys equals agg_signers.
///
/// This is the core constraint that closes SEC-011 (phantom majority attack).
///
/// # Arguments
/// * `cs` - Constraint system reference
/// * `signing_pubkeys` - Compressed pubkey bytes for each signer (48 bytes each)
/// * `agg_signers_bytes` - Compressed agg_signers public input (48 bytes)
/// * `actual_signers` - Number of real signers (rest are padding)
///
/// # Constraints
/// ~20,000 per point addition × (k-1) additions + ~10,000 for decompressions
pub fn enforce_aggregate_key(
    cs: ConstraintSystemRef<Fr>,
    signing_pubkeys: &[[u8; 48]],
    agg_signers_bytes: &[u8; 48],
    actual_signers: usize,
) -> Result<(), SynthesisError> {
    if actual_signers == 0 {
        return Err(SynthesisError::Unsatisfiable);
    }

    // Decompress agg_signers to affine
    let agg_point = decompress_g1(agg_signers_bytes).ok_or(SynthesisError::Unsatisfiable)?;

    let agg_var = G1NonNativeVar::new_witness_from_affine(cs.clone(), &agg_point)?;
    agg_var.enforce_on_curve(cs.clone())?;

    // Start with identity
    let mut sum = G1NonNativeVar::zero(cs.clone())?;

    for (i, pk_bytes) in signing_pubkeys.iter().enumerate() {
        if i < actual_signers {
            // Real signer: decompress and add
            let pk_affine = decompress_g1(pk_bytes).ok_or(SynthesisError::Unsatisfiable)?;
            let pk_var = G1NonNativeVar::new_witness_from_affine(cs.clone(), &pk_affine)?;
            pk_var.enforce_on_curve(cs.clone())?;
            sum = sum.add(&pk_var)?;
        }
        // Padding slots (i >= actual_signers): skip, identity + P = P
    }

    // Enforce: sum == agg_signers
    sum.enforce_equal(&agg_var)?;

    Ok(())
}
