# chia-l2-consensus

Groth16-based L2 validator consensus for Chia. Proves majority validator
agreement to the L1 at constant cost using zero-knowledge proofs.

**This crate builds spend bundles. It never broadcasts transactions.** The
importing project submits bundles via `push_tx()`.

## What This Crate Does

An L2 system imports this crate as a dependency. The crate handles:

- Deploying the L2 network (two singletons + configuration)
- Registering validators (BLS key + collateral lock)
- Building checkpoints (Groth16 proof + BLS aggregate signature)
- Collateral recovery (two-phase: delay coin → timed release)
- On-chain state indexing (validator set, Merkle tree, lineage verification)

The L2 system handles: broadcasting, fee selection, retry logic, timing.

## Public API

### Types

```rust
use chia_l2_consensus::{
    ConsensusClient,           // Main entry point — all operations go through this
    NetworkConfig,             // Deployment parameters (immutable after deploy)
    ConsensusError,            // Error type with actionable variants
    ConsensusResult,           // Result<T, ConsensusError>
    CheckpointSingletonState,  // On-chain state: epoch, roots, count
    ValidatorSet,              // Active validators from last sync
    ValidatorInfo,             // Pubkey + registration coin
    DeploymentArtifacts,       // JSON-serializable deployment output
    VkJson,                    // Verification key in publishable format
    Bytes32,                   // Re-exported from chia-protocol
    SpendBundle,               // Re-exported from chia-protocol
};
```

### Lifecycle

```rust
// 1. Create client with saved config
let client = ConsensusClient::new(config, IndexerCache::in_memory());

// 2. Connect to Chia blockchain (decentralized peers + coinset.org fallback)
client.connect(ChiaQueryConfig {
    network: NetworkType::Mainnet,
    ..Default::default()
}).await?;

// 3. Sync on-chain state
let validator_set = client.sync().await?;

// 4. Operations (all return SpendBundle — caller broadcasts)
let bundle = client.register_validator(&pk, &wallet, "name", 0, fee).await?;
let bundle = client.build_checkpoint(state_root, signers, sigs).await?;
let bundle = client.recover_collateral(&pubkey).await?;
let bundle = client.release_collateral(delay_coin, dest, amount).await?;
```

### ConsensusClient Methods

| Method | Returns | When to Call |
|--------|---------|-------------|
| `new(config, cache)` | `ConsensusClient` | Once at startup |
| `connect(query_config)` | `Result<()>` | Before sync |
| `sync()` | `Result<ValidatorSet>` | Before every operation |
| `deploy()` | `Result<SpendBundle>` | Once per L2 network (genesis) |
| `register_validator(pk, wallet, name, idx, fee)` | `Result<SpendBundle>` | Validator joining |
| `build_checkpoint(state_root, signers, sigs)` | `Result<SpendBundle>` | L2 → L1 settlement |
| `recover_collateral(pubkey)` | `Result<SpendBundle>` | Validator exiting (Phase 1) |
| `release_collateral(coin, dest, amount)` | `Result<SpendBundle>` | After delay period (Phase 2) |
| `epoch()` | `Result<u64>` | Health monitoring |
| `state_root()` | `Result<Bytes32>` | Read current L2 state |
| `validator_merkle_root()` | `Result<Bytes32>` | Read validator set root |
| `validator_count()` | `Result<u64>` | Read validator count |
| `is_active(pubkey)` | `Result<bool>` | Check membership (local, no RPC) |
| `checkpoint_message(sr, mr, vc)` | `Result<[u8; 32]>` | Compute what validators sign |
| `validator_signing_message(sr, mr, vc)` | `Result<[u8; 96]>` | Full AGG_SIG_ME message |
| `config()` | `&NetworkConfig` | Read deployment params |
| `set_cache_path(path)` | `()` | Enable persistent indexer cache |

### NetworkConfig

Immutable after deployment. Saved as JSON and loaded on every startup.

```rust
pub struct NetworkConfig {
    pub network_coin_launcher_id: Bytes32,   // Network coin singleton ID
    pub checkpoint_launcher_id: Bytes32,     // Checkpoint singleton ID
    pub registration_coin_mod_hash: Bytes32, // Uncurried registration coin hash
    pub checkpoint_inner_mod_hash: Bytes32,  // Uncurried checkpoint inner hash
    pub withdraw_delay_mod_hash: Bytes32,    // Uncurried withdraw delay coin hash
    pub collateral_amount: u64,              // Per-validator collateral (mojos)
    pub tree_depth: u32,                     // Sparse Merkle tree depth (32)
    pub max_signers: usize,                  // Circuit max signers (20,000)
    pub verification_key_hex: String,        // Groth16 VK (672 bytes as hex)
    pub genesis_challenge: Bytes32,          // Chia network ID
    pub withdraw_delay_blocks: u64,          // Blocks before collateral release (24,000 ≈ 5 days)
}
```

### Error Variants

| Variant | Meaning | Recovery Action |
|---------|---------|----------------|
| `NotDeployed` | `sync()` not called yet | Call `sync()` |
| `AlreadyRegistered` | Pubkey already in active set | Check `is_active()` first |
| `ValidatorNotFound` | Pubkey not in active set | Verify registration succeeded |
| `BelowThreshold` | Not enough signers for majority | Collect more signatures |
| `StateMismatch` | Local Merkle root ≠ on-chain | Delete cache, re-sync |
| `InsufficientFunds` | Wallet cannot fund collateral + fee | Add XCH to wallet |
| `RpcError` | Blockchain query failed | Retry or check connectivity |
| `InvalidLineage` | Registration coin has wrong parent | Ignore this coin |
| `InvalidMerkleProof` | Merkle proof verification failed | Rebuild tree |
| `ProvingError` | Groth16 proof generation failed | Check proving key loaded |
| `SerializationError` | Type size mismatch | Check wire format constants |
| `SpendRejected` | Node rejected the bundle | Re-sync, check epoch |
| `SlotCollision` | Two validators hash to same slot | Reject second registration |

## On-Chain Architecture

Four Rue puzzles compiled to CLVM, deployed as Chia coins:

```
Network Coin (singleton)
  │ Spent to register validator
  │ Creates registration coin with collateral
  │ Recreates itself for next registration
  │
  └──► Registration Coin (one per validator)
         │ Holds collateral until validator exits
         │ Asserts non-membership announcement from checkpoint
         │ Creates withdraw delay coin (NOT direct destination)
         │
         └──► Withdraw Delay Coin (time-locked)
                │ Holds collateral for 24,000 blocks (~5 days)
                │ ASSERT_HEIGHT_RELATIVE enforces delay
                │ Permissionless release after delay
                │
                └──► Destination Coin (validator's wallet)

Checkpoint Singleton (singleton)
  ├── Spend Path 1: Checkpoint
  │     Verifies Groth16 proof (membership + majority)
  │     Verifies BLS aggregate signature
  │     Updates state, increments epoch
  │
  └── Spend Path 2: Membership Query (permissionless)
        Verifies Merkle proof
        Emits membership/non-membership announcement
        Recreates singleton unchanged
```

### Checkpoint Message Format

```
sha256(state_root[32] || merkle_root[32] || count_be[8] || epoch_be[8] || network_id[32])
= 112 bytes → 32-byte hash
```

The `network_id` (network coin launcher ID) prevents cross-network replay.
The epoch is computed internally by the puzzle (`old_epoch + 1`), never
accepted from the solution.

### Two-Phase Collateral Recovery

```
Phase 1: recover_collateral()
  ├── Checkpoint membership query → non-membership announcement
  └── Registration coin spend → creates withdraw delay coin
       └── Delay coin has: destination, amount, delay (all curried, immutable)

[wait 24,000 L1 blocks ≈ 5 days]

Phase 2: release_collateral()
  └── Withdraw delay coin spend → funds arrive at destination
       └── Permissionless (no signature needed)
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `chia-wallet-sdk` | SpendContext, Launcher, singleton drivers |
| `chia-protocol` | Bytes32, Coin, SpendBundle |
| `chia-query` | Decentralized blockchain queries (peers + coinset.org) |
| `dig-l1-wallet` | Coin selection for collateral funding |
| `ark-groth16` | Groth16 proving system |
| `ark-bls12-381` | BLS12-381 curve for ZK proofs |
| `blst` | BLS aggregate signatures |
| `clvmr` | CLVM runtime for puzzle execution |

## Key Constants

| Constant | Value | Meaning |
|----------|-------|---------|
| `TREE_DEPTH` | 32 | Sparse Merkle tree depth (4B validator slots) |
| `MAX_SIGNERS` | 20,000 | Max signers per checkpoint (circuit limit) |
| `DEFAULT_WITHDRAW_DELAY_BLOCKS` | 24,000 | ~5 days at ~18s/block |
| `PUBLIC_INPUT_COUNT` | 6 | Groth16 circuit public inputs |
| `EMPTY_LEAF_HASH` | `sha256([0; 48])` | Empty Merkle tree slot marker |

## Security Properties

- **Groth16 proof**: Proves k pubkeys are registered AND form a majority AND
  their G1 sum matches the aggregate key. Cannot be forged without the proving
  key AND valid validator keys.
- **BLS signature**: Proves the majority actually signed this specific
  checkpoint. Binds proof to state transition.
- **Collateral delay**: 5-day window for slashing before funds leave the system.
- **No condition injection**: Registration coin, checkpoint, and delay coin
  puzzles accept no solution-provided conditions (SEC-008).
- **Cross-network replay prevention**: Checkpoint message includes network ID.
- **Epoch replay prevention**: Membership announcements include epoch number.
- **Trusted setup**: Requires multi-party computation (MPC) ceremony. At least
  one honest participant needed for soundness.

## Project Structure

```
puzzles/                          # Rue source → compiled CLVM
  ├── network_coin_inner.rue      # Registration authority
  ├── registration_coin.rue       # Collateral holder
  ├── checkpoint_inner.rue        # State authority + membership oracle
  ├── withdraw_delay_coin.rue     # Time-locked release
  └── compiled/                   # .hex (bytecode) + .hash (tree hash)

src/
  ├── client.rs                   # ConsensusClient (public API facade)
  ├── config.rs                   # NetworkConfig, DeploymentArtifacts
  ├── error.rs                    # ConsensusError variants
  ├── state.rs                    # CheckpointSingletonState, ValidatorSet
  ├── puzzles/                    # Spend bundle construction drivers
  ├── merkle/                     # Sparse Merkle tree (depth 32, SHA-256)
  ├── prover/                     # Groth16 circuit + proof generation
  ├── indexer/                    # On-chain state tracking + lineage verification
  ├── validator/                  # Key generation, signing, exit flows
  └── testing.rs                  # Internal type re-exports for VV tests

tests/                            # 1,074 tests across 99 VV test files
docs/
  ├── resources/                  # CHIP + 15 spec documents
  └── requirements/               # 108 requirements across 12 domains
```

## License

MIT
