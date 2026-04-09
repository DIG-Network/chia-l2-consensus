# Trusted Setup Ceremony Runbook

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Depends on** | [spec-groth16-circuit](spec-groth16-circuit.md) | The circuit defines what is being set up. MAX_SIGNERS and TREE_DEPTH are circuit parameters. |
| **Enables** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | The VK produced here is curried into the checkpoint singleton at deployment |
| **Enables** | [spec-deployment-runbook](spec-deployment-runbook.md) | The VK and proving key must exist before Step 2 of the deployment runbook |
| **Enables** | [spec-consensus-crate](spec-consensus-crate.md) | The proving key is loaded by ConsensusClient.load_proving_key() |
| **Referenced by** | [spec-security](spec-security.md) | Assumption 2 covers trusted setup soundness. Failure modes cover compromise recovery. |
| **Referenced by** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | Trusted Setup section and Proving the correct circuit was used security section |
| **Referenced by** | [spec-groth16-circuit](spec-groth16-circuit.md) | Circuit parameters fixed at setup time |

---

## Overview

The Groth16 proving system requires a one-time trusted setup ceremony to
generate the proving key and verification key for the consensus circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md)). This document covers
what the trusted setup is, why it matters, how to run it, and what to do with
the output.

The setup must be completed before the network can be deployed
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 1). The
verification key produced by the setup gets curried into the checkpoint
singleton puzzle permanently
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Curried In
Parameters). If you use a compromised or incorrectly generated VK the security
of the entire system is broken
(→ see [spec-security](spec-security.md) — Assumption 2).

---

## What the Trusted Setup Is

The Groth16 trusted setup generates two things:

- **Proving key (pk)**: Used by the checkpoint submitter to generate proofs.
  Large, typically 100-500MB. Does not need to be public but can be. Loaded by
  `ConsensusClient.load_proving_key()`
  (→ see [spec-consensus-crate](spec-consensus-crate.md) — Startup).
- **Verification key (vk)**: Used on-chain to verify proofs. Small, 672 bytes
  as defined in
  (→ see [spec-wire-format](spec-wire-format.md) — Verification Key Format).
  Must be published. Gets curried into the checkpoint singleton
  (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md)).

The setup process involves generating random secret values called toxic waste.
If an attacker knows the toxic waste they can generate fake proofs that verify
as valid. The toxic waste must be destroyed after the ceremony
(→ see [spec-security](spec-security.md) — Assumption 2 and Trusted Setup
Compromise failure mode).

A single-party setup is insecure because one person controls all the toxic
waste. A multi-party computation ceremony distributes the toxic waste across
many participants. As long as at least one participant honestly destroys their
contribution, the toxic waste is irrecoverable and the setup is secure.

---

## Single-Party Setup (Development/Testing Only)

Never use this in production. For local development and testing only. The
security implications are covered in
(→ see [spec-security](spec-security.md) — Assumption 2).

```bash
cargo build --release --bin l2-consensus-setup

# MAX_SIGNERS and TREE_DEPTH must match what you will deploy
# TREE_DEPTH must match spec-sparse-merkle-tree TREE_DEPTH parameter
# MAX_SIGNERS bounds the circuit size per spec-groth16-circuit
./target/release/l2-consensus-setup \
  --max-signers 10 \
  --tree-depth 32 \
  --pk-path proving_key.bin \
  --vk-path verification_key.bin

# Expected output:
# Generating circuit parameters...
# Constraint count: ~8,850,000 (per spec-groth16-circuit estimates)
# Setup complete
# proving_key.bin: ~347MB
# verification_key.bin: 672 bytes (per spec-wire-format VK format)
# VK hash (sha256): <hex>
```

---

## Multi-Party Ceremony (Production)

For production you need multiple independent parties each contributing
randomness. The standard approach uses the Powers of Tau followed by a
circuit-specific phase 2.

### Phase 1: Powers of Tau

Powers of Tau is a universal setup that any Groth16 circuit can use as its
starting point. You can reuse an existing Powers of Tau output rather than
running your own phase 1.

Download the Hermez ceremony output (supports up to 2^28 constraints). At
approximately 8.85 million constraints as estimated in
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Constraint Count
Estimates) the Hermez 2^28 file is sufficient:

```bash
wget https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final.ptau
sha256sum powersOfTau28_hez_final.ptau
# Must match the published hash from the Hermez ceremony
```

### Phase 2: Circuit-Specific Setup

Phase 2 specializes the Powers of Tau output to the specific circuit defined
in
(→ see [spec-groth16-circuit](spec-groth16-circuit.md)). The blank circuit
is used for setup
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Trusted Setup).

#### Option A: snarkjs

```bash
npm install -g snarkjs

# Start phase 2 from phase 1 transcript
snarkjs groth16 setup circuit.r1cs powersOfTau28_hez_final.ptau ceremony_0000.zkey

# Participant 1 contributes
snarkjs zkey contribute ceremony_0000.zkey ceremony_0001.zkey \
  --name "Participant 1" \
  -e "$(head -c 1000 /dev/urandom | base64)"

# Add as many participants as you want - more = more security
# per spec-security Assumption 2

# Apply random beacon (use a future Bitcoin block hash)
snarkjs zkey beacon ceremony_000N.zkey ceremony_final.zkey \
  <BITCOIN_BLOCK_HASH> "Final Beacon" 10

# Export VK (672 bytes per spec-wire-format)
snarkjs zkey export verificationkey ceremony_final.zkey verification_key.json

# Export proving key in Arkworks format
snarkjs zkey export arkworks ceremony_final.zkey proving_key.bin
```

#### Option B: Arkworks native MPC

```bash
cargo install ark-mpc-cli

ark-mpc-cli phase2 init \
  --circuit circuit.r1cs \
  --ptau powersOfTau28_hez_final.ptau \
  --output ceremony_0000.params

# Each participant runs in sequence
ark-mpc-cli phase2 contribute \
  --input ceremony_NNNN.params \
  --output ceremony_MMMM.params \
  --entropy "$(head -c 1000 /dev/urandom | base64)"

# Verify each contribution
ark-mpc-cli phase2 verify \
  --input ceremony_MMMM.params \
  --prev  ceremony_NNNN.params

# Finalize
ark-mpc-cli phase2 finalize \
  --input ceremony_final.params \
  --pk-output proving_key.bin \
  --vk-output verification_key.bin
```

---

## Verifying the Output

Before deploying, verify the setup output is correct. These checks are also
required by the deployment runbook
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 1):

### 1. Verify the constraint count matches the circuit spec

```rust
use ark_relations::r1cs::ConstraintSystem;

let cs = ConstraintSystem::<Fr>::new_ref();
ConsensusCircuit::blank(MAX_SIGNERS, TREE_DEPTH)
    .generate_constraints(cs.clone())?;

println!("Constraint count: {}", cs.num_constraints());
// Must match estimates in spec-groth16-circuit — Constraint Count Estimates
println!("Public inputs: {}", cs.num_instance_variables());
// Must equal 7 (6 public inputs + constant term per spec-wire-format VK format)
```

### 2. Verify the VK has the correct structure

```rust
let vk = load_verification_key("verification_key.bin")?;
assert_eq!(vk.gamma_abc_g1.len(), 7,
    "Must have 7 IC points per spec-wire-format — IC Point Order");
```

### 3. Generate a test proof and verify it

```rust
let test_proof = generate_test_proof(&pk, MAX_SIGNERS, TREE_DEPTH)?;

let valid = Groth16::<Bls12_381>::verify(
    &vk,
    &test_proof.public_inputs,
    &test_proof.proof,
)?;
assert!(valid, "Test proof must verify");
```

### 4. Verify serialized bytes match CLVM expectations

The serialized VK must conform to the format defined in
(→ see [spec-wire-format](spec-wire-format.md) — Verification Key Format):

```rust
let clvm_vk = serialize_vk(&vk)?;
assert_eq!(clvm_vk.alpha_g1.len(), 48);  // G1 per spec-wire-format
assert_eq!(clvm_vk.beta_g2.len(), 96);   // G2 per spec-wire-format
assert_eq!(clvm_vk.ic.len(), 7);         // 6 public inputs + constant
```

### 5. Compute and publish the VK hash

```rust
let vk_bytes = std::fs::read("verification_key.bin")?;
let vk_hash  = sha256(&vk_bytes);
println!("VK sha256: {}", hex::encode(vk_hash));
```

This hash is published as part of the deployment artifacts
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 6). Users
and wallets verify the on-chain checkpoint singleton contains this VK before
trusting checkpoints
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Proving
the correct circuit was used).

---

## What to Publish

After the ceremony, publish the following. This is required for the deployment
runbook Step 6
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 6):

1. **The verification key** in hex-encoded JSON format as defined in
   (→ see [spec-wire-format](spec-wire-format.md) — Verification Key Format —
   Storage Format). This is what gets curried into the checkpoint singleton.

2. **The VK hash** (SHA-256 of `verification_key.bin`). Short enough to tweet.
   Anyone can verify the checkpoint singleton uses this VK
   (→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 7).

3. **The ceremony transcript**. The sequence of `.zkey` or `.params` files.
   Anyone can re-verify the ceremony was conducted correctly
   (→ see [spec-security](spec-security.md) — Assumption 2).

4. **Participant attestations**. Each participant should publish a signed
   statement confirming they destroyed their toxic waste.

5. **The circuit source code** at the exact git commit used to generate the
   constraint system
   (→ see [spec-groth16-circuit](spec-groth16-circuit.md)). Anyone should be
   able to reproduce the circuit and verify the constraint count matches.

---

## When to Rerun the Ceremony

A new ceremony is required if any of these change:
- `MAX_SIGNERS` needs to increase
  (→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Circuit Parameters)
- `TREE_DEPTH` needs to increase
  (→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Parameters)
- A bug is found in the circuit that changes the constraint count
  (→ see [spec-groth16-circuit](spec-groth16-circuit.md))
- The ceremony is believed to be compromised
  (→ see [spec-security](spec-security.md) — Trusted Setup Compromise)

A new ceremony is **not** required if:
- New validators join or leave (`validator_count` is a runtime input
  → see [spec-groth16-circuit](spec-groth16-circuit.md) — Constraint 3)
- The L2 state changes
- The checkpoint singleton is redeployed (the VK can be reused if it matches)

---

## Ceremony Participant Selection

Pick participants who are:
- Geographically distributed
- Running different operating systems and hardware
- Economically independent from each other and from the L2 operator
- Willing to publish attestations

A minimum of 5 independent participants is recommended. More is better.
The security model is covered in
(→ see [spec-security](spec-security.md) — Assumption 2).

---

## Proving Key Distribution

The proving key is needed by any node that submits checkpoints
(→ see [spec-l2-integration](spec-l2-integration.md) — Important Notes:
Proof generation is blocking and
[spec-consensus-crate](spec-consensus-crate.md) — Startup). At 100-500MB it
is large. Options:

- Store in a CDN and have nodes download on startup
- Distribute via IPFS with a pinned CID
- Bundle with the node software

The proving key is not secret. It is safe to distribute publicly. Only the
toxic waste is secret, and that must be destroyed.
