# L2 Integration Guide

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Depends on** | [spec-consensus-crate](spec-consensus-crate.md) | All L2 interaction goes through ConsensusClient |
| **Depends on** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | Checkpoint submission targets this puzzle. Event loop monitors its epoch. |
| **Depends on** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | compute_new_validator_set() updates the tree |
| **Depends on** | [spec-wire-format](spec-wire-format.md) | Signing message format used in signature collection |
| **Depends on** | [spec-groth16-circuit](spec-groth16-circuit.md) | Proof generation is the slow step in checkpoint submission |
| **Depends on** | [spec-deployment-runbook](spec-deployment-runbook.md) | Network must be deployed before integration |
| **Depends on** | [spec-validator-onboarding](spec-validator-onboarding.md) | Assumes validators are registered before checkpoints are submitted |
| **Referenced by** | [spec-security](spec-security.md) | Proof generation centralization limitation |
| **Referenced by** | [spec-consensus-crate](spec-consensus-crate.md) | Integration patterns described here |

---

## Overview

This guide covers how an L2 system integrates the `chia-l2-consensus` crate
(→ see [spec-consensus-crate](spec-consensus-crate.md)). It covers the event
loop pattern, signature collection across validators, handling proof generation
failures, validator set transitions at checkpoint boundaries, and error
recovery. The crate does not handle P2P networking — signature collection and
validator coordination are the L2's responsibility.

---

## Adding the Crate

```toml
[dependencies]
chia-l2-consensus = { path = "../chia-l2-consensus" }
```

The public interface is intentionally narrow
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Public Re-Exports):
`ConsensusClient`, `NetworkConfig`, `ValidatorSet`, `SpendBundle`, `Bytes32`,
`PublicKey`. The L2 does not interact with any internal puzzle or prover
modules directly.

---

## Startup Sequence

```rust
use chia_l2_consensus::{ConsensusClient, NetworkConfig};

async fn start_l2_node(
    config_path: &str,
    node_url: &str,
    proving_key_path: Option<&str>,
) -> anyhow::Result<ConsensusClient> {

    let config: NetworkConfig = serde_json::from_str(
        &std::fs::read_to_string(config_path)?
    )?;

    let node = FullNodeClient::connect(node_url).await?;
    let mut client = ConsensusClient::new(node, config);

    // Load proving key only if this node submits checkpoints
    // Large file (100-500MB) per spec-trusted-setup — Single-Party Setup
    // Proving key distribution: spec-trusted-setup — Proving Key Distribution
    if let Some(pk_path) = proving_key_path {
        client.load_proving_key(pk_path)?;
    }

    // sync() drives the indexer (spec-indexer — Sync Algorithm)
    // and verifies Merkle root consistency (spec-indexer — Merkle Root Consistency Check)
    client.sync().await?;

    println!("synced at epoch {}", client.epoch()?);
    println!("{} active validators", client.validator_count()?);

    Ok(client)
}
```

---

## Event Loop

Run a background task that keeps the client synced and triggers checkpoint
submission when the L2 decides it is time. The epoch accessor
`ConsensusClient.epoch()`
(→ see [spec-consensus-crate](spec-consensus-crate.md) — State Accessors) is
the primary signal that a new checkpoint has been confirmed.

```rust
pub async fn run_consensus_loop(
    client: Arc<Mutex<ConsensusClient>>,
    node: FullNodeClient,
    l2_state: Arc<L2State>,
    is_checkpoint_submitter: bool,
) {
    let mut last_epoch = {
        let c = client.lock().await;
        c.epoch().unwrap_or(0)
    };

    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;

        {
            let mut c = client.lock().await;
            if let Err(e) = c.sync().await {
                eprintln!("sync error: {} - retrying", e);
                continue;
            }

            let current_epoch = c.epoch().unwrap();
            if current_epoch > last_epoch {
                println!("new checkpoint confirmed at epoch {}", current_epoch);
                last_epoch = current_epoch;
                l2_state.on_checkpoint_confirmed(current_epoch).await;
            }
        }

        if is_checkpoint_submitter {
            let should_checkpoint = l2_state.should_submit_checkpoint().await;
            if should_checkpoint {
                if let Err(e) = try_submit_checkpoint(
                    &client, &node, &l2_state
                ).await {
                    eprintln!("checkpoint submission failed: {}", e);
                }
            }
        }
    }
}
```

---

## Checkpoint Submission Flow

The most complex part of integration. The L2 is responsible for P2P signature
collection. The crate handles proof generation
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Proof Generation),
BLS aggregation
(→ see [spec-wire-format](spec-wire-format.md) — Aggregate Signature), and
spend bundle assembly
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Checkpoint
Spend).

```rust
async fn try_submit_checkpoint(
    client: &Arc<Mutex<ConsensusClient>>,
    node: &FullNodeClient,
    l2_state: &Arc<L2State>,
) -> anyhow::Result<()> {

    // Step 1: Determine new L2 state root
    let new_state_root = l2_state.compute_current_state_root().await?;

    // Step 2: Determine validator set changes
    let (entries, exits) = l2_state.pending_validator_changes().await?;

    // Step 3: Compute new Merkle root
    // Uses SparseMerkleTree per spec-sparse-merkle-tree — Tree Updates
    let (new_merkle_root, new_count, _) = {
        let c = client.lock().await;
        c.compute_new_validator_set(&entries, &exits)?
    };

    // Step 4: Get the signing message
    // Format per spec-wire-format — Individual Signatures
    let signing_message = {
        let c = client.lock().await;
        c.validator_signing_message(new_state_root, new_merkle_root, new_count)?
    };

    // Step 5: Collect signatures (L2-defined P2P, not provided by crate)
    let signatures = collect_signatures(
        &signing_message,
        &client.lock().await.validator_set()?.validators,
        l2_state,
    ).await?;

    // Step 6: Generate proof and build spend bundle
    // Proof generation: 5-15 minutes per spec-groth16-circuit — Constraint Count Estimates
    // Runs in spawn_blocking per spec-consensus-crate — submit_checkpoint()
    println!("generating proof ({} signers)...", signatures.len());

    let pubkeys: Vec<_> = signatures.iter().map(|(pk, _)| *pk).collect();
    let sigs: Vec<_>    = signatures.iter().map(|(_, s)| s.clone()).collect();

    let bundle = {
        let c = client.lock().await;
        c.submit_checkpoint(
            new_state_root,
            new_merkle_root,
            new_count,
            &pubkeys,
            &sigs,
        ).await?
    };

    // Step 7: Submit — the crate returns the bundle but does not submit
    node.push_tx(bundle).await?;
    println!("checkpoint submitted");

    Ok(())
}
```

---

## Signature Collection

The crate does not handle signature collection or P2P. A basic approach for a
permissioned L2 where all validators are known. Each validator signs using the
message format from
[spec-wire-format](spec-wire-format.md) — Individual Signatures:

```rust
async fn collect_signatures(
    signing_message: &[u8],
    validators: &[ValidatorInfo],
    l2_state: &Arc<L2State>,
) -> anyhow::Result<Vec<(PublicKey, Signature)>> {

    let mut signatures = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(30);
    let validator_count = validators.len() as u64;

    l2_state.broadcast_signing_request(signing_message).await?;

    while Instant::now() < deadline {
        let responses = l2_state.collect_signature_responses().await;

        for (pubkey, sig) in responses {
            if validators.iter().any(|v| v.pubkey == pubkey) {
                if verify_bls_signature(&pubkey, signing_message, &sig) {
                    if !signatures.iter().any(|(pk, _)| pk == &pubkey) {
                        signatures.push((pubkey, sig));
                    }
                }
            }
        }

        // Majority check: 2k > validator_count per spec-groth16-circuit — Constraint 3
        if 2 * signatures.len() as u64 > validator_count {
            return Ok(signatures);
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    anyhow::bail!(
        "timeout: only collected {}/{} signatures before deadline",
        signatures.len(),
        validator_count
    );
}
```

---

## Handling Proof Generation Failure

Proof generation can fail due to OOM, proving key not loaded, or unsatisfiable
constraints (which indicates a circuit bug). The crate surfaces this as
`ConsensusError::ProvingError`
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Error Type).

```rust
async fn submit_checkpoint_with_retry(
    client: &Arc<Mutex<ConsensusClient>>,
    node: &FullNodeClient,
    l2_state: &Arc<L2State>,
) -> anyhow::Result<()> {

    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        attempts += 1;

        match try_submit_checkpoint(client, node, l2_state).await {
            Ok(()) => return Ok(()),

            Err(e) if e.to_string().contains("ProvingError") => {
                eprintln!("proof generation failed (attempt {}): {}", attempts, e);
                if attempts >= max_attempts {
                    anyhow::bail!("proof generation failed after {} attempts", attempts);
                }
                // Wait before retry - may be transient OOM
                tokio::time::sleep(Duration::from_secs(60)).await;
            }

            Err(e) if e.to_string().contains("BelowThreshold") => {
                // Not enough signers - do not retry, wait for more signatures
                eprintln!("insufficient signers: {}", e);
                return Ok(());
            }

            Err(e) if e.to_string().contains("SpendRejected") => {
                // Bundle rejected - sync to check state
                eprintln!("spend rejected: {} - syncing", e);
                client.lock().await.sync().await?;
                return Ok(());
            }

            Err(e) => return Err(e),
        }
    }
}
```

---

## Validator Set Transitions

New validators and exits take effect at checkpoint boundaries. The L2 is
responsible for tracking pending registrations and exits. The `compute_new_validator_set()`
method applies changes to the sparse Merkle tree
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Tree Updates):

```rust
async fn compute_checkpoint_validator_changes(
    client: &ConsensusClient,
    l2_state: &Arc<L2State>,
) -> anyhow::Result<(Vec<PublicKey>, Vec<PublicKey>)> {

    let current_validators = client.validator_set()?.pubkeys();
    let all_registration_coins = l2_state.get_all_registration_coins().await?;

    // Entries: registration coins not yet in the active set
    // These are coins whose lineage is valid (spec-indexer verified)
    // but the previous checkpoint didn't include them
    let entries: Vec<PublicKey> = all_registration_coins
        .iter()
        .filter(|(_, pk)| !current_validators.contains(pk))
        .map(|(_, pk)| *pk)
        .collect();

    // Exits: validators in the active set whose registration coins are spent
    let exits: Vec<PublicKey> = current_validators
        .iter()
        .filter(|pk| !all_registration_coins.iter().any(|(_, p)| p == *pk))
        .copied()
        .collect();

    Ok((entries, exits))
}
```

The majority of validators must sign the checkpoint message that commits to the
new `validator_merkle_root` and `new_validator_count`. This signature is itself
the trustless proof that the new validator set is correct
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Why the
validator set lives off-chain and [spec-security](spec-security.md) —
Assumption 3).

---

## Monitoring

Track these metrics to know if the system is healthy. The epoch metric is the
primary health signal — if it stops advancing, checkpoints are stalled and the
L2 is not settling state on L1.

```rust
pub struct ConsensusMetrics {
    pub epoch:              u64,
    pub validator_count:    u64,
    pub last_sync_height:   u32,
    pub pending_entries:    usize,
    pub pending_exits:      usize,
}

pub async fn collect_metrics(
    client: &ConsensusClient,
    l2_state: &Arc<L2State>,
) -> ConsensusMetrics {
    let (entries, exits) = compute_checkpoint_validator_changes(client, l2_state)
        .await
        .unwrap_or_default();

    ConsensusMetrics {
        epoch:           client.epoch().unwrap_or(0),
        validator_count: client.validator_count().unwrap_or(0),
        last_sync_height: client.synced_at().unwrap_or(0),
        pending_entries: entries.len(),
        pending_exits:   exits.len(),
    }
}
```

Alert on:
- Epoch not advancing for more than expected time: checkpoints are stalled.
  Possible causes: insufficient signers, proving key unavailable, node issues.
  Recovery: [spec-security](spec-security.md) — Checkpoint Singleton is Stuck.
- `pending_entries > 0` for more than one epoch: registrations are not being
  processed into the validator set.
- `StateMismatch` error from `sync()`: local Merkle tree does not match
  on-chain root. Trigger a full re-index per
  [spec-indexer](spec-indexer.md) — Reorg Handling: full_reindex.

---

## Important Notes

**Only one checkpoint in-flight at a time**

Do not submit a second checkpoint while one is being proved or submitted. The
epoch is read from the checkpoint singleton at proof time. If the chain
advances an epoch during proof generation, the solution will reference a stale
epoch and be rejected. Use a mutex or semaphore.

**Proof generation is blocking**

`submit_checkpoint()` runs proof generation in `spawn_blocking` but still
consumes a thread for 5–15 minutes per
[spec-groth16-circuit](spec-groth16-circuit.md) — Constraint Count Estimates.
Make sure your thread pool is large enough. Do not set a short timeout on the
`submit_checkpoint()` future.

**The crate does not submit bundles**

`submit_checkpoint()` and `recover_collateral()` return `SpendBundle` but do
not submit to the node. The L2 is responsible for submission. This keeps the
crate testable and lets the L2 inspect or modify the bundle if needed.

**Always sync before checkpoint submission**

Call `sync()` immediately before starting checkpoint submission. If the chain
state changed since your last sync, the proof inputs will be wrong. The
indexer's Merkle root consistency check
(→ see [spec-indexer](spec-indexer.md) — Merkle Root Consistency Check) will
catch most staleness issues.

**Handle the empty validator set**

At network launch the validator count is 0. The majority threshold `2k > 0`
is satisfied by k ≥ 1. Make sure your signature collection logic does not
divide by zero when computing the required majority.

**The crate does not do P2P**

Signature collection, validator communication, and checkpoint coordination are
your L2's responsibility. The crate only handles the Chia L1 interaction.
