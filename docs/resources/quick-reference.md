# chia-l2-consensus — Quick Reference Guide

---

## How to Use This Document

The first section is a short primer that builds the mental model for the whole
system — read it once, in order. The reference tables that follow are for
lookup: find the thing you need, see exactly what it is, and jump to the spec
that defines it in full. Every entry in every table cites its source.

**Specs referenced throughout:**

| Short name | Full document |
|-----------|--------------|
| CHIP | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) |
| network-coin | [spec-network-coin](spec-network-coin.md) |
| reg-coin | [spec-registration-coin](spec-registration-coin.md) |
| checkpoint | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) |
| smt | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) |
| wire | [spec-wire-format](spec-wire-format.md) |
| circuit | [spec-groth16-circuit](spec-groth16-circuit.md) |
| setup | [spec-trusted-setup](spec-trusted-setup.md) |
| indexer | [spec-indexer](spec-indexer.md) |
| crate | [spec-consensus-crate](spec-consensus-crate.md) |
| costs | [spec-clvm-costs](spec-clvm-costs.md) |
| deploy | [spec-deployment-runbook](spec-deployment-runbook.md) |
| onboard | [spec-validator-onboarding](spec-validator-onboarding.md) |
| l2 | [spec-l2-integration](spec-l2-integration.md) |
| security | [spec-security](spec-security.md) |

---

# Part 1 — Primer

## The Problem This System Solves

Chia L2 systems need validators to collectively sign checkpoints that settle
L2 state on the L1 blockchain. The naive approach — aggregate all signing keys
into one BLS pubkey and verify one signature — fails because an aggregate G1
point carries no information about how many keys were combined into it. A
single validator key could fake a majority. The system needs to prove that a
signing majority exists without iterating over all individual keys on-chain,
which would be too expensive.

The solution is a Groth16 zero-knowledge proof. The checkpoint submitter
generates a proof off-chain that demonstrates k validator pubkeys are all
members of the registered set and that 2k > total_count. On-chain, CLVM
verifies the proof at constant cost — roughly 17 million cost units regardless
of how large the validator set is, which is 0.16% of the block limit.

---

## The Four On-Chain Components

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         NETWORK COIN (singleton)                         │
│                 One per L2 network. Never holds value.                   │
│     Spent to register a validator → creates their registration coin.     │
│        Recreates itself so the next registration can go through it.      │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │  parent coin ID = lineage proof
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     REGISTRATION COIN (one per validator)                │
│           Holds collateral. Curried: validator pubkey + singleton ID.    │
│    Can only be spent when the checkpoint singleton emits a              │
│    non-membership announcement for this validator's pubkey.              │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                    CHECKPOINT SINGLETON (singleton)                      │
│  Tracks: state_root | epoch | validator_merkle_root | validator_count   │
│                                                                          │
│  Spend Path 1 — Checkpoint:                                             │
│    Verifies Groth16 proof (membership + majority)                        │
│    Verifies BLS aggregate signature                                       │
│    Updates all state, increments epoch                                   │
│                                                                          │
│  Spend Path 2 — Membership Query (permissionless):                      │
│    Verifies a Merkle proof (membership or non-membership)                │
│    Recreates singleton UNCHANGED                                          │
│    Emits announcement ← registration coin asserts this to recover        │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                    GROTH16 PROOF (off-chain, 192 bytes)                  │
│  Proves: k pubkeys ∈ validator_merkle_root                              │
│          G1 sum of those pubkeys = agg_signers                          │
│          2k > validator_count                                            │
│  Does NOT prove signature validity — bls_verify handles that separately  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## The Two Security Mechanisms

The system has two independent on-chain checks that together give complete security:

**Check 1 — Groth16 proof via `bls_pairing_identity`**
Proves the claimed aggregate pubkey (`agg_signers`) is actually the G1 sum of
k legitimate, registered validator pubkeys, and that k is a strict majority of
`validator_count`. This is what prevents a single key from faking a quorum.

**Check 2 — BLS aggregate signature via `bls_verify`**
Proves that the majority represented by `agg_signers` actually signed the
specific checkpoint message. This binds the proof to a specific state
transition.

Neither check alone is sufficient. The ZK proof does not prove the signature
was made. The signature alone does not prove the signers are a legitimate
majority. Together they provide the complete guarantee.

→ see [security](spec-security.md) — Completeness of the Two-Check Design

---

## The Validator Set Lives Off-Chain

The checkpoint singleton stores only the root of a sparse Merkle tree and a
count. The actual list of validators is never stored on L1. Instead:

1. The indexer queries all registration coins and verifies each one's lineage
   traces back to a network coin spend.
2. It builds the sparse Merkle tree from only the verified coins.
3. The submitter includes the new Merkle root in the checkpoint message.
4. A majority of validators sign that message.

The majority signature over the checkpoint message is itself the trustless
proof that the Merkle root is correct — any fraudulent root would never gather
a majority signature from the real validator set.

→ see [CHIP](chip-groth16-l2-consensus.md) — Why the validator set lives off-chain

---

## The Five Things That Must Be Byte-Identical Between Rust and Rue

These are the most common source of silent failures. If any of these diverges
between the off-chain Rust code and the on-chain Rue puzzle, verification will
fail with no helpful error.

| # | What must match | Defined in |
|---|----------------|-----------|
| 1 | Sibling ordering in Merkle path verification: **left child always first** | [smt](spec-sparse-merkle-tree.md) — Tree Structure |
| 2 | Leaf value for active validator: `sha256(pubkey)` | [smt](spec-sparse-merkle-tree.md) — Leaf Values |
| 3 | Leaf value for empty slot: `sha256(0x00 * 48)` | [smt](spec-sparse-merkle-tree.md) — Leaf Values |
| 4 | Checkpoint message: `sha256(state_root + merkle_root + count_be8 + epoch_be8)` | [wire](spec-wire-format.md) — Checkpoint Message |
| 5 | `scalar(bytes)` = `sha256(bytes)` as big-endian u256, mod r | [wire](spec-wire-format.md) — The scalar() Function |

---

## The Lifecycle in Eight Steps

```
1. DEPLOY
   Run trusted setup → get proving key + VK
   Derive IDs from genesis coin (resolves circular dependency)
   Deploy network coin + checkpoint singleton in one bundle
   Publish VK hash, verify it is correctly curried on-chain
   → spec-deployment-runbook

2. REGISTER
   Validator: spend network coin with their secret key
   Network coin creates registration coin with approved puzzle hash + collateral
   Indexer detects the new registration coin, verifies lineage
   → spec-network-coin, spec-registration-coin, spec-indexer

3. SYNC (runs before every checkpoint)
   Indexer fetches all registration coins, verifies each lineage
   Builds sparse Merkle tree off-chain
   Verifies computed root == on-chain validator_merkle_root
   Fails with StateMismatch if they differ
   → spec-indexer, spec-sparse-merkle-tree

4. COLLECT SIGNATURES
   L2 broadcasts checkpoint message to validators
   Each validator signs: sha256(checkpoint_msg + genesis_challenge + coin_id)
   L2 collects until 2k > validator_count
   → spec-wire-format — Individual Signatures

5. PROVE (5-15 minutes, off-chain)
   Generate Merkle membership proof for each signer
   Aggregate pubkeys (G1 sum) and signatures (G2 sum)
   Run Groth16 prover → 192-byte proof
   → spec-groth16-circuit, spec-sparse-merkle-tree

6. CHECKPOINT
   Submit spend bundle to Chia node
   On-chain: bls_pairing_identity verifies Groth16 proof
   On-chain: bls_verify verifies aggregate signature
   Epoch increments, new state committed
   → spec-checkpoint-singleton — Spend Path 1

7. EXIT
   Next checkpoint excludes the exiting validator's pubkey
   Validator spends checkpoint singleton (membership query) to get announcement
   Validator spends registration coin asserting that announcement
   Collateral returned
   → spec-checkpoint-singleton — Spend Path 2, spec-registration-coin

8. MONITOR
   Watch epoch counter — stalled epoch means checkpoints are stuck
   Watch for StateMismatch on sync — indicates indexer inconsistency
   → spec-l2-integration — Monitoring
```

---

# Part 2 — Reference Tables

---

## Table 1 — Constants

| Constant | Value | Description | Source |
|---------|-------|-------------|--------|
| `TREE_DEPTH` | 32 | Depth of the sparse Merkle tree. Fixed at circuit compile time. Supports 2^32 ≈ 4 billion validator slots. Cannot change without a new trusted setup and redeployment. | [smt](spec-sparse-merkle-tree.md) — Parameters |
| `EMPTY_LEAF` | `sha256(0x00 * 48)` = `0x7d4e3eec...` | Leaf value for empty Merkle tree slots. Curried into checkpoint singleton as `EMPTY_LEAF_HASH`. | [smt](spec-sparse-merkle-tree.md) — Leaf Values |
| `MAX_SIGNERS` | Set at deployment | Max simultaneous signers the circuit supports. Fixed at trusted setup time. Cannot increase without a new ceremony. | [circuit](spec-groth16-circuit.md) — Circuit Parameters |
| `PUBKEY_SIZE` | 48 bytes | BLS12-381 G1 point, compressed, ZCash format. | [wire](spec-wire-format.md) — G1 Points |
| `SIGNATURE_SIZE` | 96 bytes | BLS12-381 G2 point, compressed. | [wire](spec-wire-format.md) — G2 Points |
| `PROOF_SIZE` | 192 bytes | Groth16 proof: A(48) + B(96) + C(48). | [wire](spec-wire-format.md) — Groth16 Proof Format |
| `VK_SIZE` | 672 bytes | Verification key: alpha_g1(48) + beta_g2(96) + gamma_g2(96) + delta_g2(96) + 7×ic(48). | [wire](spec-wire-format.md) — Verification Key Format |
| `IC_COUNT` | 7 | One IC point per public input (6) plus the constant term (1). | [wire](spec-wire-format.md) — IC Point Order |
| `BLS12-381 scalar field order r` | `0x73eda753...00000001` | All scalars reduced mod r. | [wire](spec-wire-format.md) — Public Input Encoding |
| Block cost limit | 11,000,000,000 | Maximum CLVM cost per block. | [costs](spec-clvm-costs.md) — Overview |

---

## Table 2 — Byte Formats: Points and Proof

| Field | Type | Size | Encoding | Notes | Source |
|-------|------|------|----------|-------|--------|
| G1 point | PublicKey atom | 48 bytes | ZCash compressed BLS12-381: bit7=compress flag, bit6=infinity, bit5=sign, remaining=x coord big-endian | Arkworks `G1Affine::serialize_compressed()` | [wire](spec-wire-format.md) — G1 Points |
| G2 point | Signature atom | 96 bytes | Same flags as G1 but over Fp2: first 48 = x1, second 48 = x0 | Arkworks `G2Affine::serialize_compressed()` | [wire](spec-wire-format.md) — G2 Points |
| `proof.a` | G1 | 48 bytes | Compressed | First Groth16 proof element | [wire](spec-wire-format.md) — Groth16 Proof Format |
| `proof.b` | G2 | 96 bytes | Compressed | Second Groth16 proof element | [wire](spec-wire-format.md) — Groth16 Proof Format |
| `proof.c` | G1 | 48 bytes | Compressed | Third Groth16 proof element | [wire](spec-wire-format.md) — Groth16 Proof Format |
| `agg_signers` | G1 | 48 bytes | Compressed | G1 sum of k signing pubkeys | [wire](spec-wire-format.md) — Aggregate Public Key |
| `agg_sig` | G2 | 96 bytes | Compressed | G2 sum of k individual signatures | [wire](spec-wire-format.md) — Aggregate Signature |

---

## Table 3 — Byte Formats: Message Hashes

All message hashes are SHA-256 (FIPS 180-4). All integers are big-endian fixed-width.
All string literals are UTF-8, no null terminator, no length prefix.

| Message | Input (in concatenation order) | Output | Source |
|---------|-------------------------------|--------|--------|
| **checkpoint_message** | `new_state_root` (32) + `new_validator_merkle_root` (32) + `new_validator_count` (8, u64 be) + `new_epoch` (8, u64 be) = **80 bytes total** | 32 bytes | [wire](spec-wire-format.md) — Checkpoint Message |
| **validator signing message** | `checkpoint_message` (32) + `genesis_challenge` (32) + `checkpoint_singleton_coin_id` (32) = **96 bytes total** | Vec\<u8\> (signed directly, not hashed again) | [wire](spec-wire-format.md) — Individual Signatures |
| **registration_message** | `"register"` (8) + `pubkey` (48) = **56 bytes total** | 32 bytes | [wire](spec-wire-format.md) — Registration Message Format |
| **agg_sig_me for registration** | `registration_message` (32) + `genesis_challenge` (32) + `network_coin_coin_id` (32) = **96 bytes total** | Vec\<u8\> | [wire](spec-wire-format.md) — Registration Message Format |
| **membership announcement inner** | `"membership"` (10) + `epoch` (8, u64 be) + `pubkey` (48) + `is_member` (1: 0x01=member, 0x00=not) = **67 bytes total** | 32 bytes | [wire](spec-wire-format.md) — Membership Announcement Format |
| **membership announcement (full)** | `checkpoint_singleton_coin_id` (32) + `announcement_inner` (32) = **64 bytes total** | 32 bytes — used in `AssertCoinAnnouncement` | [wire](spec-wire-format.md) — Membership Announcement Format |
| **checkpoint state announcement** | `"checkpoint"` (10) + `new_epoch` (8) + `new_state_root` (32) + `new_validator_merkle_root` (32) + `new_validator_count` (8) = **90 bytes total** | 32 bytes | [wire](spec-wire-format.md) — Checkpoint State Announcement Format |
| **scalar(bytes)** | Any bytes | SHA-256(bytes) as big-endian u256, mod r | Fr field element | [wire](spec-wire-format.md) — The scalar() Function |

---

## Table 4 — Byte Formats: Verification Key IC Points

The IC point order in the VK is fixed at trusted setup time and cannot change.
It must match the public input allocation order in the Arkworks circuit.

| IC Index | Public Input | Encoding passed to `scalar()` | Source |
|----------|-------------|------------------------------|--------|
| `ic[0]` | Constant term | N/A — used as-is, no scalar multiplication | [wire](spec-wire-format.md) — IC Point Order |
| `ic[1]` | `validator_merkle_root` | 32 bytes raw | [wire](spec-wire-format.md) — Public Input Encoding Per Field |
| `ic[2]` | `validator_count` | 8 bytes, big-endian u64 | [wire](spec-wire-format.md) — Public Input Encoding Per Field |
| `ic[3]` | `new_validator_merkle_root` | 32 bytes raw | [wire](spec-wire-format.md) — Public Input Encoding Per Field |
| `ic[4]` | `new_validator_count` | 8 bytes, big-endian u64 | [wire](spec-wire-format.md) — Public Input Encoding Per Field |
| `ic[5]` | `agg_signers` | 48 bytes, G1 compressed | [wire](spec-wire-format.md) — Public Input Encoding Per Field |
| `ic[6]` | `checkpoint_message` | 32 bytes raw | [wire](spec-wire-format.md) — Public Input Encoding Per Field |

`vk_input = ic[0] + scalar(inputs[0])*ic[1] + ... + scalar(inputs[5])*ic[6]`
→ see [wire](spec-wire-format.md) — VK Input Computation

---

## Table 5 — Sparse Merkle Tree

| Item | Definition | Source |
|------|-----------|--------|
| **Slot assignment** | `slot = first_8_bytes_as_u64_be(sha256(pubkey)) mod 2^TREE_DEPTH` | [smt](spec-sparse-merkle-tree.md) — Slot Assignment |
| **Active leaf** | `sha256(pubkey)` — 48-byte compressed G1 input to sha256 | [smt](spec-sparse-merkle-tree.md) — Leaf Values |
| **Empty leaf** | `sha256(0x00 * 48)` = `EMPTY_LEAF` constant | [smt](spec-sparse-merkle-tree.md) — Leaf Values |
| **Empty node at level i** | `sha256(empty_node[i-1] + empty_node[i-1])` | [smt](spec-sparse-merkle-tree.md) — Empty Node Hashes |
| **Parent node** | `sha256(left_child + right_child)` — **left always first** | [smt](spec-sparse-merkle-tree.md) — Tree Structure |
| **Left child test** | Node at index I is left child if `I % 2 == 0` | [smt](spec-sparse-merkle-tree.md) — Sibling Ordering |
| **Proof format** | `{ leaf_index: u64, siblings: Vec<[u8;32]> }` — siblings length == TREE_DEPTH | [smt](spec-sparse-merkle-tree.md) — Proof Format |
| **Membership verify** | Path from `sha256(pubkey)` at `leaf_index` must reach `validator_merkle_root` | [smt](spec-sparse-merkle-tree.md) — Proof Verification |
| **Non-membership verify** | Path from `EMPTY_LEAF` at `leaf_index` must reach `validator_merkle_root` | [smt](spec-sparse-merkle-tree.md) — Non-Membership Proof |
| **Proof verification cost** | Exactly 32 SHA-256 operations at TREE_DEPTH=32 | [costs](spec-clvm-costs.md) — Spend Path 3 |
| **Root computation complexity** | O(n × depth) where n = active validators; empty subtrees short-circuit | [smt](spec-sparse-merkle-tree.md) — Root Computation |
| **Cross-impl requirement** | Rust root must equal Rue root for same validator set (CI test required) | [security](spec-security.md) — Assumption 5 |

---

## Table 6 — Groth16 Circuit

| Item | Value / Description | Source |
|------|---------------------|--------|
| **Library** | Arkworks, targeting BLS12-381 | [circuit](spec-groth16-circuit.md) — Dependencies |
| **Parameters fixed at setup** | `MAX_SIGNERS`, `TREE_DEPTH` | [circuit](spec-groth16-circuit.md) — Circuit Parameters |
| **Constraint 1** | Merkle membership: each of k pubkeys has a valid inclusion proof against `validator_merkle_root` | [circuit](spec-groth16-circuit.md) — Constraint 1 |
| **Constraint 2** | Aggregate consistency: G1 sum of k pubkeys == `agg_signers` | [circuit](spec-groth16-circuit.md) — Constraint 2 |
| **Constraint 3** | Majority threshold: `2k > validator_count` | [circuit](spec-groth16-circuit.md) — Constraint 3 |
| **validator_count is runtime** | Comes from checkpoint singleton state at proof time — circuit does not need to be redeployed when validators join/leave | [CHIP](chip-groth16-l2-consensus.md) — Why count is a runtime input |
| **SHA-256 per pubkey→leaf** | ~25,000 constraints | [circuit](spec-groth16-circuit.md) — Constraint Count Estimates |
| **Merkle path (depth 32)** | ~800,000 constraints | [circuit](spec-groth16-circuit.md) — Constraint Count Estimates |
| **G1 decompression** | ~50,000 constraints | [circuit](spec-groth16-circuit.md) — Constraint Count Estimates |
| **G1 addition** | ~10,000 constraints | [circuit](spec-groth16-circuit.md) — Constraint Count Estimates |
| **Total (MAX_SIGNERS=10, depth=32)** | ~8,850,000 constraints | [circuit](spec-groth16-circuit.md) — Constraint Count Estimates |
| **Proof generation time** | 5-15 minutes on a modern server (BLS12-381) | [circuit](spec-groth16-circuit.md) — Constraint Count Estimates |
| **Future: BLS12-377** | ~50,000 constraints, < 10 seconds — requires new CLVM opcode | [circuit](spec-groth16-circuit.md) — Constraint Count Estimates |
| **Blank circuit for setup** | All zeros but same constraint count as a real MAX_SIGNERS circuit | [circuit](spec-groth16-circuit.md) — Trusted Setup |
| **Proof randomization** | Two calls with identical inputs produce different proofs that both verify | [circuit](spec-groth16-circuit.md) — Important Notes |

---

## Table 7 — Network Coin Puzzle

| Item | Value / Description | Source |
|------|---------------------|--------|
| **Curried in** | `REGISTRATION_COIN_MOD_HASH` (Bytes32), `COLLATERAL_AMOUNT` (u64), `CHECKPOINT_SINGLETON_ID` (Bytes32) | [network-coin](spec-network-coin.md) — Curried In Parameters |
| **Solution** | `new_validator_pubkey` (PublicKey), `conditions` (List) | [network-coin](spec-network-coin.md) — Solution Parameters |
| **AggSigMe message** | `sha256("register" + pubkey)` then `+ genesis_challenge + network_coin_coin_id` | [wire](spec-wire-format.md) — Registration Message Format |
| **Creates** | Registration coin with `curry_hash(REGISTRATION_COIN_MOD_HASH, pubkey, CHECKPOINT_SINGLETON_ID)` | [network-coin](spec-network-coin.md) — What the Puzzle Does |
| **Recreates** | Itself at `MY_PUZZLE_HASH` with `MY_AMOUNT` (1 mojo) | [network-coin](spec-network-coin.md) — What the Puzzle Does |
| **Memo convention** | Driver must include pubkey (48 bytes) as first memo on CreateCoin — not enforced on-chain but required by indexer | [network-coin](spec-network-coin.md) — Important Notes: Memo convention |
| **Lineage check** | Enforced off-chain by indexer — parent coin ID must be a network coin spend | [indexer](spec-indexer.md) — Registration Coin Detection |
| **CLVM cost** | ~5,300,000 units ≈ 0.048% of block limit ≈ 0.0000053 XCH fee | [costs](spec-clvm-costs.md) — Spend Path 1 |

---

## Table 8 — Registration Coin Puzzle

| Item | Value / Description | Source |
|------|---------------------|--------|
| **Curried in** | `VALIDATOR_PUBKEY` (PublicKey, 48 bytes), `CHECKPOINT_SINGLETON_ID` (Bytes32 — coin ID, not launcher ID) | [reg-coin](spec-registration-coin.md) — Curried In Parameters |
| **Solution** | `epoch` (u64 be), `collateral_destination` (Bytes32), `collateral_amount` (u64), `conditions` (List) | [reg-coin](spec-registration-coin.md) — Solution Parameters |
| **Asserts** | `AssertCoinAnnouncement(sha256(CHECKPOINT_SINGLETON_ID + sha256("membership" + epoch_be8 + VALIDATOR_PUBKEY + 0x00)))` | [reg-coin](spec-registration-coin.md) — What the Puzzle Does |
| **Creates** | Coin at `collateral_destination` with `collateral_amount` | [reg-coin](spec-registration-coin.md) — What the Puzzle Does |
| **Epoch** | Must match the epoch in the membership announcement exactly — replay protection | [reg-coin](spec-registration-coin.md) — Important Notes: Epoch Matching |
| **Coin ID is deterministic** | From: `network_coin_spend_id`, `validator_pubkey`, `checkpoint_singleton_id`, `registration_coin_mod_hash`, `collateral_amount` | [reg-coin](spec-registration-coin.md) — Computing the Registration Coin Puzzle Hash |
| **Spent coin = exited validator** | Indexer removes from active set when the coin is spent | [indexer](spec-indexer.md) — Process Removals |
| **CLVM cost** | ~3,300,000 units ≈ 0.030% of block limit | [costs](spec-clvm-costs.md) — Spend Path 4 |
| **Combined recovery bundle** | ~7,400,000 units (membership query + reg coin spend) | [costs](spec-clvm-costs.md) — Combined Collateral Recovery Bundle |

---

## Table 9 — Checkpoint Singleton Puzzle

### Curried In Parameters

| Parameter | Type | Notes | Source |
|-----------|------|-------|--------|
| `VK` | VerificationKey (672 bytes) | Produced by trusted setup. Never changes. | [checkpoint](spec-checkpoint-singleton.md) — Curried In Parameters |
| `TREE_DEPTH` | Int | Must match SMT and circuit TREE_DEPTH exactly | [checkpoint](spec-checkpoint-singleton.md) — Curried In Parameters |
| `EMPTY_LEAF_HASH` | Bytes32 = `sha256(0x00 * 48)` | Precomputed, curried in at deployment | [checkpoint](spec-checkpoint-singleton.md) — Curried In Parameters |

### State (curried in on each recreation)

| Field | Type | Notes | Source |
|-------|------|-------|--------|
| `state_root` | Bytes32 | Current L2 state rollup hash | [checkpoint](spec-checkpoint-singleton.md) — Singleton State |
| `epoch` | u64 | Increments by exactly 1 on every checkpoint spend | [checkpoint](spec-checkpoint-singleton.md) — Singleton State |
| `validator_merkle_root` | Bytes32 | Sparse Merkle root of active validator set | [checkpoint](spec-checkpoint-singleton.md) — Singleton State |
| `validator_count` | u64 | Number of active validators | [checkpoint](spec-checkpoint-singleton.md) — Singleton State |

### Spend Path 1 — Checkpoint

| Step | What happens | Source |
|------|-------------|--------|
| 1 | `new_epoch = epoch + 1` | [checkpoint](spec-checkpoint-singleton.md) — What the Puzzle Does |
| 2 | `checkpoint_message = sha256(new_state_root + new_validator_merkle_root + new_validator_count_be8 + new_epoch_be8)` | [wire](spec-wire-format.md) — Checkpoint Message |
| 3 | `vk_input = ic[0] + scalar(inputs[0])*ic[1] + ... + scalar(inputs[5])*ic[6]` | [wire](spec-wire-format.md) — VK Input Computation |
| 4 | `bls_pairing_identity(proof.a, VK.beta_g2, -VK.alpha_g1, proof.b, -vk_input, VK.gamma_g2, -proof.c, VK.delta_g2)` | [checkpoint](spec-checkpoint-singleton.md) — Puzzle Source |
| 5 | `bls_verify(agg_sig, agg_signers, checkpoint_message)` | [checkpoint](spec-checkpoint-singleton.md) — Puzzle Source |
| 6 | Recreate singleton with new state; emit checkpoint state announcement | [wire](spec-wire-format.md) — Checkpoint State Announcement Format |
| **CLVM cost** | ~17,229,000 units ≈ 0.157% of block limit ≈ 0.0000172 XCH | [costs](spec-clvm-costs.md) — Spend Path 2 |
| **aggregated_signature** | G2 identity — no AGG_SIG_ME conditions | [checkpoint](spec-checkpoint-singleton.md) — Important Notes |

### Spend Path 2 — Membership Query

| Step | What happens | Source |
|------|-------------|--------|
| 1 | Compute leaf: `sha256(query_pubkey)` if member, else `EMPTY_LEAF_HASH` | [checkpoint](spec-checkpoint-singleton.md) — What the Puzzle Does |
| 2 | Verify Merkle path from leaf to `validator_merkle_root` (32 SHA-256 ops) | [smt](spec-sparse-merkle-tree.md) — On-Chain Verification |
| 3 | Emit announcement: `sha256("membership" + epoch_be8 + query_pubkey + is_member)` | [wire](spec-wire-format.md) — Membership Announcement Format |
| 4 | Recreate singleton **unchanged** — state does not change | [checkpoint](spec-checkpoint-singleton.md) — Spend Path 2 |
| **Permissionless** | No signature required. Anyone can call this. | [checkpoint](spec-checkpoint-singleton.md) — Important Notes |
| **No timing pressure** | Singleton persists same state until next checkpoint | [CHIP](chip-groth16-l2-consensus.md) — Rationale |
| **CLVM cost** | ~4,103,000 units ≈ 0.037% of block limit | [costs](spec-clvm-costs.md) — Spend Path 3 |

---

## Table 10 — Indexer

| Item | Description | Source |
|------|-------------|--------|
| **Tracks** | Network coin state, checkpoint singleton state, all valid registration coins (keyed by pubkey), checkpoint history | [indexer](spec-indexer.md) — What the Indexer Tracks |
| **Lineage verification** | Parent coin ID must be in `network_coin_spend_ids` set, pubkey must be in memo, puzzle hash must match computed hash, amount must equal collateral | [indexer](spec-indexer.md) — Registration Coin Detection |
| **Memo extraction** | First 48-byte memo on the `CreateCoin` condition matching the child coin — set by network coin driver | [indexer](spec-indexer.md) — Registration Coin Detection |
| **Merkle consistency check** | After every sync, rebuild tree from registration coins; root must equal `checkpoint.validator_merkle_root`. Returns `StateMismatch` if not. | [indexer](spec-indexer.md) — Merkle Root Consistency Check |
| **Reorg handling** | Roll back to last checkpoint confirmed before new peak; re-index forward. Full reindex from genesis if no safe checkpoint found. | [indexer](spec-indexer.md) — Reorg Handling |
| **Persistent cache** | JSON file. Atomic write via temp file. Loaded on restart to avoid full re-index. | [indexer](spec-indexer.md) — Persistent Cache |
| **Validator set sort** | Sorted by pubkey bytes for deterministic Merkle tree slot ordering | [indexer](spec-indexer.md) — Validator Set Construction |
| **Spent reg coin** | Removed from active set — validator exited and recovered collateral | [indexer](spec-indexer.md) — Process Removals |
| **Spent = checkpoint creation** | When a new checkpoint singleton coin appears, decode state from parent spend's solution fields | [indexer](spec-indexer.md) — Checkpoint State Updates |

---

## Table 11 — ConsensusClient Public API

| Method | What it does | Key details | Source |
|--------|-------------|-------------|--------|
| `new(node, config)` | Create client. Does not sync. | Call `sync()` before anything else. | [crate](spec-consensus-crate.md) — ConsensusClient |
| `set_cache_path(path)` | Set indexer cache file path | Enables fast restarts | [crate](spec-consensus-crate.md) — ConsensusClient |
| `load_proving_key(path)` | Load Groth16 proving key (100-500MB) | Only needed on nodes that submit checkpoints | [crate](spec-consensus-crate.md) — ConsensusClient |
| `sync()` | Update all local state from chain | Drives indexer, rebuilds SMT, verifies Merkle consistency, returns `StateMismatch` if inconsistent | [crate](spec-consensus-crate.md) — ConsensusClient |
| `deploy(...)` | One-time network deployment | Returns `(SpendBundle, NetworkConfig)`. Runs setup if VK not found. | [crate](spec-consensus-crate.md) — Deployment |
| `register_validator(sk)` | Build registration spend bundle | Requires validator's secret key. Includes pubkey memo. | [crate](spec-consensus-crate.md) — Validator Registration |
| `build_checkpoint(...)` | Build checkpoint spend bundle | Runs proof generation (5-15 min) in `spawn_blocking`. Returns SpendBundle — caller broadcasts. | [crate](spec-consensus-crate.md) — Checkpoint Submission |
| `checkpoint_message(...)` | Compute the 32-byte message to commit to | Input to `sha256`, not the signing message itself | [crate](spec-consensus-crate.md) — Checkpoint Submission |
| `validator_signing_message(...)` | Compute the full message each validator signs | = checkpoint_message + genesis_challenge + coin_id | [crate](spec-consensus-crate.md) — Checkpoint Submission |
| `compute_new_validator_set(entries, exits)` | Apply entries and exits to current SMT | Returns `(new_root, new_count, new_tree)`. Detects slot collisions. | [crate](spec-consensus-crate.md) — Validator Set Construction |
| `recover_collateral(pubkey, dest)` | Build collateral recovery bundle | Membership query spend + registration coin spend. No signatures. | [crate](spec-consensus-crate.md) — Collateral Recovery |
| `query_membership_on_chain(pubkey, is_member)` | Build membership query spend bundle | Permissionless. Emits announcement other coins can assert. | [crate](spec-consensus-crate.md) — Membership Queries |
| `membership_announcement(pubkey, is_member)` | Compute announcement hash for `AssertCoinAnnouncement` | Uses current checkpoint coin ID (not launcher ID) | [crate](spec-consensus-crate.md) — Membership Queries |
| `is_active(pubkey)` | Fast local membership check | No RPC. Call `sync()` first. | [crate](spec-consensus-crate.md) — Membership Queries |
| `epoch()` | Current epoch from checkpoint singleton | Primary health signal — stalled epoch = checkpoints stuck | [crate](spec-consensus-crate.md) — State Accessors |
| `state_root()` | Current L2 state root | | [crate](spec-consensus-crate.md) — State Accessors |
| `validator_merkle_root()` | Current on-chain Merkle root | Off-chain tree root verified equal to this on sync | [crate](spec-consensus-crate.md) — State Accessors |
| `validator_count()` | On-chain count from checkpoint singleton | May differ from local registration coin count | [crate](spec-consensus-crate.md) — State Accessors |
| `synced_at()` | Block height of last sync | Use to decide if re-sync is needed | [crate](spec-consensus-crate.md) — State Accessors |

---

## Table 12 — NetworkConfig Fields

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `network_coin_launcher_id` | Bytes32 | Launcher ID of the network coin singleton | [crate](spec-consensus-crate.md) — Configuration |
| `checkpoint_launcher_id` | Bytes32 | Launcher ID of the checkpoint singleton | [crate](spec-consensus-crate.md) — Configuration |
| `registration_coin_mod_hash` | Bytes32 | Tree hash of base registration coin puzzle before currying | [crate](spec-consensus-crate.md) — Configuration |
| `checkpoint_inner_mod_hash` | Bytes32 | Tree hash of base checkpoint inner puzzle before currying | [crate](spec-consensus-crate.md) — Configuration |
| `collateral_amount` | u64 | Exact collateral per validator in mojos. Cannot change without redeployment. | [crate](spec-consensus-crate.md) — Configuration |
| `tree_depth` | u32 | Must match TREE_DEPTH in circuit and checkpoint singleton | [crate](spec-consensus-crate.md) — Configuration |
| `max_signers` | usize | Must match MAX_SIGNERS in circuit. Cannot increase without new ceremony. | [crate](spec-consensus-crate.md) — Configuration |
| `verification_key_hex` | String | Groth16 VK as hex-encoded JSON. Curried into checkpoint singleton. | [crate](spec-consensus-crate.md) — Configuration |
| `genesis_challenge` | Bytes32 | Chia network genesis challenge for AGG_SIG_ME construction | [crate](spec-consensus-crate.md) — Configuration |

---

## Table 13 — ConsensusError Variants

| Variant | When it occurs | Action |
|---------|---------------|--------|
| `NotDeployed` | `state()` called before `sync()` | Call `sync()` first |
| `AlreadyRegistered` | Pubkey already in active set, or validator still active during collateral recovery | Check `is_active()` before registering or recovering |
| `ValidatorNotFound` | Signing pubkey not in current tree, or no registration coin found | Verify validator is registered and sync is current |
| `BelowThreshold` | `2k ≤ validator_count` | Collect more signatures before calling `build_checkpoint()` |
| `StateMismatch` | Local Merkle root ≠ on-chain `validator_merkle_root` after sync | Delete cache, trigger full re-index |
| `InvalidLineage` | Registration coin parent not a network coin spend | Do not use this coin — it was not created through the approved flow |
| `InvalidMerkleProof` | Merkle proof verification failed or slot not empty for non-membership | Rebuild tree from current registration coins |
| `ProvingError` | Proof generation failed — OOM, key not loaded, or circuit bug | Load proving key; if persists, circuit implementation bug |
| `SerializationError` | Arkworks serialization mismatch | Check point sizes match spec-wire-format constants |
| `SpendRejected` | Node rejected the spend bundle | Re-sync and check epoch; another checkpoint may have been submitted |
| `PuzzleHashMismatch` | Detected coin with wrong puzzle hash during lineage check | Coin was not created correctly — skip it |
| `SlotCollision` | Two validators hash to the same Merkle tree slot | Reject the second registration at the L2 level |

→ see [crate](spec-consensus-crate.md) — Error Type

---

## Table 14 — CLVM Cost Summary

| Spend Path | Cost (units) | % of 11B limit | Fee estimate | Source |
|-----------|--------------|----------------|-------------|--------|
| Network coin registration | ~5,300,000 | 0.048% | ~0.0000053 XCH | [costs](spec-clvm-costs.md) — Spend Path 1 |
| Checkpoint (full) | ~17,229,000 | 0.157% | ~0.0000172 XCH | [costs](spec-clvm-costs.md) — Spend Path 2 |
| Membership query (standalone) | ~4,103,000 | 0.037% | ~0.0000041 XCH | [costs](spec-clvm-costs.md) — Spend Path 3 |
| Registration coin (collateral) | ~3,300,000 | 0.030% | ~0.0000033 XCH | [costs](spec-clvm-costs.md) — Spend Path 4 |
| **Collateral recovery bundle** | **~7,403,000** | **0.067%** | **~0.0000074 XCH** | [costs](spec-clvm-costs.md) — Combined Bundle |

**Safety margin**: The checkpoint spend (most expensive) is ~640× below the block limit.

### Individual CLVM operator costs for reference

| Operator | Cost |
|----------|------|
| SHA-256 (per 64 bytes) | 87 |
| `bls_pairing_identity` base | 3,000,000 |
| `bls_pairing_identity` per pair | 1,200,000 |
| `bls_verify` base | 3,000,000 |
| `bls_verify` per pair | 1,200,000 |
| `g1_add` per point | 101,094 |
| `g1_multiply` base | 706 |
| `g1_multiply` per scalar byte | 92 |
| `CREATE_COIN` | 1,800,000 |
| `CREATE_COIN_ANNOUNCEMENT` | 1,800,000 |
| `ASSERT_COIN_ANNOUNCEMENT` | 1,200,000 |
| `AGG_SIG_ME` | 1,200,000 |

→ see [costs](spec-clvm-costs.md) — Cost Reference

---

## Table 15 — Security Assumptions and Failure Modes

### Assumptions (all must hold for the system to be secure)

| # | Assumption | What breaks if it fails | Recovery | Source |
|---|-----------|------------------------|----------|--------|
| 1 | Majority of validators are honest | Fraudulent checkpoints can be submitted | Remove compromised validators via majority exit | [security](spec-security.md) — Assumption 1 |
| 2 | Trusted setup is sound (at least one MPC participant honest) | Attacker can generate valid-looking proofs for false statements | New MPC ceremony + redeploy checkpoint singleton | [security](spec-security.md) — Assumption 2 |
| 3 | Validator set off-chain verification is correct | Fraudulent Merkle root could pass — but requires corrupting majority (Assumption 1) | Covered by Assumption 1 | [security](spec-security.md) — Assumption 3 |
| 4 | Groth16 circuit is correctly implemented | False proofs might verify | Audit circuit; CI cross-verification test | [security](spec-security.md) — Assumption 4 |
| 5 | Rust and Rue SMT implementations are byte-identical | Valid proofs fail on-chain silently | Cross-impl CI test; canonical spec is [smt](spec-sparse-merkle-tree.md) | [security](spec-security.md) — Assumption 5 |

### Security Properties Enforced

| Property | Where enforced | Source |
|----------|---------------|--------|
| Registration lineage | Off-chain by indexer — parent must be network coin spend | [security](spec-security.md) — Lineage Proof Enforcement |
| Correct circuit commitment | VK curried into checkpoint singleton puzzle hash | [security](spec-security.md) — Proving the Correct Circuit Was Used |
| Merkle root correctness | Majority BLS signature over checkpoint message | [security](spec-security.md) — Validator Merkle Root Correctness |
| Collateral lock | Registration coin can only be spent with membership announcement | [security](spec-security.md) — Collateral Security |
| Two-check completeness | ZK proof + BLS verify are both required | [security](spec-security.md) — Completeness of the Two-Check Design |
| Epoch replay protection | Membership announcement includes epoch number | [security](spec-security.md) — Epoch Replay Protection |

### Failure Modes and Recovery

| Failure | Cause | Recovery | Source |
|---------|-------|----------|--------|
| Checkpoint singleton is stuck | Not enough signers, proving key unavailable, submitter bug | Restore signers; fix key distribution; no on-chain override | [security](spec-security.md) — Checkpoint Singleton Is Stuck |
| Validator key compromise | Private key stolen | Force-exit compromised validator in next checkpoint | [security](spec-security.md) — Validator Key Compromise |
| Registration coin lost | Validator loses wallet access | Force-exit via checkpoint; collateral is permanently locked | [security](spec-security.md) — Registration Coin Lost |
| Trusted setup compromise | All MPC participants colluded | New ceremony + redeploy checkpoint singleton | [security](spec-security.md) — Trusted Setup Compromise |
| Chain reorg | Chia blockchain reorganizes | Indexer rolls back to last safe checkpoint and re-syncs | [security](spec-security.md) — Chain Reorganization |
| CLVM cost limit exceeded | Future Chia update lowering block limit | 640× safety margin makes this unlikely; monitor Chia releases | [security](spec-security.md) — CLVM Cost Limit Exceeded |

---

## Table 16 — Deployment Steps

| Step | What happens | Verify before proceeding | Source |
|------|-------------|--------------------------|--------|
| 1 | Run trusted setup (MPC for production, single-party for dev only) | `sha256sum verification_key.bin`; VK has 7 IC points; test proof verifies | [deploy](spec-deployment-runbook.md) — Step 1 |
| 2 | Choose genesis coin (≥ 2 XCH) | Coin is unspent | [deploy](spec-deployment-runbook.md) — Step 2 |
| 3 | Derive deployment parameters from genesis coin ID | TREE_DEPTH in config matches circuit and SMT | [deploy](spec-deployment-runbook.md) — Step 3 |
| 4 | Submit deploy bundle (network coin + checkpoint singleton) | Transaction submitted | [deploy](spec-deployment-runbook.md) — Step 4 |
| 5 | Verify on-chain presence | Both coins found; epoch=0; validator_count=0; merkle_root=empty root | [deploy](spec-deployment-runbook.md) — Step 5 |
| 6 | Publish deployment artifacts | network_config.json, VK hex, VK hash (in multiple places), transcript, circuit source | [deploy](spec-deployment-runbook.md) — Step 6 |
| 7 | Verify VK is correctly curried | Decurry checkpoint singleton; VK matches local file; 7 IC points OK | [deploy](spec-deployment-runbook.md) — Step 7 |
| 8 | First sync | Indexer reports merkle_root=empty root, matches on-chain | [deploy](spec-deployment-runbook.md) — Step 8 |

---

## Table 17 — Validator Onboarding Steps

| Step | What happens | Source |
|------|-------------|--------|
| 1 | Install validator node software | [onboard](spec-validator-onboarding.md) — Step 1 |
| 2 | Generate BLS keypair; back up immediately | [onboard](spec-validator-onboarding.md) — Step 2 |
| 3 | Configure node with network_config.json | [onboard](spec-validator-onboarding.md) — Step 3 |
| 4 | Fund wallet with ≥ collateral_amount XCH | [onboard](spec-validator-onboarding.md) — Step 4 |
| 5 | Sync with chain | [onboard](spec-validator-onboarding.md) — Step 5 |
| 6 | Register: spend network coin, lock collateral | [onboard](spec-validator-onboarding.md) — Step 6 |
| 7 | Verify registration coin exists; status shows PENDING | [onboard](spec-validator-onboarding.md) — Step 7 |
| 8 | Wait for next checkpoint; status shows ACTIVE | [onboard](spec-validator-onboarding.md) — Step 8 |
| 9 | Sign checkpoint messages when requested by coordinator | [onboard](spec-validator-onboarding.md) — Step 9 |
| **Exit 1** | Signal intent to L2 coordinator | [onboard](spec-validator-onboarding.md) — Voluntary Exit |
| **Exit 2** | Wait for checkpoint that excludes your pubkey | [onboard](spec-validator-onboarding.md) — Voluntary Exit |
| **Exit 3** | Submit collateral recovery bundle (membership query + reg coin spend) | [onboard](spec-validator-onboarding.md) — Voluntary Exit |

---

## Table 18 — Common Implementation Mistakes

| Mistake | Effect | Correct behavior | Source |
|---------|--------|-----------------|--------|
| Wrong sibling ordering (right child first) | Membership proofs fail for some validators but not others | Left child always first in `sha256(left + right)` | [smt](spec-sparse-merkle-tree.md) — Common Implementation Mistakes |
| Wrong empty node level (off-by-one) | Wrong root for empty subtrees | `empty_nodes[0]` = EMPTY_LEAF; `empty_nodes[TREE_DEPTH]` = empty root | [smt](spec-sparse-merkle-tree.md) — Common Implementation Mistakes |
| Variable-length integer encoding | Message hash mismatch between Rust and Rue | All integers are fixed-width big-endian (u64 = 8 bytes, always) | [wire](spec-wire-format.md) — Common Mistakes |
| Using launcher ID instead of coin ID in announcement | AssertCoinAnnouncement fails silently | Announcement uses current checkpoint singleton coin ID, which changes each checkpoint | [wire](spec-wire-format.md) — Common Mistakes |
| Swapping G1 and G2 | `bls_verify` fails with no helpful error | `agg_signers` is G1 (48 bytes); `agg_sig` is G2 (96 bytes) | [wire](spec-wire-format.md) — Common Mistakes |
| TREE_DEPTH mismatch between circuit and singleton | On-chain proof verification fails | Set both from the same NetworkConfig value | [circuit](spec-groth16-circuit.md) — Important Notes |
| Forgetting pubkey memo on registration coin creation | Indexer cannot build validator set | Network coin driver must include pubkey as first memo on CreateCoin | [indexer](spec-indexer.md) — Important Notes |
| Missing scalar reduction mod r | Incorrect vk_input G1 point | `scalar(bytes) = sha256(bytes) as u256 mod r` — Rust must explicitly reduce; Rue relies on g1_multiply | [wire](spec-wire-format.md) — Common Mistakes |
| Submitting two checkpoints in-flight | Second is rejected (stale epoch) | Mutex or semaphore to allow only one checkpoint submission at a time | [l2](spec-l2-integration.md) — Important Notes |
| Calling build_checkpoint() without syncing first | Wrong public inputs to proof generation | Always call `sync()` immediately before starting checkpoint construction | [l2](spec-l2-integration.md) — Important Notes |
