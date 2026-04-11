//! Poseidon-based Merkle tree for in-circuit verification (CIR-002).
//!
//! This is a SEPARATE tree from the on-chain SHA-256 SMT (sparse.rs).
//! It uses the Poseidon hash (ZK-friendly, ~300 R1CS constraints per hash)
//! for leaf and internal node computation. The Poseidon root is used as
//! a private witness in the Groth16 circuit.
//!
//! See DESIGN_DECISIONS.md Decision 1 for rationale.

use ark_bls12_381::Fr;
use ark_crypto_primitives::crh::poseidon::{TwoToOneCRH, CRH};
use ark_crypto_primitives::crh::{CRHScheme, TwoToOneCRHScheme};
use ark_crypto_primitives::sponge::poseidon::PoseidonConfig;
use ark_ff::PrimeField;
use num_bigint::BigUint;

/// Generate Poseidon parameters for BLS12-381 Fr.
///
/// Uses rate=2, alpha=17, 8 full rounds, 31 partial rounds.
/// These parameters provide 128-bit security.
pub fn poseidon_config() -> PoseidonConfig<Fr> {
    use ark_crypto_primitives::sponge::poseidon::find_poseidon_ark_and_mds;

    let rate = 2;
    let full_rounds: u64 = 8;
    let partial_rounds: u64 = 31;
    let alpha = 17;
    let skip_matrices = 0;

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
        1, // capacity = 1
    )
}

/// Compute Poseidon leaf hash for a 48-byte BLS pubkey.
///
/// Splits the pubkey into two Fr field elements (24 bytes each, big-endian)
/// and hashes them with Poseidon CRH.
pub fn poseidon_leaf(config: &PoseidonConfig<Fr>, pubkey: &[u8; 48]) -> Fr {
    let lo = Fr::from(BigUint::from_bytes_be(&pubkey[0..24]));
    let hi = Fr::from(BigUint::from_bytes_be(&pubkey[24..48]));
    CRH::<Fr>::evaluate(config, [lo, hi]).expect("Poseidon CRH failed")
}

/// Compute Poseidon two-to-one hash (for internal Merkle nodes).
pub fn poseidon_two_to_one(config: &PoseidonConfig<Fr>, left: Fr, right: Fr) -> Fr {
    TwoToOneCRH::<Fr>::compress(config, left, right).expect("Poseidon 2:1 failed")
}

/// Empty leaf value in the Poseidon tree (hash of zero pubkey).
pub fn poseidon_empty_leaf(config: &PoseidonConfig<Fr>) -> Fr {
    poseidon_leaf(config, &[0u8; 48])
}

/// A Poseidon Merkle proof (sibling hashes from leaf to root).
#[derive(Debug, Clone)]
pub struct PoseidonMerkleProof {
    /// Sibling Fr values at each level, from leaf to root.
    pub siblings: Vec<Fr>,
    /// Slot index (determines left/right at each level).
    pub index: u64,
}

/// Poseidon sparse Merkle tree for off-chain witness generation.
///
/// Fixed depth, uses slot assignment from `compute_slot(pubkey)`.
/// Separate from the on-chain SHA-256 SMT.
pub struct PoseidonMerkleTree {
    config: PoseidonConfig<Fr>,
    depth: u32,
    /// Leaf values indexed by slot. Missing slots = empty leaf.
    leaves: std::collections::HashMap<u64, Fr>,
    /// Precomputed empty subtree hashes at each level.
    empty_hashes: Vec<Fr>,
}

impl PoseidonMerkleTree {
    /// Create a new empty Poseidon Merkle tree.
    pub fn new(config: PoseidonConfig<Fr>, depth: u32) -> Self {
        let empty_leaf = poseidon_empty_leaf(&config);
        let mut empty_hashes = vec![Fr::default(); (depth + 1) as usize];
        empty_hashes[0] = empty_leaf;
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
    pub fn insert_validator(&mut self, pubkey: &[u8; 48]) -> u64 {
        let raw_slot = super::sparse::compute_slot(pubkey);
        let slot = raw_slot % (1u64 << self.depth);
        let leaf = poseidon_leaf(&self.config, pubkey);
        self.leaves.insert(slot, leaf);
        slot
    }

    /// Compute the Merkle root.
    pub fn root(&self) -> Fr {
        self.compute_subtree(0, 1u64 << self.depth, self.depth)
    }

    /// Generate a Merkle proof for a given slot (bottom-up sibling collection).
    pub fn prove(&self, slot: u64) -> PoseidonMerkleProof {
        let mut siblings = Vec::with_capacity(self.depth as usize);
        let mut index = slot;

        for level in 0..self.depth {
            let sibling_index = index ^ 1; // flip the lowest bit to get sibling
            let sibling_start = sibling_index << level; // not needed, sibling is at same level
                                                        // Compute the sibling's subtree hash
            let level_size = 1u64 << level;
            let sibling_subtree_start = sibling_index * level_size;
            let sibling_hash = self.compute_subtree(sibling_subtree_start, level_size, level);
            siblings.push(sibling_hash);
            index /= 2;
        }

        PoseidonMerkleProof {
            siblings,
            index: slot,
        }
    }

    /// Verify a Poseidon Merkle proof (off-chain).
    pub fn verify(&self, leaf: Fr, proof: &PoseidonMerkleProof) -> bool {
        let mut current = leaf;
        let mut index = proof.index;
        for sibling in &proof.siblings {
            current = if index.is_multiple_of(2) {
                poseidon_two_to_one(&self.config, current, *sibling)
            } else {
                poseidon_two_to_one(&self.config, *sibling, current)
            };
            index /= 2;
        }
        current == self.root()
    }

    fn compute_subtree(&self, start: u64, size: u64, depth: u32) -> Fr {
        if depth == 0 {
            self.leaves
                .get(&start)
                .copied()
                .unwrap_or(self.empty_hashes[0])
        } else {
            let half = size / 2;
            let left = self.compute_subtree(start, half, depth - 1);
            let right = self.compute_subtree(start + half, half, depth - 1);
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
    pub fn config(&self) -> &PoseidonConfig<Fr> {
        &self.config
    }

    /// Number of inserted validators.
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }
}
