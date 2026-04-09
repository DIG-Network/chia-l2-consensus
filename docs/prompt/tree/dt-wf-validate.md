# Workflow — validate

## By component

| Component | Validation |
|-----------|------------|
| Rust code | `cargo test`, `cargo clippy`, `cargo fmt --check` |
| Chialisp | CLVM tests, `run` / `brun` verification |
| Circuit | Constraint satisfaction, proof generation/verification |
| Wire format | Test vectors in both Rust and Chialisp |

## Cross-implementation tests

**Critical:** SMT, wire format, and hash computations must produce identical results in Rust and Chialisp.

```bash
# Run all Rust tests
cargo test

# Run cross-impl tests specifically
cargo test cross_impl

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

## Test vector verification

Each wire format and hash computation should have:
1. Known input values
2. Expected output (computed manually or from reference implementation)
3. Test in Rust: `assert_eq!(compute(...), expected)`
4. Test in Chialisp: `(= (compute ...) expected)`

## Circuit validation

1. Generate proof with valid witness → success
2. Generate proof with invalid witness → failure
3. Verify proof with correct public inputs → success
4. Verify proof with wrong public inputs → failure

Fix all failures before **`dt-wf-update-tracking.md`**.

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-wf-implement.md`](dt-wf-implement.md) |
| **Next** | [`dt-wf-update-tracking.md`](dt-wf-update-tracking.md) |

*Back to [`tree/README.md`](README.md).*
