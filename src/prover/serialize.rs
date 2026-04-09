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

// ============================================================================
// WIRE-003: Groth16 Proof Format
// ============================================================================

/// Total Groth16 proof size in bytes: A (48) + B (96) + C (48) = 192.
///
/// The proof consists of exactly three curve points. This constant-size proof
/// is verified on-chain regardless of validator set size.
///
/// Source: spec-wire-format.md Lines 122-183
pub const GROTH16_PROOF_SIZE: usize = G1_COMPRESSED_SIZE + G2_COMPRESSED_SIZE + G1_COMPRESSED_SIZE;

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
///
/// In the checkpoint singleton solution, the proof is passed as three separate atoms:
/// `(proof_a proof_b proof_c ...)`
///
/// Where `a` and `c` are 48-byte G1 atoms and `b` is a 96-byte G2 atom.
/// CLVM passes these directly to `bls_pairing_identity` as G1 and G2 arguments.
///
/// Source: spec-wire-format.md Lines 122-183
#[derive(Debug, Clone)]
pub struct ClvmProof {
    /// First proof element (G1 compressed, 48 bytes).
    pub a: Vec<u8>,
    /// Second proof element (G2 compressed, 96 bytes).
    pub b: Vec<u8>,
    /// Third proof element (G1 compressed, 48 bytes).
    pub c: Vec<u8>,
}

impl ClvmProof {
    /// Total size of the proof when concatenated (192 bytes).
    pub fn total_size(&self) -> usize {
        self.a.len() + self.b.len() + self.c.len()
    }

    /// Concatenate A || B || C into a single byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(GROTH16_PROOF_SIZE);
        bytes.extend_from_slice(&self.a);
        bytes.extend_from_slice(&self.b);
        bytes.extend_from_slice(&self.c);
        bytes
    }
}

/// A verification key serialized for CLVM consumption.
#[derive(Debug, Clone)]
pub struct ClvmVerificationKey {
    /// VK bytes in CLVM format.
    pub bytes: Vec<u8>,
}

// ============================================================================
// WIRE-004: Membership Announcement Format
// ============================================================================

/// Membership announcement prefix (10 bytes UTF-8, no null terminator).
pub const MEMBERSHIP_PREFIX: &[u8] = b"membership";

/// Membership announcement input size (67 bytes).
/// "membership" (10) + epoch (8) + pubkey (48) + is_member (1)
pub const MEMBERSHIP_INPUT_SIZE: usize = 10 + 8 + 48 + 1;

/// Compute the membership announcement message.
///
/// Membership announcements are emitted by the checkpoint singleton during
/// membership query spends. They enable validators to prove their membership
/// status for collateral recovery.
///
/// Format: `sha256("membership" || epoch_be8 || pubkey || is_member_byte)`
/// where is_member_byte is 0x01 for member, 0x00 for non-member.
///
/// Source: spec-wire-format.md Lines 548-597
pub fn compute_membership_announcement_message(
    epoch: u64,
    pubkey: &[u8; 48],
    is_member: bool,
) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Field order is critical - must match Rue implementation exactly
    hasher.update(MEMBERSHIP_PREFIX); // 10 bytes
    hasher.update(epoch.to_be_bytes()); // 8 bytes, big-endian
    hasher.update(pubkey); // 48 bytes
    hasher.update([if is_member { 0x01 } else { 0x00 }]); // 1 byte

    // Total: 67 bytes input, 32 bytes output
    hasher.finalize().into()
}
