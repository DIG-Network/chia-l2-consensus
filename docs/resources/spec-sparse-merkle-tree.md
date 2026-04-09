# Sparse Merkle Tree - Canonical Specification

## Document Relationships

**This document is foundational.** Nearly every other document in the system
depends on it. Any divergence between the Rust and Rue implementations of this
spec breaks the entire proof verification pipeline.

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Implemented by** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | The on-chain Rue puzzle uses `verify_merkle_path` defined here for both spend paths |
| **Implemented by** | [spec-groth16-circuit](spec-groth16-circuit.md) | The ZK circuit encodes Merkle membership as R1CS constraints using this exact structure |
| **Implemented by** | [spec-consensus-crate](spec-consensus-crate.md) | The `SparseMerkleTree` struct in `merkle/sparse.rs` must match this spec exactly |
| **Implemented by** | [spec-indexer](spec-indexer.md) | The indexer reconstructs the tree from registration coins and validates it against chain state |
| **Referenced by** | [spec-wire-format](spec-wire-format.md) | Wire format defines how proof siblings are serialized for transmission to CLVM |
| **Referenced by** | [spec-clvm-costs](spec-clvm-costs.md) | Proof verification cost is exactly 32 SHA-256 operations as defined here |
| **Referenced by** | [spec-security](spec-security.md) | Assumption 5 covers cross-implementation consistency requirements |
| **Referenced by** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | The CHIP defines the sparse Merkle tree as the canonical validator set commitment |
| **Constrained by** | [spec-groth16-circuit](spec-groth16-circuit.md) | `TREE_DEPTH` is fixed at circuit compile time and cannot change without a new trusted setup |
| **Constrained by** | [spec-trusted-setup](spec-trusted-setup.md) | A new trusted setup is required if `TREE_DEPTH` changes |

---

## Overview

This document is the canonical specification for the sparse Merkle tree used
by the chia-l2-consensus system. Both the on-chain Rue puzzle
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md)) and the
off-chain Rust implementation
(→ see [spec-consensus-crate](spec-consensus-crate.md)) must produce identical
results for all operations. Any divergence between the two will cause valid
membership proofs to fail on-chain silently or raise an exception in
`bls_pairing_identity`.

This spec defines exact byte encodings, hash constructions, slot assignment,
leaf values, empty node values, path ordering, and proof formats. When in
doubt, this document takes precedence over any code.

The `EMPTY_LEAF_HASH` constant defined here is curried directly into the
checkpoint singleton puzzle
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Curried In
Parameters). It must be precomputed and hardcoded before deployment.

---

## Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| `TREE_DEPTH` | 32 | Fixed at circuit compile time. Supports 2^32 ≈ 4 billion slots. Cannot change without a new trusted setup (→ see [spec-trusted-setup](spec-trusted-setup.md)). |
| `HASH_FUNCTION` | SHA-256 | All hashing uses SHA-256 with standard FIPS 180-4. Encoding rules in (→ see [spec-wire-format](spec-wire-format.md)). |
| `PUBKEY_SIZE` | 48 bytes | BLS12-381 G1 compressed point. Encoding defined in (→ see [spec-wire-format](spec-wire-format.md) — G1 Points). |
| `HASH_SIZE` | 32 bytes | SHA-256 output |

`TREE_DEPTH` is fixed at deployment and cannot change without redeploying the
checkpoint singleton and rerunning the trusted setup
(→ see [spec-trusted-setup](spec-trusted-setup.md) — When to Rerun the
Ceremony and [spec-deployment-runbook](spec-deployment-runbook.md) — Step 1).
Choose this value carefully before running the trusted setup ceremony.

---

## Slot Assignment

Each validator occupies a deterministic slot in the tree derived from their
pubkey. The slot computation must be identical in both Rust and Rue. The
network coin puzzle
(→ see [spec-network-coin](spec-network-coin.md)) does not enforce slot
uniqueness on-chain, so the L2 and indexer
(→ see [spec-indexer](spec-indexer.md) — Registration Coin Detection) must
detect slot collisions during registration.

The slot is computed as:

```
slot = first_8_bytes_as_u64_big_endian(sha256(pubkey)) mod 2^TREE_DEPTH
```

In Rust:
```rust
pub fn validator_slot(pubkey: &PublicKey) -> u64 {
    let hash = sha256(pubkey.to_bytes());
    let first_8 = &hash[0..8];
    u64::from_be_bytes(first_8.try_into().unwrap()) % (1u64 << TREE_DEPTH)
}
```

In Rue (on-chain, used in the checkpoint singleton membership query path):
```rust
fn validator_slot(pubkey: PublicKey, tree_depth: Int) -> Int {
    let hash = sha256(pubkey);
    // Take first 8 bytes as big-endian u64, mod 2^depth
    bytes_to_u64_be(substr(hash, 0, 8)) % pow(2, tree_depth)
}
```

**Slot collisions**: Two validators can theoretically hash to the same slot.
The probability is negligible for any realistic validator set size but the
system must handle it. If a collision occurs the second validator to register
cannot be included in the tree at that slot. Collision detection is the
responsibility of the L2 integration layer
(→ see [spec-l2-integration](spec-l2-integration.md) — Validator Set
Transitions).

---

## Leaf Values

```
active_leaf(pubkey)  = sha256(pubkey)
empty_leaf()         = sha256(0x00 * 48)
```

An active validator at slot S has leaf value `sha256(pubkey)`.
An empty slot S has leaf value `sha256(0x00 * 48)`.

The empty leaf is constant across all empty slots at all depths. It is
precomputed once and stored as a constant. This value is curried into the
checkpoint singleton puzzle as `EMPTY_LEAF_HASH`
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Curried In
Parameters):

```rust
pub const EMPTY_LEAF: [u8; 32] = sha256([0u8; 48]);
// = 0x7d4e3eec80026719639ed4dba68916eb94c7a49a053e05c8f9578fe4e5a3d7e
```

Note: the exact value depends on the SHA-256 of 48 zero bytes. Implementors
must verify this value matches before deployment
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 7).

The `active_leaf` formula is also used inside the Groth16 circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Constraint 1:
Merkle Membership) where SHA-256 is implemented as an R1CS gadget. The same
formula is used in the checkpoint singleton membership query spend path
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend Path
2: Membership Query).

---

## Empty Node Hashes

Empty nodes at higher levels are computed recursively from the empty leaf.
These are precomputed at startup and never change. They must be identical
between Rust and Rue:

```
empty_node[0] = empty_leaf()                               // leaf level
empty_node[1] = sha256(empty_node[0] + empty_node[0])     // one level up
empty_node[i] = sha256(empty_node[i-1] + empty_node[i-1]) // general case
```

Where `+` denotes byte concatenation.

```rust
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
// nodes[0] = empty leaf
// nodes[1] = empty node at depth-1
// nodes[TREE_DEPTH] = empty root (all slots empty)
```

The empty root (all slots empty) is the initial value of `validator_merkle_root`
in the checkpoint singleton when the network is first deployed
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 8 and
[spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Singleton State).

---

## Tree Structure

The tree has `TREE_DEPTH` levels plus the leaf level. Level 0 is the leaf
level. Level `TREE_DEPTH` is the root:

```
Level 32 (root):    [          root          ]
Level 31:           [    L    ][    R    ]
...
Level 1:            [...][...][...][...]
Level 0 (leaves):   [0][1][2][3][4][5]...[2^32 - 1]
```

The root is computed by combining pairs of nodes bottom-up:

```
parent = sha256(left_child + right_child)
```

**Critical invariant**: Left child always comes first in the SHA-256
concatenation. This ordering must be identical in Rust, Rue, and inside the
Groth16 circuit constraints
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Merkle Path
Verification Gadget). A sibling ordering mismatch between any two
implementations is the most common cause of proof verification failures.

---

## Root Computation

Given a set of active validators. This function is called by the indexer
after every sync to verify the local tree matches the on-chain root
(→ see [spec-indexer](spec-indexer.md) — Merkle Root Consistency Check):

```rust
pub fn compute_root(
    active_slots: &HashMap<u64, [u8; 32]>,  // slot -> leaf hash
    depth: u32,
    empty_nodes: &[[u8; 32]],
) -> [u8; 32] {
    compute_subtree(active_slots, 0, 1u64 << depth, depth, empty_nodes)
}

fn compute_subtree(
    active_slots: &HashMap<u64, [u8; 32]>,
    start: u64,
    end: u64,
    level: u32,
    empty_nodes: &[[u8; 32]],
) -> [u8; 32] {
    if level == 0 {
        return active_slots
            .get(&start)
            .copied()
            .unwrap_or(EMPTY_LEAF);
    }

    let has_active = active_slots.keys().any(|&k| k >= start && k < end);
    if !has_active {
        return empty_nodes[level as usize];
    }

    let mid = start + (end - start) / 2;
    let left  = compute_subtree(active_slots, start, mid, level - 1, empty_nodes);
    let right = compute_subtree(active_slots, mid, end, level - 1, empty_nodes);

    sha256(&[left, right].concat())
}
```

The optimization of returning precomputed empty nodes for empty subtrees is
critical for performance. Without it, computing the root over 2^32 slots is
infeasible. The `compute_new_validator_set()` method in the consensus crate
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Validator Set
Construction) calls this after applying pending entries and exits to produce
the `new_validator_merkle_root` for the next checkpoint.

---

## Membership Proof

A membership proof for slot S proves that the leaf at slot S contains
`sha256(pubkey)` and that this leaf is consistent with the root. Membership
proofs are used inside the Groth16 circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Constraint 1) and
as the private witness during checkpoint proof generation
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Checkpoint
Submission). The serialization of `siblings` for transmission to CLVM is
defined in
(→ see [spec-wire-format](spec-wire-format.md) — Membership Proof Siblings).

### Proof Format

```rust
pub struct MerkleProof {
    /// The slot index of the leaf being proven
    pub leaf_index: u64,
    /// Sibling hashes from leaf level up to (but not including) root level
    /// Length must equal TREE_DEPTH
    pub siblings: Vec<[u8; 32]>,
}
```

`siblings[0]` is the sibling of the leaf at level 0.
`siblings[TREE_DEPTH - 1]` is the sibling at the level just below the root.

### Sibling Ordering

At each level, the sibling is the node on the opposite side:
- If the current node is at an even index (left child), the sibling is at
  index + 1 (right sibling).
- If the current node is at an odd index (right child), the sibling is at
  index - 1 (left sibling).

### Proof Generation

Called by the indexer
(→ see [spec-indexer](spec-indexer.md) — Registration Coin Detection) and by
the consensus crate before proof generation
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Checkpoint
Submission):

```rust
pub fn prove(
    active_slots: &HashMap<u64, [u8; 32]>,
    slot: u64,
    depth: u32,
    empty_nodes: &[[u8; 32]],
) -> MerkleProof {
    let mut siblings = Vec::with_capacity(depth as usize);
    let mut index = slot;
    let mut start = 0u64;
    let mut end = 1u64 << depth;

    for level in 0..depth {
        let mid = start + (end - start) / 2;

        let sibling = if index < mid {
            compute_subtree(active_slots, mid, end, level, empty_nodes)
        } else {
            compute_subtree(active_slots, start, mid, level, empty_nodes)
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
```

### Proof Verification (Rust)

Used by the consensus crate to verify proofs before submitting them to the
prover, and by the indexer to validate internal consistency:

```rust
pub fn verify_membership(
    proof: &MerkleProof,
    pubkey: &PublicKey,
    root: [u8; 32],
    depth: u32,
) -> bool {
    let leaf = sha256(pubkey.to_bytes());
    verify_path(leaf, proof.leaf_index, &proof.siblings, root, depth)
}

pub fn verify_non_membership(
    proof: &MerkleProof,
    root: [u8; 32],
    depth: u32,
) -> bool {
    verify_path(EMPTY_LEAF, proof.leaf_index, &proof.siblings, root, depth)
}

fn verify_path(
    leaf: [u8; 32],
    index: u64,
    siblings: &[[u8; 32]],
    root: [u8; 32],
    depth: u32,
) -> bool {
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

### Proof Verification (Rue — On-Chain)

This function is embedded in the checkpoint singleton inner puzzle
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Puzzle
Source). It must produce identical results to the Rust implementation above.
The CLVM cost of this function is exactly 32 SHA-256 operations at depth 32
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Spend Path 3: Membership
Query):

```rust
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
        let parent = if index % 2 == 0 {
            sha256(node + sibling)     // left child: node first
        } else {
            sha256(sibling + node)     // right child: sibling first
        };
        verify_merkle_path(
            parent,
            index / 2,
            siblings,
            root,
            depth,
            level + 1,
        )
    }
}
```

---

## Non-Membership Proof

A non-membership proof for slot S proves that the leaf at slot S contains
`EMPTY_LEAF` and that this is consistent with the root. The proof format is
identical to a membership proof. The only difference is the expected leaf value
used during verification.

A validator uses a non-membership proof to recover their collateral. The proof
demonstrates their slot is empty in the current `validator_merkle_root`,
meaning they are no longer in the active set. The full collateral recovery
flow is:

1. Call `prove_non_membership(slot)` here to generate the proof
2. Pass `siblings` to the checkpoint singleton membership query spend
   (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
   Path 2)
3. The checkpoint singleton emits an announcement
4. The registration coin spend asserts that announcement
   (→ see [spec-registration-coin](spec-registration-coin.md) — What the
   Puzzle Does)
5. The full bundle is described in
   (→ see [spec-consensus-crate](spec-consensus-crate.md) — Collateral
   Recovery) and the validator perspective in
   (→ see [spec-validator-onboarding](spec-validator-onboarding.md) — Step 7)

---

## Tree Updates

### Adding a Validator

Called during `compute_new_validator_set()` in the consensus crate
(→ see [spec-consensus-crate](spec-consensus-crate.md)):

```rust
pub fn insert(&mut self, pubkey: &PublicKey) {
    let slot = validator_slot(pubkey);
    self.active_slots.insert(slot, sha256(pubkey.to_bytes()));
}
```

### Removing a Validator

Called when processing exits in `compute_new_validator_set()`
(→ see [spec-consensus-crate](spec-consensus-crate.md)):

```rust
pub fn remove(&mut self, pubkey: &PublicKey) {
    let slot = validator_slot(pubkey);
    self.active_slots.remove(&slot);
}
```

After any update the root changes. The new root becomes the
`new_validator_merkle_root` included in the checkpoint message
(→ see [spec-wire-format](spec-wire-format.md) — Checkpoint Message). A
majority of validators must sign this message, which is how the correctness
of the new root is trustlessly attested
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Why the
validator set lives off-chain).

---

## Test Vectors

These test vectors must pass for both the Rust and Rue implementations before
deployment. The cross-implementation consistency test is a CI requirement
described in
(→ see [spec-security](spec-security.md) — Assumption 5):

### Empty Tree Root (depth = 4 for brevity)

```
EMPTY_LEAF = sha256(0x00 * 48)

empty_node[0] = EMPTY_LEAF
empty_node[1] = sha256(EMPTY_LEAF + EMPTY_LEAF)
empty_node[2] = sha256(empty_node[1] + empty_node[1])
empty_node[3] = sha256(empty_node[2] + empty_node[2])
empty_node[4] = sha256(empty_node[3] + empty_node[3])

root of empty tree = empty_node[4]
```

This empty root is the initial `validator_merkle_root` committed to the
checkpoint singleton at deployment
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 2).

### Single Validator Tree (depth = 4)

```
pubkey = 0xb7f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb

slot = first_8_bytes_as_u64_be(sha256(pubkey)) mod 16

leaf = sha256(pubkey)

// All other slots are EMPTY_LEAF
// Compute root by combining pairs up the tree
// Left child always comes first in sha256
```

### Membership Proof Roundtrip

For any validator in the tree, generating a proof and then verifying it must
return true. For any pubkey not in the tree, the non-membership proof must
verify against EMPTY_LEAF. The `query_membership_on_chain()` method in the
consensus crate
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Membership Queries)
exercises this path end to end.

### Cross-Implementation Consistency

The root computed by the Rust implementation must equal the root computed by
the Rue implementation for the same set of active validators. Run this check
in CI by generating a root in Rust, feeding it to a CLVM runner with the
checkpoint singleton puzzle, and asserting equality. This requirement is
captured as a security assumption
(→ see [spec-security](spec-security.md) — Assumption 5).

---

## Performance Notes

For a tree of depth 32 with a small validator set (< 10,000 validators), the
root computation takes roughly O(n * depth) hash operations where n is the
number of active validators. This is because most subtrees are empty and return
precomputed empty nodes without recursing.

For a validator set of 1,000 validators at depth 32:
- Root computation: approximately 32,000 SHA-256 operations
- Proof generation: exactly 32 SHA-256 operations
- Proof verification: exactly 32 SHA-256 operations (on-chain and off-chain)

Proof verification cost on-chain is constant at 32 SHA-256 operations
regardless of validator set size. This is what keeps the checkpoint puzzle
cost bounded and is the basis for the cost estimate in
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Spend Path 3: Membership
Query). The CHIP discusses why constant cost is a core requirement
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Rationale:
Why not iterate over individual signatures?).

---

## Common Implementation Mistakes

**Wrong sibling ordering**: The most common mistake. Left child must always
come first. If you get membership proofs that verify correctly for some
validators but not others, this is almost certainly the cause. The invariant
is stated in
(→ see [spec-wire-format](spec-wire-format.md) — Common Mistakes).

**Wrong index arithmetic**: When traversing up the tree, `index / 2` gives
the parent index. Make sure integer division is used, not floating point.

**Wrong empty node level**: `empty_nodes[0]` is the empty leaf.
`empty_nodes[TREE_DEPTH]` is the empty root. Off-by-one errors here produce
a wrong root for all-empty subtrees.

**Pubkey byte format**: The pubkey passed to `sha256` must be the 48-byte
compressed BLS12-381 G1 point. Do not use uncompressed (96-byte) format.
The exact encoding is defined in
(→ see [spec-wire-format](spec-wire-format.md) — G1 Points).

**Slot mod overflow**: The slot computation takes the first 8 bytes of the
SHA-256 hash as a u64 and mods by `2^TREE_DEPTH`. For depth 32 this is
`mod 4294967296`. Use `u64` in Rust throughout. In CLVM integers are arbitrary
precision so no overflow risk.

**TREE_DEPTH mismatch**: The `TREE_DEPTH` curried into the checkpoint singleton
must match the depth used by the Groth16 circuit. They are set independently
and must be manually kept in sync. A mismatch will cause on-chain proof
verification to fail. This is enforced by the deployment runbook
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 3).
