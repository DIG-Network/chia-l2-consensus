# SETUP-001 — Rust Toolchain

> **Authoritative requirement:** [SETUP-001](../NORMATIVE.md#SETUP-001)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [spec-consensus-crate.md](../../../../resources/spec-consensus-crate.md)

## Summary

The project requires Rust 2021 edition with stable toolchain. Developers must have `rustfmt` and `clippy` installed for code formatting and linting.

## Specification

### Required Toolchain

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | stable 1.70+ | Compilation |
| Cargo | bundled | Build system |
| rustfmt | bundled | Code formatting |
| clippy | bundled | Linting |

### Installation

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ensure stable toolchain
rustup default stable

# Install components
rustup component add rustfmt
rustup component add clippy
```

### Verification Commands

```bash
# Check Rust version
rustc --version
# Expected: rustc 1.70.0 or higher

# Check cargo
cargo --version

# Check rustfmt
rustfmt --version

# Check clippy
cargo clippy --version
```

## Acceptance Criteria

- [ ] Rust stable 1.70+ installed
- [ ] `rustfmt` component available
- [ ] `clippy` component available
- [ ] `cargo build` runs without toolchain errors

## Implementation Notes

- Use `rustup` for toolchain management
- CI should pin a specific Rust version via `rust-toolchain.toml`
- Windows: use MSVC toolchain for best compatibility

### Optional: rust-toolchain.toml

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

## Verification

1. Run `rustc --version` — should show 1.70.0+
2. Run `rustfmt --version` — should be available
3. Run `cargo clippy --version` — should be available
4. Run `cargo build` on empty project — should succeed

## Source Citations

- [spec-consensus-crate.md Lines 60-109](../../../../resources/spec-consensus-crate.md) — Cargo.toml specifying edition = "2021"

## References

- [SETUP-002](SETUP-002.md) — Cargo.toml configuration
- [SETUP-006](SETUP-006.md) — Build configuration with clippy/fmt checks
