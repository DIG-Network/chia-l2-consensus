//! Trusted setup operations.
//!
//! See [spec-trusted-setup.md](../../docs/resources/spec-trusted-setup.md).

use std::path::Path;

use crate::error::{ConsensusError, ConsensusResult};

/// Load a proving key from disk.
pub fn load_proving_key(_path: &Path) -> ConsensusResult<Vec<u8>> {
    // TODO: Implement
    Err(ConsensusError::ProvingError("not implemented".into()))
}

/// Load a verification key from disk.
pub fn load_verification_key(_path: &Path) -> ConsensusResult<Vec<u8>> {
    // TODO: Implement
    Err(ConsensusError::ProvingError("not implemented".into()))
}
