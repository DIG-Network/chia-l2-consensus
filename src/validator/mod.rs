//! Validator operations: key generation, signing, registration, verification.
//!
//! See [spec-validator-onboarding.md](../../docs/resources/spec-validator-onboarding.md).

mod exit;
mod keygen;
mod registration;
mod signing;

pub use exit::{
    compute_exit_announcement, is_validator_excluded, prepare_collateral_recovery,
    prepare_forced_exit, CollateralRecoveryParams, ForcedExitParams, ForcedExitReason,
};
pub use keygen::{
    generate_validator_keypair, pubkey_from_secret, sign_message, verify_signature,
    ValidatorKeypair,
};
pub use registration::{
    compute_registration_signing_message, sign_registration, verify_registration_signature,
};
pub use signing::{
    aggregate_checkpoint_signatures, compute_checkpoint_signing_message, sign_checkpoint,
    verify_checkpoint_signature,
};
