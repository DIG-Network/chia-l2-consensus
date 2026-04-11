//! CLVM serialization for proofs and verification keys.
//!
//! See [spec-wire-format.md](../../docs/resources/spec-wire-format.md).

use ark_bls12_381::Fr;
use ark_ff::PrimeField;
use num_bigint::BigUint;
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

// ============================================================================
// WIRE-005: Registration Message Format
// ============================================================================

/// Registration message prefix (8 bytes UTF-8, no null terminator).
pub const REGISTER_PREFIX: &[u8] = b"register";

/// Registration message input size (56 bytes).
/// "register" (8) + pubkey (48)
pub const REGISTRATION_INPUT_SIZE: usize = 8 + 48;

/// Compute the registration message that validators sign during registration.
///
/// The registration message proves ownership of the BLS key and prevents
/// unauthorized registrations on behalf of another validator.
///
/// Format: `sha256("register" || pubkey)` where "register" is 8-byte UTF-8.
///
/// Source: spec-wire-format.md Lines 620-639
pub fn compute_registration_message(pubkey: &[u8; 48]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Field order is critical - must match Rue implementation exactly
    hasher.update(REGISTER_PREFIX); // 8 bytes
    hasher.update(pubkey); // 48 bytes

    // Total: 56 bytes input, 32 bytes output
    hasher.finalize().into()
}

// ============================================================================
// WIRE-006: scalar() Function
// ============================================================================

// ============================================================================
// Ark → ZCash/Chia BLS12-381 Point Format Conversion
// ============================================================================

/// Convert an arkworks-compressed G1 point (48 bytes) to ZCash/Chia format.
///
/// Arkworks: little-endian x-coordinate, flags in MSB of last byte.
/// ZCash/Chia: big-endian x-coordinate, flags in MSB of first byte.
///
/// Flag mapping:
/// - ark byte[47] bit 7 → zcash byte[0] bit 5 (y-sign / "largest")
/// - ark byte[47] bit 6 → zcash byte[0] bit 6 (point at infinity)
/// - zcash byte[0] bit 7 = 1 (compressed flag, always set)
pub fn ark_g1_to_zcash(ark_bytes: &[u8]) -> [u8; 48] {
    assert_eq!(ark_bytes.len(), 48, "G1 compressed must be 48 bytes");
    let mut zcash = [0u8; 48];

    // Extract ark flags from last byte
    let ark_flags = ark_bytes[47];
    let y_largest = (ark_flags >> 7) & 1;
    let infinity = (ark_flags >> 6) & 1;

    // Reverse byte order (little-endian → big-endian)
    for i in 0..48 {
        zcash[i] = ark_bytes[47 - i];
    }

    // Clear flag bits from byte[0] (was byte[47] with ark flags)
    zcash[0] &= 0x1F;

    // Set ZCash flags in byte[0]
    zcash[0] |= 0x80; // bit 7: compressed (always 1)
    if infinity == 1 {
        zcash[0] |= 0x40; // bit 6: infinity
    }
    if y_largest == 1 {
        zcash[0] |= 0x20; // bit 5: y-sign (largest)
    }

    zcash
}

/// Convert an arkworks-compressed G2 point (96 bytes) to ZCash/Chia format.
///
/// Arkworks Fp2 = c0 + c1*u, serialized as: c0 (LE, 48 bytes) || c1 (LE, 48 bytes).
/// Flags in MSB of byte[95] (last byte of c1).
///
/// ZCash Fp2 serialized as: c1 (BE, 48 bytes) || c0 (BE, 48 bytes).
/// Flags in MSB of byte[0] (first byte of c1).
pub fn ark_g2_to_zcash(ark_bytes: &[u8]) -> [u8; 96] {
    assert_eq!(ark_bytes.len(), 96, "G2 compressed must be 96 bytes");
    let mut zcash = [0u8; 96];

    // Extract flags from ark byte[95] (last byte of c1)
    let ark_flags = ark_bytes[95];
    let y_largest = (ark_flags >> 7) & 1;
    let infinity = (ark_flags >> 6) & 1;

    // Reverse c1 (ark[48..96]) → zcash[0..48] (ZCash puts c1 first, big-endian)
    for i in 0..48 {
        zcash[i] = ark_bytes[95 - i];
    }

    // Reverse c0 (ark[0..48]) → zcash[48..96]
    for i in 0..48 {
        zcash[48 + i] = ark_bytes[47 - i];
    }

    // Clear flag bits from zcash[0] (was byte[95] with ark flags)
    zcash[0] &= 0x1F;

    // Set ZCash flags
    zcash[0] |= 0x80; // compressed
    if infinity == 1 {
        zcash[0] |= 0x40; // infinity
    }
    if y_largest == 1 {
        zcash[0] |= 0x20; // y-sign
    }

    zcash
}

/// Convert bytes to a BLS12-381 scalar field element.
///
/// The `scalar()` function converts public input values to BLS12-381 scalar
/// field elements. It is used in both off-chain proof generation and on-chain
/// Rue puzzle to compute the linear combination of IC points for Groth16
/// verification.
///
/// Formula: `scalar(bytes) = SHA-256(bytes) interpreted as 256-bit big-endian integer, mod r`
///
/// Where `r` is the BLS12-381 scalar field order:
/// `r = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001`
///
/// Source: spec-wire-format.md Lines 285-401
pub fn bytes_to_scalar(bytes: &[u8]) -> Fr {
    // Compute SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash: [u8; 32] = hasher.finalize().into();

    // CLVM's g1_multiply interprets scalar atoms as SIGNED big-endian integers.
    // A 32-byte atom with MSB set is treated as negative (two's complement).
    // We must match this convention so the circuit's public inputs produce
    // the same vk_input as the Rue puzzle's on-chain computation.
    if hash[0] & 0x80 != 0 {
        // Negative in CLVM: compute absolute value via two's complement
        let complement: Vec<u8> = hash.iter().map(|&b| !b).collect();
        let abs_val = BigUint::from_bytes_be(&complement) + 1u64;
        -Fr::from(abs_val)
    } else {
        // Positive: interpret directly as unsigned big-endian
        Fr::from(BigUint::from_bytes_be(&hash))
    }
}
