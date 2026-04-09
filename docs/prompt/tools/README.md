# Tools — chia-l2-consensus

## Development tools

| Tool | Purpose | Usage |
|------|---------|-------|
| `cargo` | Rust build/test | `cargo build`, `cargo test`, `cargo clippy` |
| `clvm_rs` | CLVM execution | Rust library for running Chialisp |
| `run` / `brun` | Chialisp REPL | Testing puzzle logic |
| `arkworks` | ZK proofs | Groth16 circuit implementation |

## Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test smt::

# Run with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

## Chialisp tools

```bash
# Compile puzzle
run -i puzzles/include puzzles/checkpoint_inner.clsp

# Run with solution
brun <compiled> <solution>

# Get tree hash
run -i puzzles/include --dump-tree-hash puzzles/checkpoint_inner.clsp
```

## Circuit development

```bash
# Generate test proof
cargo run --bin prove -- --input witness.json --output proof.bin

# Verify proof
cargo run --bin verify -- --proof proof.bin --public-inputs inputs.json

# Constraint count
cargo run --bin constraint-count
```

## Cross-implementation testing

Ensure Rust and Chialisp produce identical results:

1. **Hash computations** — Same SHA-256 outputs
2. **SMT operations** — Same slot, leaf, root calculations
3. **Wire format** — Same byte sequences
4. **VK input** — Same scalar() and G1 point math

Test vectors should be defined once and verified in both implementations.
