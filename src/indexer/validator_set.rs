//! Validator set building with lineage verification.
//!
//! REG-002: Registration coin lineage is verified off-chain by the indexer.
//! A registration coin is only valid if its parent coin ID is a known network
//! coin spend. This prevents attackers from creating fake registration coins
//! directly without going through the network coin.
//!
//! See [spec-indexer.md](../../docs/resources/spec-indexer.md) — Registration
//! Coin Detection and Lineage Verification.

use std::collections::HashSet;

use chia_protocol::{Bytes32, Coin};
use clvm_utils::{curry_tree_hash, tree_hash_atom, tree_hash_pair, TreeHash};

use crate::error::ConsensusResult;
use crate::indexer::IndexerCache;
use crate::state::ValidatorSet;

/// REG-002: Tracks network coin spends and verifies registration coin lineage.
///
/// The lineage check is the core security mechanism: a registration coin is
/// only valid if its `parent_coin_id` is a known network coin spend. Coins
/// without valid lineage are silently ignored.
///
/// See [spec-indexer.md Lines 266-355](../../docs/resources/spec-indexer.md) —
/// Registration Coin Detection and Lineage Verification.
#[derive(Debug, Clone)]
pub struct LineageChecker {
    /// All valid network coin spend coin IDs. O(1) lookup.
    /// Each entry corresponds to one registration event where the network
    /// coin was spent to create a registration coin.
    network_coin_spend_ids: HashSet<Bytes32>,
}

impl LineageChecker {
    /// Create a new empty lineage checker.
    pub fn new() -> Self {
        Self {
            network_coin_spend_ids: HashSet::new(),
        }
    }

    /// Record a network coin spend. Called by the indexer when it detects
    /// a network coin singleton spend on-chain. The spend ID is the coin ID
    /// of the network coin that was spent (not the newly created coin).
    pub fn record_network_coin_spend(&mut self, spend_id: Bytes32) {
        self.network_coin_spend_ids.insert(spend_id);
    }

    /// Check if a given coin ID is a known network coin spend.
    pub fn is_network_coin_spend(&self, coin_id: &Bytes32) -> bool {
        self.network_coin_spend_ids.contains(coin_id)
    }

    /// REG-002: Verify that a registration coin's parent is a valid network
    /// coin spend. Returns true if and only if `parent_coin_id` is in the
    /// set of recorded network coin spends.
    ///
    /// This is the core lineage check. A registration coin that fails this
    /// check MUST be ignored — it was not created through the approved
    /// registration process via the network coin.
    pub fn verify_registration_coin_lineage(&self, parent_coin_id: &Bytes32) -> bool {
        self.network_coin_spend_ids.contains(parent_coin_id)
    }

    /// Number of recorded network coin spends (for diagnostics).
    pub fn network_coin_spend_count(&self) -> usize {
        self.network_coin_spend_ids.len()
    }
}

impl Default for LineageChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// IDX-002: Registration Coin Puzzle Hash Computation
// ============================================================================

/// Compute the expected puzzle hash for a registration coin.
///
/// The registration coin puzzle is curried with (pubkey, checkpoint_singleton_id).
/// This must match the on-chain `curry_tree_hash` in `network_coin_inner.rue`.
///
/// Formula: `curry_tree_hash(reg_mod_hash, [tree_hash(pubkey), tree_hash(ckpt_id)])`
pub fn registration_coin_puzzle_hash(
    reg_mod_hash: Bytes32,
    pubkey: &[u8; 48],
    checkpoint_singleton_id: Bytes32,
) -> Bytes32 {
    let base = TreeHash::new(reg_mod_hash.into());
    let pk_hash = tree_hash_atom(pubkey);
    let ckpt_hash = tree_hash_atom(&<[u8; 32]>::from(checkpoint_singleton_id));
    let result: [u8; 32] = curry_tree_hash(base, &[pk_hash, ckpt_hash]).into();
    result.into()
}

// ============================================================================
// IDX-002: Registration Coin Parsing with Lineage Verification
// ============================================================================

/// A parsed registration coin record.
#[derive(Debug, Clone)]
pub struct RegistrationCoinRecord {
    /// The registration coin itself.
    pub coin: Coin,
    /// The validator's BLS public key (48 bytes).
    pub pubkey: Vec<u8>,
}

/// Try to parse a coin as a valid registration coin.
///
/// Performs the full IDX-002 lineage verification:
/// 1. Parent must be a known network coin spend
/// 2. Puzzle hash must match expected for the given pubkey
/// 3. Amount must equal required collateral
///
/// Returns `None` if the coin is not a valid registration coin (silently ignored).
/// The `pubkey` is provided by the caller (extracted from the parent spend memo).
pub fn try_parse_registration_coin(
    checker: &LineageChecker,
    coin: &Coin,
    pubkey: &[u8; 48],
    reg_mod_hash: Bytes32,
    checkpoint_singleton_id: Bytes32,
    required_collateral: u64,
) -> Option<RegistrationCoinRecord> {
    // Step 1: Check parent is a known network coin spend
    if !checker.verify_registration_coin_lineage(&coin.parent_coin_info) {
        return None;
    }

    // Step 2: Verify puzzle hash matches expected for this pubkey
    let expected_ph = registration_coin_puzzle_hash(reg_mod_hash, pubkey, checkpoint_singleton_id);
    if coin.puzzle_hash != expected_ph {
        return None;
    }

    // Step 3: Verify collateral amount
    if coin.amount != required_collateral {
        return None;
    }

    Some(RegistrationCoinRecord {
        coin: *coin,
        pubkey: pubkey.to_vec(),
    })
}

// ============================================================================
// IDX-003: Merkle Consistency Verification
// ============================================================================

/// Verify that a set of validator pubkeys produces the expected Merkle root.
///
/// Rebuilds the sparse Merkle tree from the given pubkeys and compares the
/// computed root against `on_chain_root`. Returns the tree on success, or
/// `StateMismatch` error on mismatch.
///
/// Insertion order does not matter — the SMT assigns slots deterministically
/// based on `compute_slot(pubkey)`.
pub fn verify_merkle_consistency(
    pubkeys: &[[u8; 48]],
    on_chain_root: Bytes32,
) -> ConsensusResult<crate::merkle::SparseMerkleTree> {
    use crate::merkle::SparseMerkleTree;

    let mut tree = SparseMerkleTree::new();
    for pk in pubkeys {
        tree.insert_validator(pk);
    }

    let computed_root: [u8; 32] = tree.root();
    let expected: [u8; 32] = on_chain_root.into();

    if computed_root != expected {
        return Err(crate::error::ConsensusError::StateMismatch);
    }

    Ok(tree)
}

/// Build the current validator set from cached state.
pub fn build_validator_set(_cache: &IndexerCache) -> ConsensusResult<ValidatorSet> {
    // TODO: Implement with lineage verification using LineageChecker
    todo!()
}
