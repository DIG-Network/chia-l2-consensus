# Validator Onboarding Guide

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Depends on** | [spec-deployment-runbook](spec-deployment-runbook.md) | Network must be fully deployed and verified before validators can register |
| **Depends on** | [spec-network-coin](spec-network-coin.md) | Registration is a network coin spend |
| **Depends on** | [spec-registration-coin](spec-registration-coin.md) | Registration creates a registration coin. Collateral recovery spends it. |
| **Depends on** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | Membership query spend used for collateral recovery |
| **Depends on** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | Non-membership proof needed for collateral recovery |
| **Depends on** | [spec-wire-format](spec-wire-format.md) | Signing message format, membership announcement format |
| **Depends on** | [spec-consensus-crate](spec-consensus-crate.md) | All on-chain interactions go through ConsensusClient |
| **Referenced by** | [spec-l2-integration](spec-l2-integration.md) | Signature collection assumes validators are onboarded |
| **Referenced by** | [spec-security](spec-security.md) | Key security, collateral recovery, validator key compromise failure mode |

---

## Overview

This guide covers everything a new validator needs to do to join the L2
network, from generating keys to confirming active status. Read the whole
thing before starting. The network must be fully deployed and verified before
any of these steps apply
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Post-Deployment
Checklist).

---

## Prerequisites

- Chia full node running and synced (or access to a trusted node RPC)
- Rust toolchain installed (stable, 1.75+)
- The `network_config.json` file from the network operator
  (→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 6)
- Enough XCH in your wallet to cover the collateral requirement plus fees.
  The collateral amount is in `network_config.json` under `collateral_amount`.
  Fee estimates: [spec-clvm-costs](spec-clvm-costs.md) — Spend Path 1.

---

## Step 1: Install the Validator Node Software

```bash
git clone https://github.com/<org>/chia-l2-consensus
cd chia-l2-consensus
cargo build --release

./target/release/l2-validator --version
```

---

## Step 2: Generate Your Validator Key

Your validator key is a BLS12-381 G1 keypair. The public key (48 bytes
compressed per [spec-wire-format](spec-wire-format.md) — G1 Points) is your
identity in the network and determines your slot in the sparse Merkle tree
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Slot
Assignment). The private key signs checkpoint messages
(→ see [spec-wire-format](spec-wire-format.md) — Individual Signatures).

```bash
./target/release/l2-validator keygen \
  --output validator_key.json

# Output:
# Public key:  <48-byte hex>
# Private key: stored in validator_key.json
```

**Back up `validator_key.json` immediately.** If you lose it you lose access
to your collateral. There is no recovery path. The security implications of
key loss and compromise are in
[spec-security](spec-security.md) — Validator Key Compromise.

```bash
./target/release/l2-validator keyinfo --key validator_key.json
# Expected: Public key: <48-byte hex>, Key type: BLS12-381
```

---

## Step 3: Configure the Node

Create `validator_config.toml` using the `NetworkConfig` structure from
[spec-consensus-crate](spec-consensus-crate.md) — Configuration:

```toml
[network]
config_path = "network_config.json"
node_url = "https://your-chia-node:8555"

[validator]
key_path  = "validator_key.json"
cache_path = "indexer_cache.json"

[checkpoint]
# Only set this if your node will submit checkpoints
# proving_key_path = "proving_key.bin"
# Proving key distribution: spec-trusted-setup — Proving Key Distribution

[logging]
level = "info"
```

---

## Step 4: Fund Your Wallet

You need a Chia coin with at least the collateral amount to register:

```bash
cat network_config.json | grep collateral_amount
# e.g. 10000000000000 = 10 XCH

chia wallet coins list --min-amount 10000000000000
# You need at least one coin with >= collateral_amount
```

The collateral is locked in the registration coin
(→ see [spec-registration-coin](spec-registration-coin.md) — Overview) until
you exit the network.

---

## Step 5: Sync with the Network

Before registering, sync your node with the current chain state. This drives
the indexer (→ see [spec-indexer](spec-indexer.md) — Sync Algorithm) and
verifies the Merkle root is consistent
(→ see [spec-indexer](spec-indexer.md) — Merkle Root Consistency Check):

```bash
./target/release/l2-validator sync --config validator_config.toml

# Expected output:
# Syncing from block <N> to <M>...
# Active validators: <count>
# Current epoch: <N>
# Merkle root matches on-chain: YES
# Sync complete at block <M>
```

---

## Step 6: Register

This submits a network coin spend
(→ see [spec-network-coin](spec-network-coin.md) — Registration) that:

1. Requires your signature to prove key ownership using the registration
   message format (→ see [spec-wire-format](spec-wire-format.md) — Registration
   Message Format)
2. Locks your collateral in a registration coin
   (→ see [spec-registration-coin](spec-registration-coin.md))
3. Includes your pubkey as a memo for indexer efficiency
   (→ see [spec-indexer](spec-indexer.md) — Important Notes: Memo is required
   for indexing)

This is implemented by `ConsensusClient.register_validator()`
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Validator
Registration). The CLVM cost is approximately 5.3M units
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Spend Path 1).

```bash
./target/release/l2-validator register \
  --config validator_config.toml \
  --collateral-coin <coin_id_of_your_xch_coin>

# Output:
# Building registration spend bundle...
# Submit this bundle? [y/N]: y
# Submitting to node...
# Bundle ID: <hex>
# Registration submitted. Wait for confirmation.
```

Wait for the transaction to confirm (1–3 blocks).

---

## Step 7: Verify Registration

After confirmation, sync and verify your registration coin exists and you
appear in the local validator set. This re-runs the indexer lineage check
(→ see [spec-indexer](spec-indexer.md) — Registration Coin Detection):

```bash
./target/release/l2-validator status --config validator_config.toml

# Expected output:
# Your public key: <hex>
# Registration status: REGISTERED
# Registration coin: <coin_id>
# Collateral amount: 10 XCH
# Active in validator set: PENDING
#   (will be included in next checkpoint)
```

`PENDING` is expected. Your pubkey is in the registration coins on L1 and the
indexer has verified your lineage, but the checkpoint singleton's
`validator_merkle_root` will not include you until the next checkpoint is
submitted. This is by design
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Why the
validator set lives off-chain).

---

## Step 8: Wait for the Next Checkpoint

The network includes you in the validator set at the next checkpoint. You can
monitor this with `ConsensusClient.epoch()`
(→ see [spec-consensus-crate](spec-consensus-crate.md) — State Accessors):

```bash
./target/release/l2-validator watch --config validator_config.toml

# After a new checkpoint:
# New checkpoint at epoch <N>
# Validator count: <N>
# Your status: ACTIVE
```

Once `ACTIVE`, you are a full member of the validator set and your pubkey is
in the `validator_merkle_root` committed to the checkpoint singleton.

---

## Step 9: Participate in Consensus

Once active, your node participates in signing checkpoints. How signing is
coordinated is defined by the L2 system you are joining, not by this crate
(→ see [spec-l2-integration](spec-l2-integration.md) — Signature Collection).
Your node signs checkpoint messages using the signing message format from
[spec-wire-format](spec-wire-format.md) — Individual Signatures:

```bash
./target/release/l2-validator sign \
  --config validator_config.toml \
  --message <checkpoint_message_hex>

# Output:
# Signing message: <hex>
# Signature: <96-byte hex G2 point>
# Public key: <48-byte hex G1 point>
```

Share the signature and your public key with the checkpoint coordinator. The
coordinator aggregates signatures using the process in
[spec-wire-format](spec-wire-format.md) — Aggregate Signature and Aggregate
Public Key.

---

## Voluntary Exit

To exit the network and recover your collateral. The full bundle is built by
`ConsensusClient.recover_collateral()`
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Collateral Recovery).

### Step 1: Signal intent to exit at the L2 level

Contact the L2 coordinator. Your exit will be included in the next
checkpoint's validator set transition
(→ see [spec-l2-integration](spec-l2-integration.md) — Validator Set
Transitions).

### Step 2: Wait for a checkpoint that excludes you

```bash
./target/release/l2-validator watch --config validator_config.toml
# Wait until: Your status: INACTIVE
```

Your pubkey must be absent from the `validator_merkle_root` in a confirmed
checkpoint before you can recover collateral.

### Step 3: Recover your collateral

This submits a two-spend bundle atomically:

**Spend 1**: Checkpoint singleton membership query spend
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend Path
2: Membership Query). Uses a non-membership Merkle proof
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Non-Membership
Proof). Permissionless — no signature required. Emits a membership
announcement (→ see [spec-wire-format](spec-wire-format.md) — Membership
Announcement Format).

**Spend 2**: Registration coin spend
(→ see [spec-registration-coin](spec-registration-coin.md) — Spending the
Registration Coin). Asserts the membership announcement from Spend 1. Returns
your collateral.

CLVM cost of the bundle: approximately 7.4M units
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Combined Collateral Recovery
Bundle).

```bash
./target/release/l2-validator exit \
  --config validator_config.toml \
  --destination <your_wallet_puzzle_hash>

# Output:
# Building collateral recovery spend bundle...
# Membership query spend: OK
# Registration coin spend: OK
# Submitting bundle...
# Bundle ID: <hex>
# Collateral recovery submitted
```

There is no timing pressure on this spend. The checkpoint singleton is
recreated unchanged by the membership query spend, so the same state remains
queryable until the next checkpoint changes it
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Rationale:
Why the checkpoint singleton has a membership query spend path).

---

## Key Security

Your validator key controls your participation in consensus and your access to
the collateral. Security analysis:
[spec-security](spec-security.md) — Validator Key Compromise.

- Never share the private key
- Back it up offline in at least two separate secure locations
- The key is not recoverable if lost — and neither is the collateral
- If you believe your key has been compromised, signal exit immediately at the
  L2 level. A compromised key that is still in the active set can be used by
  an attacker to contribute toward a malicious majority.

---

## Troubleshooting

**Registration bundle rejected**

Check that your collateral coin is unspent and has the exact required amount.
Check that the node is synced. If the network coin was spent since your last
sync, re-sync before retrying (→ see [spec-indexer](spec-indexer.md) — Sync
Algorithm).

**Status shows REGISTERED but not ACTIVE after several checkpoints**

The checkpoint submitter may not be including your key in the new validator set
transitions. Contact the L2 coordinator. Also possible: no checkpoints have
been submitted since you registered. Monitor epoch advancement
(→ see [spec-l2-integration](spec-l2-integration.md) — Monitoring).

**Exit spend rejected**

The membership query spend requires a valid Merkle non-membership proof against
the current `validator_merkle_root`
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Non-Membership
Proof). If this fails, your node may be out of sync. Run `sync` and try again.
If you were not excluded from the last checkpoint yet, you need to wait for a
checkpoint that excludes you first.

**Cannot find my collateral after exit**

Check the destination puzzle hash you provided. Verify in your wallet.
