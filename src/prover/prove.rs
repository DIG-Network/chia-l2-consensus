//! Proof generation.
//!
//! Generates Groth16 proofs using arkworks. The proof is 192 bytes:
//! A (G1, 48 bytes) + B (G2, 96 bytes) + C (G1, 48 bytes).
//!
//! The proof attests that a majority of registered validators signed the
//! checkpoint, without revealing which validators signed. The on-chain
//! checkpoint singleton verifies this proof via `bls_pairing_identity`.
//!
//! Source: [spec-groth16-circuit.md Lines 560-581](../../docs/resources/spec-groth16-circuit.md)
//! (Proof Generation section).
//! Wire format: [spec-wire-format.md Lines 122-183](../../docs/resources/spec-wire-format.md)
//! (Groth16 Proof Format).

use ark_bls12_381::Bls12_381;
use ark_groth16::{Groth16, Proof, ProvingKey};
use ark_serialize::CanonicalSerialize;
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;

use crate::error::{ConsensusError, ConsensusResult};
use crate::prover::circuit::ConsensusCircuit;

/// Generate a Groth16 proof for the given circuit and proving key.
///
/// Returns the proof as compressed bytes: A(48) + B(96) + C(48) = 192 bytes.
///
/// The `circuit` must have its public inputs and private witnesses populated
/// by the caller (typically [`ConsensusCircuit`] built from checkpoint data).
/// The `proving_key` is produced by [`crate::prover::setup::run_test_setup`]
/// or loaded from the trusted setup ceremony output via
/// [`crate::prover::setup::deserialize_proving_key`].
///
/// Source: [spec-groth16-circuit.md Lines 560-581](../../docs/resources/spec-groth16-circuit.md)
/// (Proof Generation).
///
/// # Cross-references
///
/// - Circuit definition: [`crate::prover::circuit::ConsensusCircuit`]
/// - Trusted setup: [`crate::prover::setup::run_test_setup`]
/// - Serialization format: [`serialize_proof`] and
///   [spec-wire-format.md Lines 122-183](../../docs/resources/spec-wire-format.md)
pub fn generate_proof(
    circuit: ConsensusCircuit,
    proving_key: &ProvingKey<Bls12_381>,
) -> ConsensusResult<Vec<u8>> {
    // Deterministic RNG for reproducible proofs (same circuit + key = same proof).
    // Production deployments may use a non-deterministic RNG for added blinding.
    let mut rng = StdRng::seed_from_u64(1337);

    // Groth16 proof generation:
    //   1. Evaluate the circuit to produce the R1CS assignment (witness + public inputs)
    //   2. Compute A, B, C group elements via multi-scalar multiplication (MSM)
    //   3. The resulting (A, B, C) satisfy the pairing equation:
    //      e(A, B) = e(alpha, beta) * e(vk_input, gamma) * e(C, delta)
    let proof =
        Groth16::<Bls12_381>::create_random_proof_with_reduction(circuit, proving_key, &mut rng)
            .map_err(|e| ConsensusError::ProvingError(format!("Proof generation failed: {}", e)))?;

    serialize_proof(&proof)
}

/// Serialize a Groth16 proof to compressed bytes.
///
/// Returns the concatenation A(48) + B(96) + C(48) = 192 bytes.
///
/// The compressed format uses the ZCash serialization convention where
/// the top two bits of the first byte encode flags:
///   - bit 7: always 1 (indicates compressed form)
///   - bit 6: 1 if the point is the identity (point at infinity)
///   - bit 5: selects the larger y-coordinate (for G1) or y0 (for G2)
///
/// arkworks `serialize_compressed` already produces this ZCash/Chia-compatible
/// format, so no manual conversion is needed (see memory note:
/// "Ark BLS12-381 = ZCash format").
///
/// Source: [spec-wire-format.md Lines 122-183](../../docs/resources/spec-wire-format.md)
/// (Groth16 Proof Format).
///
/// # Cross-references
///
/// - Proof generation: [`generate_proof`]
/// - CLVM proof struct: [`crate::prover::ClvmProof`]
/// - On-chain verification: checkpoint_inner.rue `bls_pairing_identity`
pub fn serialize_proof(proof: &Proof<Bls12_381>) -> ConsensusResult<Vec<u8>> {
    // A is a G1 point (48 bytes compressed)
    let mut a_bytes = Vec::new();
    proof
        .a
        .serialize_compressed(&mut a_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    // B is a G2 point (96 bytes compressed: two Fq elements for the extension field)
    let mut b_bytes = Vec::new();
    proof
        .b
        .serialize_compressed(&mut b_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    // C is a G1 point (48 bytes compressed)
    let mut c_bytes = Vec::new();
    proof
        .c
        .serialize_compressed(&mut c_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    // Concatenate in order: A || B || C (192 bytes total)
    let mut result = Vec::with_capacity(192);
    result.extend_from_slice(&a_bytes);
    result.extend_from_slice(&b_bytes);
    result.extend_from_slice(&c_bytes);
    Ok(result)
}
