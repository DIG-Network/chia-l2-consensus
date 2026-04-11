//! Voluntary exit and collateral recovery for validators.
//!
//! VAL-004: Validators exit by proving non-membership after checkpoint
//! exclusion, then spending their registration coin with the announcement.
//!
//! The recovery bundle is two atomic spends:
//! 1. Checkpoint singleton (membership query) → emits non-membership announcement
//! 2. Registration coin → asserts announcement, returns collateral
//!
//! See [spec-validator-onboarding.md](../../docs/resources/spec-validator-onboarding.md) — Voluntary Exit.
//! See [spec-registration-coin.md](../../docs/resources/spec-registration-coin.md) — Spending the Registration Coin.

use sha2::{Digest, Sha256};

use crate::error::{ConsensusError, ConsensusResult};
use crate::merkle::{MerkleProof, SparseMerkleTree, EMPTY_LEAF};
use crate::prover::compute_membership_announcement_message;

/// Check if a validator is excluded from the current validator set.
///
/// A validator is excluded if their slot in the Merkle tree contains
/// `EMPTY_LEAF` rather than `active_leaf(pubkey)`.
pub fn is_validator_excluded(tree: &SparseMerkleTree, pubkey: &[u8; 48]) -> bool {
    let proof = tree.prove_validator(pubkey);
    proof.leaf == EMPTY_LEAF
}

/// Compute the full exit announcement hash.
///
/// This is the value used in `ASSERT_COIN_ANNOUNCEMENT` by the registration
/// coin. It combines the checkpoint singleton coin ID with the inner
/// non-membership announcement.
///
/// Format: `sha256(checkpoint_coin_id + sha256("membership" + epoch_be8 + pubkey + 0x00))`
///
/// See spec-wire-format.md — Membership Announcement Format.
pub fn compute_exit_announcement(
    epoch: u64,
    pubkey: &[u8; 48],
    checkpoint_coin_id: &[u8; 32],
) -> [u8; 32] {
    let inner = compute_membership_announcement_message(epoch, pubkey, false);

    let mut hasher = Sha256::new();
    hasher.update(checkpoint_coin_id);
    hasher.update(inner);
    hasher.finalize().into()
}

/// Parameters for building the collateral recovery spend bundle.
///
/// Contains all data needed for both spends in the recovery bundle:
/// - Membership query spend (checkpoint singleton)
/// - Registration coin spend
pub struct CollateralRecoveryParams {
    /// Validator's BLS pubkey (48 bytes).
    pub pubkey: [u8; 48],

    /// Current checkpoint epoch.
    pub epoch: u64,

    /// Current checkpoint singleton coin ID.
    pub checkpoint_coin_id: [u8; 32],

    /// Destination puzzle hash for recovered collateral.
    pub destination: [u8; 32],

    /// Collateral amount in mojos.
    pub collateral_amount: u64,

    /// Non-membership Merkle proof for the validator's slot.
    pub merkle_proof: MerkleProof,

    /// The expected exit announcement hash (for registration coin assertion).
    pub announcement_hash: [u8; 32],
}

// ============================================================================
// VAL-005: Forced Exit
// ============================================================================

/// Reason for a forced exit (L2 governance level).
///
/// This is metadata only — the on-chain mechanism is identical to voluntary
/// exit. The reason is recorded for auditing and governance transparency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForcedExitReason {
    /// Validator's private key was compromised.
    KeyCompromise,
    /// Validator has been offline and cannot participate in signing.
    ValidatorOffline,
    /// Validator signed conflicting checkpoints or violated protocol rules.
    Misbehavior,
    /// Network governance decision (policy enforcement, capacity management).
    GovernanceDecision,
}

/// Parameters for a forced exit, wrapping `CollateralRecoveryParams` with a reason.
pub struct ForcedExitParams {
    /// The collateral recovery parameters (same as voluntary exit).
    pub params: CollateralRecoveryParams,
    /// The reason for the forced exit (governance metadata).
    pub reason: ForcedExitReason,
}

/// Prepare a forced exit for an excluded validator.
///
/// Mechanically identical to `prepare_collateral_recovery()` (VAL-004),
/// but records the governance reason and allows specifying a slash
/// destination (governance address instead of validator's own address).
///
/// The validator must already be excluded from the Merkle tree by a
/// checkpoint that the majority signed. This function cannot force-exit
/// an active validator — the exclusion happens at the checkpoint level.
pub fn prepare_forced_exit(
    tree: &SparseMerkleTree,
    pubkey: &[u8; 48],
    epoch: u64,
    checkpoint_coin_id: &[u8; 32],
    slash_destination: &[u8; 32],
    collateral_amount: u64,
    reason: ForcedExitReason,
) -> ConsensusResult<ForcedExitParams> {
    let params = prepare_collateral_recovery(
        tree,
        pubkey,
        epoch,
        checkpoint_coin_id,
        slash_destination,
        collateral_amount,
    )?;

    Ok(ForcedExitParams { params, reason })
}

// ============================================================================
// VAL-004: Voluntary Exit — Collateral Recovery
// ============================================================================

/// Prepare collateral recovery parameters for an excluded validator.
///
/// Verifies the validator is excluded (slot contains EMPTY_LEAF), generates
/// the non-membership proof, and computes the expected announcement hash.
///
/// Returns an error if the validator is still active (slot is not empty).
///
/// The caller uses these params to build the two-spend bundle:
/// 1. Checkpoint membership query with `merkle_proof`
/// 2. Registration coin spend asserting `announcement_hash`
pub fn prepare_collateral_recovery(
    tree: &SparseMerkleTree,
    pubkey: &[u8; 48],
    epoch: u64,
    checkpoint_coin_id: &[u8; 32],
    destination: &[u8; 32],
    collateral_amount: u64,
) -> ConsensusResult<CollateralRecoveryParams> {
    // Generate proof
    let proof = tree.prove_validator(pubkey);

    // Verify non-membership (slot must be empty)
    if proof.leaf != EMPTY_LEAF {
        return Err(ConsensusError::AlreadyRegistered(
            "Validator is still active (leaf = active_leaf, not EMPTY_LEAF). \
             Cannot recover collateral while active."
                .to_string(),
        ));
    }

    // Compute announcement hash
    let announcement_hash = compute_exit_announcement(epoch, pubkey, checkpoint_coin_id);

    Ok(CollateralRecoveryParams {
        pubkey: *pubkey,
        epoch,
        checkpoint_coin_id: *checkpoint_coin_id,
        destination: *destination,
        collateral_amount,
        merkle_proof: proof,
        announcement_hash,
    })
}
