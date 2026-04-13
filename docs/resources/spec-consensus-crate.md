# chia-l2-consensus - Unified Crate Specification

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Integrates** | [spec-network-coin](spec-network-coin.md) | puzzles/network_coin.rs: deploy_network_coin(), register_validator(), fetch_network_coin_state(), fetch_valid_registration_coins() |
| **Integrates** | [spec-registration-coin](spec-registration-coin.md) | puzzles/registration_coin.rs: spend_registration_coin(), fetch_registration_coin(), registration_coin_puzzle_hash(), registration_coin_id() |
| **Integrates** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | puzzles/checkpoint.rs: spend_checkpoint_singleton(), spend_checkpoint_singleton_membership_query(), fetch_checkpoint_singleton_state(), serialize_vk_for_clvm() |
| **Integrates** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | merkle/sparse.rs: SparseMerkleTree, MerkleProof, validator_slot(), EMPTY_LEAF, compute_empty_nodes(), all proof operations |
| **Integrates** | [spec-wire-format](spec-wire-format.md) | prover/serialize.rs: ClvmProof, ClvmVerificationKey, serialize_proof(), serialize_vk(), compute_checkpoint_message(), compute_membership_announcement(), aggregate_pubkeys(), aggregate_signatures(), bytes_to_scalar(), compute_vk_input() |
| **Integrates** | [spec-groth16-circuit](spec-groth16-circuit.md) | prover/circuit.rs: ConsensusCircuit, generate_proof(); prover/setup.rs: run_setup(), load_proving_key(), load_verification_key() |
| **Integrates** | [spec-indexer](spec-indexer.md) | indexer/: IndexerState, sync algorithm, lineage verification, reorg handling, IndexerCache, build_validator_set() |
| **Depends on** | [spec-trusted-setup](spec-trusted-setup.md) | Proving key loaded at startup. VK curried into checkpoint singleton at deployment. |
| **Depends on** | [spec-clvm-costs](spec-clvm-costs.md) | Cost figures documented inline throughout; fee planning for operations |
| **Enables** | [spec-l2-integration](spec-l2-integration.md) | The L2 system uses only ConsensusClient from this crate |
| **Enables** | [spec-validator-onboarding](spec-validator-onboarding.md) | Validator tooling wraps this crate's public API |
| **Referenced by** | [spec-deployment-runbook](spec-deployment-runbook.md) | ConsensusClient.deploy() implements the deployment steps |
| **Referenced by** | [spec-security](spec-security.md) | Lineage verification, Merkle consistency check, proof generation centralization |

---

## Overview

`chia-l2-consensus` is a single Rust crate that packages all three L1 puzzle
implementations, their driver code, the Groth16 circuit and prover, the sparse
Merkle tree, and the chain indexer into a unified public interface that the L2
system calls directly. The L2 system uses only `ConsensusClient` and the types
it returns. Everything else is `pub(crate)` or private.

The crate owns the full lifecycle: network deployment, validator registration,
checkpoint construction, validator set construction with lineage verification,
collateral recovery, and on-chain state indexing. Every cross-cutting concern
that individual puzzle specs leave as an implementation detail — memo
conventions, lineage verification, Merkle tree management, state indexing,
proof generation, serialization — is handled inside this crate.

**All public methods that produce coin spends return `SpendBundle` values.
The crate never broadcasts transactions. The importing project is responsible
for submitting bundles to a Chia full node via `push_tx()` or equivalent.**

```
L2 system
    |
    | uses only: ConsensusClient, NetworkConfig, ValidatorSet,
    |            SpendBundle, Bytes32, PublicKey
    v
ConsensusClient (public facade)
    |
    | internally coordinates:
    |   puzzles::network_coin      (→ see spec-network-coin)
    |   puzzles::registration_coin (→ see spec-registration-coin)
    |   puzzles::checkpoint        (→ see spec-checkpoint-singleton)
    |   merkle::sparse             (→ see spec-sparse-merkle-tree)
    |   prover::circuit            (→ see spec-groth16-circuit)
    |   prover::serialize          (→ see spec-wire-format)
    |   indexer                    (→ see spec-indexer)
    v
Chia full node RPC
```

---

## Cargo.toml

```toml
[package]
name    = "chia-l2-consensus"
version = "0.1.0"
edition = "2021"

[dependencies]
# Chia
chia-wallet-sdk = "0.18"
chia-protocol   = "0.18"
chia-puzzles    = "0.18"
clvm-traits     = "0.18"
clvmr           = "0.6"

# ZK proving (→ see spec-groth16-circuit — Dependencies)
ark-groth16           = "0.4"
ark-bls12-381         = "0.4"
ark-r1cs-std          = "0.4"
ark-relations         = "0.4"
ark-ff                = "0.4"
ark-ec                = "0.4"
ark-std               = "0.4"
ark-serialize         = "0.4"
ark-crypto-primitives = { version = "0.4", features = ["crh"] }

# BLS aggregation off-chain (→ see spec-wire-format — Aggregate Signature)
blst = "0.3"

# Async
tokio   = { version = "1", features = ["full"] }
futures = "0.3"

# Serialization
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
hex          = "0.4"
num-bigint   = "0.4"

# Error handling
thiserror = "1"
anyhow    = "1"

sha2 = "0.10"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
rand  = "0.8"
```

---

## Crate Structure

```
chia-l2-consensus/
  Cargo.toml
  src/
    lib.rs                    - public API re-exports
    client.rs                 - ConsensusClient, the main entry point
    config.rs                 - NetworkConfig, all deployment parameters
    state.rs                  - NetworkState, CheckpointSingletonState, NetworkCoinState
    error.rs                  - ConsensusError enum
    puzzles/
      mod.rs
      network_coin.rs         - per spec-network-coin
      registration_coin.rs    - per spec-registration-coin
      checkpoint.rs           - per spec-checkpoint-singleton
    merkle/
      mod.rs
      sparse.rs               - SparseMerkleTree per spec-sparse-merkle-tree
      proof.rs                - MerkleProof type
    prover/
      mod.rs
      circuit.rs              - ConsensusCircuit per spec-groth16-circuit
      setup.rs                - run_setup(), load_proving_key() per spec-trusted-setup
      prove.rs                - generate_proof()
      serialize.rs            - ClvmProof, ClvmVerificationKey per spec-wire-format
    indexer/
      mod.rs                  - IndexerState, sync() per spec-indexer
      chain.rs                - raw chain queries
      validator_set.rs        - build_validator_set(), lineage verification
      reorg.rs                - handle_reorg(), full_reindex()
      cache.rs                - IndexerCache, persistent state
  tests/
    integration.rs            - full end-to-end test
```

---

## Error Type

```rust
// src/error.rs

#[derive(thiserror::Error, Debug)]
pub enum ConsensusError {
    #[error("network not deployed - call deploy() first")]
    NotDeployed,

    #[error("validator already registered: {0}")]
    AlreadyRegistered(String),

    #[error("validator not found in active set: {0}")]
    ValidatorNotFound(String),

    /// Majority threshold enforced here before proof generation.
    /// Circuit enforces 2k > validator_count as Constraint 3.
    /// (→ see spec-groth16-circuit — Constraint 3: Majority Threshold)
    #[error("below majority threshold: need more than {count}/2 signers, got {actual}")]
    BelowThreshold { count: u64, actual: usize },

    /// Local Merkle tree does not match the validator_merkle_root in the
    /// checkpoint singleton. Indexer is out of sync — trigger full_reindex().
    /// (→ see spec-indexer — Merkle Root Consistency Check)
    #[error("on-chain state mismatch: local merkle root does not match on-chain root")]
    StateMismatch,

    /// Registration coin parent coin ID does not trace back to a network coin spend.
    /// (→ see spec-security — Lineage Proof Enforcement)
    #[error("invalid lineage proof for registration coin")]
    InvalidLineage,

    /// Merkle proof verification failed, either membership or non-membership.
    /// (→ see spec-sparse-merkle-tree — Proof Verification)
    #[error("merkle proof verification failed")]
    InvalidMerkleProof,

    /// Groth16 proof generation failed. May be OOM, proving key not loaded,
    /// or unsatisfiable constraints (indicates a circuit bug).
    /// (→ see spec-groth16-circuit — Proof Generation)
    #[error("proof generation failed: {0}")]
    ProvingError(String),

    #[error("node rpc error: {0}")]
    NodeError(String),

    /// Serialization mismatch between Rust types and CLVM format.
    /// (→ see spec-wire-format — Common Mistakes)
    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("spend bundle rejected by node: {0}")]
    SpendRejected(String),

    /// Puzzle hash mismatch during registration coin detection.
    /// (→ see spec-indexer — Registration Coin Detection)
    #[error("puzzle hash mismatch for registration coin")]
    PuzzleHashMismatch,

    /// Indexer cache is corrupted or incompatible.
    #[error("indexer cache error: {0}")]
    CacheError(String),

    /// Slot collision: two validators hash to the same Merkle tree slot.
    /// (→ see spec-sparse-merkle-tree — Slot Assignment: Slot collisions)
    #[error("slot collision for pubkey: {0}")]
    SlotCollision(String),
}

pub type ConsensusResult<T> = Result<T, ConsensusError>;
```

---

## Configuration

```rust
// src/config.rs

use ark_groth16::VerifyingKey;

/// All parameters that define a specific L2 network deployment.
/// Fixed at deployment time and never change for the life of the deployment.
/// Produced by ConsensusClient::deploy() and saved to disk.
/// Loaded on every subsequent node startup.
/// Changing collateral_amount, tree_depth, or VK requires a full redeployment.
/// (→ see spec-deployment-runbook)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfig {
    /// Launcher ID of the network coin singleton.
    /// Used to derive the current network coin puzzle hash and find the coin on-chain.
    /// (→ see spec-network-coin — Deployment)
    pub network_coin_launcher_id:   Bytes32,

    /// Launcher ID of the checkpoint singleton.
    /// Used to find the current checkpoint coin and derive checkpoint_singleton_id().
    /// (→ see spec-checkpoint-singleton — Deployment)
    pub checkpoint_launcher_id:     Bytes32,

    /// Tree hash of the base registration coin puzzle before currying.
    /// Every valid registration coin derives its puzzle hash from this.
    /// Used by the indexer for lineage verification.
    /// (→ see spec-registration-coin — Computing the Registration Coin Puzzle Hash)
    /// (→ see spec-indexer — Registration Coin Detection)
    pub registration_coin_mod_hash: Bytes32,

    /// Tree hash of the base checkpoint inner puzzle before currying.
    /// Used to rebuild the inner puzzle with current state on each spend.
    /// (→ see spec-checkpoint-singleton — Deployment)
    pub checkpoint_inner_mod_hash:  Bytes32,

    /// Required collateral per validator in mojos.
    /// Enforced exactly by the network coin puzzle on every registration spend.
    /// Cannot change without redeploying the network coin.
    /// (→ see spec-network-coin — Curried In Parameters: COLLATERAL_AMOUNT)
    pub collateral_amount:          u64,

    /// Depth of the sparse Merkle tree.
    /// Fixed at circuit compile time. Must match TREE_DEPTH in the Groth16
    /// circuit and the depth curried into the checkpoint singleton.
    /// (→ see spec-sparse-merkle-tree — Parameters: TREE_DEPTH)
    /// (→ see spec-groth16-circuit — Circuit Parameters: TREE_DEPTH)
    /// (→ see spec-checkpoint-singleton — Curried In Parameters: TREE_DEPTH)
    pub tree_depth:                 u32,

    /// Maximum simultaneous signers supported by the Groth16 circuit.
    /// Fixed at trusted setup time. Cannot increase without a new ceremony.
    /// Actual k can be anything from majority threshold up to this value.
    /// (→ see spec-groth16-circuit — Circuit Parameters: MAX_SIGNERS)
    /// (→ see spec-trusted-setup — When to Rerun the Ceremony)
    pub max_signers:                usize,

    /// Groth16 verification key from the trusted setup ceremony.
    /// Stored as hex-encoded JSON per spec-wire-format — VK Format — Storage Format.
    /// This exact value is curried into the checkpoint singleton at deployment.
    /// Wallets and users should verify the on-chain VK matches this value.
    /// (→ see spec-wire-format — Verification Key Format)
    /// (→ see spec-trusted-setup — What to Publish)
    /// (→ see spec-deployment-runbook — Step 7)
    pub verification_key_hex:       String,

    /// Chia network genesis challenge (mainnet or testnet constant).
    /// Used in AGG_SIG_ME message construction for all signed conditions.
    /// (→ see spec-wire-format — Individual Signatures)
    pub genesis_challenge:          Bytes32,
}

impl NetworkConfig {
    /// Deserialize the verification key from hex for use in Arkworks.
    pub fn verification_key(&self) -> ConsensusResult<VerifyingKey<ark_bls12_381::Bls12_381>> {
        deserialize_vk_from_hex(&self.verification_key_hex)
            .map_err(|e| ConsensusError::SerializationError(e.to_string()))
    }

    /// The checkpoint singleton coin ID derived from its launcher.
    /// This is the ID curried into every registration coin at creation time.
    /// Distinct from checkpoint_launcher_id — this is the actual coin ID.
    /// (→ see spec-registration-coin — Curried In Parameters: CHECKPOINT_SINGLETON_ID)
    /// (→ see spec-wire-format — Common Mistakes: Coin ID vs launcher ID)
    pub fn checkpoint_singleton_id(&self) -> Bytes32 {
        singleton_launcher_coin_id(self.checkpoint_launcher_id)
    }
}
```

---

## On-Chain State Types

```rust
// src/state.rs

/// Current on-chain state of the network coin singleton.
/// Refreshed by sync() on every call.
/// (→ see spec-network-coin — Querying the Network Coin State)
#[derive(Debug, Clone)]
pub struct NetworkCoinState {
    /// The current unspent network coin.
    pub coin:          Coin,
    /// The inner puzzle with config curried in. Used to build spends.
    pub inner_puzzle:  NodePtr,
    /// Lineage proof needed for the next singleton spend.
    pub lineage_proof: LineageProof,
}

/// Current on-chain state of the checkpoint singleton.
/// Refreshed by sync() on every call.
/// State changes on every checkpoint spend (puzzle hash changes because
/// state is curried in). Track via indexer checkpoint history.
/// (→ see spec-checkpoint-singleton — Singleton State)
/// (→ see spec-indexer — Checkpoint State Updates)
#[derive(Debug, Clone)]
pub struct CheckpointSingletonState {
    /// The current unspent checkpoint singleton coin.
    pub coin:                  Coin,
    /// Lineage proof needed for the next singleton spend.
    pub lineage_proof:         LineageProof,
    /// Current L2 state root committed on-chain.
    pub state_root:            Bytes32,
    /// Current epoch, incremented by 1 on every checkpoint spend.
    pub epoch:                 u64,
    /// Sparse Merkle root of the current active validator set.
    /// Canonical tree spec: spec-sparse-merkle-tree.
    pub validator_merkle_root: Bytes32,
    /// Number of active validators. Used for majority threshold: 2k > validator_count.
    pub validator_count:       u64,
}

/// All relevant cached on-chain state.
/// Populated by sync() via the indexer. Always call sync() before any
/// operation that depends on current state.
#[derive(Debug, Clone)]
pub struct NetworkState {
    pub network_coin:  NetworkCoinState,
    pub checkpoint:    CheckpointSingletonState,
    /// Active validator set. Only includes validators whose registration
    /// coin lineage traces back to the network coin.
    /// (→ see spec-security — Lineage Proof Enforcement)
    pub validators:    ValidatorSet,
    /// Sparse Merkle tree built from the current validator set.
    /// Root is verified against checkpoint.validator_merkle_root on every sync.
    /// (→ see spec-sparse-merkle-tree)
    /// (→ see spec-indexer — Merkle Root Consistency Check)
    pub merkle_tree:   SparseMerkleTree,
    /// Block height at which this state was last synced.
    pub synced_at:     u32,
}

/// The verified active validator set.
/// Sorted by pubkey bytes for deterministic Merkle slot ordering.
/// (→ see spec-indexer — Validator Set Construction)
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    pub validators: Vec<ValidatorInfo>,
}

/// A single active validator with their on-chain registration coin.
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    /// BLS12-381 G1 pubkey, 48 bytes compressed.
    /// (→ see spec-wire-format — G1 Points)
    pub pubkey:            PublicKey,
    /// The unspent registration coin holding their collateral.
    /// (→ see spec-registration-coin)
    pub registration_coin: Coin,
}

impl ValidatorSet {
    pub fn count(&self) -> u64 { self.validators.len() as u64 }

    pub fn contains(&self, pubkey: &PublicKey) -> bool {
        self.validators.iter().any(|v| &v.pubkey == pubkey)
    }

    pub fn pubkeys(&self) -> Vec<PublicKey> {
        self.validators.iter().map(|v| v.pubkey).collect()
    }
}
```

---

## Merkle Module

The full implementation of the sparse Merkle tree as defined in
[spec-sparse-merkle-tree](spec-sparse-merkle-tree.md). Every detail of this
implementation must match the canonical spec exactly. Any divergence from the
Rue on-chain implementation in the checkpoint singleton puzzle will cause
proof verification failures.

```rust
// src/merkle/sparse.rs

use sha2::{Sha256, Digest};

/// sha256(0x00 * 48) — the empty leaf value for all empty slots.
/// This exact constant is curried into the checkpoint singleton as EMPTY_LEAF_HASH.
/// (→ see spec-checkpoint-singleton — Curried In Parameters: EMPTY_LEAF_HASH)
/// Verify this value before deployment per spec-deployment-runbook — Step 7.
pub const EMPTY_LEAF: [u8; 32] = {
    // sha256([0u8; 48]) - computed at compile time
    // = 0x7d4e3eec80026719639ed4dba68916eb94c7a49a053e05c8f9578fe4e5a3d7e
    // Must be verified against spec-sparse-merkle-tree — Leaf Values
    [
        0x7d, 0x4e, 0x3e, 0xec, 0x80, 0x02, 0x67, 0x19,
        0x63, 0x9e, 0xd4, 0xdb, 0xa6, 0x89, 0x16, 0xeb,
        0x94, 0xc7, 0xa4, 0x9a, 0x05, 0x3e, 0x05, 0xc8,
        0xf9, 0x57, 0x8f, 0xe4, 0xe5, 0xa3, 0xd7, 0xe0,
    ]
};

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().into()
}

/// Precomputed empty node hashes for all levels.
/// empty_nodes[0] = EMPTY_LEAF (leaf level)
/// empty_nodes[TREE_DEPTH] = empty root (all slots empty)
/// Used as initial validator_merkle_root at deployment.
/// (→ see spec-sparse-merkle-tree — Empty Node Hashes)
/// (→ see spec-deployment-runbook — Step 8)
pub fn compute_empty_nodes(depth: u32) -> Vec<[u8; 32]> {
    let mut nodes = Vec::with_capacity(depth as usize + 1);
    let mut current = EMPTY_LEAF;
    nodes.push(current);
    for _ in 0..depth {
        current = sha256(&[current, current].concat());
        nodes.push(current);
    }
    nodes
}

/// Compute the deterministic slot index for a validator pubkey.
/// slot = first_8_bytes_as_u64_be(sha256(pubkey)) mod 2^TREE_DEPTH
///
/// CRITICAL: This must be identical in Rust and in the Rue checkpoint singleton.
/// (→ see spec-sparse-merkle-tree — Slot Assignment)
pub fn validator_slot(pubkey: &PublicKey) -> u64 {
    let hash = sha256(pubkey.to_bytes().as_slice());
    let first_8: [u8; 8] = hash[0..8].try_into().unwrap();
    // Use TREE_DEPTH constant from config at call site
    u64::from_be_bytes(first_8)
    // Caller applies mod (1u64 << tree_depth)
}

/// Compute the active leaf value for a validator pubkey: sha256(pubkey).
/// (→ see spec-sparse-merkle-tree — Leaf Values)
pub fn active_leaf(pubkey: &PublicKey) -> [u8; 32] {
    sha256(pubkey.to_bytes().as_slice())
}

/// The sparse Merkle tree. Stores only active (non-empty) slots.
/// Computes roots and proofs on demand using precomputed empty nodes for
/// empty subtrees — critical for O(n*depth) performance rather than O(2^depth).
/// (→ see spec-sparse-merkle-tree — Root Computation)
pub struct SparseMerkleTree {
    /// slot -> sha256(pubkey) for each active validator
    active_slots: HashMap<u64, [u8; 32]>,
    /// Precomputed empty node hashes, indexed by level
    empty_nodes:  Vec<[u8; 32]>,
    depth:        u32,
}

impl SparseMerkleTree {
    pub fn new(depth: u32) -> Self {
        Self {
            active_slots: HashMap::new(),
            empty_nodes:  compute_empty_nodes(depth),
            depth,
        }
    }

    /// Build from a list of active validators.
    /// Validators are sorted by slot, not by pubkey — slot is derived from pubkey.
    /// Called by ConsensusClient::sync() after the indexer builds the validator set.
    /// (→ see spec-indexer — Validator Set Construction)
    pub fn from_validators(validators: &[ValidatorInfo], depth: u32) -> Self {
        let mut tree = Self::new(depth);
        for v in validators {
            tree.insert(&v.pubkey, depth);
        }
        tree
    }

    /// Insert an active validator. Sets their slot to sha256(pubkey).
    /// Called by compute_new_validator_set() when processing entries.
    pub fn insert(&mut self, pubkey: &PublicKey) {
        let slot = validator_slot(pubkey) % (1u64 << self.depth);
        self.active_slots.insert(slot, active_leaf(pubkey));
    }

    /// Remove a validator. Sets their slot back to empty (removes from map).
    /// Called by compute_new_validator_set() when processing exits.
    pub fn remove(&mut self, pubkey: &PublicKey) {
        let slot = validator_slot(pubkey) % (1u64 << self.depth);
        self.active_slots.remove(&slot);
    }

    /// Compute the current root.
    /// The root becomes new_validator_merkle_root in the next checkpoint.
    /// Verified against on-chain root on every sync() call.
    pub fn root(&self) -> [u8; 32] {
        self.compute_subtree(0, 1u64 << self.depth, self.depth)
    }

    fn compute_subtree(&self, start: u64, end: u64, level: u32) -> [u8; 32] {
        if level == 0 {
            return *self.active_slots.get(&start).unwrap_or(&EMPTY_LEAF);
        }

        let has_active = self.active_slots.keys().any(|&k| k >= start && k < end);
        if !has_active {
            return self.empty_nodes[level as usize];
        }

        let mid = start + (end - start) / 2;
        let left  = self.compute_subtree(start, mid, level - 1);
        let right = self.compute_subtree(mid, end, level - 1);

        // CRITICAL: left child always first in SHA-256 concatenation.
        // Must match the Rue verify_merkle_path in the checkpoint singleton.
        // (→ see spec-sparse-merkle-tree — Tree Structure: Critical invariant)
        sha256(&[left, right].concat())
    }

    /// Generate a membership proof for an active validator.
    /// Returns error if pubkey is not in the tree.
    /// The proof is passed as private witness to the Groth16 circuit.
    /// (→ see spec-sparse-merkle-tree — Proof Generation)
    /// (→ see spec-groth16-circuit — Private Witness)
    pub fn prove_membership(&self, pubkey: &PublicKey) -> ConsensusResult<MerkleProof> {
        let slot = validator_slot(pubkey) % (1u64 << self.depth);
        if !self.active_slots.contains_key(&slot) {
            return Err(ConsensusError::ValidatorNotFound(
                hex::encode(pubkey.to_bytes())
            ));
        }
        Ok(self.prove(slot))
    }

    /// Generate a non-membership proof for a slot that must be empty.
    /// Returns error if the slot is not empty (validator is still active).
    /// Used during collateral recovery.
    /// (→ see spec-sparse-merkle-tree — Non-Membership Proof)
    pub fn prove_non_membership(&self, slot: u64) -> ConsensusResult<MerkleProof> {
        if self.active_slots.contains_key(&slot) {
            return Err(ConsensusError::InvalidMerkleProof);
        }
        Ok(self.prove(slot))
    }

    fn prove(&self, slot: u64) -> MerkleProof {
        let mut siblings = Vec::with_capacity(self.depth as usize);
        let mut index = slot;
        let mut start = 0u64;
        let mut end = 1u64 << self.depth;

        for level in 0..self.depth {
            let mid = start + (end - start) / 2;

            let sibling = if index < mid {
                self.compute_subtree(mid, end, level)
            } else {
                self.compute_subtree(start, mid, level)
            };

            siblings.push(sibling);

            if index < mid {
                end = mid;
            } else {
                start = mid;
                index -= mid - start + (mid - start);
            }
        }

        MerkleProof { leaf_index: slot, siblings }
    }

    /// Verify a membership proof for a pubkey against a root.
    /// (→ see spec-sparse-merkle-tree — Proof Verification)
    pub fn verify_membership(proof: &MerkleProof, pubkey: &PublicKey, root: [u8; 32], depth: u32) -> bool {
        let leaf = active_leaf(pubkey);
        verify_path(leaf, proof.leaf_index, &proof.siblings, root, depth)
    }

    /// Verify a non-membership proof against a root.
    pub fn verify_non_membership(proof: &MerkleProof, root: [u8; 32], depth: u32) -> bool {
        verify_path(EMPTY_LEAF, proof.leaf_index, &proof.siblings, root, depth)
    }
}

fn verify_path(leaf: [u8; 32], index: u64, siblings: &[[u8; 32]], root: [u8; 32], depth: u32) -> bool {
    assert_eq!(siblings.len(), depth as usize);
    let mut node = leaf;
    let mut current_index = index;

    for sibling in siblings {
        node = if current_index % 2 == 0 {
            sha256(&[node, *sibling].concat())    // left child: node first
        } else {
            sha256(&[*sibling, node].concat())    // right child: sibling first
        };
        current_index /= 2;
    }

    node == root
}
```

```rust
// src/merkle/proof.rs

/// Merkle membership or non-membership proof.
/// (→ see spec-sparse-merkle-tree — Proof Format)
#[derive(Debug, Clone)]
pub struct MerkleProof {
    /// Slot index of the leaf being proven.
    /// Derived from pubkey via validator_slot().
    pub leaf_index: u64,
    /// Sibling hashes from leaf level (index 0) up to just below root.
    /// Length must equal TREE_DEPTH.
    /// siblings[0] is the sibling at level 0 (leaf level).
    /// siblings[TREE_DEPTH - 1] is the sibling just below the root.
    pub siblings:   Vec<[u8; 32]>,
}
```

---

## Serialization Module

Full implementation of the wire format as defined in
[spec-wire-format](spec-wire-format.md). Every function here has an exact
counterpart in either the Rue checkpoint singleton puzzle or the Arkworks
library. Any mismatch causes silent on-chain verification failure.

```rust
// src/prover/serialize.rs

use ark_serialize::CanonicalSerialize;
use ark_bls12_381::{G1Affine, G2Affine, Fr};
use ark_ff::PrimeField;
use blst::min_pk::{AggregatePublicKey, AggregateSignature, PublicKey as BlstPk, Signature as BlstSig};

/// Serialized Groth16 proof ready for the CLVM checkpoint singleton solution.
/// Total size: 192 bytes (48 + 96 + 48).
/// (→ see spec-wire-format — Groth16 Proof Format)
pub struct ClvmProof {
    pub a: Vec<u8>,  // 48 bytes, G1 compressed
    pub b: Vec<u8>,  // 96 bytes, G2 compressed
    pub c: Vec<u8>,  // 48 bytes, G1 compressed
}

/// Serialized verification key ready for currying into the checkpoint singleton.
/// Total size: 672 bytes (48 + 96 + 96 + 96 + 7*48).
/// IC point order must match public input allocation order in the circuit.
/// (→ see spec-wire-format — Verification Key Format)
/// (→ see spec-groth16-circuit — Public Inputs for IC point order)
pub struct ClvmVerificationKey {
    pub alpha_g1: Vec<u8>,    // 48 bytes, G1 compressed
    pub beta_g2:  Vec<u8>,    // 96 bytes, G2 compressed
    pub gamma_g2: Vec<u8>,    // 96 bytes, G2 compressed
    pub delta_g2: Vec<u8>,    // 96 bytes, G2 compressed
    pub ic:       Vec<Vec<u8>>, // 7 × 48 bytes, G1 compressed
    // ic[0] = constant term
    // ic[1] = validator_merkle_root
    // ic[2] = validator_count
    // ic[3] = new_validator_merkle_root
    // ic[4] = new_validator_count
    // ic[5] = agg_signers
    // ic[6] = checkpoint_message
}

/// Serialize a Groth16 proof from Arkworks format to CLVM-compatible bytes.
/// (→ see spec-wire-format — Groth16 Proof Format — Serialization)
pub fn serialize_proof(
    proof: &ark_groth16::Proof<ark_bls12_381::Bls12_381>,
) -> ConsensusResult<ClvmProof> {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut c = Vec::new();

    proof.a.serialize_compressed(&mut a)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    proof.b.serialize_compressed(&mut b)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    proof.c.serialize_compressed(&mut c)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    verify_proof_sizes(&ClvmProof { a: a.clone(), b: b.clone(), c: c.clone() });
    Ok(ClvmProof { a, b, c })
}

/// Serialize the verification key from Arkworks format to CLVM-compatible bytes.
/// The result is what gets curried into the checkpoint singleton at deployment.
/// (→ see spec-wire-format — Verification Key Format — Serialization)
pub fn serialize_vk(
    vk: &ark_groth16::VerifyingKey<ark_bls12_381::Bls12_381>,
) -> ConsensusResult<ClvmVerificationKey> {
    let mut alpha_g1 = Vec::new();
    let mut beta_g2  = Vec::new();
    let mut gamma_g2 = Vec::new();
    let mut delta_g2 = Vec::new();

    vk.alpha_g1.serialize_compressed(&mut alpha_g1)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    vk.beta_g2.serialize_compressed(&mut beta_g2)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    vk.gamma_g2.serialize_compressed(&mut gamma_g2)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    vk.delta_g2.serialize_compressed(&mut delta_g2)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let ic = vk.gamma_abc_g1.iter().map(|pt| {
        let mut buf = Vec::new();
        pt.serialize_compressed(&mut buf)
            .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
        Ok(buf)
    }).collect::<ConsensusResult<Vec<_>>>()?;

    assert_eq!(ic.len(), 7, "VK must have 7 IC points (6 public inputs + constant term)");

    Ok(ClvmVerificationKey { alpha_g1, beta_g2, gamma_g2, delta_g2, ic })
}

/// Serialize VK to hex-encoded JSON string for storage in NetworkConfig.
/// (→ see spec-wire-format — Verification Key Format — Storage Format)
pub fn serialize_vk_to_hex(
    vk: &ark_groth16::VerifyingKey<ark_bls12_381::Bls12_381>,
) -> ConsensusResult<String> {
    let clvm_vk = serialize_vk(vk)?;
    let json = serde_json::json!({
        "alpha_g1": hex::encode(&clvm_vk.alpha_g1),
        "beta_g2":  hex::encode(&clvm_vk.beta_g2),
        "gamma_g2": hex::encode(&clvm_vk.gamma_g2),
        "delta_g2": hex::encode(&clvm_vk.delta_g2),
        "ic": clvm_vk.ic.iter().map(hex::encode).collect::<Vec<_>>(),
    });
    Ok(json.to_string())
}

fn verify_proof_sizes(proof: &ClvmProof) {
    assert_eq!(proof.a.len(), 48, "proof.a must be 48 bytes (G1 compressed)");
    assert_eq!(proof.b.len(), 96, "proof.b must be 96 bytes (G2 compressed)");
    assert_eq!(proof.c.len(), 48, "proof.c must be 48 bytes (G1 compressed)");
}

/// Convert bytes to a BLS12-381 scalar field element.
/// scalar(bytes) = sha256(bytes) as big-endian u256, reduced mod r.
/// This is the scalar() function used in the Rue puzzle for VK input computation.
/// MUST produce the same result as the Rue scalar() function.
/// (→ see spec-wire-format — The scalar() Function)
pub fn bytes_to_scalar(bytes: &[u8]) -> Fr {
    let hash = sha256(bytes);
    let big = num_bigint::BigUint::from_bytes_be(&hash);
    Fr::from(big)
}

/// Compute the vk_input G1 point from public inputs and the VK.
/// This is computed identically in Rust (here) and in the Rue checkpoint puzzle.
/// Any divergence causes bls_pairing_identity to fail on-chain.
/// (→ see spec-wire-format — VK Input Computation)
/// (→ see spec-checkpoint-singleton — Puzzle Source: vk_input computation)
pub fn compute_vk_input(
    vk: &ClvmVerificationKey,
    validator_merkle_root:     [u8; 32],
    validator_count:           u64,
    new_validator_merkle_root: [u8; 32],
    new_validator_count:       u64,
    agg_signers:               &[u8; 48],   // G1 compressed
    checkpoint_message:        [u8; 32],
) -> ConsensusResult<G1Affine> {
    use ark_ec::AffineCurve;

    let inputs = [
        bytes_to_scalar(&validator_merkle_root),
        bytes_to_scalar(&validator_count.to_be_bytes()),
        bytes_to_scalar(&new_validator_merkle_root),
        bytes_to_scalar(&new_validator_count.to_be_bytes()),
        bytes_to_scalar(agg_signers.as_slice()),
        bytes_to_scalar(&checkpoint_message),
    ];

    let ic_points: Vec<G1Affine> = vk.ic.iter()
        .map(|b| G1Affine::deserialize_compressed(b.as_slice())
            .map_err(|e| ConsensusError::SerializationError(e.to_string())))
        .collect::<ConsensusResult<_>>()?;

    let mut result = ic_points[0].into_projective();
    for (scalar, ic_point) in inputs.iter().zip(ic_points[1..].iter()) {
        result += ic_point.mul(*scalar);
    }

    Ok(result.into_affine())
}

/// Compute the checkpoint message that validators sign.
/// sha256(new_state_root + new_validator_merkle_root + new_validator_count_be8 + new_epoch_be8)
/// Total input: 80 bytes. Output: 32 bytes.
/// Must match the Rue checkpoint_message() function in the checkpoint singleton puzzle exactly.
/// (→ see spec-wire-format — Checkpoint Message)
pub fn compute_checkpoint_message(
    new_state_root:            [u8; 32],
    new_validator_merkle_root: [u8; 32],
    new_validator_count:       u64,
    new_epoch:                 u64,
) -> [u8; 32] {
    let mut input = Vec::with_capacity(80);
    input.extend_from_slice(&new_state_root);
    input.extend_from_slice(&new_validator_merkle_root);
    input.extend_from_slice(&new_validator_count.to_be_bytes());
    input.extend_from_slice(&new_epoch.to_be_bytes());
    sha256(&input)
}

/// Compute the full AGG_SIG_ME message each validator actually signs.
/// = checkpoint_message + genesis_challenge + checkpoint_singleton_coin_id
/// (→ see spec-wire-format — Individual Signatures)
pub fn compute_validator_signing_message(
    new_state_root:            [u8; 32],
    new_validator_merkle_root: [u8; 32],
    new_validator_count:       u64,
    new_epoch:                 u64,
    genesis_challenge:         [u8; 32],
    checkpoint_singleton_coin_id: [u8; 32],
) -> Vec<u8> {
    let checkpoint_message = compute_checkpoint_message(
        new_state_root, new_validator_merkle_root, new_validator_count, new_epoch,
    );
    let mut msg = Vec::new();
    msg.extend_from_slice(&checkpoint_message);
    msg.extend_from_slice(&genesis_challenge);
    msg.extend_from_slice(&checkpoint_singleton_coin_id);
    msg
}

/// Compute the membership announcement hash for use in AssertCoinAnnouncement.
/// sha256(sha256("membership" + epoch_be8 + pubkey_48 + is_member_1) prepended with singleton_coin_id)
/// is_member: 0x01 = member, 0x00 = not member.
/// Must match the Rue announcement in the checkpoint singleton membership query spend.
/// (→ see spec-wire-format — Membership Announcement Format)
pub fn compute_membership_announcement(
    epoch:                       u64,
    pubkey:                      &PublicKey,
    is_member:                   bool,
    checkpoint_singleton_coin_id: [u8; 32],
) -> [u8; 32] {
    let mut msg = Vec::new();
    msg.extend_from_slice(b"membership");
    msg.extend_from_slice(&epoch.to_be_bytes());
    msg.extend_from_slice(pubkey.to_bytes().as_slice());
    msg.push(if is_member { 1 } else { 0 });
    let announcement = sha256(&msg);
    sha256(&[checkpoint_singleton_coin_id.as_ref(), announcement.as_ref()].concat())
}

/// Aggregate BLS public keys into a single G1 point.
/// G1 sum of k signing pubkeys = agg_signers.
/// Proven by the Groth16 circuit's Constraint 2.
/// (→ see spec-wire-format — Aggregate Public Key)
/// (→ see spec-groth16-circuit — Constraint 2: Aggregate Consistency)
pub fn aggregate_pubkeys(pubkeys: &[PublicKey]) -> ConsensusResult<PublicKey> {
    let blst_pks: Vec<&BlstPk> = pubkeys.iter()
        .map(|pk| unsafe { &*(pk as *const PublicKey as *const BlstPk) })
        .collect();

    AggregatePublicKey::aggregate(&blst_pks, false)
        .map(|agg| {
            let pk = agg.to_public_key();
            unsafe { std::mem::transmute(pk) }
        })
        .map_err(|e| ConsensusError::ProvingError(format!("pubkey aggregation failed: {:?}", e)))
}

/// Aggregate BLS signatures into a single G2 point.
/// G2 sum of k individual signatures = agg_sig.
/// Standard (not rogue-key safe) aggregation since all validators sign the same message.
/// (→ see spec-wire-format — Aggregate Signature)
pub fn aggregate_signatures(sigs: &[Signature]) -> ConsensusResult<Signature> {
    let blst_sigs: Vec<&BlstSig> = sigs.iter()
        .map(|s| unsafe { &*(s as *const Signature as *const BlstSig) })
        .collect();

    AggregateSignature::aggregate(&blst_sigs, false)
        .map(|agg| {
            let sig = agg.to_signature();
            unsafe { std::mem::transmute(sig) }
        })
        .map_err(|e| ConsensusError::ProvingError(format!("signature aggregation failed: {:?}", e)))
}
```

---

## Prover Module

```rust
// src/prover/setup.rs
// (→ see spec-groth16-circuit — Trusted Setup)
// (→ see spec-trusted-setup — Single-Party Setup and Multi-Party Ceremony)

use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_bls12_381::Bls12_381;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};

/// Run the Groth16 trusted setup for the consensus circuit.
/// For production use: run the MPC ceremony instead.
/// NEVER use single-party setup in production.
/// (→ see spec-trusted-setup — Single-Party Setup (Development/Testing Only))
/// (→ see spec-security — Assumption 2: Trusted Setup Is Sound)
pub fn run_setup(
    max_signers: usize,
    tree_depth:  u32,
    pk_path:     &str,
    vk_path:     &str,
) -> ConsensusResult<()> {
    let mut rng = ark_std::rand::thread_rng();

    // Blank circuit: all zeros, same structure as real circuit.
    // Constraint count must match a real MAX_SIGNERS-active circuit.
    // (→ see spec-groth16-circuit — Trusted Setup)
    let blank = ConsensusCircuit::blank(max_signers, tree_depth);

    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(blank, &mut rng)
        .map_err(|e| ConsensusError::ProvingError(e.to_string()))?;

    // VK must have 7 IC points: constant term + 6 public inputs
    // (→ see spec-wire-format — Verification Key Format — IC Point Order)
    assert_eq!(vk.gamma_abc_g1.len(), 7,
        "VK must have 7 IC points (6 public inputs + constant term)");

    let mut pk_bytes = Vec::new();
    pk.serialize_uncompressed(&mut pk_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    std::fs::write(pk_path, &pk_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    let mut vk_bytes = Vec::new();
    vk.serialize_uncompressed(&mut vk_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    std::fs::write(vk_path, &vk_bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;

    Ok(())
}

/// Load the Groth16 proving key from disk.
/// Large file: 100-500MB depending on MAX_SIGNERS and TREE_DEPTH.
/// (→ see spec-trusted-setup — Proving Key Distribution)
/// Only needed on nodes that submit checkpoints.
/// (→ see spec-l2-integration — Important Notes: Proof generation is blocking)
pub fn load_proving_key(path: &str) -> ConsensusResult<ProvingKey<Bls12_381>> {
    let bytes = std::fs::read(path)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    ProvingKey::deserialize_uncompressed(&*bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))
}

/// Load the Groth16 verification key from disk.
/// Small: 672 bytes per spec-wire-format — Verification Key Format.
/// (→ see spec-trusted-setup — Verifying the Output)
pub fn load_verification_key(path: &str) -> ConsensusResult<VerifyingKey<Bls12_381>> {
    let bytes = std::fs::read(path)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))?;
    VerifyingKey::deserialize_uncompressed(&*bytes)
        .map_err(|e| ConsensusError::SerializationError(e.to_string()))
}
```

```rust
// src/prover/prove.rs

/// Generate a Groth16 proof for the consensus circuit.
/// Takes 5-15 minutes for MAX_SIGNERS=10, TREE_DEPTH=32 on BLS12-381.
/// (→ see spec-groth16-circuit — Constraint Count Estimates)
/// Called inside spawn_blocking by ConsensusClient::build_checkpoint().
/// Proof output is randomized: two calls with identical inputs produce
/// different proofs that both verify correctly.
/// (→ see spec-groth16-circuit — Important Notes: Deterministic proof generation)
pub fn generate_proof(
    circuit: ConsensusCircuit,
    pk:      &ProvingKey<Bls12_381>,
) -> ConsensusResult<ark_groth16::Proof<Bls12_381>> {
    let mut rng = ark_std::rand::thread_rng();
    Groth16::<Bls12_381>::prove(pk, circuit, &mut rng)
        .map_err(|e| ConsensusError::ProvingError(e.to_string()))
}
```

---

## Puzzle Modules

### Network Coin Driver

```rust
// src/puzzles/network_coin.rs
// Full puzzle spec: spec-network-coin

/// All fixed configuration for the network coin singleton.
/// A subset of NetworkConfig, passed to network coin functions directly.
pub struct NetworkCoinConfig {
    pub registration_coin_mod_hash: Bytes32,
    pub collateral_amount:          u64,
    pub checkpoint_singleton_id:    Bytes32,
}

/// Deploy both the network coin and checkpoint singleton in one spend bundle.
/// Resolves the circular dependency (each needs the other's ID) via the
/// genesis coin approach: derive both IDs from the genesis coin before spending.
/// (→ see spec-deployment-runbook — Step 3)
/// (→ see spec-network-coin — Deployment)
pub async fn deploy_network(
    ctx:           &mut SpendContext,
    genesis_coin:  Coin,
    genesis_sk:    &SecretKey,
    config:        &NetworkConfig,
    vk:            &ark_groth16::VerifyingKey<Bls12_381>,
) -> ConsensusResult<SpendBundle> {
    // ... deploy both singletons in one bundle
    // network coin curried with: REGISTRATION_COIN_MOD_HASH, COLLATERAL_AMOUNT, CHECKPOINT_SINGLETON_ID
    // checkpoint singleton curried with: VK (serialized per spec-wire-format), TREE_DEPTH, EMPTY_LEAF_HASH
    // (→ see spec-network-coin — Deployment)
    // (→ see spec-checkpoint-singleton — Deployment)
    todo!()
}

/// Derive launcher IDs before spending, using genesis coin as the source.
/// Allows computing IDs before the spend bundle is submitted.
/// (→ see spec-deployment-runbook — Step 3)
pub fn derive_launcher_id(genesis_coin: &Coin, index: u8) -> Bytes32 {
    todo!() // deterministic from genesis_coin.coin_id() and index
}

/// Register a validator by spending the network coin.
/// Requires validator_sk for the AggSigMe condition.
/// The registration message format is per spec-wire-format — Registration Message Format.
/// Includes validator pubkey as memo for indexer efficiency.
/// (→ see spec-network-coin — Registration)
/// (→ see spec-indexer — Important Notes: Memo is required for indexing)
/// CLVM cost: ~5.3M units (→ see spec-clvm-costs — Spend Path 1)
pub async fn register_validator(
    ctx:          &mut SpendContext,
    network_coin: &NetworkCoinState,
    validator_sk: &SecretKey,
    config:       &NetworkConfig,
) -> ConsensusResult<(SpendBundle, Coin)> {
    todo!()
}

/// Compute the puzzle hash of a registration coin for a given pubkey.
/// Must match the curry_hash call in the network coin Rue puzzle exactly.
/// Also used by the indexer for lineage verification.
/// (→ see spec-network-coin — Computing the Registration Coin Puzzle Hash)
/// (→ see spec-indexer — Registration Coin Detection)
pub fn registration_coin_puzzle_hash(
    registration_coin_mod_hash: Bytes32,
    validator_pubkey:           PublicKey,
    checkpoint_singleton_id:    Bytes32,
) -> Bytes32 {
    curry_puzzle_hash(
        registration_coin_mod_hash,
        &[clvm_encode(&validator_pubkey), clvm_encode(&checkpoint_singleton_id)],
    )
}

/// Fetch current network coin state from the Chia node.
/// Called on every sync() to get the current unspent network coin.
/// (→ see spec-network-coin — Querying the Network Coin State)
pub async fn fetch_network_coin_state(
    node:          &FullNodeClient,
    launcher_id:   Bytes32,
    config:        &NetworkConfig,
) -> ConsensusResult<NetworkCoinState> {
    todo!()
}

/// Fetch all valid registration coins whose lineage traces back to the network coin.
/// Returns (coin, pubkey) pairs sorted by pubkey bytes.
/// This is the primary lineage verification step.
/// (→ see spec-network-coin — Fetching All Valid Registration Coins)
/// (→ see spec-security — Lineage Proof Enforcement)
pub async fn fetch_valid_registration_coins(
    node:                       &FullNodeClient,
    launcher_id:                Bytes32,
    registration_coin_mod_hash: Bytes32,
    checkpoint_singleton_id:    Bytes32,
) -> ConsensusResult<Vec<(Coin, PublicKey)>> {
    todo!()
}
```

### Registration Coin Driver

```rust
// src/puzzles/registration_coin.rs
// Full puzzle spec: spec-registration-coin

/// Everything needed to spend a registration coin.
pub struct RegistrationCoinSpend {
    pub coin:                    Coin,
    pub validator_pubkey:        PublicKey,
    pub checkpoint_singleton_id: Bytes32,
}

/// Compute the deterministic coin ID of a registration coin.
/// (→ see spec-registration-coin — Computing the Registration Coin Puzzle Hash)
pub fn registration_coin_id(
    network_coin_spend_id:      Bytes32,
    validator_pubkey:           PublicKey,
    checkpoint_singleton_id:    Bytes32,
    registration_coin_mod_hash: Bytes32,
    collateral_amount:          u64,
) -> Bytes32 {
    let puzzle_hash = registration_coin_puzzle_hash_from_components(
        validator_pubkey, checkpoint_singleton_id, registration_coin_mod_hash,
    );
    Coin::new(network_coin_spend_id, puzzle_hash, collateral_amount).coin_id()
}

/// Build the registration coin spend for collateral recovery.
/// Must be submitted in the same bundle as a checkpoint singleton membership
/// query spend that emits a non-membership announcement.
/// (→ see spec-registration-coin — Spending the Registration Coin)
/// (→ see spec-checkpoint-singleton — Spend Path 2: Membership Query)
/// epoch must match the epoch in the membership announcement exactly.
/// (→ see spec-registration-coin — Important Notes: Epoch Matching)
/// CLVM cost: ~3.3M units (→ see spec-clvm-costs — Spend Path 4)
pub fn spend_registration_coin(
    ctx:                    &mut SpendContext,
    registration:           &RegistrationCoinSpend,
    registration_coin_mod:  NodePtr,
    epoch:                  u64,
    collateral_destination: Bytes32,
    extra_conditions:       Vec<Condition>,
) -> ConsensusResult<CoinSpend> {
    todo!()
}

/// Fetch the current unspent registration coin for a validator.
/// Verifies lineage: parent must be a network coin spend.
/// Returns None if not registered or already exited.
/// (→ see spec-registration-coin — Fetching a Validator's Registration Coin)
pub async fn fetch_registration_coin(
    node:                       &FullNodeClient,
    validator_pubkey:           PublicKey,
    config:                     &NetworkConfig,
) -> ConsensusResult<Option<Coin>> {
    todo!()
}

/// Check that a coin's parent is a network coin spend.
/// (→ see spec-security — Lineage Proof Enforcement)
async fn is_valid_registration_coin_parent(
    node:                    &FullNodeClient,
    parent_coin_id:          Bytes32,
    network_coin_launcher_id: Bytes32,
) -> ConsensusResult<bool> {
    let parent = node.get_coin_record_by_name(parent_coin_id).await
        .map_err(|e| ConsensusError::NodeError(e.to_string()))?;
    let Some(parent) = parent else { return Ok(false); };
    let network_coin_ph = singleton_puzzle_hash(network_coin_launcher_id);
    Ok(parent.coin.puzzle_hash == network_coin_ph)
}
```

### Checkpoint Singleton Driver

```rust
// src/puzzles/checkpoint.rs
// Full puzzle spec: spec-checkpoint-singleton

/// Build the checkpoint singleton spend for the checkpoint path.
/// Verifies Groth16 proof on-chain via bls_pairing_identity (4 pairs).
/// Verifies BLS aggregate signature on-chain via bls_verify.
/// Emits checkpoint state announcement for the indexer to parse.
/// Returns SpendBundle with identity aggregated_signature (no AGG_SIG_ME conditions).
/// (→ see spec-checkpoint-singleton — Checkpoint Spend)
/// (→ see spec-checkpoint-singleton — Important Notes: No signatures on checkpoint spend)
/// CLVM cost: ~17.2M units (→ see spec-clvm-costs — Spend Path 2)
pub fn spend_checkpoint_singleton(
    ctx:                       &mut SpendContext,
    checkpoint_state:          &CheckpointSingletonState,
    config:                    &NetworkConfig,
    proof:                     ClvmProof,         // per spec-wire-format
    new_state_root:            Bytes32,
    new_validator_merkle_root: Bytes32,
    new_validator_count:       u64,
    agg_signers:               PublicKey,          // G1 compressed per spec-wire-format
    agg_sig:                   Signature,          // G2 compressed per spec-wire-format
) -> ConsensusResult<CoinSpend> {
    todo!()
}

/// Build the checkpoint singleton membership query spend.
/// Permissionless: no signature required. Recreates singleton unchanged.
/// Emits membership announcement for the registration coin to assert.
/// (→ see spec-checkpoint-singleton — Membership Query Spend)
/// (→ see spec-checkpoint-singleton — Important Notes: Membership query is permissionless)
/// CLVM cost: ~4.1M units (→ see spec-clvm-costs — Spend Path 3)
/// The siblings slice must have length == tree_depth.
/// (→ see spec-sparse-merkle-tree — Proof Format)
pub fn spend_checkpoint_singleton_membership_query(
    ctx:              &mut SpendContext,
    checkpoint_state: &CheckpointSingletonState,
    config:           &NetworkConfig,
    query_pubkey:     PublicKey,
    leaf_index:       u64,
    siblings:         Vec<Bytes32>,
    is_member:        bool,
) -> ConsensusResult<CoinSpend> {
    todo!()
}

/// Fetch current checkpoint singleton state from the Chia node.
/// State is decoded from the checkpoint coin's parent spend (solution fields).
/// (→ see spec-checkpoint-singleton — Fetching the Current Checkpoint Singleton State)
/// (→ see spec-indexer — Checkpoint State Updates)
pub async fn fetch_checkpoint_singleton_state(
    node:          &FullNodeClient,
    launcher_id:   Bytes32,
    config:        &NetworkConfig,
) -> ConsensusResult<CheckpointSingletonState> {
    todo!()
}

/// Serialize the Groth16 VK from Arkworks format into the form the Rue puzzle expects.
/// The result is what gets curried into the checkpoint singleton at deployment.
/// (→ see spec-checkpoint-singleton — Serializing the Verification Key)
/// (→ see spec-wire-format — Verification Key Format)
pub fn serialize_vk_for_clvm(
    vk: &ark_groth16::VerifyingKey<Bls12_381>,
) -> ConsensusResult<ClvmVerificationKey> {
    crate::prover::serialize::serialize_vk(vk)
}
```

---

## Indexer Module

Full implementation as defined in [spec-indexer](spec-indexer.md). The indexer
maintains the local view of on-chain state that `ConsensusClient::sync()` uses
to populate `NetworkState`.

```rust
// src/indexer/mod.rs

/// All indexed on-chain state. Refreshed by sync().
/// (→ see spec-indexer — IndexerState)
pub struct IndexerState {
    pub last_synced_height:    u32,
    pub network_coin:          NetworkCoinState,
    pub checkpoint:            CheckpointSingletonState,
    /// All valid registration coins keyed by pubkey.
    /// Only includes coins whose parent traces to a network coin spend.
    /// (→ see spec-security — Lineage Proof Enforcement)
    pub registration_coins:    HashMap<PublicKey, RegistrationCoinRecord>,
    /// Ordered checkpoint history, most recent last. Used for reorg recovery.
    pub checkpoint_history:    Vec<CheckpointRecord>,
    cache:                     IndexerCache,
    /// IDs of all network coin spends seen. Used for fast parent ID lookup.
    network_coin_spend_ids:    HashSet<Bytes32>,
}

pub struct RegistrationCoinRecord {
    pub coin:                 Coin,
    pub pubkey:               PublicKey,
    pub registered_at_height: u32,
    pub registered_at_epoch:  u64,
}

pub struct CheckpointRecord {
    pub epoch:                 u64,
    pub state_root:            Bytes32,
    pub validator_merkle_root: Bytes32,
    pub validator_count:       u64,
    pub confirmed_at_height:   u32,
    /// Coin ID of the checkpoint singleton coin that carried this state.
    pub coin_id:               Bytes32,
}

impl IndexerState {
    /// Sync from last known state to current chain tip.
    /// Detects and handles reorgs.
    /// On completion, verifies local Merkle root matches on-chain root.
    /// Returns StateMismatch if they differ.
    /// (→ see spec-indexer — Sync Algorithm)
    /// (→ see spec-indexer — Merkle Root Consistency Check)
    pub async fn sync(
        &mut self,
        node:   &FullNodeClient,
        config: &NetworkConfig,
    ) -> ConsensusResult<()> {
        let peak = node.get_blockchain_state().await
            .map_err(|e| ConsensusError::NodeError(e.to_string()))?.peak_height;

        if peak < self.last_synced_height {
            self.handle_reorg(node, config, peak).await?;
            return Ok(());
        }

        let batch_size = 100u32;
        let mut height = self.last_synced_height + 1;
        while height <= peak {
            let end = (height + batch_size).min(peak + 1);
            self.process_block_range(node, config, height, end).await?;
            height = end;
        }

        self.last_synced_height = peak;
        self.cache.save(self)?;
        Ok(())
    }

    async fn process_block_range(
        &mut self, node: &FullNodeClient, config: &NetworkConfig, start: u32, end: u32,
    ) -> ConsensusResult<()> {
        let additions = node.get_additions_and_removals_by_height(start, end).await
            .map_err(|e| ConsensusError::NodeError(e.to_string()))?;

        for (height, added, removed) in additions {
            self.process_additions(node, config, height, &added).await?;
            self.process_removals(&removed)?;
        }
        Ok(())
    }

    async fn process_additions(
        &mut self, node: &FullNodeClient, config: &NetworkConfig, height: u32, added: &[Coin],
    ) -> ConsensusResult<()> {
        for coin in added {
            if self.is_network_coin(coin, config) {
                self.update_network_coin(node, coin, height).await?;
                continue;
            }
            if self.is_checkpoint_coin(coin, config) {
                self.update_checkpoint(node, coin, height).await?;
                continue;
            }
            if let Some(record) = self.try_parse_registration_coin(node, coin, config, height).await? {
                self.registration_coins.insert(record.pubkey, record);
            }
        }
        Ok(())
    }

    fn process_removals(&mut self, removed: &[Coin]) -> ConsensusResult<()> {
        // Spent registration coin = validator exited and recovered collateral.
        // Remove from active set.
        // (→ see spec-registration-coin — Important Notes: Spent registration coins)
        for coin in removed {
            if let Some(pk) = self.registration_coins.iter()
                .find(|(_, r)| r.coin == *coin)
                .map(|(pk, _)| *pk)
            {
                self.registration_coins.remove(&pk);
            }
        }
        Ok(())
    }

    async fn update_checkpoint(
        &mut self, node: &FullNodeClient, coin: &Coin, height: u32,
    ) -> ConsensusResult<()> {
        let parent_spend = node.get_puzzle_and_solution(coin.parent_coin_info, height).await
            .map_err(|e| ConsensusError::NodeError(e.to_string()))?
            .ok_or(ConsensusError::NodeError("missing parent spend".into()))?;

        let lineage_proof = extract_lineage_proof(&parent_spend)?;
        let new_state = parse_checkpoint_solution_fields(&parent_spend)?;

        self.checkpoint_history.push(CheckpointRecord {
            epoch:                 new_state.epoch,
            state_root:            new_state.state_root,
            validator_merkle_root: new_state.validator_merkle_root,
            validator_count:       new_state.validator_count,
            confirmed_at_height:   height,
            coin_id:               coin.coin_id(),
        });

        self.checkpoint = CheckpointSingletonState {
            coin:                  *coin,
            lineage_proof,
            state_root:            new_state.state_root,
            epoch:                 new_state.epoch,
            validator_merkle_root: new_state.validator_merkle_root,
            validator_count:       new_state.validator_count,
        };

        Ok(())
    }

    /// Verify a potential registration coin against all lineage and puzzle hash requirements.
    /// Steps: parent must be a network coin spend, pubkey extractable from memo,
    /// puzzle hash must match registration_coin_puzzle_hash(), amount must match collateral.
    /// (→ see spec-indexer — Registration Coin Detection and Lineage Verification)
    /// (→ see spec-security — Lineage Proof Enforcement)
    async fn try_parse_registration_coin(
        &self, node: &FullNodeClient, coin: &Coin, config: &NetworkConfig, height: u32,
    ) -> ConsensusResult<Option<RegistrationCoinRecord>> {
        // Step 1: Parent must be a known network coin spend ID
        if !self.network_coin_spend_ids.contains(&coin.parent_coin_info) {
            return Ok(None);
        }

        // Step 2: Extract pubkey from memo on parent spend
        // Memo convention: first memo on CreateCoin condition matching this child
        // (→ see spec-network-coin — Important Notes: Memo convention)
        let parent_spend = node.get_puzzle_and_solution(coin.parent_coin_info, height).await
            .map_err(|e| ConsensusError::NodeError(e.to_string()))?
            .ok_or(ConsensusError::NodeError("missing parent spend".into()))?;

        let pubkey = extract_pubkey_from_memo(&parent_spend, coin)
            .ok_or(ConsensusError::NodeError("missing pubkey memo".into()))?;

        // Step 3: Verify puzzle hash
        let expected_hash = crate::puzzles::network_coin::registration_coin_puzzle_hash(
            config.registration_coin_mod_hash,
            pubkey,
            config.checkpoint_singleton_id(),
        );
        if coin.puzzle_hash != expected_hash {
            return Err(ConsensusError::PuzzleHashMismatch);
        }

        // Step 4: Verify collateral amount
        if coin.amount != config.collateral_amount {
            return Ok(None); // wrong amount, not a valid registration coin
        }

        Ok(Some(RegistrationCoinRecord {
            coin:                 *coin,
            pubkey,
            registered_at_height: height,
            registered_at_epoch:  self.checkpoint.epoch,
        }))
    }

    /// Verify local Merkle root matches on-chain root after sync.
    /// Returns the SparseMerkleTree if consistent.
    /// Returns StateMismatch error if they diverge.
    /// (→ see spec-indexer — Merkle Root Consistency Check)
    /// (→ see spec-sparse-merkle-tree — Root Computation)
    pub fn verify_merkle_consistency(&self, config: &NetworkConfig) -> ConsensusResult<SparseMerkleTree> {
        let validators: Vec<ValidatorInfo> = self.registration_coins.values()
            .map(|r| ValidatorInfo { pubkey: r.pubkey, registration_coin: r.coin })
            .collect();

        let tree = SparseMerkleTree::from_validators(&validators, config.tree_depth);
        let computed_root = tree.root();

        if computed_root != self.checkpoint.validator_merkle_root {
            return Err(ConsensusError::StateMismatch);
        }

        Ok(tree)
    }

    /// Handle a chain reorganization by rolling back to the last safe checkpoint.
    /// If no safe checkpoint exists, triggers a full re-index from genesis.
    /// (→ see spec-indexer — Reorg Handling)
    async fn handle_reorg(
        &mut self, node: &FullNodeClient, config: &NetworkConfig, new_peak: u32,
    ) -> ConsensusResult<()> {
        let safe = self.checkpoint_history.iter().rev()
            .find(|c| c.confirmed_at_height <= new_peak)
            .cloned();

        if let Some(safe) = safe {
            self.checkpoint_history.retain(|c| c.epoch <= safe.epoch);
            self.registration_coins.clear();
            self.network_coin_spend_ids.clear();
            self.last_synced_height = safe.confirmed_at_height;
            self.sync(node, config).await
        } else {
            self.full_reindex(node, config).await
        }
    }

    async fn full_reindex(&mut self, node: &FullNodeClient, config: &NetworkConfig) -> ConsensusResult<()> {
        self.registration_coins.clear();
        self.checkpoint_history.clear();
        self.network_coin_spend_ids.clear();
        self.last_synced_height = 0;

        let launcher_record = node.get_coin_record_by_name(config.network_coin_launcher_id).await
            .map_err(|e| ConsensusError::NodeError(e.to_string()))?
            .ok_or(ConsensusError::NodeError("network coin launcher not found".into()))?;

        self.last_synced_height = launcher_record.confirmed_block_index.saturating_sub(1);
        self.sync(node, config).await
    }
}

/// Build the sorted, deduplicated validator set from indexed registration coins.
/// Sorted by pubkey bytes for deterministic Merkle tree slot ordering.
/// (→ see spec-indexer — Validator Set Construction)
pub fn build_validator_set(registration_coins: &HashMap<PublicKey, RegistrationCoinRecord>) -> ValidatorSet {
    let mut validators: Vec<ValidatorInfo> = registration_coins.values()
        .map(|r| ValidatorInfo { pubkey: r.pubkey, registration_coin: r.coin })
        .collect();
    validators.sort_by_key(|v| v.pubkey.to_bytes());
    ValidatorSet { validators }
}
```

```rust
// src/indexer/cache.rs
// (→ see spec-indexer — Persistent Cache)

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IndexerCache {
    pub last_synced_height: u32,
    pub checkpoint_history: Vec<SerializableCheckpointRecord>,
    pub registration_coins: Vec<SerializableRegistrationCoinRecord>,
    pub network_coin_spend_ids: Vec<String>, // hex-encoded coin IDs
}

impl IndexerCache {
    pub fn load(path: &str) -> ConsensusResult<Option<IndexerCache>> {
        if !std::path::Path::new(path).exists() { return Ok(None); }
        let bytes = std::fs::read(path)
            .map_err(|e| ConsensusError::CacheError(e.to_string()))?;
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|e| ConsensusError::CacheError(e.to_string()))
    }

    pub fn save(&self, path: &str) -> ConsensusResult<()> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| ConsensusError::CacheError(e.to_string()))?;
        // Atomic write via temp file to avoid corruption on crash
        let tmp = format!("{}.tmp", path);
        std::fs::write(&tmp, &bytes)
            .map_err(|e| ConsensusError::CacheError(e.to_string()))?;
        std::fs::rename(&tmp, path)
            .map_err(|e| ConsensusError::CacheError(e.to_string()))
    }
}
```

---

## Public Interface — ConsensusClient

```rust
// src/client.rs

pub struct ConsensusClient {
    node:         FullNodeClient,
    config:       NetworkConfig,
    state:        Option<NetworkState>,
    proving_key:  Option<ProvingKey<Bls12_381>>,
    indexer:      Option<IndexerState>,
    cache_path:   Option<String>,
}

impl ConsensusClient {

    /// Create a new client for an existing deployed network.
    /// Call sync() after this to load on-chain state.
    pub fn new(node: FullNodeClient, config: NetworkConfig) -> Self {
        Self { node, config, state: None, proving_key: None, indexer: None, cache_path: None }
    }

    /// Set the path for the indexer cache.
    /// If set, the indexer persists state between restarts.
    /// (→ see spec-indexer — Persistent Cache)
    pub fn set_cache_path(&mut self, path: &str) {
        self.cache_path = Some(path.to_string());
    }

    /// Load the Groth16 proving key from disk.
    /// Large: 100-500MB depending on MAX_SIGNERS and TREE_DEPTH.
    /// Only needed on nodes that submit checkpoints.
    /// (→ see spec-trusted-setup — Proving Key Distribution)
    pub fn load_proving_key(&mut self, path: &str) -> ConsensusResult<()> {
        self.proving_key = Some(crate::prover::setup::load_proving_key(path)?);
        Ok(())
    }

    /// Sync local state with current chain state.
    ///
    /// Drives the indexer sync algorithm (→ see spec-indexer — Sync Algorithm).
    /// Fetches current network coin state (→ see spec-network-coin — Querying).
    /// Fetches current checkpoint singleton state (→ see spec-checkpoint-singleton).
    /// Rebuilds the sparse Merkle tree from indexed registration coins.
    /// Verifies the computed Merkle root matches the on-chain validator_merkle_root.
    /// Returns StateMismatch if they differ — do not proceed, trigger full_reindex.
    ///
    /// Call before every operation that reads current chain state.
    /// (→ see spec-l2-integration — Important Notes: Always sync before checkpoint submission)
    pub async fn sync(&mut self) -> ConsensusResult<()> {
        // Initialize or restore indexer from cache
        if self.indexer.is_none() {
            let cache = self.cache_path.as_deref()
                .and_then(|p| IndexerCache::load(p).ok().flatten());
            self.indexer = Some(IndexerState::from_cache(cache));
        }

        let indexer = self.indexer.as_mut().unwrap();
        indexer.sync(&self.node, &self.config).await?;

        let network_coin = indexer.network_coin.clone();
        let checkpoint   = indexer.checkpoint.clone();
        let validators   = build_validator_set(&indexer.registration_coins);

        // Build the Merkle tree and verify consistency
        // (→ see spec-sparse-merkle-tree — Root Computation)
        let merkle_tree = indexer.verify_merkle_consistency(&self.config)?;

        let synced_at = self.node.get_blockchain_state().await
            .map_err(|e| ConsensusError::NodeError(e.to_string()))?.peak_height;

        self.state = Some(NetworkState {
            network_coin,
            checkpoint,
            validators,
            merkle_tree,
            synced_at,
        });

        Ok(())
    }

    pub fn state(&self) -> ConsensusResult<&NetworkState> {
        self.state.as_ref().ok_or(ConsensusError::NotDeployed)
    }

    // -----------------------------------------------------------------------
    // Deployment
    // -----------------------------------------------------------------------

    /// Deploy the network coin and checkpoint singleton from a genesis coin.
    /// One-time operation per L2 network.
    ///
    /// Resolves circular dependency (network coin needs checkpoint singleton ID
    /// and vice versa) via the genesis coin approach: derive both IDs from the
    /// genesis coin before spending anything.
    /// (→ see spec-deployment-runbook — Step 3)
    ///
    /// Runs the trusted setup if vk_path does not exist yet (dev only — for
    /// production run the MPC ceremony first per spec-trusted-setup).
    /// (→ see spec-trusted-setup — When to Run)
    ///
    /// Returns (SpendBundle, NetworkConfig). The caller is responsible for
    /// broadcasting the bundle to the Chia node. Save the NetworkConfig to
    /// disk — you need it on every subsequent startup.
    /// (→ see spec-deployment-runbook — Step 4 and 6)
    pub async fn deploy(
        node:              FullNodeClient,
        genesis_coin:      Coin,
        genesis_sk:        &SecretKey,
        collateral_amount: u64,
        tree_depth:        u32,
        max_signers:       usize,
        proving_key_path:  &str,
        vk_path:           &str,
        genesis_challenge: Bytes32,
    ) -> ConsensusResult<(SpendBundle, NetworkConfig)> {

        if !std::path::Path::new(vk_path).exists() {
            // NEVER use in production — see spec-trusted-setup
            // (→ see spec-security — Assumption 2: Trusted Setup Is Sound)
            crate::prover::setup::run_setup(max_signers, tree_depth, proving_key_path, vk_path)?;
        }

        let vk = crate::prover::setup::load_verification_key(vk_path)?;

        let network_launcher_id    = crate::puzzles::network_coin::derive_launcher_id(&genesis_coin, 0);
        let checkpoint_launcher_id = crate::puzzles::network_coin::derive_launcher_id(&genesis_coin, 1);
        let checkpoint_singleton_id = singleton_launcher_coin_id(checkpoint_launcher_id);

        let registration_coin_mod_hash = compile_registration_coin_mod_hash();
        let checkpoint_inner_mod_hash  = compile_checkpoint_inner_mod_hash();

        let config = NetworkConfig {
            network_coin_launcher_id:   network_launcher_id,
            checkpoint_launcher_id,
            registration_coin_mod_hash,
            checkpoint_inner_mod_hash,
            collateral_amount,
            tree_depth,
            max_signers,
            verification_key_hex: crate::prover::serialize::serialize_vk_to_hex(&vk)?,
            genesis_challenge,
        };

        let mut ctx = SpendContext::new();
        let bundle = crate::puzzles::network_coin::deploy_network(
            &mut ctx, genesis_coin, genesis_sk, &config, &vk,
        ).await?;

        Ok((bundle, config))
    }

    // -----------------------------------------------------------------------
    // Validator Registration
    // -----------------------------------------------------------------------

    /// Build a spend bundle that registers a validator by spending the network coin.
    /// Returns the SpendBundle — the caller broadcasts it.
    /// The validator must provide their secret key — the puzzle enforces AggSigMe.
    /// Duplicate registration (pubkey already in active set) returns AlreadyRegistered.
    /// The crate includes the pubkey as a memo for the indexer.
    /// (→ see spec-network-coin — Registration)
    /// (→ see spec-indexer — Important Notes: Memo is required for indexing)
    /// CLVM cost: ~5.3M units (→ see spec-clvm-costs — Spend Path 1)
    pub async fn register_validator(
        &self,
        validator_sk: &SecretKey,
    ) -> ConsensusResult<SpendBundle> {
        let state = self.state()?;
        let validator_pk = validator_sk.public_key();

        if state.validators.contains(&validator_pk) {
            return Err(ConsensusError::AlreadyRegistered(hex::encode(validator_pk.to_bytes())));
        }

        let mut ctx = SpendContext::new();
        let (bundle, _) = crate::puzzles::network_coin::register_validator(
            &mut ctx, &state.network_coin, validator_sk, &self.config,
        ).await?;

        Ok(bundle)
    }

    pub fn is_registered(&self, pubkey: &PublicKey) -> ConsensusResult<bool> {
        Ok(self.state()?.validators.contains(pubkey))
    }

    pub fn validator_set(&self) -> ConsensusResult<&ValidatorSet> {
        Ok(&self.state()?.validators)
    }

    // -----------------------------------------------------------------------
    // Checkpoint Submission
    // -----------------------------------------------------------------------

    /// Build and return a checkpoint spend bundle.
    ///
    /// The L2 provides:
    ///   new_state_root:            the new L2 state to commit on-chain
    ///   new_validator_merkle_root: from compute_new_validator_set()
    ///   new_validator_count:       from compute_new_validator_set()
    ///   signing_pubkeys:           pubkeys of k validators that signed
    ///   signatures:                their BLS signatures over validator_signing_message()
    ///
    /// The crate handles:
    ///   - Majority check: 2k > validator_count
    ///     (→ see spec-groth16-circuit — Constraint 3: Majority Threshold)
    ///   - Checkpoint message construction
    ///     (→ see spec-wire-format — Checkpoint Message)
    ///   - Merkle proof generation for each signer
    ///     (→ see spec-sparse-merkle-tree — Proof Generation)
    ///   - BLS aggregation of pubkeys and signatures
    ///     (→ see spec-wire-format — Aggregate Public Key and Aggregate Signature)
    ///   - Groth16 proof generation in spawn_blocking (5-15 minutes)
    ///     (→ see spec-groth16-circuit — Proof Generation)
    ///     (→ see spec-groth16-circuit — Constraint Count Estimates)
    ///   - Proof serialization to CLVM format
    ///     (→ see spec-wire-format — Groth16 Proof Format)
    ///   - Spend bundle assembly
    ///     (→ see spec-checkpoint-singleton — Checkpoint Spend)
    ///
    /// The returned SpendBundle has aggregated_signature = identity.
    /// No AGG_SIG_ME conditions are emitted by the checkpoint singleton.
    /// (→ see spec-checkpoint-singleton — Important Notes: No signatures on checkpoint spend)
    ///
    /// **The crate does NOT broadcast the bundle.** The caller is responsible
    /// for submitting the returned SpendBundle to a Chia full node via
    /// `push_tx()` or equivalent. This lets the L2 inspect, modify, or
    /// combine bundles before broadcast.
    ///
    /// CLVM cost: ~17.2M units (→ see spec-clvm-costs — Spend Path 2)
    /// Only one checkpoint in-flight at a time.
    /// (→ see spec-l2-integration — Important Notes: Only one checkpoint in-flight at a time)
    pub async fn build_checkpoint(
        &self,
        new_state_root:            Bytes32,
        new_validator_merkle_root: Bytes32,
        new_validator_count:       u64,
        signing_pubkeys:           &[PublicKey],
        signatures:                &[Signature],
    ) -> ConsensusResult<SpendBundle> {

        let state = self.state()?;
        let pk = self.proving_key.as_ref()
            .ok_or_else(|| ConsensusError::ProvingError(
                "proving key not loaded — call load_proving_key() first".into()
            ))?;

        // Majority check before expensive proof generation
        let k = signing_pubkeys.len() as u64;
        if 2 * k <= state.validators.count() {
            return Err(ConsensusError::BelowThreshold {
                count: state.validators.count(),
                actual: k as usize,
            });
        }

        let new_epoch = state.checkpoint.epoch + 1;

        // Checkpoint message per spec-wire-format — Checkpoint Message
        // 80-byte input: state_root(32) + merkle_root(32) + count_be8(8) + epoch_be8(8)
        let checkpoint_message = crate::prover::serialize::compute_checkpoint_message(
            new_state_root, new_validator_merkle_root, new_validator_count, new_epoch,
        );

        // Merkle membership proofs for each signer (private witness to circuit)
        // (→ see spec-sparse-merkle-tree — Proof Generation)
        let merkle_proofs: Vec<MerkleProof> = signing_pubkeys.iter()
            .map(|pk| {
                let slot = validator_slot(pk) % (1u64 << self.config.tree_depth);
                state.merkle_tree.prove_membership(pk)
                    .map_err(|_| ConsensusError::ValidatorNotFound(hex::encode(pk.to_bytes())))
            })
            .collect::<ConsensusResult<_>>()?;

        // BLS aggregation (→ see spec-wire-format — Aggregate Public Key and Aggregate Signature)
        let agg_signers = crate::prover::serialize::aggregate_pubkeys(signing_pubkeys)?;
        let agg_sig     = crate::prover::serialize::aggregate_signatures(signatures)?;

        // Groth16 proof generation — runs in spawn_blocking, 5-15 minutes
        // Circuit encodes: Merkle membership × k, G1 aggregate, majority threshold
        // (→ see spec-groth16-circuit — Full Circuit Implementation)
        let proof = tokio::task::spawn_blocking({
            let circuit = ConsensusCircuit {
                signing_pubkeys:          signing_pubkeys.iter()
                    .map(|pk| pk.to_bytes().try_into().unwrap())
                    .collect(),
                merkle_proofs:            merkle_proofs.clone(),
                actual_signers:           signing_pubkeys.len(),
                validator_merkle_root:    state.checkpoint.validator_merkle_root,
                validator_count:          state.checkpoint.validator_count,
                new_validator_merkle_root,
                new_validator_count,
                agg_signers:              agg_signers.to_bytes().try_into().unwrap(),
                checkpoint_message,
                max_signers:              self.config.max_signers,
                tree_depth:               self.config.tree_depth,
            };
            let pk = pk.clone();
            move || crate::prover::prove::generate_proof(circuit, &pk)
        })
        .await
        .map_err(|e| ConsensusError::ProvingError(e.to_string()))??;

        // Serialize proof for CLVM (→ see spec-wire-format — Groth16 Proof Format)
        let clvm_proof = crate::prover::serialize::serialize_proof(&proof)?;

        let mut ctx = SpendContext::new();
        let spend = crate::puzzles::checkpoint::spend_checkpoint_singleton(
            &mut ctx,
            &state.checkpoint,
            &self.config,
            clvm_proof,
            new_state_root,
            new_validator_merkle_root,
            new_validator_count,
            agg_signers,
            agg_sig,
        )?;

        Ok(SpendBundle {
            coin_spends: vec![spend],
            aggregated_signature: G2Affine::identity(),
        })
    }

    /// Compute the checkpoint message (32 bytes) that must be committed to.
    /// This is sha256(new_state_root + new_merkle_root + new_count_be8 + new_epoch_be8).
    /// Validators sign the full AGG_SIG_ME message from validator_signing_message() instead.
    /// (→ see spec-wire-format — Checkpoint Message)
    pub fn checkpoint_message(
        &self,
        new_state_root:            Bytes32,
        new_validator_merkle_root: Bytes32,
        new_validator_count:       u64,
    ) -> ConsensusResult<Bytes32> {
        let new_epoch = self.state()?.checkpoint.epoch + 1;
        Ok(crate::prover::serialize::compute_checkpoint_message(
            new_state_root, new_validator_merkle_root, new_validator_count, new_epoch,
        ))
    }

    /// Compute the full message each validator actually signs.
    /// = checkpoint_message + genesis_challenge + checkpoint_singleton_coin_id
    /// Format per spec-wire-format — Individual Signatures.
    /// Note: coin_id is the current checkpoint coin's coin ID, NOT the launcher ID.
    /// (→ see spec-wire-format — Common Mistakes: Coin ID vs launcher ID)
    pub fn validator_signing_message(
        &self,
        new_state_root:            Bytes32,
        new_validator_merkle_root: Bytes32,
        new_validator_count:       u64,
    ) -> ConsensusResult<Vec<u8>> {
        let state = self.state()?;
        let new_epoch = state.checkpoint.epoch + 1;
        Ok(crate::prover::serialize::compute_validator_signing_message(
            new_state_root,
            new_validator_merkle_root,
            new_validator_count,
            new_epoch,
            self.config.genesis_challenge,
            state.checkpoint.coin.coin_id(),
        ))
    }

    // -----------------------------------------------------------------------
    // Validator Set Construction
    // -----------------------------------------------------------------------

    /// Build the new validator set state for a pending checkpoint.
    /// Applies entries (new registrations) and exits (departures) to the
    /// current sparse Merkle tree.
    /// Returns (new_root, new_count, new_tree).
    ///
    /// The new_root and new_count are what you pass to build_checkpoint().
    /// Validators must sign the checkpoint_message that commits to these values.
    /// The majority signature over that message is the trustless proof the
    /// validator set is correct.
    /// (→ see spec-sparse-merkle-tree — Tree Updates)
    /// (→ see chip-groth16-l2-consensus — Why the validator set lives off-chain)
    pub fn compute_new_validator_set(
        &self,
        entries: &[PublicKey],
        exits:   &[PublicKey],
    ) -> ConsensusResult<(Bytes32, u64, SparseMerkleTree)> {
        let state = self.state()?;
        let mut new_tree = state.merkle_tree.clone();

        // Collision detection: reject entries that would collide with an existing slot
        // (→ see spec-sparse-merkle-tree — Slot Assignment: Slot collisions)
        for pk in entries {
            let slot = validator_slot(pk) % (1u64 << self.config.tree_depth);
            if new_tree.active_slots.contains_key(&slot) {
                return Err(ConsensusError::SlotCollision(hex::encode(pk.to_bytes())));
            }
            new_tree.insert(pk);
        }

        for pk in exits {
            new_tree.remove(pk);
        }

        let new_count = (state.validators.count() as i64
            + entries.len() as i64
            - exits.len() as i64)
            .max(0) as u64;

        Ok((new_tree.root(), new_count, new_tree))
    }

    // -----------------------------------------------------------------------
    // Collateral Recovery
    // -----------------------------------------------------------------------

    /// Build a spend bundle for a validator to recover their collateral.
    /// Returns the SpendBundle — the caller broadcasts it.
    ///
    /// Combines two spends in one atomic bundle:
    ///   Spend 1: Checkpoint singleton membership query (permissionless)
    ///     - Provides non-membership Merkle proof for validator_pubkey
    ///     - Recreates singleton unchanged
    ///     - Emits non-membership announcement
    ///     CLVM cost: ~4.1M units (→ see spec-clvm-costs — Spend Path 3)
    ///
    ///   Spend 2: Registration coin
    ///     - Asserts the non-membership announcement from Spend 1
    ///     - Returns collateral to collateral_destination
    ///     CLVM cost: ~3.3M units (→ see spec-clvm-costs — Spend Path 4)
    ///
    /// Total bundle cost: ~7.4M units (→ see spec-clvm-costs — Combined Bundle)
    /// aggregated_signature = identity (no AGG_SIG_ME conditions in either spend)
    ///
    /// Validator must not be in the current active set (already excluded by checkpoint).
    /// Epoch in the announcement must match the current checkpoint singleton epoch.
    /// (→ see spec-registration-coin — Important Notes: Epoch Matching)
    ///
    /// (→ see spec-checkpoint-singleton — Spend Path 2: Membership Query)
    /// (→ see spec-registration-coin — Spending the Registration Coin)
    /// (→ see spec-validator-onboarding — Voluntary Exit)
    pub async fn recover_collateral(
        &self,
        validator_pubkey:       PublicKey,
        collateral_destination: Bytes32,
    ) -> ConsensusResult<SpendBundle> {

        let state = self.state()?;

        if state.validators.contains(&validator_pubkey) {
            return Err(ConsensusError::AlreadyRegistered(
                "validator is still in the active set — must be excluded by a checkpoint first".into()
            ));
        }

        let registration_coin = crate::puzzles::registration_coin::fetch_registration_coin(
            &self.node, validator_pubkey, &self.config,
        ).await?
        .ok_or_else(|| ConsensusError::ValidatorNotFound(hex::encode(validator_pubkey.to_bytes())))?;

        let slot = validator_slot(&validator_pubkey) % (1u64 << self.config.tree_depth);
        let proof = state.merkle_tree.prove_non_membership(slot)?;

        let mut ctx = SpendContext::new();

        // Spend 1: membership query — permissionless, emits announcement
        let query_spend = crate::puzzles::checkpoint::spend_checkpoint_singleton_membership_query(
            &mut ctx, &state.checkpoint, &self.config,
            validator_pubkey, proof.leaf_index, proof.siblings.clone(), false,
        )?;

        // Spend 2: registration coin — asserts announcement, returns collateral
        let reg_spend = crate::puzzles::registration_coin::spend_registration_coin(
            &mut ctx,
            &RegistrationCoinSpend {
                coin: registration_coin,
                validator_pubkey,
                checkpoint_singleton_id: self.config.checkpoint_singleton_id(),
            },
            state.checkpoint.epoch,
            collateral_destination,
            vec![],
        )?;

        Ok(SpendBundle {
            coin_spends: vec![query_spend, reg_spend],
            aggregated_signature: G2Affine::identity(),
        })
    }

    // -----------------------------------------------------------------------
    // Membership Queries
    // -----------------------------------------------------------------------

    /// Fast local membership check. No RPC call.
    /// Call sync() first to ensure state is current.
    pub fn is_active(&self, pubkey: &PublicKey) -> ConsensusResult<bool> {
        Ok(self.state()?.validators.contains(pubkey))
    }

    /// Build a spend bundle that queries membership on-chain and emits an announcement.
    /// Returns the SpendBundle — the caller broadcasts it.
    /// The announcement can be asserted by other coins in the same bundle.
    /// Announcement format: spec-wire-format — Membership Announcement Format.
    /// Permissionless — no signature required.
    /// (→ see spec-checkpoint-singleton — Spend Path 2: Membership Query)
    /// CLVM cost: ~4.1M units (→ see spec-clvm-costs — Spend Path 3)
    pub async fn query_membership_on_chain(
        &self,
        pubkey:    PublicKey,
        is_member: bool,
    ) -> ConsensusResult<SpendBundle> {
        let state = self.state()?;
        let slot = validator_slot(&pubkey) % (1u64 << self.config.tree_depth);

        let proof = if is_member {
            state.merkle_tree.prove_membership(&pubkey)?
        } else {
            state.merkle_tree.prove_non_membership(slot)?
        };

        let mut ctx = SpendContext::new();
        let spend = crate::puzzles::checkpoint::spend_checkpoint_singleton_membership_query(
            &mut ctx, &state.checkpoint, &self.config,
            pubkey, proof.leaf_index, proof.siblings, is_member,
        )?;

        Ok(SpendBundle {
            coin_spends: vec![spend],
            aggregated_signature: G2Affine::identity(),
        })
    }

    /// Compute the membership announcement hash for use in AssertCoinAnnouncement.
    /// The format is sha256(sha256("membership" + epoch_be8 + pubkey_48 + is_member_byte))
    /// prepended with the checkpoint singleton coin ID.
    /// (→ see spec-wire-format — Membership Announcement Format)
    /// Note: uses the current checkpoint coin ID, not the launcher ID.
    /// (→ see spec-wire-format — Common Mistakes: Coin ID vs launcher ID)
    pub fn membership_announcement(
        &self,
        pubkey:    PublicKey,
        is_member: bool,
    ) -> ConsensusResult<Bytes32> {
        let state = self.state()?;
        Ok(crate::prover::serialize::compute_membership_announcement(
            state.checkpoint.epoch,
            &pubkey,
            is_member,
            state.checkpoint.coin.coin_id(),
        ))
    }

    // -----------------------------------------------------------------------
    // State Accessors
    // -----------------------------------------------------------------------

    /// Current epoch from the checkpoint singleton. Increments by 1 per checkpoint.
    /// Primary health signal for the L2 — if it stops advancing, checkpoints are stalled.
    /// (→ see spec-l2-integration — Monitoring)
    pub fn epoch(&self) -> ConsensusResult<u64> { Ok(self.state()?.checkpoint.epoch) }

    /// Current L2 state root committed on-chain.
    pub fn state_root(&self) -> ConsensusResult<Bytes32> { Ok(self.state()?.checkpoint.state_root) }

    /// Current validator Merkle root from the checkpoint singleton.
    /// The off-chain tree's root is verified equal to this on every sync().
    pub fn validator_merkle_root(&self) -> ConsensusResult<Bytes32> {
        Ok(self.state()?.checkpoint.validator_merkle_root)
    }

    /// Current validator count from the checkpoint singleton.
    /// May differ from the local count of registration coins if new validators
    /// registered since the last checkpoint. Use compute_new_validator_set() to
    /// compute the correct count for the next checkpoint.
    /// (→ see spec-l2-integration — Validator Set Transitions)
    pub fn validator_count(&self) -> ConsensusResult<u64> { Ok(self.state()?.checkpoint.validator_count) }

    /// Block height at which local state was last synced.
    pub fn synced_at(&self) -> ConsensusResult<u32> { Ok(self.state()?.synced_at) }
}
```

---

## Public Re-Exports

```rust
// src/lib.rs

pub use client::ConsensusClient;
pub use config::NetworkConfig;
pub use error::{ConsensusError, ConsensusResult};
pub use state::{NetworkState, CheckpointSingletonState, NetworkCoinState,
                ValidatorSet, ValidatorInfo};
pub use merkle::sparse::{SparseMerkleTree, EMPTY_LEAF, validator_slot, active_leaf,
                          compute_empty_nodes};
pub use merkle::proof::MerkleProof;
pub use prover::serialize::{
    ClvmProof, ClvmVerificationKey,
    serialize_proof, serialize_vk, serialize_vk_to_hex,
    compute_checkpoint_message, compute_validator_signing_message,
    compute_membership_announcement, aggregate_pubkeys, aggregate_signatures,
    bytes_to_scalar, compute_vk_input,
};
pub use indexer::mod_::{IndexerState, RegistrationCoinRecord, CheckpointRecord,
                        build_validator_set};
pub use indexer::cache::IndexerCache;

// Primitive types the L2 needs directly
pub use chia_protocol::{Bytes32, Coin, SpendBundle};
pub use chia_wallet_sdk::PublicKey;
```

---

## Integration Test

```rust
// tests/integration.rs

#[tokio::test]
async fn test_full_lifecycle() {
    let sim = Simulator::new();
    let deployer = sim.bls(200_000_000_000); // 200 XCH

    // Deploy (→ see spec-deployment-runbook)
    let (bundle, config) = ConsensusClient::deploy(
        sim.node(), deployer.coin, &deployer.sk,
        10_000_000_000, // 10 XCH collateral
        16,             // tree_depth (small for tests)
        10,             // max_signers
        "test_pk.bin",
        "test_vk.bin",
        sim.genesis_challenge(),
    ).await.unwrap();
    sim.push_tx(bundle).unwrap();

    let mut client = ConsensusClient::new(sim.node(), config.clone());
    client.load_proving_key("test_pk.bin").unwrap();
    client.set_cache_path("test_cache.json");
    client.sync().await.unwrap();

    assert_eq!(client.epoch().unwrap(), 0);
    assert_eq!(client.validator_count().unwrap(), 0);

    // Register 10 validators (→ see spec-validator-onboarding — Step 6)
    let validators: Vec<_> = (0..10).map(|_| sim.bls(10_000_000_000)).collect();
    for v in &validators {
        client.sync().await.unwrap();
        let bundle = client.register_validator(&v.sk).await.unwrap();
        sim.push_tx(bundle).unwrap();
    }

    client.sync().await.unwrap();
    // Validators are in local registration coins but not yet in checkpoint
    assert_eq!(client.validator_set().unwrap().count(), 10);

    // Submit first checkpoint including all 10 validators, 6 signing
    // (→ see spec-l2-integration — Checkpoint Submission Flow)
    let new_state_root = [1u8; 32];
    let (new_merkle_root, new_count, _) = client
        .compute_new_validator_set(&validators.iter().map(|v| v.pk).collect::<Vec<_>>(), &[])
        .unwrap();

    let signing_message = client.validator_signing_message(
        new_state_root, new_merkle_root, new_count,
    ).unwrap();

    let signers: Vec<_> = validators[..6].iter()
        .map(|v| (v.pk, v.sk.sign(&signing_message)))
        .collect();

    let pubkeys: Vec<_> = signers.iter().map(|(pk, _)| *pk).collect();
    let sigs:    Vec<_> = signers.iter().map(|(_, s)| s.clone()).collect();

    let bundle = client.build_checkpoint(
        new_state_root, new_merkle_root, new_count, &pubkeys, &sigs,
    ).await.unwrap();
    sim.push_tx(bundle).unwrap(); // Caller broadcasts

    client.sync().await.unwrap();
    assert_eq!(client.epoch().unwrap(), 1);
    assert_eq!(client.validator_count().unwrap(), 10);
    assert_eq!(client.state_root().unwrap(), new_state_root);

    // Validator 0 exits: build checkpoint excluding them, then broadcast
    let exiting_pk = validators[0].pk;
    let (new_merkle_root2, new_count2, _) = client
        .compute_new_validator_set(&[], &[exiting_pk])
        .unwrap();

    let signing_message2 = client.validator_signing_message(
        new_state_root, new_merkle_root2, new_count2,
    ).unwrap();

    let signers2: Vec<_> = validators[1..7].iter()
        .map(|v| (v.pk, v.sk.sign(&signing_message2)))
        .collect();

    let pubkeys2: Vec<_> = signers2.iter().map(|(pk, _)| *pk).collect();
    let sigs2:    Vec<_> = signers2.iter().map(|(_, s)| s.clone()).collect();

    let bundle2 = client.build_checkpoint(
        new_state_root, new_merkle_root2, new_count2, &pubkeys2, &sigs2,
    ).await.unwrap();
    sim.push_tx(bundle2).unwrap(); // Caller broadcasts

    client.sync().await.unwrap();
    assert_eq!(client.epoch().unwrap(), 2);
    assert_eq!(client.validator_count().unwrap(), 9);
    assert!(!client.is_active(&exiting_pk).unwrap());

    // Validator 0 recovers collateral
    // (→ see spec-validator-onboarding — Voluntary Exit — Step 3)
    let recovery = client.recover_collateral(exiting_pk, validators[0].puzzle_hash).await.unwrap();
    sim.push_tx(recovery).unwrap();

    println!("full lifecycle test passed");
}
```

---

## Important Notes

**sync() before every operation**

The client caches on-chain state in memory. Chain state advances with every
block. Always call `sync()` before any operation that reads current state.
Use `synced_at()` to check when state was last refreshed. The indexer loads
from cache on first call and only processes new blocks, so restart overhead
is minimal when using `set_cache_path()`.

**Proof generation is slow**

`build_checkpoint()` runs proof generation in `spawn_blocking` so it does
not block the async runtime, but it ties up a thread for 5-15 minutes at
MAX_SIGNERS=10, TREE_DEPTH=32. See constraint counts:
[spec-groth16-circuit](spec-groth16-circuit.md) — Constraint Count Estimates.
Do not set a short timeout. Only one checkpoint can be in-flight at a time:
[spec-l2-integration](spec-l2-integration.md) — Important Notes.

**The crate builds SpendBundles but NEVER broadcasts them**

Every method that produces a spend bundle (`deploy()`, `register_validator()`,
`build_checkpoint()`, `recover_collateral()`, `query_membership_on_chain()`)
returns the `SpendBundle` to the caller. The crate has no `push_tx()`,
`send_transaction()`, or any other broadcast mechanism. The importing project
is responsible for broadcasting the bundle to a Chia full node.

This design is intentional:
- The L2 can inspect, modify, or combine bundles before broadcast.
- The L2 controls fee selection, timing, and retry logic.
- The crate remains testable in isolation without a live node.
- Bundle composition allows the L2 to include its own spends alongside
  consensus operations.

**Config is immutable after deployment**

`NetworkConfig` is fixed at deployment time. Changing `collateral_amount`,
`tree_depth`, or the VK requires a new deployment. The network coin, checkpoint
singleton, and the trusted setup are all tied to these values.

**validator_count() vs local count**

`validator_count()` returns the count from the on-chain checkpoint singleton.
`validator_set().count()` returns the local count of indexed registration coins.
These differ whenever validators register but a checkpoint has not yet included
them. Use `compute_new_validator_set()` with the correct entries and exits to
produce the count to include in the next checkpoint.

**StateMismatch after sync**

If `sync()` returns `StateMismatch`, the local Merkle tree computed from
registration coins does not match the `validator_merkle_root` stored on-chain.
The indexer state is inconsistent. Trigger a full re-index by deleting the
cache file and calling `sync()` again. If it persists, there is a bug in the
sparse Merkle tree implementation — see:
[spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Common Implementation
Mistakes.

**Slot collisions**

`compute_new_validator_set()` checks for slot collisions in the entries list.
The probability is negligible but it must be handled. If two validators hash
to the same slot, the second registration must be rejected at the L2 level:
[spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Slot Collisions and
[spec-l2-integration](spec-l2-integration.md) — Validator Set Transitions.
