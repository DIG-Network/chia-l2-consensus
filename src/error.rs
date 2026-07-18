//! Error types for chia-l2-consensus.
//!
//! Every error variant maps to a specific failure mode documented in the
//! spec. The L2 system matches on these variants to decide recovery actions.
//!
//! See [spec-consensus-crate.md Lines 151-222](../docs/resources/spec-consensus-crate.md)
//! for the full error type specification.

use thiserror::Error;

/// Errors that can occur in consensus operations.
///
/// Each variant corresponds to a spec-defined failure mode.
/// The L2 system should handle these programmatically, not just log them.
#[derive(Error, Debug)]
pub enum ConsensusError {
    /// No network state available. Call `sync()` first.
    /// Returned by state accessor methods when the client hasn't synced yet.
    #[error("network not deployed - call deploy() first")]
    NotDeployed,

    /// The validator pubkey is already in the active set.
    /// Returned by `register_validator()` when the pubkey has an existing
    /// registration coin, and by `prepare_collateral_recovery()` when the
    /// validator hasn't been excluded yet.
    ///
    /// See [spec-consensus-crate.md Lines 162-163](../docs/resources/spec-consensus-crate.md).
    #[error("validator already registered: {0}")]
    AlreadyRegistered(String),

    /// The validator pubkey is not in the current active set.
    /// Returned when trying to sign for a validator who isn't registered
    /// or whose registration coin has been spent.
    ///
    /// See [spec-consensus-crate.md Lines 165-166](../docs/resources/spec-consensus-crate.md).
    #[error("validator not found in active set: {0}")]
    ValidatorNotFound(String),

    /// Not enough signers for majority consensus.
    /// The circuit requires `2k > validator_count` (CIR-004).
    /// Collect more signatures before calling `build_checkpoint()`.
    ///
    /// See [spec-groth16-circuit.md Lines 325-357](../docs/resources/spec-groth16-circuit.md) —
    /// Constraint 3: Majority Threshold.
    #[error("below majority threshold: need more than {count}/2 signers, got {actual}")]
    BelowThreshold { count: u64, actual: usize },

    /// Local Merkle tree root ≠ on-chain `validator_merkle_root`.
    /// The indexer is out of sync. Recovery: delete cache, call `sync()` for
    /// full re-index from genesis.
    ///
    /// See [spec-indexer.md Lines 400-450](../docs/resources/spec-indexer.md) —
    /// Merkle Root Consistency Check.
    #[error("on-chain state mismatch: local merkle root does not match on-chain root")]
    StateMismatch,

    /// Registration coin's parent coin ID does not trace back to a network
    /// coin spend. The coin was not created through the approved registration
    /// process and MUST be ignored.
    ///
    /// See [spec-security.md Lines 175-200](../docs/resources/spec-security.md) —
    /// Lineage Proof Enforcement.
    #[error("invalid lineage proof for registration coin")]
    InvalidLineage,

    /// Merkle proof verification failed. Either the leaf doesn't match
    /// the expected value, or the path doesn't reconstruct to the root.
    ///
    /// See [spec-sparse-merkle-tree.md Lines 327-380](../docs/resources/spec-sparse-merkle-tree.md) —
    /// Proof Verification.
    #[error("merkle proof verification failed")]
    InvalidMerkleProof,

    /// Groth16 proof generation failed. Possible causes:
    /// - Out of memory (proof generation is memory-intensive)
    /// - Proving key not loaded (call `load_proving_key()` first)
    /// - Unsatisfiable constraints (indicates a circuit implementation bug)
    ///
    /// See [spec-groth16-circuit.md Lines 400-450](../docs/resources/spec-groth16-circuit.md) —
    /// Proof Generation.
    #[error("proof generation failed: {0}")]
    ProvingError(String),

    /// Chia full node RPC call failed.
    #[error("node rpc error: {0}")]
    NodeError(String),

    /// Serialization mismatch between Rust types and CLVM format.
    /// Check point sizes against spec-wire-format constants (G1=48, G2=96).
    ///
    /// See [spec-wire-format.md Lines 46-118](../docs/resources/spec-wire-format.md) —
    /// Point Encoding.
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Chia node rejected the spend bundle. Possible causes:
    /// stale epoch (another checkpoint submitted first),
    /// invalid proof or signature, or insufficient fees.
    /// Re-sync and check epoch before retrying.
    #[error("spend bundle rejected by node: {0}")]
    SpendRejected(String),

    /// Registration coin puzzle hash doesn't match expected value.
    /// The coin was not created correctly — skip it during indexing.
    ///
    /// See [spec-indexer.md Lines 266-355](../docs/resources/spec-indexer.md) —
    /// Registration Coin Detection.
    #[error("puzzle hash mismatch for registration coin")]
    PuzzleHashMismatch,

    /// Indexer cache is corrupted or incompatible with current version.
    /// Recovery: delete cache file and re-sync.
    #[error("indexer cache error: {0}")]
    CacheError(String),

    /// Blockchain query failed (chia-query).
    /// Wraps errors from `ChiaQuery` — peer connection failures, coinset API
    /// errors, or all sources failing.
    ///
    /// RPC-001: See [spec-consensus-crate.md](../docs/resources/spec-consensus-crate.md).
    #[error("blockchain query error: {0}")]
    RpcError(String),

    /// Two validators hash to the same Merkle tree slot.
    /// Probability: ~n²/2^64 for n validators (negligible for realistic sets).
    /// Reject the second registration at the L2 level.
    ///
    /// See [spec-sparse-merkle-tree.md Lines 63-104](../docs/resources/spec-sparse-merkle-tree.md) —
    /// Slot Assignment.
    #[error("slot collision for pubkey: {0}")]
    SlotCollision(String),

    /// Wallet does not have enough XCH to fund collateral + fees.
    /// The validator needs to add funds before registering.
    ///
    /// RPC-005: Returned by `register_validator()` when `dig-wallet-backend`
    /// coin selection (`engine::select_for_spend`) yields `InsufficientFunds`
    /// or `NeedsConsolidation`.
    #[error("insufficient funds for collateral: {0}")]
    InsufficientFunds(String),
}

/// Result type alias for all consensus operations.
pub type ConsensusResult<T> = Result<T, ConsensusError>;
