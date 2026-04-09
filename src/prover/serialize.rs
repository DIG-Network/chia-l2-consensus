//! CLVM serialization for proofs and verification keys.
//!
//! See [spec-wire-format.md](../../docs/resources/spec-wire-format.md).

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
