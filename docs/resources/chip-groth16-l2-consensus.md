# CHIP: Groth16 Proof Verification for L2 Consensus

| CHIP Number | *(leave blank - editor will assign)* |
|---|---|
| Title | Groth16 Proof Verification for L2 Consensus |
| Description | A standard for proving L2 majority validator consensus on the Chia L1 at constant cost using a network coin, registration coins, a checkpoint singleton, and Groth16 ZK proofs |
| Author | |
| Editor | *(leave blank - editor will assign)* |
| Comments-URI | *(leave blank - editor will assign)* |
| Status | *(leave blank - editor will assign)* |
| Category | Standards Track |
| Sub-Category | Primitive |
| Created | 2026-04-08 |
| Requires | CHIP-0011 |

---

## Document Relationships

This CHIP is the root standards document. All implementation specs derive from
it. The table below maps each section of this CHIP to the spec that implements
or elaborates it.

| This CHIP Section | Implemented / Elaborated By |
|-------------------|----------------------------|
| Part 1: Network Coin | [spec-network-coin](spec-network-coin.md) |
| Part 2: Registration Coin | [spec-registration-coin](spec-registration-coin.md) |
| Part 3: Checkpoint Singleton | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) |
| Part 4: Off-Chain Validator Set Construction | [spec-indexer](spec-indexer.md), [spec-consensus-crate](spec-consensus-crate.md) |
| Sparse Merkle Tree | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) |
| Circuit Public Inputs / Circuit Statement | [spec-groth16-circuit](spec-groth16-circuit.md) |
| Groth16 On-Chain Verification | [spec-wire-format](spec-wire-format.md), [spec-checkpoint-singleton](spec-checkpoint-singleton.md) |
| Trusted Setup | [spec-trusted-setup](spec-trusted-setup.md) |
| De-Registration and Collateral Recovery | [spec-registration-coin](spec-registration-coin.md), [spec-validator-onboarding](spec-validator-onboarding.md) |
| Security | [spec-security](spec-security.md) |
| CLVM costs | [spec-clvm-costs](spec-clvm-costs.md) |
| Integration | [spec-l2-integration](spec-l2-integration.md), [spec-consensus-crate](spec-consensus-crate.md) |
| Deployment | [spec-deployment-runbook](spec-deployment-runbook.md) |

---

## Abstract

This CHIP defines a standard for how Chia L2 systems can prove majority
validator consensus to the Chia L1 at constant cost. It covers four
components: a network coin that is the canonical registration authority for
the entire network (full spec: [spec-network-coin](spec-network-coin.md)), registration coins that
validators create by spending the network coin and which hold their collateral
(full spec: [spec-registration-coin](spec-registration-coin.md)), a checkpoint singleton that
tracks L2 state and the active validator set (full spec:
[spec-checkpoint-singleton](spec-checkpoint-singleton.md)), and a Groth16
zero-knowledge proof that a checkpoint submitter generates off-chain to prove
a signing majority (circuit spec: [spec-groth16-circuit](spec-groth16-circuit.md), setup:
[spec-trusted-setup](spec-trusted-setup.md)).

The lineage proof between a registration coin and its parent network coin spend
is what makes a registration coin legitimate, any coin that cannot prove this
lineage gets ignored by the L2 when constructing the validator set. The
checkpoint singleton has two spend paths, a checkpoint spend that verifies the
ZK proof and updates state, and a membership query spend that any validator can
use at any time to prove they are no longer in the active set so they can
recover their collateral. The proof is verified on-chain using existing CLVM
operators from CHIP-0011, specifically `bls_pairing_identity`, `g1_multiply`,
and `g1_add`. No new opcodes are required.

---

## Motivation

**The problem**

To build an L2 system on Chia you need a set of validators that collectively
sign off on state checkpoints. Those checkpoints get posted to the Chia L1 to
settle state and the L1 puzzle needs to verify that a majority of the
registered validator set actually signed them.

The natural approach is BLS signature aggregation, you combine all signing
keys into one aggregate public key and verify a single aggregate signature.
CLVM already supports this with `g1_add` and `bls_verify`. The problem is
that an aggregate G1 point carries no information about how many individual
keys were combined into it, you can produce a valid aggregate from 1 key or
100 keys and the resulting point looks identical. This means a single bad
actor with one valid validator key could submit a fraudulent checkpoint
claiming majority consensus and the L1 puzzle would have no way to tell the
difference. This is the foundational problem the Groth16 circuit
([spec-groth16-circuit](spec-groth16-circuit.md)) solves.

The only way to solve this without iterating over individual signatures, which
CLVM puzzles cant do at reasonable cost (see [spec-clvm-costs](spec-clvm-costs.md) for why), is
to use a zero-knowledge proof. The checkpoint submitter proves off-chain that
they have k valid validator keys from the registered set, that k is a majority
of the total registered validators, and that those keys produced a valid
aggregate signature over the checkpoint. On-chain, CLVM just verifies the
proof at constant cost regardless of how large the validator set is.

**Why the network coin matters**

Without a canonical registration authority anyone could create a coin with the
registration coin puzzle hash and claim to be a registered validator. The
network coin ([spec-network-coin](spec-network-coin.md)) solves this, its a singleton that acts
as the gatekeeper for the entire network. A registration coin is only valid if
its lineage proof traces back to a spend of the network coin. The L2 enforces
this lineage check when constructing the validator set off-chain
([spec-indexer](spec-indexer.md) — Registration Coin Detection and Lineage
Verification) and this ensures that only validators who went through the
approved registration process can participate in consensus. The security
guarantees of this check are analyzed in
[spec-security](spec-security.md) — Assumption 3.

**Why the validator set lives off-chain**

The checkpoint singleton ([spec-checkpoint-singleton](spec-checkpoint-singleton.md)) tracks the
current `validator_merkle_root` and `validator_count` as on-chain state but
the full validator set is not stored on L1. Instead the L2 queries all
registration coins and verifies their lineage back to the network coin before
each checkpoint. The submitter constructs the sparse Merkle tree
([spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)) off-chain and includes the resulting
root in the checkpoint message ([spec-wire-format](spec-wire-format.md) — Checkpoint Message).
A majority of validators must sign that message, which means a fraudulent
Merkle root would never gather a majority signature from the real validator
set. The majority consensus over the checkpoint message is itself the proof
that the Merkle root is correct. This property is a core security assumption
([spec-security](spec-security.md) — Assumption 3).

**Why the registration coin holds collateral**

Collateral gives validators skin in the game and provides a slashing
mechanism. A validator cant recover their collateral
([spec-registration-coin](spec-registration-coin.md) — Spending the Registration Coin) until
the network has collectively confirmed via a checkpoint that they are no longer
in the active set, this prevents validators from abandoning the network while
still being counted in the validator set. The collateral security property is
analyzed in [spec-security](spec-security.md) — Collateral Security.

**Why Groth16**

Groth16 is the most efficient proof system for on-chain verification. It
produces a constant-size proof of three elliptic curve points, 192 bytes total
([spec-wire-format](spec-wire-format.md) — Groth16 Proof Format), and verification requires
only a handful of pairing checks and point multiplications which are operations
CLVM already supports via CHIP-0011. The CLVM cost of Groth16 verification is
approximately 7.8 million units ([spec-clvm-costs](spec-clvm-costs.md) — Spend Path 2:
Checkpoint — Groth16 Verification), well within block limits. Other proof
systems like STARKs have larger verification overhead that doesnt map as
cleanly to existing CLVM operators.

**Why this matters beyond one project**

Any Chia L2 that uses a validator set for consensus faces this exact problem.
Without a standard every team will either reinvent the same solution or fall
back to insecure designs that trust the submitter's claimed signer count. A
shared standard means shared tooling, shared audits, and a clear path for
wallets and explorers to understand L2 checkpoints.

**Technical feasibility**

All operators needed for Groth16 verification already exist in CLVM as of
CHIP-0011. This CHIP requires no changes to the Chia node or CLVM. CLVM cost
analysis across all spend paths is documented in [spec-clvm-costs](spec-clvm-costs.md).

---

## Backwards Compatibility

This CHIP introduces no changes to CLVM or any existing on-chain primitives.
It is a standards document describing how to use existing operators and how to
structure four new puzzle types. There are no backwards incompatibilities.

---

## Rationale

**Why not iterate over individual signatures?**

CLVM puzzles can technically iterate using recursive patterns but the cost
scales linearly with the number of validators. For a validator set of any
meaningful size this becomes prohibitively expensive and sets a hard upper
bound on how large the validator set can grow. The cost analysis in
[spec-clvm-costs](spec-clvm-costs.md) shows that even the Groth16 approach costs only
~17.2M CLVM units for the full checkpoint spend, which is 0.16% of the block
limit and scales to any validator set size.

**Why not use fraud proofs?**

Fraud proofs shift the problem rather than solving it. To dispute a fraudulent
checkpoint a challenger still needs to prove on-chain that the claimed signer
set was invalid which runs into the same cardinality problem. Fraud proofs also
introduce a challenge window that delays finality which is undesirable for L2
systems that need prompt settlement.

**Why not require all validators to submit individually?**

Requiring each validator to individually spend a coin to cast a vote is a valid
alternative and may be preferable for small validator sets, however it requires
O(n) spends per checkpoint, increases blockchain footprint, and requires all
validators to be online and coordinated for each checkpoint submission. The ZK
proof approach lets a single submitter aggregate everything off-chain and post
one spend.

**Why a sparse Merkle tree**

The validator set changes over time as validators join and leave and an
append-only tree cant handle removals. A sparse Merkle tree
([spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)) assigns each validator a deterministic
slot derived from their pubkey and supports both insertions and deletions by
writing or zeroing a slot. Non-membership proofs are straightforward, you just
prove the slot is empty. The tree is constructed off-chain before each
checkpoint and the root is attested to by a majority signature.

**Circuit design choice**

The circuit ([spec-groth16-circuit](spec-groth16-circuit.md)) proves membership and majority
count but deliberately does not verify the BLS signature internally. BLS12-381
arithmetic inside a BLS12-381 Groth16 circuit requires emulating one field
inside the same field which is expensive and makes proof generation slow. By
splitting the work where the ZK proof handles membership and count and
`bls_verify` handles the signature we avoid this cost entirely. The two checks
together give complete security: the ZK proof ensures the aggregate pubkey
represents a legitimate majority of the registered set and `bls_verify` ensures
that majority actually signed the checkpoint. This completeness property is
analyzed in [spec-security](spec-security.md) — Completeness of the Two-Check
Design.

**Why the checkpoint singleton has a membership query spend path**

A validator recovering their collateral needs to prove they are no longer in
the active set. The membership query spend path
([spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend Path 2: Membership Query)
lets them do this at any time by spending the checkpoint singleton in a
read-only mode that recreates it unchanged and emits a membership announcement
([spec-wire-format](spec-wire-format.md) — Membership Announcement Format). The registration
coin then asserts that announcement in the same spend bundle
([spec-registration-coin](spec-registration-coin.md) — What the Puzzle Does) and there is no
timing pressure because the singleton persists with the same state until the
next checkpoint changes it.

**Why count is a runtime input not a circuit parameter**

Because `validator_count` comes from the checkpoint singleton state at proof
time rather than being baked into the circuit at setup time
([spec-groth16-circuit](spec-groth16-circuit.md) — Circuit Parameters), the circuit doesnt
need to be redeployed every time a validator joins or leaves. The trusted setup
([spec-trusted-setup](spec-trusted-setup.md)) only needs to be rerun if the maximum number
of simultaneous signers k needs to grow beyond what the current circuit
supports.

**A note on future improvements**

Right now verifying BLS12-381 signatures requires emulating BLS12-381 field
arithmetic as constraints inside a BLS12-381 Groth16 circuit, you are
essentially doing the same math twice in the same field which is expensive and
makes proof generation slow (see constraint estimates:
[spec-groth16-circuit](spec-groth16-circuit.md) — Constraint Count Estimates). If CLVM added
`bls12_377_pairing_identity` you could move the circuit to BLS12-377 instead.
Because BLS12-377 and BLS12-381 form a curve cycle, BLS12-381 operations would
become native constraints rather than emulated ones making proof generation
dramatically faster and the circuit much smaller. This CHIP works today with
what CLVM has but adding BLS12-377 pairing support would make it a lot more
practical at scale.

---

## Specification

### Definitions

The following terms are used throughout this CHIP and all referenced
implementation specs.

- **Network coin**: A Chia singleton that is the canonical registration
  authority for the entire L2 network. All validator registrations must go
  through it. One per network. Full spec:
  [spec-network-coin](spec-network-coin.md).

- **Registration coin**: A coin created by spending the network coin. Holds
  validator collateral. Only valid if its lineage proof traces back to a
  network coin spend. Full spec:
  [spec-registration-coin](spec-registration-coin.md).

- **Lineage proof**: The chain of parent coin IDs proving a registration coin
  was created by a legitimate network coin spend. Enforced by the indexer:
  [spec-indexer](spec-indexer.md) — Registration Coin Detection and Lineage
  Verification. Security analysis: [spec-security](spec-security.md) —
  Lineage Proof Enforcement.

- **Checkpoint singleton**: A Chia singleton that tracks the current L2 state
  root, epoch, validator Merkle root, and validator count. Full spec:
  [spec-checkpoint-singleton](spec-checkpoint-singleton.md).

- **Epoch**: An auto-incrementing integer stored in the checkpoint singleton,
  incremented by 1 on every checkpoint spend. Used for replay protection in
  membership announcements: [spec-wire-format](spec-wire-format.md) —
  Membership Announcement Format.

- **Sparse Merkle tree**: A fixed-depth Merkle tree where each validator has a
  deterministic slot derived from their pubkey, supporting both membership and
  non-membership proofs. Canonical spec:
  [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md).

- **Checkpoint message**: The message signed by validators, committing to the
  new state root, new validator Merkle root, new validator count, and new
  epoch. Exact format: [spec-wire-format](spec-wire-format.md) — Checkpoint
  Message.

- **Groth16 proof**: Three BLS12-381 curve points (A: G1 48 bytes, B: G2 96
  bytes, C: G1 48 bytes). 192 bytes total. Serialization:
  [spec-wire-format](spec-wire-format.md) — Groth16 Proof Format. Circuit:
  [spec-groth16-circuit](spec-groth16-circuit.md).

- **Verification key**: A set of BLS12-381 curve points derived from the
  trusted setup, curried into the checkpoint singleton. Format:
  [spec-wire-format](spec-wire-format.md) — Verification Key Format. Setup:
  [spec-trusted-setup](spec-trusted-setup.md).

---

### Part 1: Network Coin

Full implementation spec: [spec-network-coin](spec-network-coin.md).

The network coin is a singleton that acts as the canonical gatekeeper for
validator registration. There is exactly one network coin per L2 network.
Validators must spend the network coin to create their registration coin. The
inner puzzle validates the registration, creates the registration coin with the
approved puzzle hash and the validator's collateral, and recreates itself so
the next registration can go through it.

The approved registration coin puzzle hash is curried into the network coin at
deployment. This means every registration coin created through the network coin
has the same base puzzle just curried with different validator pubkeys. The L2
uses this to verify lineage ([spec-indexer](spec-indexer.md) — Registration
Coin Detection): a coin is a valid registration coin if and only if it has the
correct puzzle hash for its pubkey and its parent coin ID is a network coin
spend.

The CLVM cost of each network coin spend is approximately 5.3M units. See
[spec-clvm-costs](spec-clvm-costs.md) — Spend Path 1.

```rust
// network_coin_inner.rue
// Full driver code: spec-network-coin — Driver Code (Rust)

fn main(
    REGISTRATION_COIN_MOD_HASH: Bytes32,
    COLLATERAL_AMOUNT: Int,
    CHECKPOINT_SINGLETON_ID: Bytes32,

    new_validator_pubkey: PublicKey,
    conditions: List<Condition>,
) -> List<Condition> {

    // Registration coin puzzle hash - must match spec-registration-coin
    // and the lineage check in spec-indexer
    let registration_coin_puzzle_hash = curry_hash(
        REGISTRATION_COIN_MOD_HASH,
        new_validator_pubkey,
        CHECKPOINT_SINGLETON_ID,
    );

    // Registration message format per spec-wire-format — Registration Message Format
    let registration_message = sha256("register" + new_validator_pubkey);

    conditions + [
        AggSigMe(new_validator_pubkey, registration_message),
        CreateCoin(registration_coin_puzzle_hash, COLLATERAL_AMOUNT),
        CreateCoin(MY_PUZZLE_HASH, MY_AMOUNT),
    ]
}
```

---

### Part 2: Registration Coin

Full implementation spec: [spec-registration-coin](spec-registration-coin.md).

The registration coin holds the validator's collateral and commits to their
pubkey. It can only be spent when the checkpoint singleton confirms via a
membership announcement ([spec-wire-format](spec-wire-format.md) — Membership
Announcement Format) that the validator is no longer in the active set. The
coin ID is deterministic from the validator pubkey and is queryable by any
party building the validator set. Collateral recovery is described from the
validator's perspective in [spec-validator-onboarding](spec-validator-onboarding.md) — Voluntary Exit.

The CLVM cost of a registration coin spend is approximately 3.3M units. The
combined collateral recovery bundle (membership query + registration coin) is
approximately 7.4M units. See [spec-clvm-costs](spec-clvm-costs.md) — Spend
Paths 3 and 4.

```rust
// registration_coin.rue
// Full driver code: spec-registration-coin — Driver Code (Rust)

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
    let expected_announcement = sha256(
        "membership" + epoch + VALIDATOR_PUBKEY + 0
    );

    conditions + [
        AssertCoinAnnouncement(
            sha256(CHECKPOINT_SINGLETON_ID + expected_announcement)
        ),
        CreateCoin(collateral_destination, collateral_amount),
    ]
}
```

---

### Part 3: Checkpoint Singleton

Full implementation spec: [spec-checkpoint-singleton](spec-checkpoint-singleton.md).

The checkpoint singleton is the canonical on-chain source of truth for L2
state. It tracks four values curried into its inner puzzle on each recreation:

- `state_root` — the current L2 state rollup hash (32 bytes)
- `epoch` — auto-incremented by 1 on every checkpoint spend
- `validator_merkle_root` — sparse Merkle root of the current active validator
  set (32 bytes). Tree spec: [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md).
- `validator_count` — number of active validators (integer)

The singleton has two spend paths. Costs for each:
[spec-clvm-costs](spec-clvm-costs.md) — Spend Paths 2 and 3.

**Spend Path 1: Checkpoint**

Verifies majority consensus via a Groth16 proof
([spec-groth16-circuit](spec-groth16-circuit.md)) and updates all state. Can
be triggered at any time as long as majority consensus is achieved. The L1
does not impose timing rules on checkpoints, L2-level rules about when to
checkpoint are handled entirely off-chain
([spec-l2-integration](spec-l2-integration.md) — Checkpoint Submission Flow).

The checkpoint message that validators sign commits to the complete new state.
The exact byte format is defined in
[spec-wire-format](spec-wire-format.md) — Checkpoint Message:

```
checkpoint_message = sha256(
    new_state_root              (32 bytes)
    + new_validator_merkle_root (32 bytes)
    + new_validator_count_be    (8 bytes, big-endian u64)
    + new_epoch_be              (8 bytes, big-endian u64)
)
```

By signing the checkpoint message a majority of validators are collectively
attesting that the new validator Merkle root correctly reflects the current set
of valid registration coins on L1. This is how the Merkle root is trustlessly
verified without the L1 ever needing to query individual registration coins or
verify lineage proofs. This property is analyzed as Assumption 3 in
[spec-security](spec-security.md).

**Spend Path 2: Membership Query**

A read-only spend path that any party can use at any time. It verifies a
Merkle membership or non-membership proof
([spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Membership Proof and
Non-Membership Proof) against the current `validator_merkle_root`, recreates
the singleton unchanged, and emits a membership announcement
([spec-wire-format](spec-wire-format.md) — Membership Announcement Format).
Validators use this to prove they are no longer in the active set in order to
recover their registration collateral
([spec-registration-coin](spec-registration-coin.md)). Because the singleton
is recreated unchanged the same state remains queryable until the next
checkpoint changes the root so there is no timing pressure on the validator.

The full checkpoint singleton puzzle source with both spend paths is in
[spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Puzzle Source.

---

### Part 4: Off-Chain Validator Set Construction

Full implementation: [spec-indexer](spec-indexer.md) and
[spec-consensus-crate](spec-consensus-crate.md) — Checkpoint Submission.

Before each checkpoint the submitter:

1. Queries all coins with the registration coin puzzle mod hash on L1 and
   verifies lineage back to the network coin
   ([spec-indexer](spec-indexer.md) — Registration Coin Detection)
2. Constructs the sparse Merkle tree from verified registration coins only
   ([spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Root Computation)
3. Reads current checkpoint singleton state via
   [spec-consensus-crate](spec-consensus-crate.md) — State Accessors
4. Collects k validator signatures over the checkpoint message where 2k >
   validator_count using the signing message format in
   [spec-wire-format](spec-wire-format.md) — Individual Signatures
5. Computes `agg_sig` and `agg_signers` per
   [spec-wire-format](spec-wire-format.md) — Aggregate Signature and Aggregate
   Public Key
6. Generates k Merkle inclusion proofs per
   [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Proof Generation
7. Runs the Groth16 prover per
   [spec-groth16-circuit](spec-groth16-circuit.md) — Proof Generation
8. Submits the checkpoint spend

The Merkle root produced in step 2 should match the `validator_merkle_root` in
the current singleton state if the off-chain indexing is correct. The indexer
verifies this on every sync:
[spec-indexer](spec-indexer.md) — Merkle Root Consistency Check.

### Sparse Merkle Tree

Canonical spec: [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md).

The sparse Merkle tree has a fixed depth set at circuit compile time
([spec-groth16-circuit](spec-groth16-circuit.md) — Circuit Parameters:
TREE_DEPTH). Each validator occupies a slot derived from their pubkey:

```
slot = first_8_bytes_as_u64_big_endian(sha256(pubkey)) mod 2^TREE_DEPTH
```

Leaf values:

```
active leaf   = sha256(pubkey)
inactive leaf = sha256(0x00 * 48)  // EMPTY_LEAF_HASH
```

Non-membership proof: prove the leaf at the validator's slot contains
`EMPTY_LEAF_HASH`. The `EMPTY_LEAF_HASH` is curried into the checkpoint
singleton at deployment
([spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Curried In
Parameters).

### Circuit Public Inputs

The order of public inputs is fixed by the circuit definition
([spec-groth16-circuit](spec-groth16-circuit.md) — Public Inputs) and
determines the IC point assignment in the verification key
([spec-wire-format](spec-wire-format.md) — IC Point Order). The encoding of
each input to a field element uses the `scalar()` function defined in
[spec-wire-format](spec-wire-format.md) — The scalar() Function.

- `validator_merkle_root` — current active set root from checkpoint singleton
- `validator_count` — current count from checkpoint singleton
- `new_validator_merkle_root` — new active set root after this checkpoint
- `new_validator_count` — new count after this checkpoint
- `agg_signers` — G1 aggregate of k signing pubkeys (48 bytes compressed)
- `checkpoint_message` — sha256 of new state fields

### Circuit Statement

> I know k pubkeys, each with a valid Merkle inclusion proof against
> `validator_merkle_root` (per [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)), whose G1
> sum equals `agg_signers` (per [spec-wire-format](spec-wire-format.md) — Aggregate Public Key),
> and where 2k > `validator_count`. The checkpoint message commits to
> `new_validator_merkle_root` and `new_validator_count` which the majority is
> attesting to by signing.

Full circuit constraints: [spec-groth16-circuit](spec-groth16-circuit.md) —
Constraints 1, 2, and 3.

### Groth16 On-Chain Verification

A Groth16 proof `(A, B, C)` is verified by the equation:

```
e(A, B) * e(-vk_alpha, vk_beta) * e(-vk_input, vk_gamma) * e(-C, vk_delta) = 1
```

Where `vk_input` is computed using `g1_multiply` and `g1_add` from the public
inputs and the verification key IC points. The exact computation is in
[spec-wire-format](spec-wire-format.md) — VK Input Computation. This maps
directly to existing CLVM operators from CHIP-0011. CLVM cost of this
operation: ~7.8M units per
[spec-clvm-costs](spec-clvm-costs.md) — Groth16 Verification.

### Trusted Setup

Full runbook: [spec-trusted-setup](spec-trusted-setup.md).

Groth16 requires a one-time trusted setup per circuit configuration. The setup
is keyed to the maximum number of simultaneous signers k
([spec-groth16-circuit](spec-groth16-circuit.md) — Circuit Parameters:
MAX_SIGNERS). Because validator count is a runtime input the circuit doesnt
need to be redeployed when validators join or leave, it only needs redeployment
if k needs to grow beyond what the current setup supports. Applications must
conduct a multi-party computation ceremony
([spec-trusted-setup](spec-trusted-setup.md) — Multi-Party Ceremony). A
single-party setup must never be used in production
([spec-security](spec-security.md) — Assumption 2). The verification key
produced by the ceremony gets curried into the checkpoint singleton at
deployment ([spec-deployment-runbook](spec-deployment-runbook.md) — Step 2).

---

### De-Registration and Collateral Recovery

Full specs: [spec-registration-coin](spec-registration-coin.md) — Full
Collateral Recovery Spend Bundle and
[spec-validator-onboarding](spec-validator-onboarding.md) — Voluntary Exit.
Implemented by: [spec-consensus-crate](spec-consensus-crate.md) —
Collateral Recovery.

**Voluntary exit**

A validator signals intent to exit at the L2 level
([spec-l2-integration](spec-l2-integration.md) — Validator Set Transitions).
The next checkpoint includes the updated Merkle root that excludes them and
decrements the validator count. Once that checkpoint is accepted on L1 the
validator spends the checkpoint singleton via the membership query path
([spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend Path 2) to
get a non-membership announcement, then spends their registration coin in the
same bundle to recover their collateral.

**Forced exit**

If a majority of validators vote to remove a validator the next checkpoint
includes the updated Merkle root excluding that validator. The same collateral
recovery process applies. For slashing the registration coin puzzle can include
conditions that redirect collateral to a governance address. Known limitations
of the slashing model: [spec-security](spec-security.md) — Known Limitations.

**De-registration spend bundle**

The full bundle is constructed by
[spec-consensus-crate](spec-consensus-crate.md) — Collateral Recovery. The
membership announcement format is in
[spec-wire-format](spec-wire-format.md) — Membership Announcement Format:

```
Spend 1: Checkpoint singleton (membership query path)
    - Input: exiting validator pubkey + Merkle non-membership proof
      (per spec-sparse-merkle-tree — Non-Membership Proof)
    - Recreates singleton unchanged
    - Announces: sha256("membership" + epoch_be + pubkey + 0x00)

Spend 2: Registration coin
    - Asserts the membership announcement from the checkpoint singleton
    - Returns collateral to destination
```

The validator can execute this bundle at any time after a checkpoint that
excludes them is accepted, there is no block window to race against.

---

## Full Data Flow

The system-level data flow. Each step maps to one or more implementation specs.

```
Network coin deployed (one per L2 network)
(→ spec-network-coin — Deployment, spec-deployment-runbook — Step 2)
        |
        v
Validator spends network coin to register:
    - Signs their pubkey (→ spec-wire-format — Registration Message Format)
    - Network coin creates registration coin with approved puzzle hash
      (→ spec-registration-coin)
    - Network coin recreates itself for future registrations
(→ spec-validator-onboarding — Steps 5 and 6)
        |
        v
L2 queries registration coins and verifies lineage back to network coin
(→ spec-indexer — Registration Coin Detection)
L2 constructs sparse Merkle tree from valid registration coins only
(→ spec-sparse-merkle-tree — Root Computation)
        |
        v
Checkpoint time (whenever L2 achieves majority consensus):
    - Collect k > validator_count/2 signatures
      (→ spec-wire-format — Individual Signatures, spec-l2-integration — Signature Collection)
    - Run Groth16 prover off-chain (→ spec-groth16-circuit — Proof Generation)
    - Submit checkpoint spend:
        * Groth16 proof verifies membership + majority
          (→ spec-checkpoint-singleton — Spend Path 1)
        * bls_verify verifies aggregate signature
          (→ spec-wire-format — Aggregate Signature)
        * epoch increments by 1
        * new state committed on L1
        * checkpoint announcement emitted (→ spec-wire-format — Checkpoint State Announcement)
(→ spec-consensus-crate — submit_checkpoint(), spec-l2-integration — Checkpoint Submission Flow)
        |
        v
Validator exits (voluntary or forced via majority consensus):
    - Next checkpoint excludes their pubkey
      (→ spec-consensus-crate — compute_new_validator_set())
        |
        v
Validator spends checkpoint singleton via membership query path:
    - Provides non-membership Merkle proof (→ spec-sparse-merkle-tree — Non-Membership Proof)
    - Singleton recreated unchanged
    - Non-membership announcement emitted (→ spec-wire-format — Membership Announcement Format)
(→ spec-checkpoint-singleton — Spend Path 2)
        |
        v
Validator spends registration coin in same bundle:
    - Asserts non-membership announcement (→ spec-registration-coin — What the Puzzle Does)
    - Collateral returned to validator
(→ spec-consensus-crate — recover_collateral(), spec-validator-onboarding — Voluntary Exit)
```

---

## Test Cases

To be added to `assets/chip-<CHIP>/`. Reference implementations for each test
case are in [spec-groth16-circuit](spec-groth16-circuit.md),
[spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Test Vectors, and
[spec-wire-format](spec-wire-format.md) — Test Vectors.

- Valid network coin spend creates registration coin with correct puzzle hash
- Registration coin without valid network coin lineage is rejected by L2
- Validator cannot register on behalf of another validator without their signature
- Valid checkpoint spend accepted with correct proof
- Checkpoint spend with minority signers rejected (2k ≤ validator_count)
- Checkpoint spend with pubkeys not in current Merkle root rejected
- Checkpoint spend with incorrect agg_signers rejected
- Checkpoint spend with invalid BLS signature rejected
- Membership query spend returns correct announcement for active validator
- Membership query spend returns correct announcement for inactive validator
- Registration coin spend succeeds after non-membership announcement
- Registration coin spend fails without non-membership announcement
- Registration coin spend fails with non-membership announcement from wrong epoch
- Cross-implementation: Rust Merkle root == Rue Merkle root for same validator set
- End-to-end: network coin deployed, 10 validators registered, 6 sign checkpoint, 1 exits, collateral recovered

---

## Reference Implementation

To be provided before Review. Implementation is organized as the
`chia-l2-consensus` Rust crate
([spec-consensus-crate](spec-consensus-crate.md) — Crate Structure):

- Network coin inner puzzle in Rue ([spec-network-coin](spec-network-coin.md))
- Registration coin puzzle in Rue ([spec-registration-coin](spec-registration-coin.md))
- Checkpoint singleton inner puzzle in Rue ([spec-checkpoint-singleton](spec-checkpoint-singleton.md))
- Compiled CLVM bytecode for all three
- Groth16 circuit in Rust using Arkworks targeting BLS12-381 ([spec-groth16-circuit](spec-groth16-circuit.md))
- Sparse Merkle tree implementation in Rust ([spec-sparse-merkle-tree](spec-sparse-merkle-tree.md))
- Off-chain prover CLI including lineage proof verification
- Rust driver code using Chia Wallet SDK ([spec-consensus-crate](spec-consensus-crate.md))

---

## Security

Full analysis: [spec-security](spec-security.md).

**Lineage proof enforcement**

A registration coin is only valid if its parent coin ID traces back to a
network coin spend. This is enforced off-chain by the L2 indexer
([spec-indexer](spec-indexer.md) — Registration Coin Detection). Because coin
IDs are deterministic and the blockchain is immutable this check cant be gamed.
Any coin with the correct puzzle hash but the wrong parent is just ignored.
Full analysis: [spec-security](spec-security.md) — Lineage Proof Enforcement.

**Proving the correct circuit was used**

The verification key ([spec-trusted-setup](spec-trusted-setup.md) — What to
Publish) is cryptographically bound to the specific circuit
([spec-groth16-circuit](spec-groth16-circuit.md)) it was generated for during
the trusted setup ([spec-trusted-setup](spec-trusted-setup.md)). Since the VK
is curried into the checkpoint singleton at deployment
([spec-deployment-runbook](spec-deployment-runbook.md) — Step 7), the puzzle
hash on L1 commits to both the VK and the circuit permanently. Compliant
implementations must publish the circuit source and the full trusted setup
transcript. Users and wallets should independently verify the VK in the
deployed puzzle matches the VK from the published transcript before trusting
a checkpoint puzzle.

**Validator Merkle root correctness**

The new `validator_merkle_root` included in each checkpoint is not verified by
the L1 directly, its correctness is enforced by the majority signature. A
majority of validators independently construct the sparse Merkle tree
([spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)) from current valid
registration coins on L1 and would only sign a checkpoint message containing
the correct root. Constructing a fraudulent root requires corrupting a majority
of the validator set which is the core security assumption of the system
([spec-security](spec-security.md) — Assumption 1).

**Collateral security**

A validator cant recover their collateral
([spec-registration-coin](spec-registration-coin.md)) without a checkpoint
that excludes them which requires majority consensus. Full analysis:
[spec-security](spec-security.md) — Collateral Security.

**Trusted setup**

If the Groth16 trusted setup is compromised an attacker can generate fake
proofs and submit fraudulent checkpoints. Applications must conduct a proper
multi-party ceremony ([spec-trusted-setup](spec-trusted-setup.md) — Multi-Party
Ceremony). A single-party setup must never be used in production. Failure mode
and recovery: [spec-security](spec-security.md) — Trusted Setup Compromise.

**Completeness of the two-check design**

An attacker cant pass both the ZK proof check and `bls_verify` without both a
valid majority of registered keys and a valid aggregate signature from that
majority. The ZK proof alone does not prove the signature and the signature
alone does not prove the quorum is legitimate. Full analysis:
[spec-security](spec-security.md) — Completeness of the Two-Check Design.

**Epoch replay protection**

The membership announcement ([spec-wire-format](spec-wire-format.md) —
Membership Announcement Format) includes the epoch number. A non-membership
announcement from epoch N cant be used to spend a registration coin after
epoch N+1 has started if the validator rejoined in the interim. Full analysis:
[spec-security](spec-security.md) — Epoch Replay Protection.

---

## Additional Assets

To be added.

---

## Copyright

Copyright and related rights waived via [CC0](https://creativecommons.org/publicdomain/zero/1.0/).
