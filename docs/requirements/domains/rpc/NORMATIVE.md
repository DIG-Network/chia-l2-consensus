# RPC Integration — Normative Requirements

> **Master spec:** [spec-consensus-crate.md](../../../resources/spec-consensus-crate.md)

These requirements track the 16 `todo!()` stubs that require Chia full node
RPC infrastructure. They are Phase 8 — deferred until the RPC client layer
is built.

---

<a id="RPC-001"></a>**RPC-001** A Chia full node RPC client MUST be implemented or integrated to enable on-chain state queries (coin lookups, puzzle hash queries, spend submission).
> **Spec:** [`RPC-001.md`](specs/RPC-001.md)

<a id="RPC-002"></a>**RPC-002** All puzzle driver spend bundle construction stubs (8 `todo!()` in `src/puzzles/*.rs`) MUST be implemented using chia-wallet-sdk `SpendContext`.
> **Spec:** [`RPC-002.md`](specs/RPC-002.md)

<a id="RPC-003"></a>**RPC-003** The indexer sync algorithm (3 `todo!()` in `src/indexer/`) MUST be implemented to fetch coins, verify lineage, and build the Merkle tree.
> **Spec:** [`RPC-003.md`](specs/RPC-003.md)

<a id="RPC-004"></a>**RPC-004** The ConsensusClient operation methods (5 `todo!()` in `src/client.rs`) MUST be implemented to coordinate puzzle drivers, prover, and indexer.
> **Spec:** [`RPC-004.md`](specs/RPC-004.md)
