//! Proof generation.
//!
//! Generates Groth16 proofs using arkworks. The proof is 192 bytes:
//! A (G1, 48 bytes) + B (G2, 96 bytes) + C (G1, 48 bytes).
//!
//! See [spec-groth16-circuit.md](../../docs/resources/spec-groth16-circuit.md).

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
pub fn generate_proof(
    circuit: ConsensusCircuit,
    proving_key: &ProvingKey<Bls12_381>,
) -> ConsensusResult<Vec<u8>> {
    let mut rng = StdRng::seed_from_u64(1337); // Deterministic for reproducibility

    let proof =
        Groth16::<Bls12_381>::create_random_proof_with_reduction(circuit, proving_key, &mut rng)
            .map_err(|e| ConsensusError::ProvingError(format!("Proof generation failed: {}", e)))?;

    serialize_proof(&proof)
}

/// Serialize a Groth16 proof to compressed bytes.
/// Returns: A(48) + B(96) + C(48) = 192 bytes.
///
/// Note: arkworks BLS12-381 `serialize_compressed` already produces
/// ZCash/Chia-compatible format (big-endian, flags in first byte).
pub fn serialize_proof(proof: &Proof<Bls12_381>) -> ConsensusResult<Vec<u8>> {
    let mut a_bytes = Vec::new();
    proof
        .a
        .serialize_compressed(&mut a_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let mut b_bytes = Vec::new();
    proof
        .b
        .serialize_compressed(&mut b_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let mut c_bytes = Vec::new();
    proof
        .c
        .serialize_compressed(&mut c_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let mut result = Vec::with_capacity(192);
    result.extend_from_slice(&a_bytes);
    result.extend_from_slice(&b_bytes);
    result.extend_from_slice(&c_bytes);
    Ok(result)
}
