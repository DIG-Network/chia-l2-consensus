//! Validator registration: signing and verification.
//!
//! VAL-002: Validators register by spending the network coin with a BLS
//! signature proving they control the claimed pubkey.
//!
//! The registration message follows Chia's AGG_SIG_ME convention:
//! `signing_message = sha256("register" + pubkey) + genesis_challenge + coin_id`
//!
//! See [spec-wire-format.md](../../docs/resources/spec-wire-format.md) — Registration Message Format.
//! See [spec-validator-onboarding.md](../../docs/resources/spec-validator-onboarding.md) — Steps 5-6.

use crate::error::{ConsensusError, ConsensusResult};
use crate::prover::compute_registration_message;
use crate::validator::keygen::{sign_message, verify_signature};

/// Compute the full AGG_SIG_ME signing message for registration.
///
/// Layout (96 bytes):
/// - `registration_message` (32 bytes) = sha256("register" + pubkey)
/// - `genesis_challenge` (32 bytes) = Chia network identifier
/// - `network_coin_coin_id` (32 bytes) = the network coin being spent
///
/// This is the message the validator signs with their BLS key to prove
/// they control the claimed pubkey during registration.
///
/// See spec-wire-format.md — Registration Message Format.
pub fn compute_registration_signing_message(
    pubkey: &[u8; 48],
    genesis_challenge: &[u8; 32],
    network_coin_coin_id: &[u8; 32],
) -> [u8; 96] {
    let reg_msg = compute_registration_message(pubkey);

    let mut message = [0u8; 96];
    message[0..32].copy_from_slice(&reg_msg);
    message[32..64].copy_from_slice(genesis_challenge);
    message[64..96].copy_from_slice(network_coin_coin_id);
    message
}

/// Sign the registration message with the validator's secret key.
///
/// Uses BLS augmented scheme. The signature proves the validator
/// controls the private key corresponding to the pubkey being registered.
///
/// Returns a 96-byte compressed G2 signature.
pub fn sign_registration(
    secret_key: &[u8; 32],
    pubkey: &[u8; 48],
    genesis_challenge: &[u8; 32],
    network_coin_coin_id: &[u8; 32],
) -> ConsensusResult<[u8; 96]> {
    let message =
        compute_registration_signing_message(pubkey, genesis_challenge, network_coin_coin_id);
    sign_message(secret_key, &message)
}

/// Verify a registration signature.
///
/// Checks that the signature was produced by the validator's private key
/// over the correct registration message for the given network parameters.
pub fn verify_registration_signature(
    pubkey: &[u8; 48],
    genesis_challenge: &[u8; 32],
    network_coin_coin_id: &[u8; 32],
    signature: &[u8; 96],
) -> ConsensusResult<bool> {
    let message =
        compute_registration_signing_message(pubkey, genesis_challenge, network_coin_coin_id);
    verify_signature(pubkey, &message, signature)
}
