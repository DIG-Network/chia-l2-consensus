//! Checkpoint signing protocol for validators.
//!
//! VAL-003: Validators sign checkpoint messages using the AGG_SIG_ME convention.
//!
//! The signing message is: `checkpoint_message + genesis_challenge + coin_id`
//! (96 bytes total, signed directly without further hashing).
//!
//! See [spec-wire-format.md](../../docs/resources/spec-wire-format.md) — Individual Signatures.
//! See [spec-validator-onboarding.md](../../docs/resources/spec-validator-onboarding.md) — Step 9.

use crate::error::{ConsensusError, ConsensusResult};
use crate::validator::keygen::{sign_message, verify_signature};

/// Compute the full AGG_SIG_ME signing message for a checkpoint.
///
/// Layout (96 bytes):
/// - `checkpoint_message` (32 bytes) — sha256 of new state fields (WIRE-001)
/// - `genesis_challenge` (32 bytes) — Chia network identifier
/// - `checkpoint_singleton_coin_id` (32 bytes) — current singleton coin ID
///
/// This message is signed directly (not hashed again) per Chia AGG_SIG_ME
/// semantics.
///
/// See spec-wire-format.md — Individual Signatures.
pub fn compute_checkpoint_signing_message(
    checkpoint_message: &[u8; 32],
    genesis_challenge: &[u8; 32],
    checkpoint_singleton_coin_id: &[u8; 32],
) -> [u8; 96] {
    let mut msg = [0u8; 96];
    msg[0..32].copy_from_slice(checkpoint_message);
    msg[32..64].copy_from_slice(genesis_challenge);
    msg[64..96].copy_from_slice(checkpoint_singleton_coin_id);
    msg
}

/// Sign a checkpoint message with the validator's secret key.
///
/// Uses BLS augmented scheme. Returns a 96-byte compressed G2 signature.
///
/// The validator SHOULD independently verify the checkpoint message
/// contents before signing (new state root, new merkle root, epoch).
pub fn sign_checkpoint(
    secret_key: &[u8; 32],
    pubkey: &[u8; 48],
    checkpoint_message: &[u8; 32],
    genesis_challenge: &[u8; 32],
    checkpoint_singleton_coin_id: &[u8; 32],
) -> ConsensusResult<[u8; 96]> {
    let msg = compute_checkpoint_signing_message(
        checkpoint_message,
        genesis_challenge,
        checkpoint_singleton_coin_id,
    );
    sign_message(secret_key, &msg)
}

/// Verify a checkpoint signature from a single validator.
pub fn verify_checkpoint_signature(
    pubkey: &[u8; 48],
    checkpoint_message: &[u8; 32],
    genesis_challenge: &[u8; 32],
    checkpoint_singleton_coin_id: &[u8; 32],
    signature: &[u8; 96],
) -> ConsensusResult<bool> {
    let msg = compute_checkpoint_signing_message(
        checkpoint_message,
        genesis_challenge,
        checkpoint_singleton_coin_id,
    );
    verify_signature(pubkey, &msg, signature)
}

/// Aggregate multiple checkpoint signatures into a single G2 point.
///
/// The aggregate signature can be verified against the aggregate public
/// key (G1 sum of all signing pubkeys) using `bls_verify` on-chain.
///
/// At least one signature is required.
///
/// See spec-wire-format.md — Aggregate Signature.
pub fn aggregate_checkpoint_signatures(signatures: &[[u8; 96]]) -> ConsensusResult<[u8; 96]> {
    if signatures.is_empty() {
        return Err(ConsensusError::ProvingError(
            "Cannot aggregate zero signatures".to_string(),
        ));
    }

    if signatures.len() == 1 {
        return Ok(signatures[0]);
    }

    // Decompress all signatures
    let mut sigs = Vec::with_capacity(signatures.len());
    for (i, sig_bytes) in signatures.iter().enumerate() {
        let sig = blst::min_pk::Signature::uncompress(sig_bytes).map_err(|e| {
            ConsensusError::ProvingError(format!("Invalid signature {}: {:?}", i, e))
        })?;
        sigs.push(sig);
    }

    // Aggregate using blst
    let sig_refs: Vec<&blst::min_pk::Signature> = sigs.iter().collect();
    let agg = blst::min_pk::AggregateSignature::aggregate(&sig_refs, true).map_err(|e| {
        ConsensusError::ProvingError(format!("Signature aggregation failed: {:?}", e))
    })?;

    Ok(agg.to_signature().compress())
}
