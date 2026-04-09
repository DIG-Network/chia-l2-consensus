# SETUP-006 — Build Configuration

> **Authoritative requirement:** [SETUP-006](../NORMATIVE.md#SETUP-006)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [spec-consensus-crate.md](../../../../resources/spec-consensus-crate.md)

## Summary

Release builds must use optimization level 3 with LTO enabled. The project must pass `cargo clippy -- -D warnings` and `cargo fmt --check` before commits.

## Specification

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

| Setting | Value | Reason |
|---------|-------|--------|
| `opt-level` | 3 | Maximum optimization for proof generation |
| `lto` | true | Link-time optimization for smaller binary |
| `codegen-units` | 1 | Better optimization at cost of compile time |

### Code Quality Checks

All code must pass these checks before commit:

```bash
# Format check (no changes needed)
cargo fmt --check

# Lint check (no warnings)
cargo clippy -- -D warnings

# Test suite passes
cargo test
```

### CI Configuration

```yaml
# .github/workflows/ci.yml
jobs:
  check:
    steps:
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

### Pre-commit Hook (Optional)

```bash
#!/bin/sh
# .git/hooks/pre-commit

cargo fmt --check || exit 1
cargo clippy -- -D warnings || exit 1
cargo test || exit 1
```

## Acceptance Criteria

- [ ] `cargo fmt --check` passes (no formatting issues)
- [ ] `cargo clippy -- -D warnings` passes (no warnings)
- [ ] `cargo test` passes
- [ ] `cargo build --release` uses specified profile
- [ ] Release binary is optimized

## Implementation Notes

- Debug builds use default profile (faster compile)
- Release builds prioritize runtime performance
- LTO significantly increases compile time
- Consider `[profile.dev]` settings for faster iteration

### Development Profile (Optional)

```toml
[profile.dev]
opt-level = 1  # Faster iteration
debug = true   # Debug symbols
```

### Clippy Configuration

```toml
# Cargo.toml or .clippy.toml
[lints.clippy]
pedantic = "warn"
```

Or via command line:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Verification

1. `cargo fmt --check` returns success (exit 0)
2. `cargo clippy -- -D warnings` returns success
3. `cargo test` all tests pass
4. `cargo build --release` completes
5. Binary size is reasonable (LTO working)

### Verification Script

```bash
#!/bin/bash
set -e

echo "Checking format..."
cargo fmt --check

echo "Running clippy..."
cargo clippy -- -D warnings

echo "Running tests..."
cargo test

echo "Building release..."
cargo build --release

echo "All checks passed!"
```

## Source Citations

- [spec-consensus-crate.md](../../../../resources/spec-consensus-crate.md) — Crate configuration
- [quick-reference.md](../../../../resources/quick-reference.md) — Development workflow

## References

- [SETUP-001](SETUP-001.md) — Rust toolchain with rustfmt and clippy
- [SETUP-002](SETUP-002.md) — Cargo.toml configuration
