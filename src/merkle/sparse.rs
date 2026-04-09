//! Sparse Merkle Tree data structure.
//!
//! See [spec-sparse-merkle-tree.md](../../docs/resources/spec-sparse-merkle-tree.md).
//!
//! This module implements a fixed-depth sparse Merkle tree for validator set
//! management. The tree has exactly TREE_DEPTH levels, supporting 2^TREE_DEPTH
//! slots for validators.

use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::MerkleProof;

/// Tree depth (fixed at 32 for 2^32 slots).
///
/// This value is fixed at circuit compile time and cannot change without
/// a new trusted setup ceremony.
///
/// Source: spec-sparse-merkle-tree.md Lines 46-60
pub const TREE_DEPTH: u32 = 32;

/// Empty leaf hash: sha256([0u8; 48]).
///
/// An empty slot has leaf value sha256(0x00 * 48). This is constant across
/// all empty slots and is curried into the checkpoint singleton puzzle.
///
/// Source: spec-sparse-merkle-tree.md Lines 107-131
pub const EMPTY_LEAF: [u8; 32] = [
    0x17, 0xb0, 0x76, 0x1f, 0x87, 0xb0, 0x81, 0xd5, 0xcf, 0x10, 0x75, 0x7c, 0xcc, 0x89, 0xf1, 0x2b,
    0xe3, 0x55, 0xc7, 0x0e, 0x2e, 0x29, 0xdf, 0x28, 0x8b, 0x65, 0xb3, 0x07, 0x10, 0xdc, 0xbc, 0xd1,
];

/// Compute the slot for a validator pubkey.
///
/// The slot is computed as:
/// 1. Hash the pubkey: `h = sha256(pubkey)`
/// 2. Take first 8 bytes as big-endian u64
/// 3. Reduce mod 2^TREE_DEPTH
///
/// Source: spec-sparse-merkle-tree.md Lines 63-104
pub fn compute_slot(pubkey: &[u8; 48]) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    let hash: [u8; 32] = hasher.finalize().into();
    let n = u64::from_be_bytes(hash[0..8].try_into().unwrap());
    n % (1u64 << TREE_DEPTH)
}

/// Compute the active leaf value for a validator pubkey.
///
/// Active leaf = sha256(pubkey), where pubkey is 48-byte compressed BLS G1 point.
/// This value is stored in the tree when a validator is registered.
///
/// Source: spec-sparse-merkle-tree.md Lines 107-131
pub fn active_leaf(pubkey: &[u8; 48]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    hasher.finalize().into()
}

/// Compute the precomputed empty node hashes for all tree levels.
///
/// - empty_nodes[0] = EMPTY_LEAF (leaf level)
/// - empty_nodes[i] = sha256(empty_nodes[i-1] || empty_nodes[i-1])
/// - empty_nodes[TREE_DEPTH] = empty tree root
///
/// Source: spec-sparse-merkle-tree.md Lines 140-175
pub fn compute_empty_nodes(depth: u32) -> Vec<[u8; 32]> {
    let mut nodes = Vec::with_capacity(depth as usize + 1);
    let mut current = EMPTY_LEAF;
    nodes.push(current);

    for _ in 0..depth {
        let mut hasher = Sha256::new();
        hasher.update(current);
        hasher.update(current);
        current = hasher.finalize().into();
        nodes.push(current);
    }

    nodes
}

/// Sparse Merkle Tree for validator set management.
///
/// The tree has a fixed depth of TREE_DEPTH (32), supporting 2^32 slots.
/// Only active (non-empty) leaves are stored; empty slots use precomputed
/// empty node hashes.
///
/// Source: spec-sparse-merkle-tree.md Lines 178-204
#[derive(Debug, Clone)]
pub struct SparseMerkleTree {
    /// Active leaves: slot -> leaf hash (sha256(pubkey))
    leaves: HashMap<u64, [u8; 32]>,

    /// Precomputed empty node hashes for each level
    empty_nodes: Vec<[u8; 32]>,

    /// Cached root hash (recomputed on modifications)
    root: [u8; 32],
}

impl SparseMerkleTree {
    /// Create a new empty tree.
    ///
    /// The empty tree has all slots set to EMPTY_LEAF, and the root
    /// is empty_nodes[TREE_DEPTH].
    pub fn new() -> Self {
        let empty_nodes = compute_empty_nodes(TREE_DEPTH);
        let root = empty_nodes[TREE_DEPTH as usize];

        Self {
            leaves: HashMap::new(),
            empty_nodes,
            root,
        }
    }

    /// Get the tree depth.
    ///
    /// Always returns TREE_DEPTH (32).
    pub fn depth(&self) -> u32 {
        TREE_DEPTH
    }

    /// Get the current root hash.
    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    /// Get the number of active (non-empty) leaves.
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Check if the tree is empty (no active leaves).
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Get the leaf value at a slot, or EMPTY_LEAF if empty.
    pub fn get_leaf(&self, slot: u64) -> [u8; 32] {
        self.leaves.get(&slot).copied().unwrap_or(EMPTY_LEAF)
    }

    /// Insert a leaf at the given slot.
    ///
    /// Returns the proof for the new leaf position.
    pub fn insert(&mut self, slot: u64, leaf: [u8; 32]) -> MerkleProof {
        self.leaves.insert(slot, leaf);
        self.recompute_root();
        self.prove(slot)
    }

    /// Insert a validator by their pubkey.
    ///
    /// Computes the slot from pubkey hash and stores active_leaf(pubkey).
    /// Returns the proof for the new leaf position.
    pub fn insert_validator(&mut self, pubkey: &[u8; 48]) -> MerkleProof {
        let slot = compute_slot(pubkey);
        let leaf = active_leaf(pubkey);
        self.insert(slot, leaf)
    }

    /// Remove a validator by their pubkey.
    ///
    /// Computes the slot from pubkey hash and removes the leaf.
    /// Returns the proof for the now-empty slot.
    pub fn remove_validator(&mut self, pubkey: &[u8; 48]) -> MerkleProof {
        let slot = compute_slot(pubkey);
        self.remove(slot)
    }

    /// Remove a leaf at the given slot (set to empty).
    ///
    /// Returns the proof for the now-empty slot.
    pub fn remove(&mut self, slot: u64) -> MerkleProof {
        self.leaves.remove(&slot);
        self.recompute_root();
        self.prove(slot)
    }

    /// Generate a proof for the given slot.
    ///
    /// The proof contains exactly TREE_DEPTH sibling hashes.
    pub fn prove(&self, slot: u64) -> MerkleProof {
        let mut siblings = Vec::with_capacity(TREE_DEPTH as usize);
        let mut current_slot = slot;

        for level in 0..TREE_DEPTH {
            let sibling_slot = current_slot ^ 1; // flip lowest bit to get sibling
            let sibling_hash = self.compute_node_hash(sibling_slot, level);
            siblings.push(sibling_hash);
            current_slot >>= 1; // move up to parent
        }

        let leaf = self.get_leaf(slot);

        MerkleProof {
            leaf,
            slot,
            siblings,
        }
    }

    /// Recompute the root hash from all active leaves.
    fn recompute_root(&mut self) {
        self.root = self.compute_subtree_hash(0, 1u64 << TREE_DEPTH, TREE_DEPTH);
    }

    /// Compute the hash of a subtree.
    ///
    /// Source: spec-sparse-merkle-tree.md Lines 206-239
    fn compute_subtree_hash(&self, start: u64, end: u64, level: u32) -> [u8; 32] {
        if level == 0 {
            // Leaf level
            return self.leaves.get(&start).copied().unwrap_or(EMPTY_LEAF);
        }

        // Check if this subtree has any active leaves
        let has_active = self.leaves.keys().any(|&k| k >= start && k < end);
        if !has_active {
            // Return precomputed empty subtree hash
            return self.empty_nodes[level as usize];
        }

        // Compute left and right child hashes
        let mid = start + ((end - start) >> 1);
        let left = self.compute_subtree_hash(start, mid, level - 1);
        let right = self.compute_subtree_hash(mid, end, level - 1);

        // Parent = sha256(left || right)
        // Critical: left child always comes first
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }

    /// Compute the node hash at a given slot and level.
    fn compute_node_hash(&self, slot: u64, level: u32) -> [u8; 32] {
        let subtree_size = 1u64 << level;
        let start = slot * subtree_size;
        let end = start + subtree_size;
        self.compute_subtree_hash(start, end, level)
    }
}

impl Default for SparseMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_leaf_constant() {
        // Verify EMPTY_LEAF matches sha256([0u8; 48])
        let mut hasher = Sha256::new();
        hasher.update([0u8; 48]);
        let computed: [u8; 32] = hasher.finalize().into();
        assert_eq!(EMPTY_LEAF, computed);
    }

    #[test]
    fn test_empty_nodes_computation() {
        let empty_nodes = compute_empty_nodes(TREE_DEPTH);
        assert_eq!(empty_nodes.len(), (TREE_DEPTH + 1) as usize);
        assert_eq!(empty_nodes[0], EMPTY_LEAF);
    }

    #[test]
    fn test_empty_tree_root() {
        let tree = SparseMerkleTree::new();
        let empty_nodes = compute_empty_nodes(TREE_DEPTH);
        assert_eq!(tree.root(), empty_nodes[TREE_DEPTH as usize]);
    }
}
