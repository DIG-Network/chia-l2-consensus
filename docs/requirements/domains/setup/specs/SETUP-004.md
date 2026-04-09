# SETUP-004 — Core Dependencies

> **Authoritative requirement:** [SETUP-004](../NORMATIVE.md#SETUP-004)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [spec-consensus-crate.md](../../../../resources/spec-consensus-crate.md) — Cargo.toml

## Summary

The project must include all required dependencies: arkworks crates (0.4.x) for ZK proofs, chia-wallet-sdk/clvmr for Chia integration, blst for BLS aggregation, and standard utilities.

## Specification

### Chia Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `chia-wallet-sdk` | 0.18 | Wallet operations, spend bundle creation |
| `chia-protocol` | 0.18 | Protocol types (Coin, CoinSpend) |
| `chia-puzzles` | 0.18 | Standard puzzles (singleton, CAT) |
| `clvm-traits` | 0.18 | CLVM type conversions |
| `clvmr` | 0.6 | CLVM runtime for puzzle execution |

### Arkworks ZK Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ark-groth16` | 0.4 | Groth16 proving system |
| `ark-bls12-381` | 0.4 | BLS12-381 curve implementation |
| `ark-r1cs-std` | 0.4 | R1CS gadgets for circuit building |
| `ark-relations` | 0.4 | R1CS constraint system |
| `ark-ff` | 0.4 | Finite field arithmetic |
| `ark-ec` | 0.4 | Elliptic curve operations |
| `ark-std` | 0.4 | Standard utilities |
| `ark-serialize` | 0.4 | Serialization for proof elements |
| `ark-crypto-primitives` | 0.4 | SHA256 gadget (with `crh` feature) |

### Other Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `blst` | 0.3 | Off-chain BLS signature aggregation |
| `tokio` | 1.x | Async runtime |
| `futures` | 0.3 | Async utilities |
| `serde` | 1.x | Serialization framework |
| `serde_json` | 1.x | JSON serialization |
| `hex` | 0.4 | Hex encoding |
| `num-bigint` | 0.4 | Big integer arithmetic |
| `thiserror` | 1.x | Error derive macros |
| `anyhow` | 1.x | Error context |
| `sha2` | 0.10 | SHA-256 hashing |

### Dev Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `rand` | 0.8 | Random number generation for tests |
| `tokio` | 1.x | Async test runtime |

## Acceptance Criteria

- [ ] All Chia crates resolve at 0.18
- [ ] All arkworks crates resolve at 0.4
- [ ] blst 0.3 available (requires C compiler)
- [ ] tokio with "full" features enabled
- [ ] `cargo build` succeeds without version conflicts
- [ ] `cargo tree` shows consistent dependency versions

## Implementation Notes

- Keep all arkworks crates at the same version (0.4)
- Keep all chia crates at the same version (0.18)
- blst requires a C compiler (MSVC on Windows, clang/gcc on Unix)
- tokio "full" features needed for runtime, sync, time, io

### Build Requirements

```bash
# Unix: ensure C compiler available
gcc --version || clang --version

# Windows: ensure MSVC installed
# Visual Studio Build Tools with C++ workload
```

## Verification

1. `cargo check` resolves all dependencies
2. `cargo tree` shows no duplicate versions
3. `cargo build` compiles without errors
4. Test imports work:

```rust
use ark_groth16::Groth16;
use ark_bls12_381::Bls12_381;
use chia_wallet_sdk::SpendBundle;
use clvmr::Allocator;
use blst::min_pk::SecretKey;
```

## Source Citations

- [spec-consensus-crate.md Lines 60-109](../../../../resources/spec-consensus-crate.md) — Complete Cargo.toml
- [spec-consensus-crate.md Lines 68-75](../../../../resources/spec-consensus-crate.md) — Chia dependencies with versions
- [spec-consensus-crate.md Lines 77-85](../../../../resources/spec-consensus-crate.md) — Arkworks ZK dependencies (0.4.x)
- [spec-consensus-crate.md Lines 88](../../../../resources/spec-consensus-crate.md) — blst for BLS aggregation
- [spec-consensus-crate.md Lines 91-103](../../../../resources/spec-consensus-crate.md) — Async, serialization, error handling deps
- [spec-consensus-crate.md Lines 106-108](../../../../resources/spec-consensus-crate.md) — Dev dependencies
- [spec-groth16-circuit.md](../../../../resources/spec-groth16-circuit.md) — arkworks usage in circuit
- [spec-wire-format.md](../../../../resources/spec-wire-format.md) — blst for BLS aggregation

## References

- [SETUP-002](SETUP-002.md) — Cargo.toml configuration
- [CIR-001](../../circuit/specs/CIR-001.md) — Circuit using arkworks
- [WIRE-005](../../wire/specs/WIRE-005.md) — BLS aggregation using blst
