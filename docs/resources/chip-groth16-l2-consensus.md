# CHIP: Groth16 Proof Verification for L2 Consensus

| CHIP Number | *(leave blank - editor will assign)* |
|---|---|
| Title | Groth16 Proof Verification for L2 Consensus |
| Description | A standard for proving L2 majority validator consensus on the Chia L1 at constant cost using a network coin, registration coins, a checkpoint singleton, withdraw delay coins, and Groth16 ZK proofs |
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
| Part 4: Withdraw Delay Coin | [spec-withdraw-delay-coin](spec-withdraw-delay-coin.md) |
| Part 5: Off-Chain Validator Set Construction | [spec-indexer](spec-indexer.md), [spec-consensus-crate](spec-consensus-crate.md) |
| Sparse Merkle Tree | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) |
| Circuit Public Inputs / Circuit Statement | [spec-groth16-circuit](spec-groth16-circuit.md) |
| Groth16 On-Chain Verification | [spec-wire-format](spec-wire-format.md), [spec-checkpoint-singleton](spec-checkpoint-singleton.md) |
| Trusted Setup | [spec-trusted-setup](spec-trusted-setup.md) |
| De-Registration and Collateral Recovery | [spec-registration-coin](spec-registration-coin.md), [spec-withdraw-delay-coin](spec-withdraw-delay-coin.md), [spec-validator-onboarding](spec-validator-onboarding.md) |
| Security | [spec-security](spec-security.md) |
| CLVM costs | [spec-clvm-costs](spec-clvm-costs.md) |
| Integration | [spec-l2-integration](spec-l2-integration.md), [spec-consensus-crate](spec-consensus-crate.md) |
| Deployment | [spec-deployment-runbook](spec-deployment-runbook.md) |

---

## Abstract

This CHIP defines a standard for how Chia L2 systems can prove majority
validator consensus to the Chia L1 at constant cost. It covers five
components:

1. A **network coin** that is the canonical registration authority for the
   entire network (full spec: [spec-network-coin](spec-network-coin.md)).
2. **Registration coins** that validators create by spending the network coin
   and which hold their collateral (full spec:
   [spec-registration-coin](spec-registration-coin.md)).
3. A **checkpoint singleton** that tracks L2 state and the active validator
   set (full spec: [spec-checkpoint-singleton](spec-checkpoint-singleton.md)).
4. **Withdraw delay coins** that enforce a mandatory waiting period before
   collateral can be released after a validator exits (full spec:
   [spec-withdraw-delay-coin](spec-withdraw-delay-coin.md)).
5. A **Groth16 zero-knowledge proof** that a checkpoint submitter generates
   off-chain to prove a signing majority (circuit spec:
   [spec-groth16-circuit](spec-groth16-circuit.md), setup:
   [spec-trusted-setup](spec-trusted-setup.md)).

The lineage proof between a registration coin and its parent network coin spend
is what makes a registration coin legitimate. Any coin that cannot prove this
lineage gets ignored by the L2 when constructing the validator set.

The checkpoint singleton has two spend paths: a checkpoint spend that verifies
the ZK proof and updates state, and a membership query spend that any party can
use at any time to prove a validator is no longer in the active set.

When a validator exits, their registration coin does not release collateral
directly. Instead it creates a withdraw delay coin that holds the collateral
for a configurable number of L1 blocks (default: 24,000 blocks, approximately
5 days). This delay window gives the network time to detect and respond to
misbehavior before funds leave the system.

The proof is verified on-chain using existing CLVM operators from CHIP-0011,
specifically `bls_pairing_identity`, `g1_multiply`, and `g1_add`. No new
opcodes are required.

---

## Motivation

**The problem**

To build an L2 system on Chia you need a set of validators that collectively
sign off on state checkpoints. Those checkpoints get posted to the Chia L1 to
settle state and the L1 puzzle needs to verify that a majority of the
registered validator set actually signed them.

The natural approach is BLS signature aggregation: combine all signing keys
into one aggregate public key and verify a single aggregate signature. CLVM
already supports this with `g1_add` and `bls_verify`. The problem is that an
aggregate G1 point carries no information about how many individual keys were
combined into it — you can produce a valid aggregate from 1 key or 100 keys
and the resulting point looks identical. This means a single bad actor with
one valid validator key could submit a fraudulent checkpoint claiming majority
consensus and the L1 puzzle would have no way to tell the difference.

The only way to solve this without iterating over individual signatures (which
CLVM puzzles cannot do at reasonable cost — see
[spec-clvm-costs](spec-clvm-costs.md)) is to use a zero-knowledge proof. The
checkpoint submitter proves off-chain that they have k valid validator keys
from the registered set, that k is a majority of the total registered
validators, and that those keys produced a valid aggregate signature over the
checkpoint. On-chain, CLVM just verifies the proof at constant cost regardless
of how large the validator set is.

**Why the network coin matters**

Without a canonical registration authority anyone could create a coin with the
registration coin puzzle hash and claim to be a registered validator. The
network coin solves this — it is a singleton that acts as the gatekeeper for
the entire network. A registration coin is only valid if its lineage proof
traces back to a spend of the network coin. The L2 enforces this lineage check
when constructing the validator set off-chain.

**Why collateral recovery has a delay**

Without a delay, a malicious validator could misbehave, immediately exit, and
recover their collateral before the network can react. The withdraw delay coin
introduces a mandatory 5-day waiting period (configurable per deployment)
between collateral recovery initiation and actual fund release. This gives the
network time to detect misbehavior and potentially slash the collateral.

**Why Groth16**

Groth16 is the most efficient proof system for on-chain verification. It
produces a constant-size proof of three elliptic curve points, 192 bytes total,
and verification requires only a handful of pairing checks and point
multiplications which are operations CLVM already supports via CHIP-0011.

**Technical feasibility**

All operators needed for Groth16 verification already exist in CLVM as of
CHIP-0011. This CHIP requires no changes to the Chia node or CLVM.

---

## Backwards Compatibility

This CHIP introduces no changes to CLVM or any existing on-chain primitives.
It is a standards document describing how to use existing operators and how to
structure five new puzzle types. There are no backwards incompatibilities.

---

## Specification

### Definitions

- **Network coin**: A Chia singleton that is the canonical registration
  authority for the entire L2 network. All validator registrations must go
  through it. One per network. Full spec:
  [spec-network-coin](spec-network-coin.md).

- **Registration coin**: A coin created by spending the network coin. Holds
  validator collateral. Only valid if its lineage proof traces back to a
  network coin spend. When spent, creates a withdraw delay coin instead of
  releasing collateral directly. Full spec:
  [spec-registration-coin](spec-registration-coin.md).

- **Withdraw delay coin**: A time-locked coin created by spending a
  registration coin. Holds collateral for `WITHDRAW_DELAY_BLOCKS` L1 blocks
  (default: 24,000, approximately 5 days) before allowing release to the
  destination address. Permissionless to spend after the delay. Full spec:
  [spec-withdraw-delay-coin](spec-withdraw-delay-coin.md).

- **Lineage proof**: The chain of parent coin IDs proving a registration coin
  was created by a legitimate network coin spend. Enforced by the indexer.

- **Checkpoint singleton**: A Chia singleton that tracks the current L2 state
  root, epoch, validator Merkle root, validator count, and network identity.

- **Epoch**: An auto-incrementing integer stored in the checkpoint singleton,
  incremented by 1 on every checkpoint spend. Computed internally by the
  puzzle, not accepted from the solution.

- **Sparse Merkle tree**: A fixed-depth (32) Merkle tree where each validator
  has a deterministic slot derived from `sha256(pubkey)`, supporting both
  membership and non-membership proofs.

- **Checkpoint message**: The message signed by validators:
  `sha256(state_root + merkle_root + count_be8 + epoch_be8 + network_coin_launcher_id)`.
  112 bytes hashed to 32 bytes. The network ID prevents cross-network replay.

- **Groth16 proof**: Three BLS12-381 curve points (A: G1 48 bytes, B: G2 96
  bytes, C: G1 48 bytes). 192 bytes total.

- **Verification key**: BLS12-381 curve points from the trusted setup (672
  bytes total: alpha_g1 + beta_g2 + gamma_g2 + delta_g2 + 7 IC points),
  curried into the checkpoint singleton at deployment.

---

### Part 1: Network Coin

Full implementation spec: [spec-network-coin](spec-network-coin.md).

The network coin is a singleton that acts as the canonical gatekeeper for
validator registration. There is exactly one network coin per L2 network.

The inner puzzle has six curried parameters and two solution parameters:

```rue
fn main(
    // Curried parameters (fixed at deployment)
    INNER_MOD_HASH: Bytes32,              // Self-reference for singleton morphing
    registration_coin_mod_hash: Bytes32,  // Tree hash of uncurried registration coin
    collateral_amount: Int,               // Required collateral per validator
    checkpoint_singleton_id: Bytes32,     // Checkpoint singleton launcher ID
    withdraw_delay_mod_hash: Bytes32,     // Tree hash of withdraw delay coin puzzle
    withdraw_delay_blocks: Int,           // L1 blocks delay (default: 24,000)

    // Solution parameters
    new_validator_pubkey: PublicKey,       // Registering validator's BLS key
    conditions: List<Condition>,          // Pass-through (for fees)
) -> List<Condition> {

    // Registration coin puzzle hash includes all 4 curried params
    let registration_coin_puzzle_hash = curry_tree_hash(
        registration_coin_mod_hash,
        [
            tree_hash(new_validator_pubkey),
            tree_hash(checkpoint_singleton_id),
            tree_hash(withdraw_delay_mod_hash),
            tree_hash(withdraw_delay_blocks),
        ],
    );

    let registration_message = sha256("register" + new_validator_pubkey);

    [
        AggSigMe(new_validator_pubkey, registration_message),
        CreateCoin(registration_coin_puzzle_hash, collateral_amount),
        CreateCoin(INNER_MOD_HASH, 1),  // Singleton self-recreation
        ...conditions
    ]
}
```

---

### Part 2: Registration Coin

Full implementation spec: [spec-registration-coin](spec-registration-coin.md).

The registration coin holds the validator's collateral and commits to their
pubkey. It can only be spent when the checkpoint singleton confirms the
validator is no longer in the active set. When spent, it creates a **withdraw
delay coin** rather than releasing collateral directly — this enforces a
mandatory waiting period before funds can be claimed.

The puzzle has four curried parameters and three solution parameters. No
conditions pass-through is permitted (SEC-008: prevents condition injection).

```rue
fn main(
    // Curried parameters (set by network coin at creation)
    VALIDATOR_PUBKEY: PublicKey,           // 48-byte BLS G1 point
    CHECKPOINT_SINGLETON_ID: Bytes32,     // Coin ID (not launcher ID)
    WITHDRAW_DELAY_MOD_HASH: Bytes32,     // Tree hash of withdraw delay puzzle
    WITHDRAW_DELAY_BLOCKS: Int,           // L1 blocks to wait (default: 24,000)

    // Solution parameters
    epoch: Int,                           // Current checkpoint epoch
    collateral_destination: Bytes32,      // Where collateral goes (via delay coin)
    collateral_amount: Int,               // Amount to return
) -> List<Condition> {

    // Compute withdraw delay coin puzzle hash on-chain
    let withdraw_delay_puzzle_hash = curry_tree_hash(
        WITHDRAW_DELAY_MOD_HASH,
        [
            tree_hash(collateral_destination),
            tree_hash(collateral_amount),
            tree_hash(WITHDRAW_DELAY_BLOCKS),
        ],
    );

    // Non-membership announcement from checkpoint singleton
    let expected_announcement = sha256(
        "membership" + int_to_8_bytes_be(epoch) + VALIDATOR_PUBKEY + 0x00
    );

    // Conflict-resistant hint for delay coin discovery
    let hint = sha256(CHECKPOINT_SINGLETON_ID + collateral_destination);

    [
        AssertCoinAnnouncement(sha256(CHECKPOINT_SINGLETON_ID + expected_announcement)),
        CreateCoin(withdraw_delay_puzzle_hash, collateral_amount, hint),
    ]
}
```

**Key changes from earlier designs:**
- Four curried parameters (was two) — added `WITHDRAW_DELAY_MOD_HASH` and
  `WITHDRAW_DELAY_BLOCKS` for on-chain delay coin hash computation.
- Creates a **withdraw delay coin**, not a direct destination coin.
- No `conditions` pass-through — prevents condition injection attacks (SEC-008).
- Includes a conflict-resistant hint memo:
  `sha256(CHECKPOINT_SINGLETON_ID + destination)` for indexed lookup via
  `get_coin_records_by_hint()`.

---

### Part 3: Checkpoint Singleton

Full implementation spec:
[spec-checkpoint-singleton](spec-checkpoint-singleton.md).

The checkpoint singleton is the canonical on-chain source of truth for L2
state. It tracks these values in a `STATE` struct curried into its inner puzzle:

- `state_root` — the current L2 state rollup hash (32 bytes)
- `epoch` — auto-incremented by 1 on every checkpoint spend
- `validator_merkle_root` — sparse Merkle root of the active validator set
- `validator_count` — number of active validators

Additional curried constants (immutable after deployment):

- `INNER_MOD_HASH` — self-reference for singleton morphing
- `VK` — Groth16 verification key (alpha, beta, gamma, delta)
- `IC` — 7 IC points for public input linear combination
- `TREE_DEPTH` — sparse Merkle tree depth (32)
- `EMPTY_LEAF_HASH` — `sha256([0x00; 48])` = `0x17b076...`
- `NETWORK_COIN_LAUNCHER_ID` — network identity for cross-network replay
  prevention (CHK-012)

**Spend Path 1: Checkpoint**

Verifies majority consensus via Groth16 proof + BLS signature and updates
state. Anyone can submit a checkpoint (permissionless submission) but only
bundles with valid proofs and majority signatures are accepted.

The checkpoint message includes the network identity:

```
checkpoint_message = sha256(
    new_state_root                (32 bytes)
    + new_validator_merkle_root   (32 bytes)
    + new_validator_count_be      (8 bytes, big-endian u64)
    + new_epoch_be                (8 bytes, big-endian u64)
    + network_coin_launcher_id    (32 bytes)
)
```

Total preimage: **112 bytes** → sha256 → 32-byte message.

The puzzle computes `new_epoch = old_epoch + 1` internally — the epoch is
never accepted from the solution, preventing manipulation (CHK-009).

Verification steps:
1. **Scalar verification**: Assert 6 scalars match sha256 of corresponding
   public inputs.
2. **VK input computation**: `vk_input = IC[0] + Σ(IC[i] * scalar_i)` using
   `g1_multiply` and `point_add`.
3. **Groth16 pairing check**: `bls_pairing_identity` with 4 pairs verifying
   `e(A,B) * e(-α,β) * e(-vk_input,γ) * e(-C,δ) = 1`.
4. **BLS signature verification**: `bls_verify(agg_sig, agg_signers,
   checkpoint_message)`.

On success: recreates singleton with updated state, emits checkpoint
announcement.

**Spend Path 2: Membership Query**

A permissionless (no signature required) spend path for querying validator
membership status. Verifies a Merkle proof against the current
`validator_merkle_root`, recreates the singleton unchanged, and emits an
announcement.

Announcement format:
```
sha256("membership" + epoch_be8 + pubkey + is_member_byte)
```
Where `is_member_byte` is 0x01 for member, 0x00 for non-member.

---

### Part 4: Withdraw Delay Coin

Full implementation spec:
[spec-withdraw-delay-coin](spec-withdraw-delay-coin.md).

The withdraw delay coin is a time-locked container for validator collateral.
It is created by the registration coin when a validator exits and enforces a
mandatory waiting period before funds can be released.

The puzzle has three curried parameters, no solution parameters, and no
conditions pass-through. All behavior is locked at creation time.

```rue
fn main(
    DESTINATION: Bytes32,             // Where funds go after delay
    AMOUNT: Int,                      // Collateral amount in mojos
    WITHDRAW_DELAY_BLOCKS: Int,       // L1 blocks to wait (default: 24,000)
) -> List<Condition> {
    [
        AssertHeightRelative(WITHDRAW_DELAY_BLOCKS),
        CreateCoin(DESTINATION, AMOUNT, "DIG Network Collateral Release"),
    ]
}
```

**Properties:**
- **Time lock**: `ASSERT_HEIGHT_RELATIVE` enforced by Chia consensus (not CLVM).
  Cannot be bypassed.
- **Permissionless**: No signature required after delay expires. Anyone can
  submit the release transaction.
- **Immutable**: Destination, amount, and delay are curried — cannot be changed.
- **Identifiable**: Memo "DIG Network Collateral Release" tags the transaction
  in wallets and explorers.

**Default delay**: 24,000 blocks ≈ 5 days at ~18 seconds per block.

---

### Part 5: Off-Chain Validator Set Construction

Before each checkpoint the submitter:

1. Queries all coins with the registration coin puzzle mod hash on L1 and
   verifies lineage back to the network coin.
2. Constructs the sparse Merkle tree from verified registration coins only.
3. Reads current checkpoint singleton state.
4. Collects k validator signatures over the checkpoint message where
   `2k > validator_count`.
5. Computes `agg_sig` (G2 aggregate) and `agg_signers` (G1 aggregate).
6. Generates k Merkle inclusion proofs.
7. Runs the Groth16 prover.
8. Builds the checkpoint spend bundle and returns it to the caller for
   broadcasting.

The `chia-l2-consensus` crate builds all spend bundles but never broadcasts
them. The importing project is responsible for submitting bundles to the Chia
network via `push_tx()` or equivalent.

### Sparse Merkle Tree

The sparse Merkle tree has fixed depth 32, supporting 2^32 validator slots.
Each validator occupies a slot derived from their pubkey:

```
slot = first_8_bytes_as_u64_big_endian(sha256(pubkey)) mod 2^32
```

Leaf values:
- Active leaf: `sha256(pubkey)` (48-byte compressed G1 input)
- Empty leaf: `sha256([0x00; 48])` = `EMPTY_LEAF_HASH`

The tree root is computed as `sha256(left || right)` at each level, with
left child always first. Empty subtrees use precomputed hashes for efficiency.

### Circuit Public Inputs

The Groth16 circuit accepts exactly 6 public inputs in fixed order:

1. `validator_merkle_root` — current active set root
2. `validator_count` — current count
3. `new_validator_merkle_root` — new active set root
4. `new_validator_count` — new count
5. `agg_signers` — G1 aggregate of k signing pubkeys
6. `checkpoint_message` — sha256 of new state fields + network ID

### Circuit Statement

> I know k pubkeys, each with a valid Merkle inclusion proof against
> `validator_merkle_root`, whose G1 sum equals `agg_signers`, and where
> `2k > validator_count`.

The circuit enforces three constraints:
1. **Merkle membership** (CIR-002): Each signer has a valid inclusion proof.
2. **Aggregate key binding** (CIR-003): G1 sum of k pubkeys == `agg_signers`.
   This prevents phantom majority attacks where an attacker forges a proof
   claiming more signers than actually exist.
3. **Majority threshold** (CIR-004): `2k > validator_count`.

Circuit parameters fixed at trusted setup: `MAX_SIGNERS` = 20,000,
`TREE_DEPTH` = 32.

### Groth16 On-Chain Verification

A Groth16 proof `(A, B, C)` is verified by the equation:

```
e(A, B) * e(-vk_alpha, vk_beta) * e(-vk_input, vk_gamma) * e(-C, vk_delta) = 1
```

Where `vk_input` is computed using `g1_multiply` and `point_add` from the
public inputs, IC points, and scalar values. Each scalar is
`sha256(public_input)` interpreted as a big-endian integer.

---

### De-Registration and Collateral Recovery

Collateral recovery is a **two-phase process**:

**Phase 1: Initiate Recovery (creates withdraw delay coin)**

```
Spend 1: Checkpoint singleton (membership query path)
    - Input: exiting validator pubkey + Merkle non-membership proof
    - Recreates singleton unchanged
    - Announces: sha256("membership" + epoch_be8 + pubkey + 0x00)

Spend 2: Registration coin
    - Asserts the non-membership announcement
    - Creates WITHDRAW DELAY COIN (not direct destination)
    - Delay coin puzzle hash = curry_hash(WDC_MOD, dest, amount, delay)
    - Memo hint = sha256(checkpoint_id + destination)
```

**Phase 2: Release Collateral (after delay period)**

```
Spend 1: Withdraw delay coin
    - ASSERT_HEIGHT_RELATIVE(WITHDRAW_DELAY_BLOCKS) — enforces time lock
    - CREATE_COIN(DESTINATION, AMOUNT) — releases funds
    - Memo: "DIG Network Collateral Release"
    - No signature needed — permissionless after delay
```

The validator can execute Phase 2 at any time after `WITHDRAW_DELAY_BLOCKS`
L1 blocks have passed since Phase 1 was confirmed. There is no timing pressure
because the delay coin persists until spent.

**Voluntary exit**: Validator signals intent at L2 level → next checkpoint
excludes them → Phase 1 → wait 24,000 blocks (~5 days) → Phase 2.

**Forced exit**: Majority votes to remove validator → same process.

---

## Full Data Flow

```
Network coin deployed (one per L2 network)
        |
        v
Validator spends network coin to register:
    - Signs their pubkey (AggSigMe)
    - Network coin creates registration coin with collateral
      (puzzle hash includes withdraw delay params)
    - Network coin recreates itself
        |
        v
L2 queries registration coins and verifies lineage back to network coin
L2 constructs sparse Merkle tree from valid registration coins only
        |
        v
Checkpoint time:
    - Collect k > validator_count/2 signatures
    - Run Groth16 prover off-chain (circuit proves membership + majority)
    - Build checkpoint spend bundle (returned to caller, not broadcast)
    - Caller submits: bls_pairing_identity verifies proof,
      bls_verify verifies aggregate signature
    - Epoch increments, new state committed on L1
        |
        v
Validator exits:
    - Next checkpoint excludes their pubkey
        |
        v
Phase 1 — Collateral Recovery:
    - Membership query: non-membership announcement emitted
    - Registration coin: asserts announcement, creates WITHDRAW DELAY COIN
    - Delay coin holds collateral for 24,000 blocks (~5 days)
        |
        v
Phase 2 — Collateral Release (after delay):
    - Withdraw delay coin: ASSERT_HEIGHT_RELATIVE passes
    - Funds released to validator's destination address
    - Permissionless — anyone can submit the release transaction
```

---

## Security

Full analysis: [spec-security](spec-security.md).

**Lineage proof enforcement**: A registration coin is only valid if its parent
coin ID traces back to a network coin spend. Enforced off-chain by the indexer.

**Two-check design**: The ZK proof alone does not prove the signature was made.
The signature alone does not prove the quorum is legitimate. Together they
provide the complete guarantee.

**Aggregate key binding** (CIR-003 / SEC-011): The circuit enforces that
`agg_signers` equals the G1 sum of the k signing pubkeys. This prevents
phantom majority attacks where an attacker with the proving key forges a proof
claiming an arbitrary number of signers.

**Collateral security**: A validator cannot recover their collateral without:
(a) a checkpoint that excludes them, (b) a non-membership Merkle proof, and
(c) waiting `WITHDRAW_DELAY_BLOCKS` L1 blocks after creating the delay coin.

**Condition injection protection** (SEC-008): The registration coin, checkpoint
singleton, and withdraw delay coin puzzles accept no conditions pass-through
from solutions. Only the network coin allows pass-through conditions, protected
by AggSigMe.

**Cross-network replay protection** (CHK-012): The checkpoint message includes
`network_coin_launcher_id`, preventing proofs generated for one L2 network from
being replayed on another.

**Epoch replay protection**: The membership announcement includes the epoch
number. A non-membership announcement from epoch N cannot be used after
epoch N+1 if the validator rejoined.

**Trusted setup**: Groth16 requires a one-time trusted setup per circuit
configuration. Applications MUST conduct a multi-party computation (MPC)
ceremony. A single-party setup MUST NOT be used in production — if the setup
is compromised, an attacker can generate fake proofs. At least one honest
participant in the MPC ceremony is sufficient for soundness.

**Withdraw delay security**: The 5-day default delay window provides time for
the network to detect and respond to misbehavior. The delay is enforced by
Chia's `ASSERT_HEIGHT_RELATIVE` condition (consensus-level, not CLVM-level).
Destination and amount are curried into the delay coin — they cannot be changed
after creation.

---

## Test Cases

Reference implementations for each test case are in the `tests/` directory of
the `chia-l2-consensus` crate. 1,074 test functions across 99 VV test files
verify all 108 requirements.

Key test coverage:
- CLVM execution tests for all four puzzles (deserialize hex, curry, run,
  assert conditions)
- Simulator tests using chia-sdk-test for cross-coin spend bundles
- Cross-implementation tests (Rust hash == CLVM hash for Merkle roots,
  announcement hashes, puzzle hashes)
- Groth16 proof generation and verification round-trip
- Failure case coverage (invalid proofs, missing announcements, wrong epochs)

---

## Reference Implementation

Implementation is the `chia-l2-consensus` Rust crate:

- Network coin inner puzzle in Rue (`puzzles/network_coin_inner.rue`)
- Registration coin puzzle in Rue (`puzzles/registration_coin.rue`)
- Checkpoint singleton inner puzzle in Rue (`puzzles/checkpoint_inner.rue`)
- Withdraw delay coin puzzle in Rue (`puzzles/withdraw_delay_coin.rue`)
- Compiled CLVM bytecode for all four (`puzzles/compiled/*.hex`)
- Groth16 circuit in Rust using Arkworks targeting BLS12-381
- Sparse Merkle tree implementation in Rust (depth 32, SHA-256)
- Chain indexer using `chia-query` for decentralized blockchain queries
- Wallet integration using `dig-l1-wallet` for collateral coin selection
- Rust driver code using `chia-wallet-sdk`

The crate builds all spend bundles but never broadcasts transactions. The
importing L2 project is responsible for broadcasting via `push_tx()`.

---

## Copyright

Copyright and related rights waived via
[CC0](https://creativecommons.org/publicdomain/zero/1.0/).
