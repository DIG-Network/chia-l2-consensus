# Security Assumptions

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Synthesizes** | All specs | This document is the security analysis that draws on every other spec. Read all others first. |
| **Analyzes** | [spec-network-coin](spec-network-coin.md) | Lineage proof enforcement property |
| **Analyzes** | [spec-registration-coin](spec-registration-coin.md) | Collateral security, epoch replay protection |
| **Analyzes** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | Two-check design completeness, membership query permissionlessness |
| **Analyzes** | [spec-groth16-circuit](spec-groth16-circuit.md) | Circuit correctness assumption, trusted setup soundness |
| **Analyzes** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | Cross-implementation consistency requirement |
| **Analyzes** | [spec-wire-format](spec-wire-format.md) | Serialization consistency, epoch replay protection |
| **Analyzes** | [spec-trusted-setup](spec-trusted-setup.md) | Trusted setup compromise failure mode and recovery |
| **Analyzes** | [spec-indexer](spec-indexer.md) | Off-chain lineage verification enforcement |
| **Analyzes** | [spec-clvm-costs](spec-clvm-costs.md) | CLVM cost limit exceeded failure mode |
| **Analyzes** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | Security section of the CHIP |
| **Referenced by** | [spec-deployment-runbook](spec-deployment-runbook.md) | Step 7 VK verification is a security requirement |
| **Referenced by** | [spec-trusted-setup](spec-trusted-setup.md) | Assumption 2 drives MPC ceremony requirements |
| **Referenced by** | [spec-validator-onboarding](spec-validator-onboarding.md) | Key security guidance |

---

## Overview

This document defines the threat model for the chia-l2-consensus system, the
assumptions the system makes about validator behavior and infrastructure, and
the failure modes with recovery paths. Read this before deploying to
production. It cross-references specific sections across all implementation
specs where each property is enforced.

---

## Trust Model

### What the L1 Enforces Unconditionally

The Chia L1 enforces these properties regardless of validator behavior:

- A registration coin can only be created by spending the network coin
  singleton (→ see [spec-network-coin](spec-network-coin.md) — What the
  Puzzle Does)
- A registration coin can only be spent when the checkpoint singleton emits a
  non-membership announcement for that validator's pubkey
  (→ see [spec-registration-coin](spec-registration-coin.md) — What the
  Puzzle Does)
- The checkpoint singleton state can only be updated by a spend that provides
  a valid Groth16 proof verified by `bls_pairing_identity` and a valid BLS
  aggregate signature verified by `bls_verify`
  (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
  Path 1: Checkpoint)
- The epoch counter increments by exactly 1 on every checkpoint spend
  (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — What the
  Puzzle Does: Checkpoint Spend Path)
- The network coin and checkpoint singleton are singletons and can only exist
  once each

### What the ZK Proof Enforces

The Groth16 proof, if the circuit is sound
(→ see [spec-groth16-circuit](spec-groth16-circuit.md)):

- The k signing pubkeys are all members of the current `validator_merkle_root`
  (Constraint 1: → see [spec-groth16-circuit](spec-groth16-circuit.md) —
  Constraint 1: Merkle Membership)
- The G1 sum of the k pubkeys equals the claimed `agg_signers`
  (Constraint 2: → see [spec-groth16-circuit](spec-groth16-circuit.md) —
  Constraint 2: Aggregate Consistency)
- `2k > validator_count` (strict majority)
  (Constraint 3: → see [spec-groth16-circuit](spec-groth16-circuit.md) —
  Constraint 3: Majority Threshold)

### What Majority Signature Enforces

The BLS aggregate signature over the checkpoint message, verified by
`bls_verify` in the checkpoint singleton
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend Path
1: Checkpoint), enforces:

- A majority of the current active validator set signed the specific checkpoint
  message (→ see [spec-wire-format](spec-wire-format.md) — Checkpoint Message)
- The checkpoint message commits to `new_state_root`,
  `new_validator_merkle_root`, `new_validator_count`, and `new_epoch`
- Therefore a majority is attesting that the new validator Merkle root
  correctly reflects the current registration coins on L1
  (→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Why the
  validator set lives off-chain)

### What the System Does Not Enforce

- The L2 state root is correct or meaningful (the L1 just stores whatever the
  majority signs)
- Validators are online or responsive
- The L2 system behind the checkpoint is functioning correctly
- The proving key is only held by authorized parties
  (→ see Known Limitations: Proof generation is centralized)

---

## Security Assumptions

### Assumption 1: Majority of Validators Are Honest

The system is secure as long as fewer than half of the registered validators
collude to produce fraudulent checkpoints. A coalition of exactly half or more
validators can submit any state root they want. This is the fundamental
assumption of any BFT consensus system.

**Mitigation**: Collateral makes attacking expensive
(→ see [spec-registration-coin](spec-registration-coin.md) — Overview). A
majority coalition would need to sacrifice their collective collateral if the
attack is detected and the chain is rolled back. Economic independence of
validators and network topology reduce collusion risk.

### Assumption 2: Trusted Setup Is Sound

If any participant in the MPC trusted setup ceremony honestly destroys their
contribution, the toxic waste is irrecoverable and the setup is secure. If all
participants collude, they can generate fake proofs.

**Mitigation**: Run the ceremony with many independent participants from
diverse geographic locations, organizations, and technical backgrounds per
[spec-trusted-setup](spec-trusted-setup.md) — Ceremony Participant Selection.
Publish the ceremony transcript so anyone can verify it was conducted correctly
per [spec-trusted-setup](spec-trusted-setup.md) — What to Publish. Use the
Hermez Powers of Tau phase 1 output which has already been reviewed by
thousands of people per
[spec-trusted-setup](spec-trusted-setup.md) — Phase 1: Powers of Tau.

**Failure mode**: If compromised, an attacker can generate Groth16 proofs for
false statements. Recovery path: → see Trusted Setup Compromise below.

### Assumption 3: Validator Set Off-Chain Verification

The on-chain puzzle does not verify that the `validator_merkle_root` was
constructed from legitimate registration coins. This is verified off-chain by
the L2 indexer (→ see [spec-indexer](spec-indexer.md) — Registration Coin
Detection) when building the validator set before each checkpoint. A majority
of validators independently verify the Merkle root before signing.

**Why this is secure**: Constructing a fraudulent Merkle root requires getting
a majority of real validators to sign the checkpoint message containing that
root. Real validators independently compute the correct root from L1
registration coins and will reject a fraudulent one. This requires corrupting
the majority, which is Assumption 1.

### Assumption 4: Circuit Implementation Is Correct

The Arkworks Groth16 implementation and the Rue `bls_pairing_identity` call
must correctly implement the Groth16 verification equation. Bugs in either
could allow false proofs to verify.

**Mitigation**: Use audited libraries. Add a cross-verification test that
generates a proof in Rust and verifies it in both Rust and CLVM simulator.
Run this test in CI. Audit recommendation:
→ see Audit Recommendations below.

### Assumption 5: Sparse Merkle Tree Cross-Implementation Consistency

The Rust and Rue sparse Merkle tree implementations must produce identical
roots and proofs for the same inputs. Any divergence causes valid proofs to
fail on-chain silently.

**Mitigation**: The sparse Merkle tree spec
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)) is the
canonical reference. Both implementations must pass all test vectors in that
spec (→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Test
Vectors). Add a cross-language consistency test to CI that generates a root
in Rust and verifies a proof in CLVM simulator. The critical invariant is
sibling ordering: left child always comes first in SHA-256 concatenation
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Tree
Structure).

---

## Security Properties

### Lineage Proof Enforcement

A registration coin is only valid if its parent coin ID traces back to a
network coin spend. This is enforced off-chain by the L2 indexer
(→ see [spec-indexer](spec-indexer.md) — Registration Coin Detection and
Lineage Verification). Because coin IDs are deterministic and the blockchain
is immutable this check cannot be gamed. Any coin with the correct puzzle hash
but the wrong parent is ignored. Puzzle hash computation:
[spec-network-coin](spec-network-coin.md) — Computing the Registration Coin
Puzzle Hash.

### Proving the Correct Circuit Was Used

The verification key is cryptographically bound to the specific circuit during
the trusted setup (→ see [spec-trusted-setup](spec-trusted-setup.md)). If you
change anything about the circuit — the number of constraints, the Merkle
depth, the public input order — you get a completely different VK. There is no
way to produce a valid Groth16 proof for circuit A using the VK from circuit B.

Since the VK is curried into the checkpoint singleton at deployment
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Curried In
Parameters) and the singleton is hashed to produce its puzzle hash, the puzzle
hash on L1 commits to both the VK and the circuit permanently. Verified in
[spec-deployment-runbook](spec-deployment-runbook.md) — Step 7.

Compliant implementations must publish:
- The circuit source code at the exact git commit
  (→ see [spec-groth16-circuit](spec-groth16-circuit.md))
- The full trusted setup transcript
  (→ see [spec-trusted-setup](spec-trusted-setup.md) — What to Publish)

### Collateral Security

A validator cannot recover their collateral
(→ see [spec-registration-coin](spec-registration-coin.md)) without a
checkpoint that excludes them which requires majority consensus. This prevents
validators from abandoning the network unilaterally and gives the network a
credible slashing mechanism for misbehaving validators.

### Completeness of the Two-Check Design

An attacker cannot pass both the ZK proof check and `bls_verify` without both
a valid majority of registered keys and a valid aggregate signature from that
majority. The ZK proof alone does not prove the signature and the signature
alone does not prove the quorum is legitimate. These two checks together give
complete security. Design rationale:
[chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Circuit design
choice.

### Epoch Replay Protection

The membership announcement
(→ see [spec-wire-format](spec-wire-format.md) — Membership Announcement
Format) includes the epoch number encoded as an 8-byte big-endian u64. A
non-membership announcement from epoch N cannot be used to spend a registration
coin after epoch N+1 has started if the validator rejoined in the interim,
because the epoch in the announcement would not match the current singleton
epoch. Implementation: [spec-registration-coin](spec-registration-coin.md) —
Important Notes: Epoch Matching.

### Groth16 Soundness

Under the knowledge-of-exponent assumption and the generic group model,
Groth16 is sound. A malicious prover cannot produce a valid proof for a false
statement without breaking the underlying hardness assumption. This is
well-studied and deployed in production systems including Zcash.

---

## Failure Modes and Recovery

### Checkpoint Singleton Is Stuck

**Scenario**: No valid checkpoint is submitted for an extended period. Causes:
validator set falls below majority (too many validators exit or go offline),
proving key unavailable, bug in checkpoint submitter.

**Impact**: L2 state is not settled on L1. L2 operations can continue
off-chain but are not finalized on Chia.

**Recovery**: Bring enough validators online to restore majority per
[spec-validator-onboarding](spec-validator-onboarding.md). Fix proving key
distribution per [spec-trusted-setup](spec-trusted-setup.md) — Proving Key
Distribution. There is no on-chain emergency override. If the validator set is
permanently below majority the network is halted and requires governance to
redeploy.

**Monitoring**: Alert if epoch stops advancing per
[spec-l2-integration](spec-l2-integration.md) — Monitoring.

### Validator Key Compromise

**Scenario**: A validator's private key is stolen. The attacker can sign
checkpoints on the validator's behalf.

**Impact**: The attacker contributes one vote toward a potentially malicious
majority. If combined with other compromised or colluding validators this could
enable fraudulent checkpoints.

**Recovery**: The honest validators should collectively exit the compromised
validator in the next checkpoint, removing their pubkey from the Merkle root
(→ see [spec-l2-integration](spec-l2-integration.md) — Validator Set
Transitions). The compromised validator's collateral remains locked until this
exit is processed. Validator guidance:
[spec-validator-onboarding](spec-validator-onboarding.md) — Key Security.

### Registration Coin Lost

**Scenario**: A validator loses access to their wallet and cannot spend their
registration coin.

**Impact**: The collateral is permanently locked. The validator's pubkey
remains in the Merkle root unless the network exits them.

**Recovery**: The network can force-exit the validator in a checkpoint
(removing their pubkey from the Merkle root) but the collateral remains
unspendable since only the registration coin's owner can provide the spend
conditions per [spec-registration-coin](spec-registration-coin.md) — What the
Puzzle Does. There is no recovery path for lost collateral.

### Trusted Setup Compromise

**Scenario**: All participants in the MPC ceremony colluded or the ceremony
was otherwise compromised (→ see Assumption 2 above).

**Impact**: An attacker can generate valid-looking Groth16 proofs for false
statements, allowing fraudulent checkpoints claiming majority consensus when
they have none.

**Recovery**: This is the most severe failure mode. Steps:
1. Detect the compromise (usually by observing fraudulent checkpoints)
2. Rerun the MPC ceremony with new participants
   (→ see [spec-trusted-setup](spec-trusted-setup.md) — Multi-Party Ceremony)
3. Redeploy the checkpoint singleton with the new VK
   (→ see [spec-deployment-runbook](spec-deployment-runbook.md))
4. Coordinate migration at the L2 governance level

### Chain Reorganization

**Scenario**: The Chia blockchain reorganizes, reversing confirmed blocks
including checkpoint spends or registration coin creation events.

**Impact**: Local indexer state may be ahead of actual chain state. Checkpoints
believed confirmed may be reversed.

**Recovery**: The indexer handles reorgs by rolling back to the last safe
checkpoint state and re-syncing forward
(→ see [spec-indexer](spec-indexer.md) — Reorg Handling). Deep reorgs are
extremely rare on Chia but handled gracefully.

### CLVM Cost Limit Exceeded

**Scenario**: A future Chia update lowers the block cost limit, or CLVM costs
increase significantly.

**Impact**: Checkpoint spends cannot be included in blocks.

**Mitigation**: The current checkpoint spend costs approximately 17.2M CLVM
units against an 11B block limit — a safety margin of roughly 640×
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Spend Path 2). A CLVM update
would need to reduce the block limit by 640× for this to become an issue.
Monitor CLVM cost changes in Chia release notes.

---

## Known Limitations

**Collateral recovery timing**

A validator must wait for a checkpoint that excludes them before recovering
collateral. If checkpoints are infrequent this could be a long wait. This is
by design — the collateral lock prevents premature exit — but validators should
understand the timing expectation before registering.

**No on-chain validator set limit**

The network coin does not limit how many validators can register. A very large
validator set increases proof generation time
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Constraint Count
Estimates). The L2 governance should define a maximum validator count and
enforce it at the L2 level before proof generation becomes impractical.

**Proof generation is centralized**

Only nodes with the proving key can submit checkpoints. This is a
centralization risk if the proving key is held by a small number of parties.
Mitigate by distributing the proving key to all validators or a diverse set of
checkpoint submitters per
[spec-trusted-setup](spec-trusted-setup.md) — Proving Key Distribution.

**No slashing enforcement on-chain**

The L1 does not enforce slashing conditions. The registration coin puzzle can
be extended to support slashing (redirecting collateral to a governance
address) but the specific conditions must be defined and enforced at the L2
level. The `conditions` field in the registration coin solution provides the
extension point
(→ see [spec-registration-coin](spec-registration-coin.md) — Solution
Parameters).

---

## Audit Recommendations

Before mainnet deployment, the following should be independently audited:

1. **Rue puzzle correctness**: All three puzzles
   (→ see [spec-network-coin](spec-network-coin.md),
   [spec-registration-coin](spec-registration-coin.md),
   [spec-checkpoint-singleton](spec-checkpoint-singleton.md)) for logic errors
   and attack vectors.

2. **Groth16 circuit correctness**: The constraint system
   (→ see [spec-groth16-circuit](spec-groth16-circuit.md)) to verify
   constraints correctly encode the intended statement and there are no
   unsatisfied constraints that allow false proofs.

3. **Serialization consistency**: The wire format
   (→ see [spec-wire-format](spec-wire-format.md)) between Rust and CLVM for
   encoding mismatches that could cause security failures.

4. **Driver code**: Rust driver code
   (→ see [spec-consensus-crate](spec-consensus-crate.md)) for correctness,
   particularly lineage verification logic and signature aggregation.

5. **Cross-implementation consistency**: CI test that Rust Merkle root equals
   Rue Merkle root for the same validator set. This directly tests Assumption 5.
