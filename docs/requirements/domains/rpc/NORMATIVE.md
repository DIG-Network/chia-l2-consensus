# RPC Integration — Normative Requirements

> **Master spec:** [spec-consensus-crate.md](../../../resources/spec-consensus-crate.md)

These requirements implement the 16 `todo!()` stubs using `chia-query` for
blockchain queries and `dig-l1-wallet` for coin selection and fee estimation.

**Dependency crates:**
- [`chia-query`](https://crates.io/crates/chia-query) — decentralized Chia
  blockchain queries via peer network with coinset.org fallback
- [`dig-l1-wallet`](https://crates.io/crates/dig-l1-wallet) — self-custodial
  wallet with coin selection and transaction support

---

## §1 Blockchain Query Client

<a id="RPC-001"></a>**RPC-001** The crate MUST use `chia-query::ChiaQuery` as the blockchain query client, configured via `ChiaQueryConfig` with network type matching deployment (mainnet/testnet11). The `ChiaQuery` instance MUST be shared across the indexer and all puzzle drivers. All chain reads go through `ChiaQuery`; the crate MUST NOT open direct peer connections or implement custom RPC.
> **Spec:** [`RPC-001.md`](specs/RPC-001.md)

---

## §2 Puzzle Driver Spend Bundle Construction

<a id="RPC-002"></a>**RPC-002** All puzzle driver spend bundle construction stubs (8 `todo!()` in `src/puzzles/*.rs`) MUST be implemented using `chia-wallet-sdk::SpendContext` for CLVM allocation and `chia-query::ChiaQuery` for coin state queries (singleton lookups, lineage proofs). The drivers MUST return `SpendBundle` values without broadcasting (per API-008).
> **Spec:** [`RPC-002.md`](specs/RPC-002.md)

---

## §3 Indexer Sync Algorithm

<a id="RPC-003"></a>**RPC-003** The indexer sync algorithm (4 `todo!()` in `src/indexer/`) MUST be implemented using `ChiaQuery::get_coin_records_by_puzzle_hash` for registration coin discovery, `ChiaQuery::get_puzzle_and_solution` for lineage verification, and `ChiaQuery::get_blockchain_state` for peak height and reorg detection.
> **Spec:** [`RPC-003.md`](specs/RPC-003.md)

---

## §4 ConsensusClient Coordination

<a id="RPC-004"></a>**RPC-004** The ConsensusClient operation methods (5 `todo!()` in `src/client.rs`) MUST be implemented to coordinate puzzle drivers, prover, and indexer. All methods MUST call `sync()` internally or require it to have been called. All methods MUST return `SpendBundle` without broadcasting (per API-008).
> **Spec:** [`RPC-004.md`](specs/RPC-004.md)

---

## §5 Wallet Integration for Collateral Funding

<a id="RPC-005"></a>**RPC-005** For validator registration, the crate MUST use `dig-l1-wallet` coin selection (`select_coins` with `CoinSelectionStrategy`) to fund the collateral amount from the validator's wallet. The selected coins MUST be included in the registration spend bundle alongside the network coin spend. Fee estimation SHOULD use `dig-l1-wallet::estimate_fee()`.
> **Spec:** [`RPC-005.md`](specs/RPC-005.md)

---

## §6 Dependency Version Alignment

<a id="RPC-006"></a>**RPC-006** `Cargo.toml` MUST add `chia-query` and `dig-l1-wallet` as dependencies. The `chia-wallet-sdk`, `chia-protocol`, and `clvmr` versions MUST be updated to align with the versions used by `chia-query` and `dig-l1-wallet` to avoid type mismatches across crate boundaries.
> **Spec:** [`RPC-006.md`](specs/RPC-006.md)
