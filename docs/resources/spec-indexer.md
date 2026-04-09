# Indexer - Technical Specification

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Depends on** | [spec-network-coin](spec-network-coin.md) | Queries network coin spends to find valid registration coin parents |
| **Depends on** | [spec-registration-coin](spec-registration-coin.md) | Tracks unspent registration coins as the active validator set. Extracts pubkeys from memos. |
| **Depends on** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | Parses checkpoint state announcements to track epoch, state_root, validator_merkle_root, validator_count |
| **Depends on** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | Rebuilds the tree from registration coins after each sync. Verifies root matches on-chain state. |
| **Depends on** | [spec-wire-format](spec-wire-format.md) | Checkpoint state announcement format, registration message format, memo conventions |
| **Enables** | [spec-consensus-crate](spec-consensus-crate.md) | ConsensusClient.sync() calls the indexer. NetworkState is produced by the indexer. |
| **Referenced by** | [spec-security](spec-security.md) | Assumption 3 describes how majority signature enforces Merkle root correctness off-chain |
| **Referenced by** | [spec-l2-integration](spec-l2-integration.md) | Event loop calls sync() which triggers the indexer |

---

## Overview

The indexer is responsible for maintaining a reliable local view of on-chain
state that the `ConsensusClient`
(→ see [spec-consensus-crate](spec-consensus-crate.md)) uses to build spend
bundles and generate proofs. It must handle chain reorgs, efficiently sync
from any point in history, and provide fast access to the current validator
set with lineage verification.

The indexer runs as a background concern inside the crate. The
`ConsensusClient` calls `sync()` which triggers the indexer to update from
the last known block. After sync completes, the indexer verifies that the
sparse Merkle tree it has built from registration coins matches the
`validator_merkle_root` stored in the checkpoint singleton. Any mismatch is
surfaced as a `StateMismatch` error
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Error Type).

---

## What the Indexer Tracks

1. **Network coin state**: Current unspent coin, lineage proof, last spend
   height. Needed before every registration
   (→ see [spec-network-coin](spec-network-coin.md) — Querying the Network
   Coin State).

2. **Checkpoint singleton state**: Current unspent coin, lineage proof, decoded
   state (epoch, state_root, validator_merkle_root, validator_count). Decoded
   from checkpoint state announcements per
   (→ see [spec-wire-format](spec-wire-format.md) — Checkpoint State
   Announcement Format).

3. **Registration coins**: All unspent registration coins whose parent is a
   network coin spend, with their associated validator pubkeys extracted from
   memos. Used to build the validator set and the sparse Merkle tree
   (→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)).

4. **Checkpoint history**: Ordered list of all past checkpoint states for
   audit and the indexer's own reorg recovery.

---

## Module Structure

```
src/indexer/
  mod.rs           - IndexerState, sync entry point
  chain.rs         - raw chain queries, event parsing
  validator_set.rs - validator set construction with lineage verification
  reorg.rs         - reorg detection and rollback
  cache.rs         - persistent local cache
```

---

## IndexerState

```rust
pub struct IndexerState {
    pub last_synced_height:  u32,
    pub network_coin:        NetworkCoinState,
    pub checkpoint:          CheckpointSingletonState,
    /// All valid registration coins, keyed by pubkey.
    /// Only includes coins whose parent traces to a network coin spend.
    /// (→ see spec-security — Lineage proof enforcement)
    pub registration_coins:  HashMap<PublicKey, RegistrationCoinRecord>,
    pub checkpoint_history:  Vec<CheckpointRecord>,
    cache:                   IndexerCache,
}

pub struct RegistrationCoinRecord {
    pub coin:                    Coin,
    pub pubkey:                  PublicKey,
    pub registered_at_height:    u32,
    pub registered_at_epoch:     u64,
}

pub struct CheckpointRecord {
    pub epoch:                 u64,
    pub state_root:            Bytes32,
    pub validator_merkle_root: Bytes32,
    pub validator_count:       u64,
    pub confirmed_at_height:   u32,
    pub coin_id:               Bytes32,
}
```

---

## Sync Algorithm

```rust
impl IndexerState {

    pub async fn sync(
        &mut self,
        node: &FullNodeClient,
        config: &NetworkConfig,
    ) -> Result<(), IndexerError> {

        let peak = node.get_blockchain_state().await?.peak_height;

        // Detect reorg: if peak is behind our last synced height
        if peak < self.last_synced_height {
            self.handle_reorg(node, config, peak).await?;
            return Ok(());
        }

        let batch_size = 100;
        let mut height = self.last_synced_height + 1;

        while height <= peak {
            let end = (height + batch_size).min(peak + 1);
            self.process_block_range(node, config, height, end).await?;
            height = end;
        }

        self.last_synced_height = peak;
        self.cache.save(&self)?;

        Ok(())
    }

    async fn process_block_range(
        &mut self,
        node: &FullNodeClient,
        config: &NetworkConfig,
        start: u32,
        end: u32,
    ) -> Result<(), IndexerError> {

        let additions = node
            .get_additions_and_removals_by_height(start, end)
            .await?;

        for (height, added, removed) in additions {
            self.process_additions(node, config, height, &added).await?;
            self.process_removals(config, &removed)?;
        }

        Ok(())
    }

    async fn process_additions(
        &mut self,
        node: &FullNodeClient,
        config: &NetworkConfig,
        height: u32,
        added: &[Coin],
    ) -> Result<(), IndexerError> {

        for coin in added {
            if self.is_network_coin(coin, config) {
                self.update_network_coin(node, coin, height).await?;
                continue;
            }

            if self.is_checkpoint_coin(coin, config) {
                self.update_checkpoint(node, coin, height).await?;
                continue;
            }

            if let Some(record) = self.try_parse_registration_coin(
                node, coin, config, height,
            ).await? {
                self.registration_coins.insert(record.pubkey, record);
            }
        }

        Ok(())
    }

    fn process_removals(
        &mut self,
        config: &NetworkConfig,
        removed: &[Coin],
    ) -> Result<(), IndexerError> {

        // Spent registration coin = validator exited and recovered collateral
        // (→ see spec-registration-coin — Important Notes: Spent registration coins)
        for coin in removed {
            let pubkey = self.registration_coins
                .iter()
                .find(|(_, r)| r.coin == *coin)
                .map(|(pk, _)| *pk);

            if let Some(pk) = pubkey {
                self.registration_coins.remove(&pk);
            }
        }

        Ok(())
    }
}
```

---

## Checkpoint State Updates

When the checkpoint singleton is spent and recreated, the new state is parsed
from the checkpoint state announcement emitted in that spend. The announcement
format is defined in
(→ see [spec-wire-format](spec-wire-format.md) — Checkpoint State Announcement
Format):

```rust
async fn update_checkpoint(
    &mut self,
    node: &FullNodeClient,
    coin: &Coin,
    height: u32,
) -> Result<(), IndexerError> {

    let parent_spend = node
        .get_puzzle_and_solution(coin.parent_coin_info, height)
        .await?
        .ok_or(IndexerError::MissingParentSpend)?;

    let lineage_proof = extract_lineage_proof(&parent_spend)?;

    // Parse the checkpoint solution fields to extract new state
    // The announcement contains sha256 of state so we parse the solution instead
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
```

---

## Registration Coin Detection and Lineage Verification

The lineage check is the core security mechanism of the system
(→ see [spec-security](spec-security.md) — Lineage proof enforcement). A
registration coin is only valid if its parent coin ID is a network coin spend.
The puzzle hash must also match what
`registration_coin_puzzle_hash()` would compute for the extracted pubkey
(→ see [spec-network-coin](spec-network-coin.md) — Computing the Registration
Coin Puzzle Hash):

```rust
async fn try_parse_registration_coin(
    &self,
    node: &FullNodeClient,
    coin: &Coin,
    config: &NetworkConfig,
    height: u32,
) -> Result<Option<RegistrationCoinRecord>, IndexerError> {

    // Step 1: Check if parent is a known network coin spend
    // (→ see spec-network-coin — Querying the Network Coin State)
    let is_valid_parent = self.network_coin_spend_ids.contains(
        &coin.parent_coin_info
    );
    if !is_valid_parent {
        return Ok(None);
    }

    // Step 2: Extract the validator pubkey from the memo on the parent spend
    // Memo convention per spec-network-coin — Important Notes: Memo convention
    // and spec-registration-coin — Important Notes: Memo convention
    let parent_spend = node
        .get_puzzle_and_solution(coin.parent_coin_info, height)
        .await?
        .ok_or(IndexerError::MissingParentSpend)?;

    let pubkey = extract_pubkey_from_memo(&parent_spend, coin)
        .ok_or(IndexerError::MissingPubkeyMemo)?;

    // Step 3: Verify puzzle hash matches expected for this pubkey
    // Must match network_coin_inner.rue curry_hash call exactly
    // (→ see spec-network-coin — What the Puzzle Does)
    let expected_hash = registration_coin_puzzle_hash(
        config.registration_coin_mod_hash,
        pubkey,
        config.checkpoint_singleton_id(),
    );
    if coin.puzzle_hash != expected_hash {
        return Err(IndexerError::PuzzleHashMismatch);
    }

    // Step 4: Verify collateral amount
    if coin.amount != config.collateral_amount {
        return Err(IndexerError::WrongCollateralAmount);
    }

    Ok(Some(RegistrationCoinRecord {
        coin:                 *coin,
        pubkey,
        registered_at_height: height,
        registered_at_epoch:  self.checkpoint.epoch,
    }))
}

fn extract_pubkey_from_memo(
    parent_spend: &CoinSpend,
    child_coin: &Coin,
) -> Option<PublicKey> {
    // First memo on the CreateCoin condition that matches this child
    // is the validator pubkey per the memo convention defined in
    // spec-network-coin — Important Notes: Memo convention
    let conditions = run_puzzle_and_solution(parent_spend).ok()?;

    for condition in conditions {
        if let Condition::CreateCoin(cc) = condition {
            if cc.puzzle_hash == child_coin.puzzle_hash
                && cc.amount == child_coin.amount
            {
                if let Some(memo) = cc.memos.first() {
                    if memo.len() == 48 {
                        return PublicKey::from_bytes(memo).ok();
                    }
                }
            }
        }
    }
    None
}
```

---

## Merkle Root Consistency Check

After every sync, the indexer verifies that the sparse Merkle tree it built
from local registration coins matches the root stored in the checkpoint
singleton. This is the critical consistency check that catches any indexing
bugs early. The tree is built using the algorithm from
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Root
Computation):

```rust
pub fn verify_merkle_consistency(
    validator_set: &ValidatorSet,
    checkpoint: &CheckpointSingletonState,
    tree_depth: u32,
) -> Result<SparseMerkleTree, IndexerError> {

    // Build tree per spec-sparse-merkle-tree exactly
    let tree = SparseMerkleTree::from_validators(
        &validator_set.validators,
        tree_depth,
    );

    let computed_root = tree.root();

    if computed_root != checkpoint.validator_merkle_root {
        return Err(IndexerError::MerkleRootMismatch {
            computed:  hex::encode(computed_root),
            on_chain:  hex::encode(checkpoint.validator_merkle_root),
        });
    }

    Ok(tree)
}
```

A `MerkleRootMismatch` error means either:
- The indexer missed some registration coin additions or removals
- There is a bug in the sparse Merkle tree implementation
  (→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Common
  Implementation Mistakes)
- The indexer cache is corrupted

In any of these cases trigger a full re-index
(→ see this document — Reorg Handling: full_reindex).

---

## Reorg Handling

A reorg occurs when the node reports a peak height lower than our last synced
height:

```rust
async fn handle_reorg(
    &mut self,
    node: &FullNodeClient,
    config: &NetworkConfig,
    new_peak: u32,
) -> Result<(), IndexerError> {

    // Find the last checkpoint confirmed at or before new_peak
    let safe_checkpoint = self.checkpoint_history
        .iter()
        .rev()
        .find(|c| c.confirmed_at_height <= new_peak)
        .cloned();

    if let Some(safe) = safe_checkpoint {
        self.checkpoint_history.retain(|c| c.epoch <= safe.epoch);
        self.registration_coins.clear();
        self.last_synced_height = safe.confirmed_at_height;
        self.sync(node, config).await?;
    } else {
        self.full_reindex(node, config).await?;
    }

    Ok(())
}

async fn full_reindex(
    &mut self,
    node: &FullNodeClient,
    config: &NetworkConfig,
) -> Result<(), IndexerError> {

    self.registration_coins.clear();
    self.checkpoint_history.clear();
    self.last_synced_height = 0;

    let launcher_height = node
        .get_coin_record_by_name(config.network_coin_launcher_id)
        .await?
        .ok_or(IndexerError::NetworkCoinNotFound)?
        .confirmed_block_index;

    self.last_synced_height = launcher_height.saturating_sub(1);
    self.sync(node, config).await
}
```

---

## Persistent Cache

```rust
#[derive(Serialize, Deserialize)]
pub struct IndexerCache {
    pub last_synced_height:  u32,
    pub checkpoint_history:  Vec<CheckpointRecord>,
    pub registration_coins:  Vec<RegistrationCoinRecord>,
    pub network_coin_state:  SerializableNetworkCoinState,
    pub checkpoint_state:    SerializableCheckpointState,
}

impl IndexerCache {
    pub fn load(path: &str) -> Result<Option<IndexerCache>, CacheError> {
        if !std::path::Path::new(path).exists() { return Ok(None); }
        let bytes = std::fs::read(path)?;
        Ok(Some(serde_json::from_slice(&bytes)?))
    }

    pub fn save(&self, path: &str) -> Result<(), CacheError> {
        let bytes = serde_json::to_vec_pretty(self)?;
        let tmp = format!("{}.tmp", path);
        std::fs::write(&tmp, &bytes)?;
        std::fs::rename(&tmp, path)?;  // atomic write
        Ok(())
    }
}
```

---

## Validator Set Construction

After syncing, the validator set is derived from the indexed registration coins.
The sort order must be consistent with the slot assignment in
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Slot
Assignment):

```rust
pub fn build_validator_set(
    registration_coins: &HashMap<PublicKey, RegistrationCoinRecord>,
) -> ValidatorSet {

    let mut validators: Vec<ValidatorInfo> = registration_coins
        .values()
        .map(|r| ValidatorInfo {
            pubkey:            r.pubkey,
            registration_coin: r.coin,
        })
        .collect();

    // Sort by pubkey bytes for deterministic ordering
    validators.sort_by_key(|v| v.pubkey.to_bytes());

    ValidatorSet { validators }
}
```

---

## Important Notes

**Memo is required for indexing**

The indexer relies on the validator pubkey being stored as a memo on the
registration coin creation condition
(→ see [spec-network-coin](spec-network-coin.md) — Important Notes: Memo
convention). If the network coin driver does not include this memo the indexer
cannot determine which pubkey a registration coin belongs to. The memo
convention is not enforced on-chain so it is the driver's responsibility.

**Registration coin deduplication**

If the same pubkey registers twice, both registration coins are indexed but
`build_validator_set()` deduplicates by pubkey. Only one entry per pubkey
appears in the validator set. This does not give that validator two votes.

**Cache invalidation on redeployment**

If the network is redeployed (new launcher IDs), the cache must be deleted.
The cache path should include the launcher IDs to prevent stale cache issues
across deployments.

**Node rate limits**

The indexer makes many RPC calls during a full re-index. Use batch queries
where possible and back off on rate limit errors. For production, point the
indexer at a dedicated node you control.
