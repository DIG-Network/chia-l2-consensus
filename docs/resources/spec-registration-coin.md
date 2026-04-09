# Registration Coin Puzzle - Technical Specification

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Created by** | [spec-network-coin](spec-network-coin.md) | The only valid creator of registration coins. Coins without this parent are ignored by the L2. |
| **Depends on** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | Must assert a membership announcement from the checkpoint singleton to spend |
| **Depends on** | [spec-wire-format](spec-wire-format.md) | Membership announcement format defined there; epoch encoding rules |
| **Implements** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | Part 2: Registration Coin of the CHIP |
| **Enables** | [spec-validator-onboarding](spec-validator-onboarding.md) | Steps 6 and 7: registration and collateral recovery |
| **Enables** | [spec-consensus-crate](spec-consensus-crate.md) | recover_collateral() and fetch_registration_coin() methods |
| **Referenced by** | [spec-indexer](spec-indexer.md) | Indexer tracks unspent registration coins as the active validator set |
| **Referenced by** | [spec-security](spec-security.md) | Lineage proof enforcement and collateral security properties |
| **Referenced by** | [spec-clvm-costs](spec-clvm-costs.md) | Spend Path 4: Registration Coin cost analysis |
| **Referenced by** | [spec-deployment-runbook](spec-deployment-runbook.md) | REGISTRATION_COIN_MOD_HASH must be known before network coin deployment |

---

## Overview

The registration coin is created by spending the network coin singleton
(→ see [spec-network-coin](spec-network-coin.md)). It holds a validator's
collateral and commits to their BLS12-381 pubkey. A validator cannot recover
their collateral until the checkpoint singleton confirms they are no longer in
the active validator set via a membership announcement
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
Path 2: Membership Query).

The registration coin asserts that announcement in the same spend bundle as
the checkpoint singleton membership query spend. The validator can do this at
any time after a checkpoint that excludes them, there is no timing pressure
because the checkpoint singleton persists with the same state until the next
checkpoint changes it
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Rationale:
Why the checkpoint singleton has a membership query spend path).

The coin ID of a registration coin is deterministic from the validator pubkey,
the checkpoint singleton ID, and the network coin spend that created it. This
makes registration coins easily queryable by the indexer
(→ see [spec-indexer](spec-indexer.md) — Registration Coin Detection).

The CLVM cost of spending a registration coin is covered in
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Spend Path 4). The combined
cost of the full collateral recovery bundle (membership query + registration
coin) is covered in
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Combined Collateral Recovery
Bundle).

---

## Puzzle Parameters

### Curried In (set by the network coin at creation time)

| Parameter | Type | Description |
|-----------|------|-------------|
| `VALIDATOR_PUBKEY` | `PublicKey` | The BLS12-381 G1 pubkey of the validator. 48 bytes compressed per [spec-wire-format](spec-wire-format.md) — G1 Points. |
| `CHECKPOINT_SINGLETON_ID` | `Bytes32` | The coin ID of the checkpoint singleton. Used to verify the membership announcement came from the correct singleton. This is the same value curried into the network coin per [spec-network-coin](spec-network-coin.md) — Curried In Parameters. |

### Solution Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `epoch` | `Int` | The epoch number from the checkpoint singleton at the time of the membership query spend. Must match the epoch in the announcement. Encoding: big-endian u64 per [spec-wire-format](spec-wire-format.md) — Integer Encoding. |
| `collateral_destination` | `Bytes32` | The puzzle hash of the coin to send the collateral to. |
| `collateral_amount` | `Int` | The amount of collateral to return. Should equal the full coin amount. |
| `conditions` | `List<Condition>` | Additional conditions. Can include slashing logic to redirect collateral per [spec-security](spec-security.md) — Known Limitations. |

---

## What the Puzzle Does

1. Computes the expected membership announcement message using the format
   defined in
   (→ see [spec-wire-format](spec-wire-format.md) — Membership Announcement
   Format):
   `sha256("membership" + epoch_be + VALIDATOR_PUBKEY + 0x00)` where `0x00`
   means not a member.
2. Wraps it in an `AssertCoinAnnouncement` attributable to the checkpoint
   singleton: `sha256(CHECKPOINT_SINGLETON_ID + announcement)`. The
   `CHECKPOINT_SINGLETON_ID` here is the coin ID of the current checkpoint
   singleton coin, not the launcher ID, per
   (→ see [spec-wire-format](spec-wire-format.md) — Common Mistakes: Coin ID
   vs launcher ID).
3. Creates a coin at `collateral_destination` with `collateral_amount`.
4. Returns any additional conditions from the solution.

The spend fails if the membership announcement is not present in the spend
bundle. The only way to get that announcement is to spend the checkpoint
singleton via its membership query spend path with a valid Merkle
non-membership proof for this validator's pubkey
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
Path 2: Membership Query and [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)
— Non-Membership Proof).

---

## Puzzle Source (Rue)

```rust
// registration_coin.rue

fn main(
    VALIDATOR_PUBKEY: PublicKey,
    CHECKPOINT_SINGLETON_ID: Bytes32,

    epoch: Int,
    collateral_destination: Bytes32,
    collateral_amount: Int,
    conditions: List<Condition>,
) -> List<Condition> {

    // Announcement format per spec-wire-format — Membership Announcement Format
    // epoch encoded as 8-byte big-endian u64
    // is_member = 0 (0x00) means not a member
    let expected_announcement = sha256(
        "membership" + int_to_8_bytes_be(epoch) + VALIDATOR_PUBKEY + 0
    );

    // CHECKPOINT_SINGLETON_ID is the current coin ID, not the launcher ID
    // per spec-wire-format — Common Mistakes: Coin ID vs launcher ID
    conditions + [
        AssertCoinAnnouncement(
            sha256(CHECKPOINT_SINGLETON_ID + expected_announcement)
        ),
        CreateCoin(collateral_destination, collateral_amount),
    ]
}
```

---

## Driver Code (Rust)

### Types

```rust
use chia_wallet_sdk::prelude::*;
use chia_protocol::{Bytes32, Coin, CoinSpend, SpendBundle};

/// Everything needed to spend a registration coin.
pub struct RegistrationCoinSpend {
    pub coin:                    Coin,
    pub validator_pubkey:        PublicKey,
    pub checkpoint_singleton_id: Bytes32,
}

/// The solution to provide when spending a registration coin.
#[derive(ToClvm, FromClvm)]
#[clvm(list)]
pub struct RegistrationCoinSolution {
    pub epoch:                  u64,
    pub collateral_destination: Bytes32,
    pub collateral_amount:      u64,
    pub conditions:             Vec<Condition>,
}
```

### Computing the Registration Coin Puzzle Hash

This must produce the exact same hash as the network coin puzzle's `curry_hash`
call on-chain
(→ see [spec-network-coin](spec-network-coin.md) — What the Puzzle Does). It
is also used by the indexer for lineage verification
(→ see [spec-indexer](spec-indexer.md) — Registration Coin Detection):

```rust
pub fn registration_coin_puzzle_hash(
    validator_pubkey: PublicKey,
    checkpoint_singleton_id: Bytes32,
    registration_coin_mod_hash: Bytes32,
) -> Bytes32 {
    curry_puzzle_hash(
        registration_coin_mod_hash,
        &[
            clvm_encode(&validator_pubkey),
            clvm_encode(&checkpoint_singleton_id),
        ],
    )
}

/// Compute the coin ID of a registration coin deterministically.
/// Used by the indexer to find registration coins without querying by puzzle hash.
pub fn registration_coin_id(
    network_coin_spend_id: Bytes32,
    validator_pubkey: PublicKey,
    checkpoint_singleton_id: Bytes32,
    registration_coin_mod_hash: Bytes32,
    collateral_amount: u64,
) -> Bytes32 {
    let puzzle_hash = registration_coin_puzzle_hash(
        validator_pubkey,
        checkpoint_singleton_id,
        registration_coin_mod_hash,
    );
    Coin::new(network_coin_spend_id, puzzle_hash, collateral_amount).coin_id()
}
```

### Spending the Registration Coin (Collateral Recovery)

This must be submitted in the same spend bundle as a checkpoint singleton
membership query spend
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
Path 2: Membership Query). The full bundle is assembled by
`ConsensusClient.recover_collateral()`
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Collateral Recovery):

```rust
pub fn spend_registration_coin(
    ctx: &mut SpendContext,
    registration: &RegistrationCoinSpend,
    registration_coin_mod: NodePtr,
    epoch: u64,
    collateral_destination: Bytes32,
    extra_conditions: Vec<Condition>,
) -> anyhow::Result<CoinSpend> {

    let puzzle = registration_coin_puzzle(
        ctx,
        registration.validator_pubkey,
        registration.checkpoint_singleton_id,
        registration_coin_mod,
    )?;

    let solution = ctx.alloc(&RegistrationCoinSolution {
        epoch,
        collateral_destination,
        collateral_amount: registration.coin.amount,
        conditions: extra_conditions,
    })?;

    Ok(CoinSpend::new(
        registration.coin,
        ctx.serialize(&puzzle)?,
        ctx.serialize(&solution)?,
    ))
}
```

### Full Collateral Recovery Spend Bundle

Assembles the membership query spend and registration coin spend into one
bundle. Called by `ConsensusClient.recover_collateral()`
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Collateral Recovery).
The validator perspective is in
(→ see [spec-validator-onboarding](spec-validator-onboarding.md) — Voluntary
Exit):

```rust
pub async fn recover_collateral(
    ctx: &mut SpendContext,
    node: &FullNodeClient,
    registration: &RegistrationCoinSpend,
    checkpoint_state: &CheckpointSingletonState,
    checkpoint_inner_mod: NodePtr,
    registration_coin_mod: NodePtr,
    merkle_tree: &SparseMerkleTree,
    collateral_destination: Bytes32,
) -> anyhow::Result<SpendBundle> {

    // Get the non-membership proof using the tree from spec-sparse-merkle-tree
    let slot = validator_slot(registration.validator_pubkey);
    let proof = merkle_tree.prove_non_membership(slot)?;

    // Build the checkpoint singleton membership query spend
    // (→ see spec-checkpoint-singleton — Spend Path 2: Membership Query)
    let membership_query_spend = spend_checkpoint_singleton_membership_query(
        ctx,
        checkpoint_state,
        checkpoint_inner_mod,
        registration.validator_pubkey,
        slot,
        proof,
        false, // is_member = false
    )?;

    // Build the registration coin spend
    let registration_spend = spend_registration_coin(
        ctx,
        registration,
        registration_coin_mod,
        checkpoint_state.epoch,
        collateral_destination,
        vec![],
    )?;

    // No signatures needed - neither spend requires one.
    // The registration coin relies on the announcement assertion.
    // The membership query spend is permissionless.
    // (→ see spec-checkpoint-singleton — Important Notes: Membership query is permissionless)
    Ok(SpendBundle {
        coin_spends: vec![membership_query_spend, registration_spend],
        aggregated_signature: G2Affine::identity(),
    })
}
```

### Fetching a Validator's Registration Coin

Called by `ConsensusClient.recover_collateral()`
(→ see [spec-consensus-crate](spec-consensus-crate.md)):

```rust
pub async fn fetch_registration_coin(
    node: &FullNodeClient,
    validator_pubkey: PublicKey,
    checkpoint_singleton_id: Bytes32,
    registration_coin_mod_hash: Bytes32,
    collateral_amount: u64,
    network_coin_launcher_id: Bytes32,
) -> anyhow::Result<Option<RegistrationCoinSpend>> {

    let puzzle_hash = registration_coin_puzzle_hash(
        validator_pubkey,
        checkpoint_singleton_id,
        registration_coin_mod_hash,
    );

    let records = node
        .get_coin_records_by_puzzle_hash(puzzle_hash, false)
        .await?;

    for record in records {
        // Verify lineage: parent must be a network coin spend
        // (→ see spec-security — Lineage proof enforcement)
        if is_valid_registration_coin_parent(
            node,
            record.coin.parent_coin_info,
            network_coin_launcher_id,
        ).await? {
            return Ok(Some(RegistrationCoinSpend {
                coin: record.coin,
                validator_pubkey,
                checkpoint_singleton_id,
            }));
        }
    }

    Ok(None)
}

async fn is_valid_registration_coin_parent(
    node: &FullNodeClient,
    parent_coin_id: Bytes32,
    network_coin_launcher_id: Bytes32,
) -> anyhow::Result<bool> {
    let parent_record = node.get_coin_record_by_name(parent_coin_id).await?;
    let Some(parent) = parent_record else { return Ok(false); };
    let network_coin_puzzle_hash = singleton_puzzle_hash(network_coin_launcher_id);
    Ok(parent.coin.puzzle_hash == network_coin_puzzle_hash)
}
```

---

## Important Notes

**Memo convention**

Always store the validator pubkey as the first memo on the `CreateCoin`
condition when the network coin driver creates the registration coin
(→ see [spec-network-coin](spec-network-coin.md) — Important Notes: Memo
convention). Without this memo, the indexer cannot efficiently determine which
pubkey a registration coin belongs to
(→ see [spec-indexer](spec-indexer.md) — Important Notes: Memo is required
for indexing).

**Slashing**

The `conditions` field in the solution allows custom slashing logic. For a
voluntary exit the `collateral_destination` is the validator's own wallet
puzzle hash. For a forced slash the L2 governance can require the
`collateral_destination` to be a designated slash address. The slashing model
is discussed in
(→ see [spec-security](spec-security.md) — Known Limitations: No slashing
enforcement on-chain).

**Spent registration coins**

A spent registration coin means the validator has exited and recovered their
collateral. The indexer only tracks unspent registration coins as the active
validator set
(→ see [spec-indexer](spec-indexer.md) — Process Removals).

**Epoch matching**

The `epoch` in the solution must exactly match the epoch in the membership
announcement from the checkpoint singleton. If the checkpoint singleton
advances to the next epoch before the registration coin is spent, the validator
needs a fresh membership announcement from the new epoch. This prevents replay
attacks across epoch boundaries
(→ see [spec-security](spec-security.md) — Epoch Replay Protection).

**Coin ID vs launcher ID in announcement**

The `CHECKPOINT_SINGLETON_ID` curried into this puzzle is the coin ID of the
checkpoint singleton at the time of registration. This value never changes
because it is curried in. However the checkpoint singleton's coin ID changes
on every checkpoint spend. The membership query spend path emits the
announcement attributable to the current coin ID, not the original curried
value. Confirm you understand this distinction before implementing the
announcement assertion
(→ see [spec-wire-format](spec-wire-format.md) — Common Mistakes: Coin ID vs
launcher ID).
