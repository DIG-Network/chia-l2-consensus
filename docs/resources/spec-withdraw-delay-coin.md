# Withdraw Delay Coin — Technical Specification

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Created by** | [spec-registration-coin](spec-registration-coin.md) | Registration coin creates the withdraw delay coin instead of sending directly to destination |
| **Depends on** | CLVM `ASSERT_HEIGHT_RELATIVE` | Enforces the block delay before funds can be released |
| **Implements** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | Collateral recovery with time lock |
| **Enables** | [spec-consensus-crate](spec-consensus-crate.md) | `release_collateral()` method for spending the delay coin |
| **Referenced by** | [spec-security](spec-security.md) | Withdraw delay prevents instant exits, gives network time to respond |
| **Referenced by** | [spec-clvm-costs](spec-clvm-costs.md) | Spend Path 5: Withdraw Delay Coin cost analysis |
| **Referenced by** | [spec-validator-onboarding](spec-validator-onboarding.md) | Exit flow now has a waiting period |

---

## Overview

The withdraw delay coin introduces a mandatory waiting period between
collateral recovery initiation and actual fund release. When a validator exits
and spends their registration coin, the collateral no longer goes directly to
the destination address. Instead, the registration coin creates a withdraw
delay coin that holds the collateral. The validator must then spend this delay
coin in a separate transaction after a configurable number of L1 blocks have
passed.

This two-phase withdrawal serves several purposes:

1. **Slashing window**: Gives the network time to detect and respond to
   misbehavior that occurred just before exit. Without a delay, a malicious
   validator could misbehave, immediately exit, and recover collateral before
   the network can react.

2. **Coordination buffer**: Provides time for the L2 to process the validator
   removal and update any dependent state before funds leave the system.

3. **Front-running prevention**: Makes it impossible to detect an upcoming
   slash and front-run it by recovering collateral in the same block.

The delay is configurable per network deployment via `withdraw_delay_blocks`
in `NetworkConfig`.

---

## Architecture

### Current Flow (Before This Change)

```
Registration Coin
    │ CreateCoin(destination, amount)
    └──► Destination Coin (immediate)
```

### New Flow (With Withdraw Delay)

```
Registration Coin
    │ CreateCoin(withdraw_delay_puzzle_hash, amount)
    └──► Withdraw Delay Coin (holds collateral)
              │ [wait WITHDRAW_DELAY_BLOCKS]
              │ AssertHeightRelative(WITHDRAW_DELAY_BLOCKS)
              │ CreateCoin(DESTINATION, AMOUNT)
              └──► Destination Coin (after delay)
```

### Spend Bundle Structure

**Bundle 1: Collateral Recovery (unchanged structure, different output)**

```
Spend 1: Checkpoint singleton membership query (permissionless)
    → emits non-membership announcement

Spend 2: Registration coin
    → asserts non-membership announcement
    → creates WITHDRAW DELAY COIN (not destination coin)
```

**Bundle 2: Release Collateral (new, after delay)**

```
Spend 1: Withdraw delay coin
    → asserts WITHDRAW_DELAY_BLOCKS have passed
    → creates coin at DESTINATION with AMOUNT
```

---

## Puzzle Parameters

### Curried In (set by the registration coin at creation time)

| Parameter | Type | Size | Description |
|-----------|------|------|-------------|
| `DESTINATION` | `Bytes32` | 32 bytes | Puzzle hash where collateral is sent after delay. Set by the exiting validator in the registration coin solution. |
| `AMOUNT` | `Int` | variable | Collateral amount in mojos. Equals the registration coin's amount. |
| `WITHDRAW_DELAY_BLOCKS` | `Int` | variable | Number of L1 blocks that must pass after coin creation before it can be spent. Set per deployment in `NetworkConfig`. |

### Solution Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| *(none required)* | | All parameters are curried in. The solution is empty. |

No solution parameters are needed because:
- `DESTINATION` is curried (cannot be changed after creation)
- `AMOUNT` is curried (cannot be changed after creation)
- `WITHDRAW_DELAY_BLOCKS` is curried (cannot be changed after creation)
- No passthrough conditions (prevents condition injection per SEC-008)

---

## What the Puzzle Does

1. Asserts that `WITHDRAW_DELAY_BLOCKS` have passed since this coin was
   created using `ASSERT_HEIGHT_RELATIVE`.
2. Creates a coin at `DESTINATION` with `AMOUNT`.
3. Returns no additional conditions.

The spend fails if the block height has not advanced by at least
`WITHDRAW_DELAY_BLOCKS` since coin creation. This is enforced by the Chia
consensus layer, not by custom logic.

---

## Puzzle Source (Rue)

```rue
// withdraw_delay_coin.rue
//
// Holds collateral during a mandatory waiting period after validator exit.
// Created by the registration coin. Can only release funds to the curried
// DESTINATION after WITHDRAW_DELAY_BLOCKS L1 blocks have passed.
//
// Security: No solution parameters, no passthrough conditions.
// All behaviour is locked at creation time via currying.

fun main(
    DESTINATION: Bytes32,
    AMOUNT: Int,
    WITHDRAW_DELAY_BLOCKS: Int,
) -> List<Condition> {
    [
        AssertHeightRelative(WITHDRAW_DELAY_BLOCKS),
        CreateCoin(DESTINATION, AMOUNT),
    ]
}
```

### CLVM Cost Estimate

| Condition | Cost |
|-----------|------|
| `ASSERT_HEIGHT_RELATIVE` | ~500,000 |
| `CREATE_COIN` | ~1,800,000 |
| Puzzle overhead | ~100,000 |
| **Total** | **~2,400,000** |

This is approximately 0.022% of the block limit.

---

## Changes to Registration Coin

### New Curried Parameters

The registration coin puzzle gains two additional curried parameters:

| Parameter | Type | Description |
|-----------|------|-------------|
| `WITHDRAW_DELAY_MOD_HASH` | `Bytes32` | Tree hash of the withdraw delay coin puzzle (uncurried). |
| `WITHDRAW_DELAY_BLOCKS` | `Int` | Block delay value. Passed through to the withdraw delay coin curry. |

### Updated Puzzle Source

```rue
// registration_coin.rue (updated)

fun main(
    VALIDATOR_PUBKEY: PublicKey,
    CHECKPOINT_SINGLETON_ID: Bytes32,
    WITHDRAW_DELAY_MOD_HASH: Bytes32,       // NEW
    WITHDRAW_DELAY_BLOCKS: Int,              // NEW

    epoch: Int,
    collateral_destination: Bytes32,
    collateral_amount: Int,
) -> List<Condition> {

    // Compute the withdraw delay coin puzzle hash
    // This locks in: destination, amount, and delay
    let withdraw_delay_puzzle_hash = curry_hash(
        WITHDRAW_DELAY_MOD_HASH,
        collateral_destination,
        collateral_amount,
        WITHDRAW_DELAY_BLOCKS,
    );

    // Announcement format per spec-wire-format — Membership Announcement Format
    let expected_announcement = sha256(
        "membership" + int_to_8_bytes_be(epoch) + VALIDATOR_PUBKEY + 0
    );

    [
        AssertCoinAnnouncement(
            sha256(CHECKPOINT_SINGLETON_ID + expected_announcement)
        ),
        // Creates the WITHDRAW DELAY COIN, not the final destination coin
        CreateCoin(withdraw_delay_puzzle_hash, collateral_amount),
    ]
}
```

### Impact on Registration Coin Puzzle Hash

Adding two curried parameters changes the registration coin's puzzle hash.
This means:
- `REGISTRATION_COIN_MOD_HASH` in `NetworkConfig` changes
- The network coin's `curry_hash` computation for registration coins changes
- The indexer's lineage verification uses the new mod hash
- All existing tests that compute registration coin puzzle hashes need updating

### Impact on Network Coin

The network coin puzzle itself does not change — it creates registration coins
using `REGISTRATION_COIN_MOD_HASH` which is curried at deployment. The mod
hash just has a different value because the registration coin puzzle changed.

The `curry_hash` computation in the network coin changes because the
registration coin now has 4 curried parameters instead of 2:

```
registration_coin_puzzle_hash = curry_hash(
    REGISTRATION_COIN_MOD_HASH,
    validator_pubkey,
    checkpoint_singleton_id,
    WITHDRAW_DELAY_MOD_HASH,       // NEW
    WITHDRAW_DELAY_BLOCKS,          // NEW
)
```

---

## Driver Code (Rust)

### Types

```rust
/// Everything needed to spend a withdraw delay coin.
pub struct WithdrawDelayCoinSpend {
    pub coin:        Coin,
    pub destination: Bytes32,
    pub amount:      u64,
    pub delay:       u64,
}

/// Solution for the withdraw delay coin (empty — all params are curried).
#[derive(ToClvm, FromClvm)]
#[clvm(list)]
pub struct WithdrawDelayCoinSolution;
```

### Computing the Withdraw Delay Coin Puzzle Hash

```rust
/// Compute the puzzle hash of a withdraw delay coin.
/// Used by the registration coin puzzle on-chain and by the driver off-chain.
pub fn withdraw_delay_puzzle_hash(
    withdraw_delay_mod_hash: Bytes32,
    destination: Bytes32,
    amount: u64,
    delay_blocks: u64,
) -> Bytes32 {
    curry_puzzle_hash(
        withdraw_delay_mod_hash,
        &[
            clvm_encode(&destination),
            clvm_encode(&amount),
            clvm_encode(&delay_blocks),
        ],
    )
}
```

### Spending the Withdraw Delay Coin

```rust
/// Spend a withdraw delay coin to release collateral to the destination.
/// Fails on-chain if WITHDRAW_DELAY_BLOCKS have not passed since creation.
pub fn spend_withdraw_delay_coin(
    ctx: &mut SpendContext,
    spend: &WithdrawDelayCoinSpend,
    withdraw_delay_mod: NodePtr,
) -> anyhow::Result<CoinSpend> {

    let puzzle = withdraw_delay_puzzle(
        ctx,
        spend.destination,
        spend.amount,
        spend.delay,
        withdraw_delay_mod,
    )?;

    // Empty solution — all behaviour is curried in
    let solution = ctx.alloc(&WithdrawDelayCoinSolution)?;

    Ok(CoinSpend::new(
        spend.coin,
        ctx.serialize(&puzzle)?,
        ctx.serialize(&solution)?,
    ))
}
```

### Building the Release Bundle

```rust
/// Build a spend bundle to release collateral from a withdraw delay coin.
/// Returns SpendBundle — the caller broadcasts it.
/// Fails if the delay period has not elapsed.
pub fn build_release_collateral(
    ctx: &mut SpendContext,
    spend: &WithdrawDelayCoinSpend,
    withdraw_delay_mod: NodePtr,
) -> anyhow::Result<SpendBundle> {

    let coin_spend = spend_withdraw_delay_coin(ctx, spend, withdraw_delay_mod)?;

    // No signatures needed — the puzzle is purely time-locked
    Ok(SpendBundle {
        coin_spends: vec![coin_spend],
        aggregated_signature: G2Affine::identity(),
    })
}
```

---

## Changes to ConsensusClient

### New Method: release_collateral()

```rust
impl ConsensusClient {
    /// Build a spend bundle to release collateral from a withdraw delay coin.
    /// Returns SpendBundle — the caller broadcasts it.
    ///
    /// This is the second step of collateral recovery:
    ///   1. recover_collateral() — creates the withdraw delay coin
    ///   2. [wait WITHDRAW_DELAY_BLOCKS]
    ///   3. release_collateral() — releases funds to destination
    ///
    /// The caller must wait until the delay period has elapsed before
    /// broadcasting this bundle. If submitted too early, the Chia node
    /// will reject it (ASSERT_HEIGHT_RELATIVE failure).
    ///
    /// CLVM cost: ~2.4M units
    pub async fn release_collateral(
        &self,
        withdraw_delay_coin: Coin,
        destination: Bytes32,
        amount: u64,
    ) -> ConsensusResult<SpendBundle> {
        // ...
    }
}
```

### Updated Method: recover_collateral()

The existing `recover_collateral()` method now creates a withdraw delay coin
instead of sending directly to the destination. The method signature is
unchanged — the `collateral_destination` parameter is forwarded into the
withdraw delay coin's curried `DESTINATION`.

### New Config Fields and Default

```rust
/// Default: 24,000 blocks ≈ 5 days at ~18s/block
pub const DEFAULT_WITHDRAW_DELAY_BLOCKS: u64 = 24_000;

pub struct NetworkConfig {
    // ... existing fields ...
    pub withdraw_delay_blocks: u64,       // Default: 24,000 (~5 days)
    pub withdraw_delay_mod_hash: Bytes32,
}
```

---

## Security Properties

### Time Lock Guarantees

| Property | Guarantee |
|----------|-----------|
| Minimum delay | `WITHDRAW_DELAY_BLOCKS` is curried in, cannot be shortened |
| Destination locked | `DESTINATION` is curried in, cannot be changed |
| Amount locked | `AMOUNT` is curried in, cannot be redirected |
| No bypass | No solution parameters, no passthrough conditions |
| Permissionless release | Anyone can spend the delay coin after the period — no key required |

### Attack Scenarios Prevented

**Instant exit attack**: Validator misbehaves and immediately exits to recover
collateral before the network can slash. The delay prevents this.

**Front-running slash**: Validator detects a pending slash transaction and
front-runs it by spending the withdraw delay coin. The delay coin cannot be
spent until the delay period has passed regardless of transaction ordering.

**Destination substitution**: An attacker cannot change the destination after
the withdraw delay coin is created because it's curried into the puzzle hash.

### New Attack Vector to Consider

**Withdraw delay coin theft**: Since the delay coin is permissionless to spend
(no signature required), anyone could potentially submit the release
transaction. However, the funds can only go to the curried `DESTINATION`, so
this is harmless — it just means anyone can "help" release the funds.

---

## Configuration

### Default: 5 Days (24,000 Blocks)

The default `WITHDRAW_DELAY_BLOCKS` is **24,000**, corresponding to
approximately 5 days at Chia's ~18-second block time:

```
5 days × 24 hours × 60 minutes × 60 seconds = 432,000 seconds
432,000 seconds ÷ 18 seconds/block = 24,000 blocks
```

### Recommended Values

| Network Type | Delay (blocks) | Approximate Time | Rationale |
|-------------|---------------|------------------|-----------|
| Testnet | 10 | ~3 minutes | Fast iteration during testing |
| Mainnet | **24,000** | **~5 days** | Default. Sufficient slashing window for detection and governance response. |

The delay MUST be set at deployment time and cannot be changed without
redeploying the registration coin puzzle (which means redeploying the network
coin and all associated infrastructure).

---

## Puzzle Inventory Update

| Puzzle | Source | Hex | Hash | Driver |
|--------|--------|-----|------|--------|
| Withdraw Delay Coin | `puzzles/withdraw_delay_coin.rue` | `compiled/withdraw_delay_coin.hex` | `compiled/withdraw_delay_coin.hash` | `src/puzzles/withdraw_delay.rs` |

---

## CLVM Cost Summary Update

| Spend Path | Cost (units) | % of 11B limit |
|-----------|--------------|----------------|
| Withdraw delay coin release | ~2,400,000 | 0.022% |
| **Updated collateral recovery bundle** | **~7,400,000** | **0.067%** |
| **New total (recovery + release)** | **~9,800,000** | **0.089%** |

The total cost of full collateral recovery is now split across two
transactions: the recovery bundle (membership query + registration coin) and
the release transaction (withdraw delay coin). Each is well within block
limits.

---

## Migration Path

This change affects:
1. Registration coin puzzle (2 new curried parameters)
2. Registration coin mod hash (changes)
3. Network coin curry_hash computation (4 params instead of 2)
4. NetworkConfig (2 new fields)
5. Deployment procedure (deploy withdraw delay coin puzzle)
6. Collateral recovery flow (now two steps)
7. Indexer (track withdraw delay coins)

Since the project has not launched, this is a clean change with no migration
needed.

---

## Important Notes

**The delay is per L1 block, not per L2 checkpoint**

The `ASSERT_HEIGHT_RELATIVE` condition counts L1 blocks, not L2 epochs. This
means the delay is independent of checkpoint frequency.

**The delay coin is permissionless to spend**

After the delay period, anyone can spend the delay coin. This is by design —
the funds can only go to the curried `DESTINATION`, so there's no security
risk. It means validators don't need to be online exactly when the delay
expires.

**The delay coin puzzle hash is deterministic**

Given the destination, amount, and delay, the puzzle hash is fully
deterministic. This means the registration coin can compute it on-chain
without any additional information.

**No AGG_SIG conditions**

Neither the withdraw delay coin creation (from registration coin) nor the
delay coin spend requires any signatures. The entire flow is permissionless
after the initial membership query.
