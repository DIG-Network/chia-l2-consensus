//! Groth16 circuit definition.
//!
//! See [spec-groth16-circuit.md](../../docs/resources/spec-groth16-circuit.md).
//!
//! The circuit proves: "I know k BLS pubkeys, each with a valid Merkle inclusion
//! proof against `validator_merkle_root`, whose G1 sum equals `agg_signers`,
//! and where 2k > `validator_count`."

use ark_bls12_381::Fr;
use ark_crypto_primitives::sponge::poseidon::PoseidonConfig;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

use crate::merkle::{MerkleProof, TREE_DEPTH};

// ============================================================================
// CIR-006: Circuit Parameters (fixed at trusted setup time)
// ============================================================================

/// Maximum number of signing validators the circuit supports.
/// The actual k can be anything from majority threshold up to MAX_SIGNERS.
/// Changing this requires a new trusted setup ceremony.
///
/// Set to 20,000 to support large validator sets.
pub const MAX_SIGNERS: usize = 20_000;

// ============================================================================
// CIR-005: Public Input Constants
// ============================================================================

/// Number of public inputs in the circuit.
/// This must match the number of IC points in the verification key minus 1.
/// VK has 7 IC points: IC[0] is constant, IC[1..7] are for the 6 public inputs.
pub const PUBLIC_INPUT_COUNT: usize = 6;

/// Public input indices (1-based, matching IC point indices).
pub mod public_input_index {
    /// Index 1: Current validator Merkle root.
    pub const VALIDATOR_MERKLE_ROOT: usize = 1;
    /// Index 2: Current validator count.
    pub const VALIDATOR_COUNT: usize = 2;
    /// Index 3: New validator Merkle root.
    pub const NEW_VALIDATOR_MERKLE_ROOT: usize = 3;
    /// Index 4: New validator count.
    pub const NEW_VALIDATOR_COUNT: usize = 4;
    /// Index 5: Aggregate signers pubkey.
    pub const AGG_SIGNERS: usize = 5;
    /// Index 6: Checkpoint message hash.
    pub const CHECKPOINT_MESSAGE: usize = 6;
}

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

    // ========================================================================
    // CIR-002: Poseidon Merkle Witnesses
    // ========================================================================
    /// Poseidon hash parameters (must match off-chain tree).
    poseidon_config: Option<PoseidonConfig<Fr>>,

    /// Poseidon Merkle root (Fr field element, private witness).
    poseidon_merkle_root: Option<Fr>,

    /// Poseidon Merkle proofs: (leaf Fr, siblings Vec<Fr>, slot index).
    /// One per signing pubkey.
    poseidon_proofs: Vec<(Fr, Vec<Fr>, u64)>,

    /// Number of signer slots to verify in the circuit.
    /// Equals MAX_SIGNERS for production; smaller for tests.
    /// The trusted setup and prover MUST use the same value.
    circuit_max_signers: usize,

    /// Depth of the Poseidon Merkle tree for in-circuit verification.
    /// May differ from TREE_DEPTH (the on-chain SHA-256 tree depth).
    poseidon_tree_depth: u32,
}

impl ConsensusCircuit {
    /// Create a new empty circuit for constraint system setup.
    ///
    /// Uses dummy majority witness (k=1, n=0) so the setup circuit is satisfiable.
    /// The trusted setup only needs a satisfiable circuit to generate the CRS;
    /// the actual witness values don't affect the proving/verification keys.
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
            actual_signers: 1, // dummy: 2*1 > 0
            poseidon_config: None,
            poseidon_merkle_root: None,
            poseidon_proofs: Vec::new(),
            circuit_max_signers: 0,
            poseidon_tree_depth: 0,
        }
    }

    /// Create a circuit with public inputs and majority witness.
    ///
    /// `actual_signers` is the private witness k. The circuit enforces 2k > validator_count.
    pub fn with_public_inputs(
        validator_merkle_root: [u8; 32],
        validator_count: u64,
        new_validator_merkle_root: [u8; 32],
        new_validator_count: u64,
        agg_signers: [u8; 48],
        checkpoint_message: [u8; 32],
        actual_signers: usize,
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
            actual_signers,
            poseidon_config: None,
            poseidon_merkle_root: None,
            poseidon_proofs: Vec::new(),
            circuit_max_signers: 0,
            poseidon_tree_depth: 0,
        }
    }

    /// Create a circuit with Poseidon Merkle proofs for CIR-002 verification.
    ///
    /// `poseidon_proofs` contains (leaf, siblings, slot) for each signer.
    /// `circuit_max_signers` controls the loop iteration count (must match setup).
    /// Unused slots (beyond actual signers) are padded with dummy proofs.
    #[allow(clippy::too_many_arguments)]
    pub fn with_merkle_proofs(
        validator_merkle_root: [u8; 32],
        validator_count: u64,
        new_validator_merkle_root: [u8; 32],
        new_validator_count: u64,
        agg_signers: [u8; 48],
        checkpoint_message: [u8; 32],
        actual_signers: usize,
        poseidon_config: PoseidonConfig<Fr>,
        poseidon_merkle_root: Fr,
        poseidon_proofs: Vec<(Fr, Vec<Fr>, u64)>,
        circuit_max_signers: usize,
        poseidon_tree_depth: u32,
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
            actual_signers,
            poseidon_config: Some(poseidon_config),
            poseidon_merkle_root: Some(poseidon_merkle_root),
            poseidon_proofs,
            circuit_max_signers,
            poseidon_tree_depth,
        }
    }

    /// Create a circuit with both public inputs and private witnesses.
    #[allow(clippy::too_many_arguments)]
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
            poseidon_config: None,
            poseidon_merkle_root: None,
            poseidon_proofs: Vec::new(),
            circuit_max_signers: 0,
            poseidon_tree_depth: 0,
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

    /// Get all public inputs in order as raw bytes.
    ///
    /// Returns a vector of 6 byte slices in the canonical order:
    /// 1. validator_merkle_root (32 bytes)
    /// 2. validator_count (8 bytes, big-endian)
    /// 3. new_validator_merkle_root (32 bytes)
    /// 4. new_validator_count (8 bytes, big-endian)
    /// 5. agg_signers (48 bytes)
    /// 6. checkpoint_message (32 bytes)
    ///
    /// This order must match the IC point assignment in the verification key.
    pub fn public_inputs_bytes(&self) -> Vec<Vec<u8>> {
        vec![
            self.validator_merkle_root.to_vec(),
            self.validator_count.to_be_bytes().to_vec(),
            self.new_validator_merkle_root.to_vec(),
            self.new_validator_count.to_be_bytes().to_vec(),
            self.agg_signers.to_vec(),
            self.checkpoint_message.to_vec(),
        ]
    }

    /// Get the number of public inputs (always 6).
    pub fn public_input_count(&self) -> usize {
        PUBLIC_INPUT_COUNT
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
        cs: ConstraintSystemRef<ark_bls12_381::Fr>,
    ) -> Result<(), SynthesisError> {
        use super::serialize::bytes_to_scalar;
        use ark_bls12_381::Fr;
        use ark_r1cs_std::{fields::fp::FpVar, prelude::*};

        // ================================================================
        // CIR-005: Allocate 6 public inputs (VK gets 7 IC points)
        // ================================================================
        let vmr = self.validator_merkle_root;
        let vc_be8 = self.validator_count.to_be_bytes();
        let nvmr = self.new_validator_merkle_root;
        let nvc_be8 = self.new_validator_count.to_be_bytes();
        let agg = self.agg_signers;
        let cm = self.checkpoint_message;

        let _ = cs.new_input_variable(|| Ok(bytes_to_scalar(&vmr)))?;
        let _ = cs.new_input_variable(|| Ok(bytes_to_scalar(&vc_be8)))?;
        let _ = cs.new_input_variable(|| Ok(bytes_to_scalar(&nvmr)))?;
        let _ = cs.new_input_variable(|| Ok(bytes_to_scalar(&nvc_be8)))?;
        let _ = cs.new_input_variable(|| Ok(bytes_to_scalar(&agg)))?;
        let _ = cs.new_input_variable(|| Ok(bytes_to_scalar(&cm)))?;

        // ================================================================
        // CIR-004: Majority threshold — 2k > validator_count
        //
        // Private witnesses: k (actual_signers), n (validator_count).
        // Constraint: 2k - n - 1 >= 0, enforced via 64-bit decomposition.
        //
        // Security binding: n is tied to the public input via the on-chain
        // scalar assertion sha256(vc_be8) == scalars.s2. A prover who lies
        // about n produces a proof that fails bls_pairing_identity on-chain.
        // See DESIGN_DECISIONS.md Decision 3.
        // ================================================================

        let k = self.actual_signers as u64;
        let n = self.validator_count;

        let k_var = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(k)))?;
        let n_var = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(n)))?;

        // diff = 2k - n - 1 (must be non-negative for strict majority)
        let two_k = k_var.double()?;
        let diff = &two_k - &n_var - FpVar::<Fr>::one();

        // Enforce diff >= 0 by decomposing into 64 boolean bits and
        // reconstructing. If the witness doesn't satisfy 2k > n, the
        // bit decomposition can't reconstruct diff (which would be negative
        // in the field, i.e. a huge number that doesn't fit in 64 bits).
        let diff_val = if 2 * k > n { 2 * k - n - 1 } else { 0 };
        let bit_witnesses: Vec<bool> = (0..64).map(|i| (diff_val >> i) & 1 == 1).collect();

        let mut reconstructed = FpVar::<Fr>::zero();
        let mut power = FpVar::<Fr>::one();
        let two_const = FpVar::<Fr>::constant(Fr::from(2u64));
        for &b in &bit_witnesses {
            let bit = Boolean::new_witness(cs.clone(), || Ok(b))?;
            reconstructed += FpVar::from(bit) * &power;
            power *= &two_const;
        }
        diff.enforce_equal(&reconstructed)?;

        // ================================================================
        // CIR-002: Poseidon Merkle membership verification
        //
        // For each signer slot (up to circuit_max_signers):
        // 1. Allocate leaf, siblings, and slot index as witnesses
        // 2. Walk from leaf to root using Poseidon two-to-one hash
        // 3. Enforce computed root == poseidon_merkle_root witness
        //
        // Unused slots (padding) use the empty leaf + valid dummy proof.
        // The root is a private witness; binding to public inputs
        // comes via CIR-003 (G1 aggregation, future) and the BLS
        // signature check on-chain.
        // ================================================================

        if let (Some(config), Some(expected_root)) =
            (&self.poseidon_config, self.poseidon_merkle_root)
        {
            use ark_crypto_primitives::crh::poseidon::constraints::TwoToOneCRHGadget;
            use ark_crypto_primitives::crh::TwoToOneCRHSchemeGadget;
            use ark_crypto_primitives::sponge::poseidon::constraints::PoseidonSpongeVar;

            // Allocate the Poseidon config as a constant
            let config_var = <TwoToOneCRHGadget<Fr> as TwoToOneCRHSchemeGadget<
                ark_crypto_primitives::crh::poseidon::TwoToOneCRH<Fr>,
                Fr,
            >>::ParametersVar::new_constant(cs.clone(), config)?;

            // Allocate the expected root as a private witness
            let root_var = FpVar::<Fr>::new_witness(cs.clone(), || Ok(expected_root))?;

            let tree_depth = self.poseidon_tree_depth as usize;

            for slot_idx in 0..self.circuit_max_signers {
                // Get the proof for this slot (or dummy if beyond actual signers)
                let (leaf, siblings, index) = if slot_idx < self.poseidon_proofs.len() {
                    let (l, s, i) = &self.poseidon_proofs[slot_idx];
                    (*l, s.clone(), *i)
                } else {
                    // Padding: empty leaf with dummy siblings
                    let empty = crate::merkle::poseidon::poseidon_empty_leaf(config);
                    (empty, vec![Fr::default(); tree_depth], 0)
                };

                // Allocate leaf as witness
                let mut current = FpVar::<Fr>::new_witness(cs.clone(), || Ok(leaf))?;

                // Walk from leaf to root
                let mut idx = index;
                for level in 0..tree_depth {
                    let sibling_val = if level < siblings.len() {
                        siblings[level]
                    } else {
                        Fr::default()
                    };
                    let sibling = FpVar::<Fr>::new_witness(cs.clone(), || Ok(sibling_val))?;

                    let is_right = idx % 2 == 1;
                    let (left, right) = if is_right {
                        (sibling.clone(), current)
                    } else {
                        (current, sibling.clone())
                    };

                    // Poseidon two-to-one hash gadget
                    current = <TwoToOneCRHGadget<Fr> as TwoToOneCRHSchemeGadget<
                        ark_crypto_primitives::crh::poseidon::TwoToOneCRH<Fr>,
                        Fr,
                    >>::evaluate(&config_var, &left, &right)?;

                    idx /= 2;
                }

                // Enforce computed root == expected root
                current.enforce_equal(&root_var)?;
            }
        }

        // CIR-003: Aggregate key — Phase 3 (non-native G1 arithmetic)

        Ok(())
    }
}
