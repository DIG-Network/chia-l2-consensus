//! Trusted setup operations.
//!
//! Generates Groth16 proving and verification keys from the circuit.
//! For production: multi-party computation ceremony.
//! For testing: single-party setup (insecure but functional).
//!
//! See [spec-trusted-setup.md](../../docs/resources/spec-trusted-setup.md).

use ark_bls12_381::Bls12_381;
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;
use sha2::{Digest, Sha256};

use crate::error::{ConsensusError, ConsensusResult};
use crate::prover::circuit::{ConsensusCircuit, PUBLIC_INPUT_COUNT};

/// Run a single-party trusted setup for testing.
/// Returns (serialized_proving_key, serialized_verification_key).
///
/// WARNING: Single-party setup is NOT secure for production. Use MPC ceremony.
pub fn run_test_setup() -> ConsensusResult<(Vec<u8>, Vec<u8>)> {
    let circuit = ConsensusCircuit::new();
    let mut rng = StdRng::seed_from_u64(42); // Deterministic for tests

    let params = Groth16::<Bls12_381>::generate_random_parameters_with_reduction(circuit, &mut rng)
        .map_err(|e| ConsensusError::ProvingError(format!("Setup failed: {}", e)))?;

    let mut pk_bytes = Vec::new();
    params
        .serialize_compressed(&mut pk_bytes)
        .map_err(|e| ConsensusError::SerializationError(format!("PK serialize: {}", e)))?;

    let mut vk_bytes = Vec::new();
    params
        .vk
        .serialize_compressed(&mut vk_bytes)
        .map_err(|e| ConsensusError::SerializationError(format!("VK serialize: {}", e)))?;

    Ok((pk_bytes, vk_bytes))
}

/// Deserialize a proving key from bytes.
pub fn deserialize_proving_key(bytes: &[u8]) -> ConsensusResult<ProvingKey<Bls12_381>> {
    ProvingKey::deserialize_compressed(bytes)
        .map_err(|e| ConsensusError::SerializationError(format!("PK deserialize: {}", e)))
}

/// Deserialize a verification key from bytes.
pub fn deserialize_verification_key(bytes: &[u8]) -> ConsensusResult<VerifyingKey<Bls12_381>> {
    VerifyingKey::deserialize_compressed(bytes)
        .map_err(|e| ConsensusError::SerializationError(format!("VK deserialize: {}", e)))
}

/// Extract the VK IC points as compressed G1 bytes for CLVM.
/// Returns: (alpha_g1, beta_g2, gamma_g2, delta_g2, ic_points)
///
/// Note: arkworks BLS12-381 `serialize_compressed` already produces
/// ZCash/Chia-compatible format (big-endian, flags in first byte).
pub fn extract_vk_components(vk: &VerifyingKey<Bls12_381>) -> ConsensusResult<VkComponents> {
    let mut alpha = Vec::new();
    vk.alpha_g1
        .serialize_compressed(&mut alpha)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let mut beta = Vec::new();
    vk.beta_g2
        .serialize_compressed(&mut beta)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let mut gamma = Vec::new();
    vk.gamma_g2
        .serialize_compressed(&mut gamma)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let mut delta = Vec::new();
    vk.delta_g2
        .serialize_compressed(&mut delta)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let mut ic_points = Vec::new();
    for ic in &vk.gamma_abc_g1 {
        let mut ic_bytes = Vec::new();
        ic.serialize_compressed(&mut ic_bytes)
            .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
        ic_points.push(ic_bytes);
    }

    Ok(VkComponents {
        alpha_g1: alpha,
        beta_g2: beta,
        gamma_g2: gamma,
        delta_g2: delta,
        ic_points,
    })
}

/// Extracted VK components for CLVM integration.
pub struct VkComponents {
    pub alpha_g1: Vec<u8>,       // 48 bytes G1 compressed
    pub beta_g2: Vec<u8>,        // 96 bytes G2 compressed
    pub gamma_g2: Vec<u8>,       // 96 bytes G2 compressed
    pub delta_g2: Vec<u8>,       // 96 bytes G2 compressed
    pub ic_points: Vec<Vec<u8>>, // Each 48 bytes G1 compressed
}

/// Expected total VK size in bytes.
/// alpha_g1(48) + beta_g2(96) + gamma_g2(96) + delta_g2(96) + 7*ic(48) = 672.
pub const VK_BYTE_SIZE: usize = 48 + 96 + 96 + 96 + 7 * 48;

/// Serialize a verification key to a flat byte vector.
///
/// Layout: alpha_g1(48) || beta_g2(96) || gamma_g2(96) || delta_g2(96) || ic[0..7](7×48)
/// Total: 672 bytes.
///
/// This is the canonical format for currying into the checkpoint singleton
/// and for computing the VK hash for publication.
///
/// See spec-wire-format.md — Verification Key Format.
pub fn vk_to_bytes(vk: &VerifyingKey<Bls12_381>) -> ConsensusResult<Vec<u8>> {
    let components = extract_vk_components(vk)?;
    let mut bytes = Vec::with_capacity(VK_BYTE_SIZE);
    bytes.extend_from_slice(&components.alpha_g1);
    bytes.extend_from_slice(&components.beta_g2);
    bytes.extend_from_slice(&components.gamma_g2);
    bytes.extend_from_slice(&components.delta_g2);
    for ic in &components.ic_points {
        bytes.extend_from_slice(ic);
    }
    Ok(bytes)
}

/// Compute the SHA-256 hash of the serialized verification key.
///
/// This hash is published as a deployment artifact so that anyone can verify
/// the on-chain checkpoint singleton contains the expected VK.
///
/// See spec-trusted-setup.md — Verifying the Output, step 5.
pub fn compute_vk_hash(vk: &VerifyingKey<Bls12_381>) -> ConsensusResult<[u8; 32]> {
    let bytes = vk_to_bytes(vk)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hasher.finalize().into())
}

/// Validate that a verification key has the expected structure.
///
/// Checks:
/// - Exactly 7 IC points (6 public inputs + 1 constant term)
/// - All components serialize to the correct sizes
///
/// This should be called after the trusted setup ceremony and before deployment.
///
/// See spec-trusted-setup.md — Verifying the Output, step 2.
pub fn validate_vk(vk: &VerifyingKey<Bls12_381>) -> ConsensusResult<()> {
    let expected_ic = PUBLIC_INPUT_COUNT + 1; // 6 + 1 = 7
    if vk.gamma_abc_g1.len() != expected_ic {
        return Err(ConsensusError::SerializationError(format!(
            "VK has {} IC points, expected {} (6 public inputs + 1 constant)",
            vk.gamma_abc_g1.len(),
            expected_ic
        )));
    }

    let components = extract_vk_components(vk)?;

    if components.alpha_g1.len() != 48 {
        return Err(ConsensusError::SerializationError(format!(
            "alpha_g1 is {} bytes, expected 48",
            components.alpha_g1.len()
        )));
    }
    if components.beta_g2.len() != 96 {
        return Err(ConsensusError::SerializationError(format!(
            "beta_g2 is {} bytes, expected 96",
            components.beta_g2.len()
        )));
    }
    if components.gamma_g2.len() != 96 {
        return Err(ConsensusError::SerializationError(format!(
            "gamma_g2 is {} bytes, expected 96",
            components.gamma_g2.len()
        )));
    }
    if components.delta_g2.len() != 96 {
        return Err(ConsensusError::SerializationError(format!(
            "delta_g2 is {} bytes, expected 96",
            components.delta_g2.len()
        )));
    }

    let total = vk_to_bytes(vk)?;
    if total.len() != VK_BYTE_SIZE {
        return Err(ConsensusError::SerializationError(format!(
            "VK total is {} bytes, expected {}",
            total.len(),
            VK_BYTE_SIZE
        )));
    }

    Ok(())
}

// ============================================================================
// DEP-004: Post-deployment VK verification (byte-level)
// ============================================================================

/// Verify that a serialized VK matches a published hash.
///
/// This is the primary post-deployment check: the operator extracts the
/// VK bytes from the deployed checkpoint singleton and compares the
/// SHA-256 hash against the hash published during the ceremony.
///
/// See spec-trusted-setup.md — Verifying the Output, step 5.
pub fn verify_vk_hash(vk_bytes: &[u8], expected_hash: &[u8; 32]) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(vk_bytes);
    let actual: [u8; 32] = hasher.finalize().into();
    actual == *expected_hash
}

/// Validate a serialized VK has the correct byte-level structure.
///
/// Checks:
/// - Total length is exactly 672 bytes
/// - Layout: alpha_g1(48) + beta_g2(96) + gamma_g2(96) + delta_g2(96) + 7×ic(48)
///
/// This works on raw bytes as extracted from a deployed singleton,
/// without needing arkworks deserialization.
///
/// See spec-wire-format.md — Verification Key Format.
pub fn validate_vk_bytes(vk_bytes: &[u8]) -> ConsensusResult<()> {
    if vk_bytes.len() != VK_BYTE_SIZE {
        return Err(ConsensusError::SerializationError(format!(
            "VK is {} bytes, expected {} (48+96+96+96+7×48)",
            vk_bytes.len(),
            VK_BYTE_SIZE
        )));
    }

    // Verify the IC section: 7 points × 48 bytes = 336 bytes
    // Starting at offset 48+96+96+96 = 336
    let ic_offset = 48 + 96 + 96 + 96;
    let ic_bytes = &vk_bytes[ic_offset..];
    let ic_count = ic_bytes.len() / 48;
    if ic_count != PUBLIC_INPUT_COUNT + 1 {
        return Err(ConsensusError::SerializationError(format!(
            "VK has {} IC points, expected {}",
            ic_count,
            PUBLIC_INPUT_COUNT + 1
        )));
    }

    Ok(())
}

/// Extract VK components from raw serialized bytes.
///
/// Splits the 672-byte VK into its constituent parts without arkworks
/// deserialization. Useful for comparing against VK components extracted
/// from an arkworks `VerifyingKey`.
///
/// Layout: alpha_g1(48) || beta_g2(96) || gamma_g2(96) || delta_g2(96) || ic[0..7](7×48)
pub fn extract_vk_components_from_bytes(vk_bytes: &[u8]) -> ConsensusResult<VkComponents> {
    validate_vk_bytes(vk_bytes)?;

    let alpha_g1 = vk_bytes[0..48].to_vec();
    let beta_g2 = vk_bytes[48..144].to_vec();
    let gamma_g2 = vk_bytes[144..240].to_vec();
    let delta_g2 = vk_bytes[240..336].to_vec();

    let mut ic_points = Vec::with_capacity(7);
    for i in 0..7 {
        let start = 336 + i * 48;
        ic_points.push(vk_bytes[start..start + 48].to_vec());
    }

    Ok(VkComponents {
        alpha_g1,
        beta_g2,
        gamma_g2,
        delta_g2,
        ic_points,
    })
}
