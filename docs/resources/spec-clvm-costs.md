# CLVM Cost Analysis

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Depends on** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | Both spend paths are the subject of cost analysis here |
| **Depends on** | [spec-network-coin](spec-network-coin.md) | Registration spend cost analyzed here |
| **Depends on** | [spec-registration-coin](spec-registration-coin.md) | Collateral recovery spend cost analyzed here |
| **Depends on** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | Proof verification at depth 32 is exactly 32 SHA-256 operations — that constant is the basis for spend path 3 cost |
| **Depends on** | [spec-wire-format](spec-wire-format.md) | Atom sizes and field encoding affect CLVM cost |
| **Depends on** | [spec-groth16-circuit](spec-groth16-circuit.md) | Constraint count estimates determine proof generation time (separate from on-chain CLVM cost) |
| **Referenced by** | [spec-security](spec-security.md) | CLVM Cost Limit Exceeded failure mode |
| **Referenced by** | [spec-deployment-runbook](spec-deployment-runbook.md) | Fee planning before mainnet deployment |
| **Referenced by** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | Motivation section references constant cost as a core requirement |
| **Referenced by** | [spec-consensus-crate](spec-consensus-crate.md) | Fee estimates used in operational planning |

---

## Overview

This document estimates the CLVM cost of each spend path in the
chia-l2-consensus system. CLVM costs determine transaction fees and whether
spends fit within block limits. All costs are in CLVM cost units.

The Chia block limit is 11 billion cost units per block. Any single spend
bundle must stay under this limit.

The on-chain verification cost is distinct from the off-chain proof generation
time. The former is measured in CLVM cost units and determines fees. The latter
is measured in seconds and is determined by constraint count
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Constraint Count
Estimates). Both are documented here for completeness.

---

## Cost Reference

| Operation | Cost |
|-----------|------|
| SHA-256 (per 64 bytes input) | 87 |
| `bls_pairing_identity` base | 3,000,000 |
| `bls_pairing_identity` per G1/G2 pair | 1,200,000 |
| `bls_verify` base | 3,000,000 |
| `bls_verify` per G1/G2 pair | 1,200,000 |
| `g1_add` per point | 101,094 |
| `g1_multiply` base | 706 |
| `g1_multiply` per byte of scalar | 92 |
| `g1_negate` | 916 |
| `AGG_SIG_ME` condition | 1,200,000 |
| `CREATE_COIN` condition | 1,800,000 |
| `ASSERT_COIN_ANNOUNCEMENT` condition | 1,200,000 |
| `CREATE_COIN_ANNOUNCEMENT` condition | 1,800,000 |

---

## Spend Path 1: Network Coin Registration

Full puzzle spec: [spec-network-coin](spec-network-coin.md).

Each time a validator registers through the network coin, one spend is
submitted. The validator's perspective on this spend is in
[spec-validator-onboarding](spec-validator-onboarding.md) — Step 6.

| Operation | Count | Unit Cost | Total |
|-----------|-------|-----------|-------|
| SHA-256 (registration_message, ~56 bytes) | 1 | ~87 | ~87 |
| `AggSigMe` condition | 1 | 1,200,000 | 1,200,000 |
| `CreateCoin` (registration coin) | 1 | 1,800,000 | 1,800,000 |
| `CreateCoin` (recreate network coin) | 1 | 1,800,000 | 1,800,000 |
| CLVM execution overhead | — | ~500,000 | 500,000 |
| **Total** | | | **~5,300,087** |

Well within limits. Fee at standard rates: approximately 0.0000053 XCH.

---

## Spend Path 2: Checkpoint Singleton — Checkpoint

Full puzzle spec: [spec-checkpoint-singleton](spec-checkpoint-singleton.md) —
Spend Path 1. This is the most expensive spend in the system due to the
Groth16 pairing verification and BLS verification. Submitted by
[spec-consensus-crate](spec-consensus-crate.md) — build_checkpoint(). The
L2 integration flow is in
[spec-l2-integration](spec-l2-integration.md) — Checkpoint Submission Flow.

### Groth16 Verification

The Groth16 verification equation requires 4 pairing checks
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Puzzle
Source and [spec-wire-format](spec-wire-format.md) — VK Input Computation):

```
e(A, B) * e(-alpha_g1, beta_g2) * e(-vk_input, gamma_g2) * e(-C, delta_g2) = 1
```

`bls_pairing_identity` with 4 G1/G2 pairs:
- Base: 3,000,000
- Per pair (4 pairs): 4 × 1,200,000 = 4,800,000
- **Pairing subtotal: 7,800,000**

### VK Input Linear Combination

Computing `vk_input` requires 6 scalar multiplications and 6 point additions
per [spec-wire-format](spec-wire-format.md) — VK Input Computation. Each
scalar is a 32-byte sha256 hash, so 32 bytes per `g1_multiply`:

| Operation | Count | Unit Cost | Total |
|-----------|-------|-----------|-------|
| `g1_multiply` (32-byte scalar each) | 6 | 706 + 32×92 = 3,650 | 21,900 |
| `g1_add` | 6 | 101,094 | 606,564 |
| SHA-256 (scalar encoding per [spec-wire-format](spec-wire-format.md)) | 6 | ~87 | ~522 |
| **VK input subtotal** | | | **~628,986** |

### BLS Verify

`bls_verify` for the aggregate signature
(→ see [spec-wire-format](spec-wire-format.md) — Aggregate Signature):
- Base: 3,000,000
- Per pair (1 pair): 1,200,000
- **bls_verify subtotal: 4,200,000**

### Conditions

| Condition | Cost |
|-----------|------|
| `CREATE_COIN` (recreate singleton) | 1,800,000 |
| `CREATE_COIN_ANNOUNCEMENT` (checkpoint state per [spec-wire-format](spec-wire-format.md)) | 1,800,000 |
| **Conditions subtotal** | **3,600,000** |

### Checkpoint Spend Total

| Component | Cost |
|-----------|------|
| Groth16 pairing | 7,800,000 |
| VK input computation | 628,986 |
| BLS verify | 4,200,000 |
| Conditions | 3,600,000 |
| CLVM execution overhead | ~1,000,000 |
| **Total** | **~17,228,986** |

**~17.2 million CLVM units. 0.157% of the 11B block limit.**

A single block could theoretically contain ~638 checkpoint spends. In practice
the L2 submits at most one per epoch
(→ see [spec-l2-integration](spec-l2-integration.md) — Important Notes: Only
one checkpoint in-flight at a time).

Fee at standard rates: approximately 0.0000172 XCH. The collateral requirement
(e.g. 100 XCH per validator) dwarfs the transaction fee entirely.

---

## Spend Path 3: Checkpoint Singleton — Membership Query

Full puzzle spec: [spec-checkpoint-singleton](spec-checkpoint-singleton.md) —
Spend Path 2. This spend is submitted when a validator exits and needs to prove
non-membership to recover collateral. Submitted by
[spec-consensus-crate](spec-consensus-crate.md) — recover_collateral() and
query_membership_on_chain().

### Merkle Path Verification

Verifying a Merkle path at depth 32 requires exactly 32 SHA-256 operations per
[spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Proof Verification
(Rue). This constant cost is the key property that makes the system scalable:
it does not grow with validator set size.

| Operation | Count | Unit Cost | Total |
|-----------|-------|-----------|-------|
| SHA-256 (leaf hash) | 1 | ~87 | 87 |
| SHA-256 (path verification, 32 levels) | 32 | ~87 | 2,784 |
| **Merkle verification subtotal** | | | **~2,871** |

### Conditions

| Condition | Cost |
|-----------|------|
| `CREATE_COIN` (recreate singleton unchanged) | 1,800,000 |
| `CREATE_COIN_ANNOUNCEMENT` (membership announcement per [spec-wire-format](spec-wire-format.md)) | 1,800,000 |
| **Conditions subtotal** | **3,600,000** |

### Membership Query Spend Total

| Component | Cost |
|-----------|------|
| Merkle verification | 2,871 |
| Conditions | 3,600,000 |
| CLVM execution overhead | ~500,000 |
| **Total** | **~4,102,871** |

**~4.1 million CLVM units. 0.037% of block limit.**

---

## Spend Path 4: Registration Coin — Collateral Recovery

Full puzzle spec: [spec-registration-coin](spec-registration-coin.md). This
spend is always submitted in the same bundle as the membership query spend
(Spend Path 3). The bundle is assembled in
[spec-consensus-crate](spec-consensus-crate.md) — Collateral Recovery. The
validator perspective is in
[spec-validator-onboarding](spec-validator-onboarding.md) — Voluntary Exit.

| Operation | Count | Cost | Total |
|-----------|-------|------|-------|
| SHA-256 (announcement message) | 1 | ~87 | 87 |
| `ASSERT_COIN_ANNOUNCEMENT` | 1 | 1,200,000 | 1,200,000 |
| `CREATE_COIN` (collateral return) | 1 | 1,800,000 | 1,800,000 |
| CLVM execution overhead | — | ~300,000 | 300,000 |
| **Total** | | | **~3,300,087** |

### Combined Collateral Recovery Bundle

Membership query (Spend Path 3) + registration coin (Spend Path 4) in one
bundle:

| Spend | Cost |
|-------|------|
| Membership query | ~4,102,871 |
| Registration coin | ~3,300,087 |
| **Bundle total** | **~7,402,958** |

**~7.4 million CLVM units. 0.067% of block limit.**

---

## Summary Table

| Spend Path | Cost (approx) | % of 11B Block Limit | Fee Estimate |
|-----------|---------------|----------------------|--------------|
| Network coin registration | ~5,300,000 | 0.048% | ~0.0000053 XCH |
| Checkpoint (full) | ~17,229,000 | 0.157% | ~0.0000172 XCH |
| Membership query only | ~4,103,000 | 0.037% | ~0.0000041 XCH |
| Collateral recovery bundle | ~7,403,000 | 0.067% | ~0.0000074 XCH |

All spend paths are comfortably within block limits with a safety margin
exceeding 60× for the most expensive path. The security failure mode for cost
limit breach and its mitigation is analyzed in
[spec-security](spec-security.md) — CLVM Cost Limit Exceeded.

---

## Off-Chain Proof Generation Time

Separate from CLVM cost, the Groth16 prover runs off-chain and takes
significant time. This is not a fee concern but an operational concern for
checkpoint submitters. Full constraint analysis:
[spec-groth16-circuit](spec-groth16-circuit.md) — Constraint Count Estimates.

| Configuration | Constraints | Proof Time (est.) |
|--------------|-------------|-------------------|
| MAX_SIGNERS=10, TREE_DEPTH=32 (BLS12-381) | ~8,850,000 | 5–15 minutes |
| MAX_SIGNERS=10, TREE_DEPTH=32 (BLS12-377, future) | ~50,000 | < 10 seconds |

The L2 integration guide covers handling this latency:
[spec-l2-integration](spec-l2-integration.md) — Handling Proof Generation
Failure and Important Notes: Proof generation is blocking.

---

## Measurement Instructions

These are estimates. Actual costs depend on the exact CLVM bytecode produced
by the Rue compiler and atom sizes. Before mainnet deployment, measure actual
costs using the Chia simulator as described in
[spec-deployment-runbook](spec-deployment-runbook.md):

```rust
#[test]
fn measure_checkpoint_cost() {
    let sim = Simulator::new();
    // ... setup per spec-deployment-runbook ...
    let result = sim.spend_coins(bundle, &[]);
    println!("Checkpoint cost: {}", result.cost);
    // Compare against estimates in this document
}
```

The Chia full node reports the cost of each spend bundle in the mempool and in
block records. Use this to calibrate fee settings before mainnet launch.
