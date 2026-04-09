//! CLVM serialization for proofs and verification keys.
//!
//! See [spec-wire-format.md](../../docs/resources/spec-wire-format.md).

use sha2::{Digest, Sha256};

// ============================================================================
// WIRE-002: Point Encoding Constants
// ============================================================================

/// G1 compressed point size in bytes (ZCash compressed BLS12-381).
///
/// G1 points represent public keys and appear in:
/// - `proof.a` and `proof.c` in Groth16 proofs
/// - IC points in the verification key
/// - Aggregate public key (`agg_signers`)
/// - Individual validator pubkeys
///
/// Source: spec-wire-format.md Lines 46-118
pub const G1_COMPRESSED_SIZE: usize = 48;

/// G2 compressed point size in bytes (ZCash compressed BLS12-381).
///
/// G2 points represent signatures and appear in:
/// - `proof.b` in Groth16 proofs
/// - `beta_g2`, `gamma_g2`, `delta_g2` in the verification key
/// - Aggregate signature (`agg_sig`)
///
/// Source: spec-wire-format.md Lines 46-118
pub const G2_COMPRESSED_SIZE: usize = 96;

/// Compute the checkpoint message that validators sign.
///
/// The checkpoint message commits to all new state including the new validator set.
/// This message is the critical link between the ZK proof and BLS signature verification.
///
/// Format: `sha256(new_state_root || new_validator_merkle_root || new_validator_count_be8 || new_epoch_be8)`
///
/// Source: spec-wire-format.md Lines 403-463
pub fn compute_checkpoint_message(
    new_state_root: [u8; 32],
    new_validator_merkle_root: [u8; 32],
    new_validator_count: u64,
    new_epoch: u64,
) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Field order is critical - must match Rue implementation exactly
    hasher.update(new_state_root); // 32 bytes
    hasher.update(new_validator_merkle_root); // 32 bytes
    hasher.update(new_validator_count.to_be_bytes()); // 8 bytes, big-endian
    hasher.update(new_epoch.to_be_bytes()); // 8 bytes, big-endian

    // Total: 80 bytes input, 32 bytes output
    hasher.finalize().into()
}

/// A Groth16 proof serialized for CLVM consumption.
#[derive(Debug, Clone)]
pub struct ClvmProof {
    /// Proof bytes in CLVM format.
    pub bytes: Vec<u8>,
}

/// A verification key serialized for CLVM consumption.
#[derive(Debug, Clone)]
pub struct ClvmVerificationKey {
    /// VK bytes in CLVM format.
    pub bytes: Vec<u8>,
}
