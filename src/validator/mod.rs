//! Validator operations: key generation, registration, checkpoint signing, exit.
//!
//! ## Sub-modules
//!
//! | Module | Purpose | Requirements |
//! |--------|---------|-------------|
//! | `keygen` | BLS12-381 keypair generation, sign/verify | VAL-001 |
//! | `registration` | Registration AGG_SIG_ME message, sign/verify | VAL-002 |
//! | `signing` | Checkpoint signing, signature aggregation | VAL-003 |
//! | `exit` | Voluntary/forced exit, collateral recovery params | VAL-004, VAL-005 |
//!
//! ## Validator Lifecycle
//!
//! ```text
//! 1. keygen::generate_validator_keypair()     → (pubkey, secret_key)
//! 2. registration::sign_registration()         → AGG_SIG_ME signature
//! 3. signing::sign_checkpoint()                → per-checkpoint signature
//! 4. exit::prepare_collateral_recovery()       → non-membership proof + params
//! ```
//!
//! See [spec-validator-onboarding.md](../../docs/resources/spec-validator-onboarding.md)
//! for the complete validator lifecycle specification.

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
