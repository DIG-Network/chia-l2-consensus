# Setup — Normative Requirements

> **Master spec:** [spec-consensus-crate.md](../../../resources/spec-consensus-crate.md) — Project Setup

---

## §1 Rust Toolchain

<a id="SETUP-001"></a>**SETUP-001** The project MUST use Rust 2021 edition with stable toolchain. Developers MUST have `rustfmt` and `clippy` installed for code formatting and linting.
> **Spec:** [`SETUP-001.md`](specs/SETUP-001.md)

---

## §2 Cargo.toml Configuration

<a id="SETUP-002"></a>**SETUP-002** The crate MUST be named `chia-l2-consensus` with version `0.1.0` and edition `2021`. All dependencies MUST be pinned to specific versions to ensure reproducible builds.
> **Spec:** [`SETUP-002.md`](specs/SETUP-002.md)

---

## §3 Project Structure

<a id="SETUP-003"></a>**SETUP-003** The project MUST follow the directory structure defined in spec-consensus-crate.md: `src/` for Rust code organized into modules (client, puzzles, merkle, prover, indexer), `puzzles/` for Rue source files (`.rue`), and `tests/` for integration tests.
> **Spec:** [`SETUP-003.md`](specs/SETUP-003.md)

---

## §4 Core Dependencies

<a id="SETUP-004"></a>**SETUP-004** The project MUST include all required dependencies: arkworks crates (0.4.x) for ZK proofs, chia-wallet-sdk/clvmr for Chia integration, blst for BLS aggregation, and standard utilities (tokio, serde, sha2).
> **Spec:** [`SETUP-004.md`](specs/SETUP-004.md)

---

## §5 Rue Tooling

<a id="SETUP-005"></a>**SETUP-005** Developers MUST have Rue tooling installed (`rue` compiler from [rue-lang.dev](https://rue-lang.dev/)) for compiling puzzles. The `rue` command MUST be available in PATH. Compiled CLVM output goes to `puzzles/compiled/`.
> **Spec:** [`SETUP-005.md`](specs/SETUP-005.md)

---

## §6 Build Configuration

<a id="SETUP-006"></a>**SETUP-006** Release builds MUST use optimization level 3 with LTO enabled. The project MUST pass `cargo clippy -- -D warnings` and `cargo fmt --check` before commits.
> **Spec:** [`SETUP-006.md`](specs/SETUP-006.md)
