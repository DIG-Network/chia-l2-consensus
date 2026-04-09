//! Error types for chia-l2-consensus.
//!
//! See [spec-consensus-crate.md Lines 151-222](../docs/resources/spec-consensus-crate.md).

use thiserror::Error;

/// Errors that can occur in consensus operations.
#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("network not deployed - call deploy() first")]
    NotDeployed,

    #[error("validator already registered: {0}")]
    AlreadyRegistered(String),

    #[error("validator not found in active set: {0}")]
    ValidatorNotFound(String),

    #[error("below majority threshold: need more than {count}/2 signers, got {actual}")]
    BelowThreshold { count: u64, actual: usize },

    #[error("on-chain state mismatch: local merkle root does not match on-chain root")]
    StateMismatch,

    #[error("invalid lineage proof for registration coin")]
    InvalidLineage,

    #[error("merkle proof verification failed")]
    InvalidMerkleProof,

    #[error("proof generation failed: {0}")]
    ProvingError(String),

    #[error("node rpc error: {0}")]
    NodeError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("spend bundle rejected by node: {0}")]
    SpendRejected(String),

    #[error("puzzle hash mismatch for registration coin")]
    PuzzleHashMismatch,

    #[error("indexer cache error: {0}")]
    CacheError(String),

    #[error("slot collision for pubkey: {0}")]
    SlotCollision(String),
}

/// Result type alias for consensus operations.
pub type ConsensusResult<T> = Result<T, ConsensusError>;
