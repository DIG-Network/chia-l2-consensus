//! BLS12-381 key generation and signing for validators.
//!
//! VAL-001: Validators must generate a BLS12-381 keypair.
//!
//! Uses the `blst` crate for BLS operations. Public keys are 48-byte
//! compressed G1 points in ZCash format. Signatures are 96-byte compressed
//! G2 points.
//!
//! See [spec-validator-onboarding.md](../../docs/resources/spec-validator-onboarding.md) — Steps 1-2.

use crate::error::{ConsensusError, ConsensusResult};

/// BLS augmented scheme DST for Chia.
const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";

/// Minimum entropy length for key generation.
const MIN_ENTROPY_LEN: usize = 32;

/// A validator's BLS12-381 keypair.
///
/// - `pubkey`: 48-byte compressed G1 point (ZCash format)
/// - `secret_key`: 32-byte scalar
#[derive(Debug, Clone)]
pub struct ValidatorKeypair {
    pub pubkey: [u8; 48],
    pub secret_key: [u8; 32],
}

/// Generate a BLS12-381 validator keypair from entropy.
///
/// The entropy (IKM — input keying material) must be at least 32 bytes.
/// The same entropy always produces the same keypair (deterministic).
///
/// See spec-validator-onboarding.md — Step 2: Generate BLS keypair.
pub fn generate_validator_keypair(entropy: &[u8]) -> ConsensusResult<ValidatorKeypair> {
    if entropy.len() < MIN_ENTROPY_LEN {
        return Err(ConsensusError::ProvingError(format!(
            "Entropy must be at least {} bytes, got {}",
            MIN_ENTROPY_LEN,
            entropy.len()
        )));
    }

    let sk = blst::min_pk::SecretKey::key_gen(entropy, &[])
        .map_err(|e| ConsensusError::ProvingError(format!("BLS key generation failed: {:?}", e)))?;

    let pk = sk.sk_to_pk();

    Ok(ValidatorKeypair {
        pubkey: pk.compress(),
        secret_key: sk.to_bytes(),
    })
}

/// Derive the public key from a secret key.
///
/// Returns the 48-byte compressed G1 point.
pub fn pubkey_from_secret(secret_key: &[u8; 32]) -> ConsensusResult<[u8; 48]> {
    let sk = blst::min_pk::SecretKey::from_bytes(secret_key)
        .map_err(|e| ConsensusError::ProvingError(format!("Invalid secret key: {:?}", e)))?;

    Ok(sk.sk_to_pk().compress())
}

/// Sign a message with a validator's secret key.
///
/// Uses the BLS augmented scheme (Chia's DST). Returns a 96-byte
/// compressed G2 signature.
///
/// See spec-wire-format.md — Individual Signatures.
pub fn sign_message(secret_key: &[u8; 32], message: &[u8]) -> ConsensusResult<[u8; 96]> {
    let sk = blst::min_pk::SecretKey::from_bytes(secret_key)
        .map_err(|e| ConsensusError::ProvingError(format!("Invalid secret key: {:?}", e)))?;

    let pk = sk.sk_to_pk();
    let sig = sk.sign(message, DST, &pk.compress());

    Ok(sig.compress())
}

/// Verify a BLS signature against a public key and message.
///
/// Uses the BLS augmented scheme (Chia's DST).
///
/// Returns `Ok(true)` if valid, `Ok(false)` if invalid signature,
/// or `Err` if the inputs are malformed.
pub fn verify_signature(
    pubkey: &[u8; 48],
    message: &[u8],
    signature: &[u8; 96],
) -> ConsensusResult<bool> {
    let pk = blst::min_pk::PublicKey::uncompress(pubkey)
        .map_err(|e| ConsensusError::ProvingError(format!("Invalid public key: {:?}", e)))?;

    let sig = blst::min_pk::Signature::uncompress(signature)
        .map_err(|e| ConsensusError::ProvingError(format!("Invalid signature: {:?}", e)))?;

    let result = sig.verify(true, message, DST, pubkey, &pk, true);

    Ok(result == blst::BLST_ERROR::BLST_SUCCESS)
}
