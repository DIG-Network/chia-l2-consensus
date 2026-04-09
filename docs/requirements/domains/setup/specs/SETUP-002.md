# SETUP-002 — Cargo.toml Configuration

> **Authoritative requirement:** [SETUP-002](../NORMATIVE.md#SETUP-002)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [spec-consensus-crate.md](../../../../resources/spec-consensus-crate.md) — Cargo.toml

## Summary

The crate must be named `chia-l2-consensus` with version `0.1.0` and edition `2021`. All dependencies must be pinned to specific versions for reproducible builds.

## Specification

### Package Metadata

```toml
[package]
name    = "chia-l2-consensus"
version = "0.1.0"
edition = "2021"
```

### Dependency Categories

| Category | Crates | Purpose |
|----------|--------|---------|
| Chia | chia-wallet-sdk, chia-protocol, chia-puzzles, clvm-traits, clvmr | Chia blockchain integration |
| ZK | ark-groth16, ark-bls12-381, ark-r1cs-std, ark-relations, ark-ff, ark-ec, ark-std, ark-serialize, ark-crypto-primitives | Groth16 proving |
| BLS | blst | BLS signature aggregation |
| Async | tokio, futures | Async runtime |
| Serialization | serde, serde_json, hex, num-bigint | Data serialization |
| Error | thiserror, anyhow | Error handling |
| Crypto | sha2 | SHA-256 hashing |

### Complete Cargo.toml

```toml
[package]
name    = "chia-l2-consensus"
version = "0.1.0"
edition = "2021"

[dependencies]
# Chia
chia-wallet-sdk = "0.18"
chia-protocol   = "0.18"
chia-puzzles    = "0.18"
clvm-traits     = "0.18"
clvmr           = "0.6"

# ZK proving
ark-groth16           = "0.4"
ark-bls12-381         = "0.4"
ark-r1cs-std          = "0.4"
ark-relations         = "0.4"
ark-ff                = "0.4"
ark-ec                = "0.4"
ark-std               = "0.4"
ark-serialize         = "0.4"
ark-crypto-primitives = { version = "0.4", features = ["crh"] }

# BLS aggregation
blst = "0.3"

# Async
tokio   = { version = "1", features = ["full"] }
futures = "0.3"

# Serialization
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
hex          = "0.4"
num-bigint   = "0.4"

# Error handling
thiserror = "1"
anyhow    = "1"

sha2 = "0.10"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
rand  = "0.8"
```

## Acceptance Criteria

- [ ] Package name is `chia-l2-consensus`
- [ ] Version is `0.1.0`
- [ ] Edition is `2021`
- [ ] All dependencies pinned to major.minor versions
- [ ] `cargo build` resolves all dependencies

## Implementation Notes

- Keep dependency versions aligned (all arkworks at 0.4, all chia at 0.18)
- Use workspace inheritance if splitting into multiple crates later
- Lock file (Cargo.lock) should be committed for reproducibility

## Verification

1. Cargo.toml exists at project root
2. Package metadata matches specification
3. `cargo check` resolves all dependencies
4. `cargo tree` shows no conflicting versions

## Source Citations

- [spec-consensus-crate.md Lines 60-109](../../../../resources/spec-consensus-crate.md) — Complete Cargo.toml with all dependencies
- [spec-consensus-crate.md Lines 68-75](../../../../resources/spec-consensus-crate.md) — Chia dependencies (chia-wallet-sdk, clvmr)
- [spec-consensus-crate.md Lines 77-85](../../../../resources/spec-consensus-crate.md) — Arkworks ZK dependencies
- [spec-consensus-crate.md Lines 88-103](../../../../resources/spec-consensus-crate.md) — Utility dependencies (tokio, serde, blst)

## References

- [SETUP-001](SETUP-001.md) — Rust toolchain requirements
- [SETUP-004](SETUP-004.md) — Core dependencies detail
- [CIR-001](../../circuit/specs/CIR-001.md) — Circuit using arkworks
