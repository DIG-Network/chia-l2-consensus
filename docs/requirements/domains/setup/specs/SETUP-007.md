# SETUP-007 — Automated Verification Tests

> **Authoritative requirement:** [SETUP-007](../NORMATIVE.md#SETUP-007)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)

## Summary

SETUP-001 (Rust toolchain) and SETUP-002 (Cargo.toml) have no automated test
files. While these are environment/configuration requirements, automated checks
ensure CI catches regressions.

## Specification

### vv_req_setup_001.rs (Rust toolchain)
- Verify `rustc --version` succeeds and returns a version
- Verify `cargo --version` succeeds
- Verify `rustfmt --version` succeeds
- Verify `cargo clippy --version` succeeds
- Parse rust-toolchain.toml and verify it specifies stable channel

### vv_req_setup_002.rs (Cargo.toml)
- Parse Cargo.toml via `toml` crate
- Verify `package.name` = "chia-l2-consensus"
- Verify `package.edition` = "2021"
- Verify key dependencies exist (arkworks, chia-wallet-sdk, clvmr, blst)
- Verify dependencies are pinned (no wildcard versions)

## Acceptance Criteria

- [x] vv_req_setup_001.rs exists with ≥3 toolchain checks — 5 tests (rustc, cargo, rustfmt, clippy, edition)
- [x] vv_req_setup_002.rs exists with ≥4 Cargo.toml checks — 9 tests (name, version, edition, deps, no wildcards, LTO)
- [x] Both files pass in CI — 14/14 pass

## References

- [SETUP-001](SETUP-001.md), [SETUP-002](SETUP-002.md) — Requirements being tested
