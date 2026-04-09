# Indexer — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Part 4: Off-Chain Validator Set Construction

---

## §1 State Tracking

<a id="IDX-001"></a>**IDX-001** The indexer MUST track: network coin state, checkpoint singleton state (epoch, roots, count), all valid registration coins keyed by pubkey, and checkpoint history.
> **Spec:** [`IDX-001.md`](../../../design/requirements/indexer/IDX-001.md)

---

## §2 Lineage Verification

<a id="IDX-002"></a>**IDX-002** The indexer MUST verify each registration coin's lineage by confirming its parent coin ID is in the set of valid network coin spend IDs; coins failing this check MUST be ignored.
> **Spec:** [`IDX-002.md`](../../../design/requirements/indexer/IDX-002.md)

---

## §3 Merkle Consistency

<a id="IDX-003"></a>**IDX-003** After every sync, the indexer MUST rebuild the sparse Merkle tree from registration coins and verify the computed root matches the on-chain `validator_merkle_root`; mismatches MUST return `StateMismatch` error.
> **Spec:** [`IDX-003.md`](../../../design/requirements/indexer/IDX-003.md)

---

## §4 Reorg Handling

<a id="IDX-004"></a>**IDX-004** On blockchain reorganization, the indexer MUST roll back to the last safe checkpoint before the reorg point and re-index forward; if no safe point exists, full re-index from genesis is required.
> **Spec:** [`IDX-004.md`](../../../design/requirements/indexer/IDX-004.md)

---

## §5 Persistent Cache

<a id="IDX-005"></a>**IDX-005** The indexer SHOULD maintain a persistent cache (JSON file) using atomic writes to enable fast restarts without full re-indexing.
> **Spec:** [`IDX-005.md`](../../../design/requirements/indexer/IDX-005.md)
