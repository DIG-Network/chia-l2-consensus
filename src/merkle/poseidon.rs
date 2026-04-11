//! Poseidon-based Merkle tree for in-circuit verification (CIR-002).
//!
//! ## Why a Separate Poseidon Tree Exists
//!
//! This is a SEPARATE tree from the on-chain SHA-256 SMT ([`super::sparse`]).
//! Two parallel Merkle trees exist for the same validator set because of a
//! fundamental cost trade-off:
//!
//!   - **SHA-256** is used on-chain (CHK-005 membership queries) because CLVM
//!     has a native `sha256` opcode. But SHA-256 costs ~25,000 R1CS constraints
//!     per hash invocation inside a Groth16 circuit, making it impractical for
//!     large validator sets (e.g., MAX_SIGNERS=1000 would need ~830M constraints).
//!
//!   - **Poseidon** is an algebraic hash designed for ZK circuits: it costs only
//!     ~300 R1CS constraints per hash. The same MAX_SIGNERS=1000 needs only ~10M
//!     constraints. The trade-off is that Poseidon has no native CLVM opcode, so
//!     it cannot be verified on-chain directly.
//!
//! The Poseidon root is committed as a public input to the Groth16 proof. The
//! on-chain SHA-256 tree (CHK-005) coexists as a separate state field for
//! membership queries that do not require ZK.
//!
//! Source: [DESIGN_DECISIONS.md Decision 1](../../docs/requirements/domains/circuit/DESIGN_DECISIONS.md)
//! (Hash Function for In-Circuit Merkle Proofs).
//! Source: [spec-groth16-circuit.md Lines 204-273](../../docs/resources/spec-groth16-circuit.md)
//! (Constraint 1: Merkle Membership).
//!
//! # Cross-references
//!
//! - On-chain SHA-256 SMT: [`super::sparse::SparseMerkleTree`]
//! - Circuit that consumes Poseidon proofs: [`crate::prover::circuit::ConsensusCircuit`]
//! - Slot assignment (shared with SHA-256 tree): [`super::sparse::compute_slot`]

use ark_bls12_381::Fr;
use ark_crypto_primitives::crh::poseidon::{TwoToOneCRH, CRH};
use ark_crypto_primitives::crh::{CRHScheme, TwoToOneCRHScheme};
use ark_crypto_primitives::sponge::poseidon::PoseidonConfig;
use ark_ff::PrimeField;
use num_bigint::BigUint;

/// Generate Poseidon parameters for BLS12-381 Fr.
///
/// Uses rate=2, alpha=17, 8 full rounds, 31 partial rounds.
/// These parameters provide 128-bit security for the BLS12-381 scalar field
/// (255-bit prime field Fr).
///
/// Parameter meanings:
///   - **rate=2**: number of field elements absorbed per permutation round
///     (determines the hash width: 2 inputs per CRH invocation)
///   - **alpha=17**: S-box exponent x^17 (chosen for BLS12-381 Fr where
///     gcd(17, p-1) = 1, ensuring the S-box is a permutation)
///   - **8 full rounds**: rounds where the S-box is applied to ALL state elements
///     (provides resistance against algebraic attacks)
///   - **31 partial rounds**: rounds where the S-box is applied to only ONE
///     state element (provides resistance against statistical attacks at lower cost)
///   - **capacity=1**: one extra state element for domain separation
///
/// The ARK (AddRoundKey) and MDS (Maximum Distance Separable) matrices are
/// derived deterministically from the field parameters via Grain LFSR.
///
/// Source: [DESIGN_DECISIONS.md Decision 1](../../docs/requirements/domains/circuit/DESIGN_DECISIONS.md).
pub fn poseidon_config() -> PoseidonConfig<Fr> {
    use ark_crypto_primitives::sponge::poseidon::find_poseidon_ark_and_mds;

    let rate = 2;
    let full_rounds: u64 = 8;
    let partial_rounds: u64 = 31;
    let alpha = 17;
    let skip_matrices = 0;

    // Derive round constants (ARK) and MDS matrix deterministically
    // from the field modulus bit size and the round parameters.
    let (ark, mds) = find_poseidon_ark_and_mds::<Fr>(
        Fr::MODULUS_BIT_SIZE as u64,
        rate,
        full_rounds,
        partial_rounds,
        skip_matrices,
    );

    PoseidonConfig::new(
        full_rounds as usize,
        partial_rounds as usize,
        alpha,
        mds,
        ark,
        rate,
        1, // capacity = 1 (extra state element for domain separation)
    )
}

/// Compute Poseidon leaf hash for a 48-byte BLS pubkey.
///
/// A BLS12-381 G1 compressed pubkey is 48 bytes (384 bits), but the
/// BLS12-381 scalar field Fr is only ~255 bits. A single Fr element
/// cannot hold 48 bytes, so the pubkey is split into two 24-byte halves
/// (each fits within 255 bits with room to spare):
///   - `lo` = first 24 bytes (big-endian) as Fr
///   - `hi` = last 24 bytes (big-endian) as Fr
///
/// These two field elements are then hashed with Poseidon CRH (rate=2).
/// This matches the in-circuit leaf computation that CIR-002 constrains.
///
/// # Cross-references
///
/// - Corresponding SHA-256 leaf: [`super::sparse::active_leaf`]
/// - In-circuit constraint: CIR-002 in [`crate::prover::circuit::ConsensusCircuit`]
pub fn poseidon_leaf(config: &PoseidonConfig<Fr>, pubkey: &[u8; 48]) -> Fr {
    // Split pubkey into two field elements that each fit in Fr (~255 bits)
    let lo = Fr::from(BigUint::from_bytes_be(&pubkey[0..24]));
    let hi = Fr::from(BigUint::from_bytes_be(&pubkey[24..48]));
    CRH::<Fr>::evaluate(config, [lo, hi]).expect("Poseidon CRH failed")
}

/// Compute Poseidon two-to-one hash (for internal Merkle nodes).
///
/// Combines a left child hash and a right child hash into a parent hash.
/// The order matters: `hash(left, right) != hash(right, left)`.
/// At each tree level, the smaller index child is always `left`.
///
/// # Cross-references
///
/// - SHA-256 equivalent: SHA-256 concatenation in [`super::sparse::SparseMerkleTree`]
pub fn poseidon_two_to_one(config: &PoseidonConfig<Fr>, left: Fr, right: Fr) -> Fr {
    TwoToOneCRH::<Fr>::compress(config, left, right).expect("Poseidon 2:1 failed")
}

/// Empty leaf value in the Poseidon tree (hash of zero pubkey).
///
/// Analogous to [`super::sparse::EMPTY_LEAF`] for the SHA-256 tree.
/// Both trees use the same convention: an empty slot contains the hash
/// of a 48-byte all-zero pubkey, not a raw zero field element.
pub fn poseidon_empty_leaf(config: &PoseidonConfig<Fr>) -> Fr {
    poseidon_leaf(config, &[0u8; 48])
}

/// A Poseidon Merkle proof (sibling hashes from leaf to root).
///
/// Analogous to [`super::proof::MerkleProof`] for the SHA-256 tree,
/// but using Fr field elements instead of 32-byte hashes.
/// The proof is consumed as a private witness in the Groth16 circuit (CIR-002).
#[derive(Debug, Clone)]
pub struct PoseidonMerkleProof {
    /// Sibling Fr values at each level, from leaf (level 0) to root (level depth-1).
    pub siblings: Vec<Fr>,
    /// Slot index (determines left/right placement at each level via bit decomposition).
    pub index: u64,
}

/// Poseidon sparse Merkle tree for off-chain witness generation.
///
/// Fixed depth, uses slot assignment from [`super::sparse::compute_slot`].
/// Separate from the on-chain SHA-256 SMT ([`super::sparse::SparseMerkleTree`]).
///
/// This tree is used exclusively for generating Merkle proofs that serve
/// as private witnesses in the Groth16 circuit (CIR-002). The root of
/// this tree becomes a public input to the circuit.
///
/// Source: [DESIGN_DECISIONS.md Decision 1](../../docs/requirements/domains/circuit/DESIGN_DECISIONS.md)
/// (Phase 2: Poseidon Merkle proofs).
pub struct PoseidonMerkleTree {
    config: PoseidonConfig<Fr>,
    depth: u32,
    /// Leaf values indexed by slot. Missing slots use [`poseidon_empty_leaf`].
    leaves: std::collections::HashMap<u64, Fr>,
    /// Precomputed empty subtree hashes at each level.
    /// `empty_hashes[0]` = empty leaf, `empty_hashes[d]` = hash of two empty subtrees at depth d-1.
    /// Used to short-circuit computation for entirely empty subtrees.
    empty_hashes: Vec<Fr>,
}

impl PoseidonMerkleTree {
    /// Create a new empty Poseidon Merkle tree.
    ///
    /// Precomputes the empty subtree hash at each level (bottom-up):
    ///   - Level 0: `poseidon_empty_leaf` (hash of zero pubkey)
    ///   - Level i: `poseidon_two_to_one(empty[i-1], empty[i-1])`
    ///
    /// This precomputation enables O(1) lookups for empty subtrees during
    /// root computation and proof generation, avoiding redundant hashing.
    pub fn new(config: PoseidonConfig<Fr>, depth: u32) -> Self {
        let empty_leaf = poseidon_empty_leaf(&config);
        let mut empty_hashes = vec![Fr::default(); (depth + 1) as usize];
        empty_hashes[0] = empty_leaf;
        // Build empty subtree hashes bottom-up: each level's empty hash
        // is the Poseidon hash of two copies of the level below's empty hash.
        for i in 1..=depth as usize {
            empty_hashes[i] =
                poseidon_two_to_one(&config, empty_hashes[i - 1], empty_hashes[i - 1]);
        }
        Self {
            config,
            depth,
            leaves: std::collections::HashMap::new(),
            empty_hashes,
        }
    }

    /// Insert a validator pubkey and return its slot.
    ///
    /// The slot is `compute_slot(pubkey) % 2^depth` to fit this tree's depth.
    /// Uses the same slot assignment as the SHA-256 tree to keep validator
    /// positions consistent across both trees.
    ///
    /// # Cross-references
    ///
    /// - Slot assignment: [`super::sparse::compute_slot`]
    /// - SHA-256 insert: [`super::sparse::SparseMerkleTree::insert_validator`]
    pub fn insert_validator(&mut self, pubkey: &[u8; 48]) -> u64 {
        // Reuse the SHA-256 tree's slot assignment so both trees agree
        // on which slot each validator occupies.
        let raw_slot = super::sparse::compute_slot(pubkey);
        let slot = raw_slot % (1u64 << self.depth);
        let leaf = poseidon_leaf(&self.config, pubkey);
        self.leaves.insert(slot, leaf);
        slot
    }

    /// Compute the Merkle root by recursively hashing all subtrees.
    ///
    /// This is the Poseidon root that becomes a public input to the
    /// Groth16 circuit, committed alongside the SHA-256 root.
    pub fn root(&self) -> Fr {
        self.compute_subtree(0, 1u64 << self.depth, self.depth)
    }

    /// Generate a Merkle proof for a given slot (bottom-up sibling collection).
    ///
    /// The proof consists of `depth` sibling hashes. At each level, the
    /// sibling is the hash of the subtree on the opposite side of the path.
    /// The prover passes this proof as a private witness to the Groth16 circuit.
    ///
    /// # Cross-references
    ///
    /// - SHA-256 proof generation: [`super::sparse::SparseMerkleTree`]
    /// - Circuit consumption: CIR-002 in [`crate::prover::circuit::ConsensusCircuit`]
    pub fn prove(&self, slot: u64) -> PoseidonMerkleProof {
        let mut siblings = Vec::with_capacity(self.depth as usize);
        let mut index = slot;

        for level in 0..self.depth {
            // XOR with 1 flips the least significant bit, giving the sibling's
            // index at this level (e.g., if index=4, sibling=5 and vice versa).
            let sibling_index = index ^ 1;
            let sibling_start = sibling_index << level; // not needed, sibling is at same level
                                                        // Compute the sibling's subtree hash (may be an empty subtree)
            let level_size = 1u64 << level;
            let sibling_subtree_start = sibling_index * level_size;
            let sibling_hash = self.compute_subtree(sibling_subtree_start, level_size, level);
            siblings.push(sibling_hash);
            // Move up one level: the parent index is floor(index / 2)
            index /= 2;
        }

        PoseidonMerkleProof {
            siblings,
            index: slot,
        }
    }

    /// Verify a Poseidon Merkle proof (off-chain).
    ///
    /// Recomputes the root from the leaf and sibling hashes, then compares
    /// against the tree's current root. At each level, the index's parity
    /// determines whether the current hash is the left or right child:
    ///   - even index: current is left child, sibling is right
    ///   - odd index: sibling is left child, current is right
    pub fn verify(&self, leaf: Fr, proof: &PoseidonMerkleProof) -> bool {
        let mut current = leaf;
        let mut index = proof.index;
        for sibling in &proof.siblings {
            // Left child is always the one with even index at this level
            current = if index.is_multiple_of(2) {
                poseidon_two_to_one(&self.config, current, *sibling)
            } else {
                poseidon_two_to_one(&self.config, *sibling, current)
            };
            index /= 2;
        }
        current == self.root()
    }

    /// Recursively compute the hash of a subtree rooted at `start` with
    /// the given `size` (number of leaf slots) and `depth`.
    ///
    /// Optimization: if both children are the precomputed empty hash for
    /// their level, return the precomputed empty hash for this level
    /// instead of actually hashing. This makes root computation O(k * depth)
    /// where k is the number of inserted leaves, not O(2^depth).
    fn compute_subtree(&self, start: u64, size: u64, depth: u32) -> Fr {
        if depth == 0 {
            // Base case: return the leaf value, or the empty leaf if no validator
            // occupies this slot.
            self.leaves
                .get(&start)
                .copied()
                .unwrap_or(self.empty_hashes[0])
        } else {
            let half = size / 2;
            let left = self.compute_subtree(start, half, depth - 1);
            let right = self.compute_subtree(start + half, half, depth - 1);
            // Short-circuit: if both children are empty, use precomputed hash
            if left == self.empty_hashes[(depth - 1) as usize]
                && right == self.empty_hashes[(depth - 1) as usize]
            {
                self.empty_hashes[depth as usize]
            } else {
                poseidon_two_to_one(&self.config, left, right)
            }
        }
    }

    /// Get the Poseidon config (needed for circuit constraints).
    ///
    /// The circuit must use the same Poseidon parameters to reproduce
    /// the leaf and internal hashes. Mismatched parameters would produce
    /// a different root, causing proof verification to fail.
    pub fn config(&self) -> &PoseidonConfig<Fr> {
        &self.config
    }

    /// Number of inserted validators (occupied leaf slots).
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Returns `true` if no validators have been inserted.
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }
}
