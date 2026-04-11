# Crate API — Normative Requirements

> **Master spec:** [spec-consensus-crate.md](../../../resources/spec-consensus-crate.md)

These requirements ensure the crate conforms to the public API specified in
spec-consensus-crate.md so that L2 systems can import it as a dependency.

---

## §1 Public API Surface

<a id="API-001"></a>**API-001** The crate MUST export only the types listed in spec-consensus-crate.md Lines 2154-2181 as the public API for L2 consumers. Internal implementation functions used by VV tests MUST be re-exported via a `pub mod testing` module, not directly from `lib.rs`.
> **Spec:** [`API-001.md`](specs/API-001.md)

---

## §2 NetworkConfig Completeness

<a id="API-002"></a>**API-002** `NetworkConfig` MUST derive `serde::Serialize` and `serde::Deserialize`, and MUST provide the `verification_key()` method (returns `VerifyingKey<Bls12_381>`) and `checkpoint_singleton_id()` method (derives coin ID from launcher) as defined in spec Lines 298-314.
> **Spec:** [`API-002.md`](specs/API-002.md)

---

## §3 State Types

<a id="API-003"></a>**API-003** The crate MUST define `NetworkState`, `ValidatorInfo`, and update `ValidatorSet` to use `ValidatorInfo` with helper methods `count()`, `contains()`, `pubkeys()`. `NetworkCoinState` and `CheckpointSingletonState` MUST include `lineage_proof` fields as specified in Lines 319-409.
> **Spec:** [`API-003.md`](specs/API-003.md)

---

## §4 ConsensusClient State Accessors

<a id="API-004"></a>**API-004** `ConsensusClient` MUST provide the state accessor methods defined in spec Lines 2129-2149: `epoch()`, `state_root()`, `validator_merkle_root()`, `validator_count()`, `synced_at()`, plus `set_cache_path()` and `load_proving_key()`.
> **Spec:** [`API-004.md`](specs/API-004.md)

---

## §5 ConsensusClient Message Computation

<a id="API-005"></a>**API-005** `ConsensusClient` MUST provide the message computation methods defined in spec Lines 1903-1960: `checkpoint_message()`, `validator_signing_message()`, `compute_new_validator_set()`, `membership_announcement()`, and `is_active()`.
> **Spec:** [`API-005.md`](specs/API-005.md)

---

## §6 Module Visibility

<a id="API-006"></a>**API-006** Internal modules (`prover`, `puzzles`, `validator`) MUST be `pub(crate)`. The `merkle` and `indexer` modules MUST be `pub(crate)` with selected types re-exported through `lib.rs` per spec Lines 24-55. Test-only exports MUST use `pub mod testing`.
> **Spec:** [`API-006.md`](specs/API-006.md)
