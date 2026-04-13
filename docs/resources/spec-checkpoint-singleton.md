# Checkpoint Singleton Puzzle - Technical Specification

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Depends on** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | verify_merkle_path gadget used in membership query spend path. TREE_DEPTH must match. |
| **Depends on** | [spec-wire-format](spec-wire-format.md) | All message formats: checkpoint message, membership announcement, checkpoint state announcement, VK input computation, G1/G2 encoding |
| **Depends on** | [spec-groth16-circuit](spec-groth16-circuit.md) | Verifies the proof produced by the circuit via bls_pairing_identity |
| **Depends on** | [spec-trusted-setup](spec-trusted-setup.md) | VK curried in at deployment comes from the trusted setup ceremony |
| **Enables** | [spec-registration-coin](spec-registration-coin.md) | Registration coin asserts membership announcements emitted by this singleton |
| **Enables** | [spec-indexer](spec-indexer.md) | Indexer parses checkpoint state announcements to track epoch, state_root, validator_merkle_root, validator_count |
| **Enables** | [spec-l2-integration](spec-l2-integration.md) | Primary interaction point for the L2 system |
| **Enables** | [spec-consensus-crate](spec-consensus-crate.md) | spend_checkpoint_singleton(), spend_checkpoint_singleton_membership_query(), fetch_checkpoint_singleton_state() |
| **Implements** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | Part 3: Checkpoint Singleton of the CHIP |
| **Referenced by** | [spec-clvm-costs](spec-clvm-costs.md) | Both spend paths have detailed cost breakdowns |
| **Referenced by** | [spec-security](spec-security.md) | Multiple security properties depend on the correctness of both spend paths |
| **Referenced by** | [spec-deployment-runbook](spec-deployment-runbook.md) | Deployed in Step 2. VK verified in Step 7. |
| **Referenced by** | [spec-validator-onboarding](spec-validator-onboarding.md) | Membership query spend used for collateral recovery in Step 7 |

---

## Overview

The checkpoint singleton is the canonical on-chain source of truth for an L2
network. It tracks the current L2 state root, an auto-incrementing epoch
counter, the sparse Merkle root of the active validator set
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)), and the
validator count. It has two spend paths:

**Spend Path 1: Checkpoint** - requires a Groth16 ZK proof of majority
consensus
(→ see [spec-groth16-circuit](spec-groth16-circuit.md)) and updates all state.
Can be triggered at any time as long as majority consensus is achieved. The L1
does not impose timing rules. This is discussed in the CHIP
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — A checkpoint
can be added anytime).

**Spend Path 2: Membership Query** - permissionless read-only path. Verifies a
Merkle proof
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Membership
Proof and Non-Membership Proof) against the current `validator_merkle_root`,
recreates the singleton unchanged, and emits a membership announcement
(→ see [spec-wire-format](spec-wire-format.md) — Membership Announcement
Format). Validators use this to prove non-membership and recover collateral
from their registration coin
(→ see [spec-registration-coin](spec-registration-coin.md)).

The CLVM cost of each spend path is covered in
(→ see [spec-clvm-costs](spec-clvm-costs.md)).

---

## Puzzle Parameters

### Curried In (fixed at deployment)

| Parameter | Type | Description |
|-----------|------|-------------|
| `VK` | `VerificationKey` | Groth16 VK from trusted setup (→ see [spec-trusted-setup](spec-trusted-setup.md)). Serialized per [spec-wire-format](spec-wire-format.md) — VK Format. Contains alpha_g1, beta_g2, gamma_g2, delta_g2, and 7 IC points. |
| `TREE_DEPTH` | `Int` | Sparse Merkle tree depth. Must match the depth in [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) and [spec-groth16-circuit](spec-groth16-circuit.md). Fixed at circuit compile time. |
| `EMPTY_LEAF_HASH` | `Bytes32` | `sha256(0x00 * 48)`. Defined in [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Leaf Values. |

### State (curried in on each recreation)

| Parameter | Type | Description |
|-----------|------|-------------|
| `state_root` | `Bytes32` | Current L2 state rollup hash |
| `epoch` | `Int` | Auto-increments by 1 on every checkpoint spend |
| `validator_merkle_root` | `Bytes32` | Sparse Merkle root per [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) |
| `validator_count` | `Int` | Number of active validators |

### Solution Parameters - Checkpoint Spend Path

| Parameter | Type | Description |
|-----------|------|-------------|
| `is_checkpoint` | `Bool` | Set to `true` |
| `proof` | `Proof` | (A: G1 48b, B: G2 96b, C: G1 48b) per [spec-wire-format](spec-wire-format.md) — Groth16 Proof Format |
| `new_state_root` | `Bytes32` | New L2 state root |
| `new_validator_merkle_root` | `Bytes32` | New validator set root per [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) |
| `new_validator_count` | `Int` | New validator count |
| `agg_signers` | `PublicKey` | G1 aggregate of k signing pubkeys per [spec-wire-format](spec-wire-format.md) — Aggregate Public Key |
| `agg_sig` | `Signature` | Aggregate BLS signature per [spec-wire-format](spec-wire-format.md) — Aggregate Signature |

### Solution Parameters - Membership Query Spend Path

| Parameter | Type | Description |
|-----------|------|-------------|
| `is_checkpoint` | `Bool` | Set to `false` |
| `query_pubkey` | `PublicKey` | Validator pubkey to check |
| `leaf_index` | `Int` | Slot index in the sparse Merkle tree per [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Slot Assignment |
| `siblings` | `List<Bytes32>` | Sibling hashes per [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Membership Proof |
| `is_member` | `Bool` | Whether proving membership or non-membership |

---

## What the Puzzle Does

### Checkpoint Spend Path

1. Computes `new_epoch = epoch + 1`.
2. Computes the checkpoint message using the format from
   (→ see [spec-wire-format](spec-wire-format.md) — Checkpoint Message):
   `sha256(new_state_root + new_validator_merkle_root + new_validator_count_be + new_epoch_be)`.
3. Computes `vk_input` using the linear combination defined in
   (→ see [spec-wire-format](spec-wire-format.md) — VK Input Computation).
4. Calls `bls_pairing_identity` with 4 G1/G2 pairs to verify the Groth16 proof
   from
   (→ see [spec-groth16-circuit](spec-groth16-circuit.md)).
5. Calls `bls_verify` to verify `agg_sig` over the checkpoint message. This
   is the critical check that ties the ZK proof to the BLS signature
   (→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Circuit
   design choice and [spec-security](spec-security.md) — Assumption 3).
6. Recreates the singleton with updated state and emits a checkpoint state
   announcement per
   (→ see [spec-wire-format](spec-wire-format.md) — Checkpoint State
   Announcement Format).

### Membership Query Spend Path

1. Computes the expected leaf value: `sha256(query_pubkey)` for membership or
   `EMPTY_LEAF_HASH` for non-membership, per
   (→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Leaf Values).
2. Verifies the Merkle path using the algorithm from
   (→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — On-Chain
   Verification).
3. Emits membership announcement per
   (→ see [spec-wire-format](spec-wire-format.md) — Membership Announcement
   Format).
4. Recreates the singleton **unchanged**. State does not change on this path.

---

## Puzzle Source (Rue)

```rust
// checkpoint_inner.rue
// Curried in: VK (per spec-wire-format VK Format), TREE_DEPTH, EMPTY_LEAF_HASH

struct VerificationKey {
    alpha_g1: PublicKey,
    beta_g2:  Signature,
    gamma_g2: Signature,
    delta_g2: Signature,
    ic:       List<PublicKey>, // 7 entries per spec-wire-format IC Point Order
}

struct Proof {
    a: PublicKey,   // G1, 48 bytes per spec-wire-format
    b: Signature,   // G2, 96 bytes per spec-wire-format
    c: PublicKey,   // G1, 48 bytes per spec-wire-format
}

// Mirrors verify_path() in spec-sparse-merkle-tree exactly
fn verify_merkle_path(
    node: Bytes32,
    index: Int,
    siblings: List<Bytes32>,
    root: Bytes32,
    depth: Int,
    level: Int,
) -> Bool {
    if level == depth {
        node == root
    } else {
        let sibling = siblings[level];
        // Left child first per spec-sparse-merkle-tree — critical invariant
        let parent = if index % 2 == 0 {
            sha256(node + sibling)
        } else {
            sha256(sibling + node)
        };
        verify_merkle_path(parent, index / 2, siblings, root, depth, level + 1)
    }
}

fn main(
    VK: VerificationKey,
    TREE_DEPTH: Int,
    EMPTY_LEAF_HASH: Bytes32,

    state_root:            Bytes32,
    epoch:                 Int,
    validator_merkle_root: Bytes32,
    validator_count:       Int,

    is_checkpoint: Bool,

    // Checkpoint spend fields (unused in membership query):
    proof:                     Proof,
    new_state_root:            Bytes32,
    new_validator_merkle_root: Bytes32,
    new_validator_count:       Int,
    agg_signers:               PublicKey,
    agg_sig:                   Signature,

    // Membership query fields (unused in checkpoint):
    query_pubkey: PublicKey,
    leaf_index:   Int,
    siblings:     List<Bytes32>,
    is_member:    Bool,

    conditions: List<Condition>,
) -> List<Condition> {

    if is_checkpoint {

        let new_epoch = epoch + 1;

        // Checkpoint message format per spec-wire-format — Checkpoint Message
        let checkpoint_message = sha256(
            new_state_root
            + new_validator_merkle_root
            + int_to_8_bytes_be(new_validator_count)
            + int_to_8_bytes_be(new_epoch)
        );

        // VK input computation per spec-wire-format — VK Input Computation
        // scalar() encoding per spec-wire-format — scalar() Function
        let vk_input = VK.ic[0]
            + VK.ic[1] * scalar(validator_merkle_root)
            + VK.ic[2] * scalar(int_to_8_bytes_be(validator_count))
            + VK.ic[3] * scalar(new_validator_merkle_root)
            + VK.ic[4] * scalar(int_to_8_bytes_be(new_validator_count))
            + VK.ic[5] * scalar(agg_signers)
            + VK.ic[6] * scalar(checkpoint_message);

        // Groth16 verification per spec-groth16-circuit — On-Chain Verification
        // Cost: ~7.8M CLVM units per spec-clvm-costs — Groth16 Verification
        bls_pairing_identity(
            proof.a,       VK.beta_g2,
            -VK.alpha_g1,  proof.b,
            -vk_input,     VK.gamma_g2,
            -proof.c,      VK.delta_g2,
        );

        // BLS aggregate signature verification
        // Cost: ~4.2M CLVM units per spec-clvm-costs
        bls_verify(agg_sig, agg_signers, checkpoint_message);

        conditions + [
            CreateCoin(
                curry_hash(
                    MY_PUZZLE_HASH,
                    new_state_root,
                    new_epoch,
                    new_validator_merkle_root,
                    new_validator_count,
                ),
                MY_AMOUNT,
            ),
            // Checkpoint state announcement per spec-wire-format
            // Parsed by the indexer per spec-indexer — Checkpoint State Updates
            CreateCoinAnnouncement(
                sha256(
                    "checkpoint"
                    + int_to_8_bytes_be(new_epoch)
                    + new_state_root
                    + new_validator_merkle_root
                    + int_to_8_bytes_be(new_validator_count)
                )
            ),
        ]

    } else {

        // Membership query spend path
        // Merkle path verification per spec-sparse-merkle-tree — On-Chain Verification
        // Cost: ~4.1M CLVM units per spec-clvm-costs — Spend Path 3

        let leaf = if is_member {
            sha256(query_pubkey)   // active_leaf per spec-sparse-merkle-tree
        } else {
            EMPTY_LEAF_HASH        // empty_leaf per spec-sparse-merkle-tree
        };

        let valid = verify_merkle_path(
            leaf, leaf_index, siblings,
            validator_merkle_root, TREE_DEPTH, 0,
        );

        assert valid;

        // Membership announcement per spec-wire-format — Membership Announcement Format
        // Asserted by the registration coin per spec-registration-coin — What the Puzzle Does
        let announcement = sha256(
            "membership"
            + int_to_8_bytes_be(epoch)
            + query_pubkey
            + if is_member { 1 } else { 0 }
        );

        conditions + [
            // Recreate singleton UNCHANGED - state does not change
            CreateCoin(
                curry_hash(
                    MY_PUZZLE_HASH,
                    state_root,
                    epoch,
                    validator_merkle_root,
                    validator_count,
                ),
                MY_AMOUNT,
            ),
            CreateCoinAnnouncement(announcement),
        ]
    }
}
```

---

## Driver Code (Rust)

### Types

```rust
pub struct CheckpointSingletonState {
    pub coin:                  Coin,
    pub lineage_proof:         LineageProof,
    pub state_root:            Bytes32,
    pub epoch:                 u64,
    pub validator_merkle_root: Bytes32,
    pub validator_count:       u64,
}

pub struct CheckpointSingletonConfig {
    pub launcher_id:     Bytes32,
    pub vk:              VerifyingKey<ark_bls12_381::Bls12_381>,
    pub tree_depth:      u32,
    pub empty_leaf_hash: Bytes32,
}
```

### Checkpoint Spend

Called by `ConsensusClient.build_checkpoint()`
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Checkpoint
Submission). The proof comes from the Groth16 circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Proof Generation)
serialized per
(→ see [spec-wire-format](spec-wire-format.md) — Groth16 Proof Format):

```rust
pub fn spend_checkpoint_singleton(
    ctx: &mut SpendContext,
    checkpoint_state: &CheckpointSingletonState,
    config: &CheckpointSingletonConfig,
    proof: ClvmProof,              // per spec-wire-format
    new_state_root: Bytes32,
    new_validator_merkle_root: Bytes32,
    new_validator_count: u64,
    agg_signers: PublicKey,        // per spec-wire-format — Aggregate Public Key
    agg_sig: Signature,            // per spec-wire-format — Aggregate Signature
) -> anyhow::Result<CoinSpend> {

    let inner_puzzle = build_checkpoint_inner_puzzle(ctx, config, checkpoint_state)?;

    let solution = ctx.alloc(&CheckpointSolution {
        is_checkpoint: true,
        proof,
        new_state_root,
        new_validator_merkle_root,
        new_validator_count,
        agg_signers,
        agg_sig,
        // membership query fields unused
        query_pubkey:   PublicKey::default(),
        leaf_index:     0,
        siblings:       vec![],
        is_member:      false,
        conditions:     vec![],
    })?;

    let full_puzzle = SingletonLayer::new(config.launcher_id, ctx.serialize(&inner_puzzle)?)
        .build_puzzle(ctx)?;
    let full_solution = SingletonLayer::solution(
        ctx, checkpoint_state.lineage_proof, ctx.serialize(&solution)?
    )?;

    Ok(CoinSpend::new(
        checkpoint_state.coin,
        ctx.serialize(&full_puzzle)?,
        ctx.serialize(&full_solution)?,
    ))
}
```

### Membership Query Spend

Called by `ConsensusClient.query_membership_on_chain()` and as part of
`ConsensusClient.recover_collateral()`
(→ see [spec-consensus-crate](spec-consensus-crate.md)). Permissionless: no
signature required. The Merkle proof comes from
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Proof
Generation):

```rust
pub fn spend_checkpoint_singleton_membership_query(
    ctx: &mut SpendContext,
    checkpoint_state: &CheckpointSingletonState,
    config: &CheckpointSingletonConfig,
    query_pubkey: PublicKey,
    leaf_index: u64,
    siblings: Vec<Bytes32>,   // length must equal TREE_DEPTH
    is_member: bool,
) -> anyhow::Result<CoinSpend> {

    let inner_puzzle = build_checkpoint_inner_puzzle(ctx, config, checkpoint_state)?;

    let solution = ctx.alloc(&MembershipQuerySolution {
        is_checkpoint: false,
        // checkpoint fields zeroed
        proof:                     ClvmProof::default(),
        new_state_root:            Bytes32::default(),
        new_validator_merkle_root: Bytes32::default(),
        new_validator_count:       0,
        agg_signers:               PublicKey::default(),
        agg_sig:                   Signature::default(),
        // membership query fields
        query_pubkey,
        leaf_index,
        siblings,
        is_member,
        conditions: vec![],
    })?;

    let full_puzzle = SingletonLayer::new(config.launcher_id, ctx.serialize(&inner_puzzle)?)
        .build_puzzle(ctx)?;
    let full_solution = SingletonLayer::solution(
        ctx, checkpoint_state.lineage_proof, ctx.serialize(&solution)?
    )?;

    Ok(CoinSpend::new(
        checkpoint_state.coin,
        ctx.serialize(&full_puzzle)?,
        ctx.serialize(&full_solution)?,
    ))
}
```

### Computing the Checkpoint Message

Must match the Rue puzzle exactly. Used by the consensus crate's
`checkpoint_message()` and `validator_signing_message()` methods
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Checkpoint
Submission). Format defined in
(→ see [spec-wire-format](spec-wire-format.md) — Checkpoint Message):

```rust
pub fn compute_checkpoint_message(
    new_state_root: Bytes32,
    new_validator_merkle_root: Bytes32,
    new_validator_count: u64,
    new_epoch: u64,
) -> Bytes32 {
    let mut input = Vec::with_capacity(80);
    input.extend_from_slice(&new_state_root);
    input.extend_from_slice(&new_validator_merkle_root);
    input.extend_from_slice(&new_validator_count.to_be_bytes());
    input.extend_from_slice(&new_epoch.to_be_bytes());
    sha256(&input)
}
```

### Serializing the Verification Key

The VK from Arkworks must be converted to the format defined in
(→ see [spec-wire-format](spec-wire-format.md) — Verification Key Format).
This serialized form is what gets curried into the puzzle at deployment:

```rust
pub fn serialize_vk_for_clvm(
    vk: &VerifyingKey<ark_bls12_381::Bls12_381>,
) -> anyhow::Result<ClvmVerificationKey> {

    let mut alpha_g1 = Vec::new();
    let mut beta_g2  = Vec::new();
    let mut gamma_g2 = Vec::new();
    let mut delta_g2 = Vec::new();

    vk.alpha_g1.serialize_compressed(&mut alpha_g1)?;
    vk.beta_g2.serialize_compressed(&mut beta_g2)?;
    vk.gamma_g2.serialize_compressed(&mut gamma_g2)?;
    vk.delta_g2.serialize_compressed(&mut delta_g2)?;

    let ic = vk.gamma_abc_g1.iter().map(|pt| {
        let mut buf = Vec::new();
        pt.serialize_compressed(&mut buf)?;
        Ok(buf)
    }).collect::<anyhow::Result<Vec<_>>>()?;

    assert_eq!(ic.len(), 7); // per spec-wire-format — IC Point Order
    Ok(ClvmVerificationKey { alpha_g1, beta_g2, gamma_g2, delta_g2, ic })
}
```

---

## Important Notes

**No signatures on checkpoint spend**

The checkpoint spend bundle sets `aggregated_signature` to the identity element.
The aggregate BLS signature from the validators is passed as a solution argument
and verified inside the puzzle by `bls_verify`. There are no `AGG_SIG_ME`
conditions. This is discussed in the L2 integration guide
(→ see [spec-l2-integration](spec-l2-integration.md) — Important Notes).

**Membership query is permissionless**

Anyone can spend the checkpoint singleton via the membership query path. No
signature required. This is intentional so validators can recover collateral
without cooperation from the rest of the network
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Rationale:
Why the checkpoint singleton has a membership query spend path).

**State tracking**

The state is curried into the puzzle on every recreation so the puzzle hash
changes on every checkpoint spend. You cannot look up the current state by
puzzle hash. The indexer maintains state history
(→ see [spec-indexer](spec-indexer.md) — Checkpoint State Updates).

**TREE_DEPTH must match the circuit**

The `TREE_DEPTH` curried in here must match the depth in the Groth16 circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Circuit Parameters)
and in the sparse Merkle tree implementation
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Parameters).
Verified by the deployment runbook
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 3).
