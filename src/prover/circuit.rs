//! Groth16 circuit definition.
//!
//! See [spec-groth16-circuit.md](../../docs/resources/spec-groth16-circuit.md).
//!
//! The circuit proves: "I know k BLS pubkeys, each with a valid Merkle inclusion
//! proof against `validator_merkle_root`, whose G1 sum equals `agg_signers`,
//! and where 2k > `validator_count`."

use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

use crate::merkle::{MerkleProof, TREE_DEPTH};

// ============================================================================
// CIR-006: Circuit Parameters (fixed at trusted setup time)
// ============================================================================

/// Maximum number of signing validators the circuit supports.
/// The actual k can be anything from majority threshold up to MAX_SIGNERS.
/// Changing this requires a new trusted setup ceremony.
pub const MAX_SIGNERS: usize = 64;

// ============================================================================
// CIR-001: Circuit Statement
// ============================================================================

/// The consensus circuit for proving validator set membership and majority.
///
/// Circuit statement (informal):
/// "I know k BLS pubkeys, each with a valid Merkle inclusion proof against
/// `validator_merkle_root`, whose G1 sum equals `agg_signers`, and where
/// 2k > `validator_count`."
///
/// Public inputs (in order, matches IC points):
/// 1. validator_merkle_root (32 bytes)
/// 2. validator_count (u64)
/// 3. new_validator_merkle_root (32 bytes)
/// 4. new_validator_count (u64)
/// 5. agg_signers (48 bytes, G1 compressed)
/// 6. checkpoint_message (32 bytes)
///
/// Private witnesses:
/// - signing_pubkeys: up to MAX_SIGNERS pubkeys
/// - merkle_proofs: corresponding Merkle proofs
/// - actual_signers: how many slots are actually used
///
/// Source: spec-groth16-circuit.md Lines 39-58
#[derive(Clone)]
pub struct ConsensusCircuit {
    // ========================================================================
    // Public Inputs (CIR-005)
    // ========================================================================
    /// Current validator Merkle root (32 bytes).
    validator_merkle_root: [u8; 32],

    /// Current validator count.
    validator_count: u64,

    /// New validator Merkle root after this checkpoint (32 bytes).
    new_validator_merkle_root: [u8; 32],

    /// New validator count after this checkpoint.
    new_validator_count: u64,

    /// Aggregate public key of signers (48 bytes, G1 compressed).
    agg_signers: [u8; 48],

    /// Checkpoint message that was signed (32 bytes).
    checkpoint_message: [u8; 32],

    // ========================================================================
    // Private Witnesses
    // ========================================================================
    /// Signing validators' pubkeys (up to MAX_SIGNERS).
    signing_pubkeys: Vec<[u8; 48]>,

    /// Merkle inclusion proofs for each signing pubkey.
    merkle_proofs: Vec<MerkleProof>,

    /// Number of actual signers (k). May be less than MAX_SIGNERS.
    actual_signers: usize,
}

impl ConsensusCircuit {
    /// Create a new empty circuit for constraint system setup.
    pub fn new() -> Self {
        Self {
            validator_merkle_root: [0u8; 32],
            validator_count: 0,
            new_validator_merkle_root: [0u8; 32],
            new_validator_count: 0,
            agg_signers: [0u8; 48],
            checkpoint_message: [0u8; 32],
            signing_pubkeys: Vec::new(),
            merkle_proofs: Vec::new(),
            actual_signers: 0,
        }
    }

    /// Create a circuit with public inputs only (for testing).
    pub fn with_public_inputs(
        validator_merkle_root: [u8; 32],
        validator_count: u64,
        new_validator_merkle_root: [u8; 32],
        new_validator_count: u64,
        agg_signers: [u8; 48],
        checkpoint_message: [u8; 32],
    ) -> Self {
        Self {
            validator_merkle_root,
            validator_count,
            new_validator_merkle_root,
            new_validator_count,
            agg_signers,
            checkpoint_message,
            signing_pubkeys: Vec::new(),
            merkle_proofs: Vec::new(),
            actual_signers: 0,
        }
    }

    /// Create a circuit with both public inputs and private witnesses.
    pub fn with_witnesses(
        validator_merkle_root: [u8; 32],
        validator_count: u64,
        new_validator_merkle_root: [u8; 32],
        new_validator_count: u64,
        agg_signers: [u8; 48],
        checkpoint_message: [u8; 32],
        signing_pubkeys: Vec<[u8; 48]>,
        merkle_proofs: Vec<MerkleProof>,
    ) -> Self {
        let actual_signers = signing_pubkeys.len();
        Self {
            validator_merkle_root,
            validator_count,
            new_validator_merkle_root,
            new_validator_count,
            agg_signers,
            checkpoint_message,
            signing_pubkeys,
            merkle_proofs,
            actual_signers,
        }
    }

    // ========================================================================
    // CIR-006: Circuit Parameters
    // ========================================================================

    /// Maximum number of signers this circuit supports.
    pub fn max_signers(&self) -> usize {
        MAX_SIGNERS
    }

    /// Tree depth (must match SMT spec).
    pub fn tree_depth(&self) -> u32 {
        TREE_DEPTH
    }

    // ========================================================================
    // CIR-005: Public Input Accessors
    // ========================================================================

    /// Get the current validator Merkle root.
    pub fn validator_merkle_root(&self) -> [u8; 32] {
        self.validator_merkle_root
    }

    /// Get the current validator count.
    pub fn validator_count(&self) -> u64 {
        self.validator_count
    }

    /// Get the new validator Merkle root.
    pub fn new_validator_merkle_root(&self) -> [u8; 32] {
        self.new_validator_merkle_root
    }

    /// Get the new validator count.
    pub fn new_validator_count(&self) -> u64 {
        self.new_validator_count
    }

    /// Get the aggregate signers pubkey.
    pub fn agg_signers(&self) -> [u8; 48] {
        self.agg_signers
    }

    /// Get the checkpoint message.
    pub fn checkpoint_message(&self) -> [u8; 32] {
        self.checkpoint_message
    }

    // ========================================================================
    // Private Witness Accessors
    // ========================================================================

    /// Get the number of actual signers (k).
    pub fn actual_signers(&self) -> usize {
        self.actual_signers
    }

    /// Get the signing pubkeys.
    pub fn signing_pubkeys(&self) -> &[[u8; 48]] {
        &self.signing_pubkeys
    }

    /// Get the Merkle proofs.
    pub fn merkle_proofs(&self) -> &[MerkleProof] {
        &self.merkle_proofs
    }
}

impl Default for ConsensusCircuit {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintSynthesizer<ark_bls12_381::Fr> for ConsensusCircuit {
    fn generate_constraints(
        self,
        _cs: ConstraintSystemRef<ark_bls12_381::Fr>,
    ) -> Result<(), SynthesisError> {
        // CIR-001: Circuit statement has three components:
        // 1. Merkle membership for each signer (CIR-002)
        // 2. G1 sum equals agg_signers (CIR-003)
        // 3. Majority threshold: 2k > validator_count (CIR-004)

        // TODO: Implement constraints in CIR-002, CIR-003, CIR-004
        // For now, this allows the circuit to be instantiated

        Ok(())
    }
}
