# Deployment Runbook

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Depends on** | [spec-trusted-setup](spec-trusted-setup.md) | Step 1 runs the trusted setup ceremony. VK and proving key must exist before Step 2. |
| **Depends on** | [spec-network-coin](spec-network-coin.md) | Step 2 deploys the network coin singleton |
| **Depends on** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | Step 2 deploys the checkpoint singleton with the VK curried in |
| **Depends on** | [spec-groth16-circuit](spec-groth16-circuit.md) | MAX_SIGNERS and TREE_DEPTH chosen in Step 1 are fixed for the life of the deployment |
| **Depends on** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | TREE_DEPTH set in Step 1 determines the tree. Empty root committed in Step 8. |
| **Depends on** | [spec-wire-format](spec-wire-format.md) | VK serialization format verified in Step 7 |
| **Depends on** | [spec-clvm-costs](spec-clvm-costs.md) | Cost analysis informs fee planning |
| **Enables** | [spec-validator-onboarding](spec-validator-onboarding.md) | Validators cannot register until Steps 1-7 are complete |
| **Enables** | [spec-l2-integration](spec-l2-integration.md) | L2 integration requires a deployed and verified network |
| **Referenced by** | [spec-consensus-crate](spec-consensus-crate.md) | ConsensusClient.deploy() implements Step 2 |
| **Referenced by** | [spec-security](spec-security.md) | VK verification in Step 7 is a security requirement |

---

## Overview

This document covers the end-to-end process for deploying a new L2 network on
Chia. Follow these steps in order. Each step has a verification check before
proceeding. The circular dependency between the network coin and checkpoint
singleton (each needs to know the other's ID) is resolved in Step 3 using a
genesis coin approach.

---

## Prerequisites

Before starting:

- Chia full node running and synced to chain tip
- Rust toolchain installed (stable, 1.75+)
- `chia-l2-consensus` crate compiled in release mode
  (→ see [spec-consensus-crate](spec-consensus-crate.md) — Crate Structure)
- At least 5 trusted parties identified for the MPC ceremony
  (→ see [spec-trusted-setup](spec-trusted-setup.md) — Ceremony Participant
  Selection)
- A funded wallet with at least 2 XCH for genesis coin plus fees

---

## Step 1: Run the Trusted Setup

This step is slow. The trusted setup for a circuit with ~8.85 million
constraints takes 10–30 minutes on a modern server. Constraint estimates:
[spec-groth16-circuit](spec-groth16-circuit.md) — Constraint Count Estimates.

`MAX_SIGNERS` and `TREE_DEPTH` are fixed for the life of this deployment.
Changing either requires a new trusted setup ceremony and a full redeployment.
(→ see [spec-trusted-setup](spec-trusted-setup.md) — When to Rerun the
Ceremony and [spec-groth16-circuit](spec-groth16-circuit.md) — Circuit
Parameters).

```bash
cargo build --release --bin l2-consensus-setup

# For development/testing only: single party setup
# Never use in production - see spec-security — Assumption 2
./target/release/l2-consensus-setup \
  --max-signers 10 \
  --tree-depth 32 \
  --pk-path proving_key.bin \
  --vk-path verification_key.bin

# For production: follow spec-trusted-setup — Multi-Party Ceremony

# Verify constraint count matches spec-groth16-circuit estimates
./target/release/l2-consensus-setup verify \
  --pk-path proving_key.bin \
  --vk-path verification_key.bin
# Expected: "Setup verification OK"

# Record the VK hash - you will need this in Step 7
sha256sum verification_key.bin
```

The VK serialization format must match
[spec-wire-format](spec-wire-format.md) — Verification Key Format: 672 bytes
total, 7 IC points, G1 points 48 bytes, G2 points 96 bytes.

**Do not proceed if verification fails.**

---

## Step 2: Choose a Genesis Coin

The genesis coin is a standard XCH coin you control that funds the deployment.
Pick a coin with at least 2 XCH. The genesis coin will be spent to create both
singletons in the same bundle.

```bash
chia wallet coins list --min-amount 2000000000000  # 2 XCH in mojos
export GENESIS_COIN_ID=<coin_id_hex>
```

---

## Step 3: Derive Deployment Parameters

Before spending anything, derive the launcher IDs and singleton IDs for both
singletons. This resolves the circular dependency:

- The network coin needs `CHECKPOINT_SINGLETON_ID` curried in
  (→ see [spec-network-coin](spec-network-coin.md) — Curried In Parameters)
- The checkpoint singleton needs `REGISTRATION_COIN_MOD_HASH` from the network
  coin puzzle (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md)
  — Curried In Parameters)
- Both IDs are derivable from the genesis coin ID before spending anything

```bash
./target/release/l2-consensus-deploy derive \
  --genesis-coin-id $GENESIS_COIN_ID \
  --collateral-amount 10000000000000 \
  --tree-depth 32 \
  --max-signers 10 \
  --vk-path verification_key.bin

# Output (save this as network_config.json):
# network_coin_launcher_id:   <hex>
# checkpoint_launcher_id:     <hex>
# checkpoint_singleton_id:    <hex>
# registration_coin_mod_hash: <hex>
# checkpoint_inner_mod_hash:  <hex>
```

Save to `network_config.json` matching the `NetworkConfig` struct from
[spec-consensus-crate](spec-consensus-crate.md) — Configuration:

```json
{
  "network_coin_launcher_id":   "<hex>",
  "checkpoint_launcher_id":     "<hex>",
  "registration_coin_mod_hash": "<hex>",
  "checkpoint_inner_mod_hash":  "<hex>",
  "collateral_amount":          10000000000000,
  "tree_depth":                 32,
  "max_signers":                10,
  "verification_key_hex":       "<hex from verification_key.bin>",
  "genesis_challenge":          "<mainnet or testnet genesis challenge hex>"
}
```

Verify that `tree_depth` here matches the `TREE_DEPTH` in the Groth16 circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Circuit Parameters)
and the depth used by the sparse Merkle tree
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Parameters).
A mismatch will cause on-chain proof verification to fail silently.

---

## Step 4: Deploy the Network

This is implemented by `ConsensusClient.deploy()` in
[spec-consensus-crate](spec-consensus-crate.md) — Deployment. Both singletons
are deployed in a single spend bundle.

```bash
./target/release/l2-consensus-deploy deploy \
  --config network_config.json \
  --genesis-coin-id $GENESIS_COIN_ID \
  --wallet-key <your_private_key_file> \
  --node-url https://your-chia-node:8555

# Output:
# Building spend bundle...
# Submitting to node...
# Spend bundle ID: <hex>
# Network coin launcher: <hex>
# Checkpoint launcher:   <hex>
```

---

## Step 5: Verify On-Chain Presence

Wait for the transaction to confirm (1–3 blocks, ~1 minute on mainnet).

```bash
./target/release/l2-consensus-deploy verify \
  --config network_config.json \
  --node-url https://your-chia-node:8555

# Expected output:
# Network coin: FOUND (coin_id: <hex>, height: <N>)
# Checkpoint singleton: FOUND (coin_id: <hex>, height: <N>)
# Epoch: 0
# Validator count: 0
# Validator merkle root: <empty tree root per spec-sparse-merkle-tree — Test Vectors>
# All checks passed
```

**Do not proceed if any check fails.**

---

## Step 6: Publish Deployment Artifacts

Publish the following publicly before allowing any validators to register.
This is a security requirement described in
[spec-security](spec-security.md) — Proving the Correct Circuit Was Used and
[spec-trusted-setup](spec-trusted-setup.md) — What to Publish:

1. **`network_config.json`** — Validators need this to configure their nodes
   (→ see [spec-validator-onboarding](spec-validator-onboarding.md) — Step 3).

2. **`verification_key.bin`** in hex-encoded JSON format
   (→ see [spec-wire-format](spec-wire-format.md) — Verification Key Format —
   Storage Format).

3. **VK hash** (SHA-256 of `verification_key.bin`). Publish this in multiple
   places. Validators and wallets verify the on-chain checkpoint singleton
   contains this VK before trusting checkpoints.

4. **Trusted setup transcript** (production) — the MPC ceremony output files
   (→ see [spec-trusted-setup](spec-trusted-setup.md) — What to Publish).

5. **Circuit source code** at the exact git commit used to generate the VK
   (→ see [spec-groth16-circuit](spec-groth16-circuit.md)).

```bash
sha256sum verification_key.bin
# Cross-check this against what you recorded in Step 1
```

---

## Step 7: Verify the VK is Correctly Curried

Fetch the checkpoint singleton puzzle reveal from the chain, decurry it, and
confirm the VK matches your local file. This is a critical security check
(→ see [spec-security](spec-security.md) — Proving the Correct Circuit Was
Used). The VK serialization format verified here is defined in
[spec-wire-format](spec-wire-format.md) — Verification Key Format.

```bash
./target/release/l2-consensus-deploy verify-vk \
  --config network_config.json \
  --vk-path verification_key.bin \
  --node-url https://your-chia-node:8555

# Expected output:
# Fetching checkpoint singleton puzzle...
# Extracting curried VK...
# VK has 7 IC points: OK
# Comparing to local verification_key.bin...
# VK MATCHES - deployment is correct
```

**If this fails the deployment is incorrect. Start over from Step 3 with a
fresh genesis coin. Do not allow validators to register.**

---

## Step 8: First Sync and Initial Checkpoint

Run a full sync to verify the indexer works correctly
(→ see [spec-indexer](spec-indexer.md) — Sync Algorithm). The empty Merkle
root that the indexer computes should match the `validator_merkle_root`
committed to the checkpoint singleton. The empty root is defined in
[spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Empty Node Hashes.

```bash
./target/release/l2-consensus-node sync \
  --config network_config.json \
  --node-url https://your-chia-node:8555 \
  --cache-path indexer_cache.json

# Expected output:
# Syncing from block 0 to <current_height>...
# Found network coin at height <N>
# Found checkpoint singleton at height <N>
# Active validators: 0
# Merkle root matches on-chain: YES
# Sync complete
```

---

## Post-Deployment Checklist

- [ ] Proving key is accessible to all nodes that will submit checkpoints
  (→ see [spec-consensus-crate](spec-consensus-crate.md) — load_proving_key())
- [ ] `network_config.json` is published and accessible to validators
  (→ see [spec-validator-onboarding](spec-validator-onboarding.md) — Step 3)
- [ ] VK hash published in multiple public locations
- [ ] Trusted setup transcript published (production only)
- [ ] Circuit source code at exact commit tagged and published
- [ ] Indexer is running and syncing on at least one node
- [ ] VK verification (Step 7) passed
- [ ] Monitoring is set up to alert if epoch stops advancing
  (→ see [spec-l2-integration](spec-l2-integration.md) — Monitoring)
- [ ] Fee estimates reviewed per [spec-clvm-costs](spec-clvm-costs.md)

---

## Troubleshooting

**Deployment spend bundle rejected**

Check that the genesis coin is unspent and has sufficient balance. Verify the
node is synced to tip. Check for fee issues if the network is congested
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Fee Estimates).

**Verify step fails with "network coin not found"**

The transaction may not be confirmed yet. Wait a few more blocks and retry.

**VK mismatch in Step 7**

The checkpoint singleton was deployed with a different VK than expected. This
means either the deployment process had a bug or the VK file was modified after
the setup. Do not allow validators to register. Start over from Step 3 with a
fresh genesis coin. The security implications of a wrong VK are analyzed in
[spec-security](spec-security.md) — Trusted Setup Compromise.

**Indexer merkle root mismatch after sync**

Should not happen on a fresh deployment with zero validators. If it does,
there is a bug in the sparse Merkle tree implementation or the indexer. See
[spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Common Implementation
Mistakes and [spec-indexer](spec-indexer.md) — Merkle Root Consistency Check.
File a bug report before proceeding.
