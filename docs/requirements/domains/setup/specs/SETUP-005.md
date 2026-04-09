# SETUP-005 — Chialisp Tooling

> **Authoritative requirement:** [SETUP-005](../NORMATIVE.md#SETUP-005)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md)

## Summary

Developers must have Chialisp tooling installed (`run`, `brun`) for compiling and testing puzzles. Include paths must be configured for `puzzles/include/`.

## Specification

### Required Tools

| Tool | Purpose | Installation |
|------|---------|--------------|
| `run` | Compile Chialisp to CLVM bytecode | `pip install chia-dev-tools` |
| `brun` | Execute CLVM bytecode with solution | `pip install chia-dev-tools` |
| `cdv` | Chia dev tools CLI | `pip install chia-dev-tools` |

### Installation

```bash
# Install chia-dev-tools (includes run, brun, cdv)
pip install chia-dev-tools

# Verify installation
run --version
brun --version
cdv --version
```

### Include Path Configuration

Puzzles use include files from `puzzles/include/`:

```bash
# Compile with include path
run -i puzzles/include puzzles/checkpoint_inner.clsp

# Get tree hash
run -i puzzles/include --dump-tree-hash puzzles/checkpoint_inner.clsp
```

### Project Puzzle Files

| File | Purpose |
|------|---------|
| `puzzles/network_coin_inner.clsp` | Network coin inner puzzle |
| `puzzles/registration_coin.clsp` | Registration coin puzzle |
| `puzzles/checkpoint_inner.clsp` | Checkpoint singleton inner puzzle |
| `puzzles/include/*.clib` | Shared include files |

### Standard Include Files

| Include | Purpose |
|---------|---------|
| `condition_codes.clib` | Standard condition codes |
| `curry-and-treehash.clib` | Curry and tree hash functions |
| `sha256tree.clib` | SHA256 tree hashing |
| `smt.clib` | SMT verification macros (project-specific) |
| `groth16.clib` | Groth16 verification (project-specific) |

## Acceptance Criteria

- [ ] `run --version` returns version info
- [ ] `brun --version` returns version info
- [ ] Puzzles compile with `-i puzzles/include`
- [ ] Tree hashes can be computed
- [ ] Puzzles execute with test solutions

## Implementation Notes

- Use Python 3.8+ for chia-dev-tools
- Consider using a virtual environment
- Include files are Chialisp libraries (`.clib`)
- Compiled output is CLVM hex bytecode

### Development Workflow

```bash
# Compile puzzle
run -i puzzles/include puzzles/checkpoint_inner.clsp > checkpoint.clvm

# Test with solution
brun "$(cat checkpoint.clvm)" "(solution args here)"

# Get puzzle hash for currying
run -i puzzles/include --dump-tree-hash puzzles/checkpoint_inner.clsp
```

### Cross-Implementation Testing

Ensure CLVM execution matches Rust implementation:

```bash
# Run puzzle and capture output
OUTPUT=$(brun "$(run -i puzzles/include puzzle.clsp)" "(solution)")

# Compare with Rust test
cargo test clvm_parity -- --nocapture
```

## Verification

1. `run --version` succeeds
2. `brun --version` succeeds
3. `run -i puzzles/include puzzles/checkpoint_inner.clsp` compiles
4. `brun` executes compiled puzzle with test solution
5. Tree hash matches expected value

## Source Citations

- [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md) — Puzzle specifications
- [spec-checkpoint-singleton.md](../../../../resources/spec-checkpoint-singleton.md) — Checkpoint puzzle
- [spec-registration-coin.md](../../../../resources/spec-registration-coin.md) — Registration puzzle

## References

- [SETUP-001](SETUP-001.md) — Rust toolchain (for CLVM parity tests)
- [CHK-001](../../checkpoint/specs/CHK-001.md) — Checkpoint puzzle requirements
- [REG-001](../../registration_coin/specs/REG-001.md) — Registration puzzle requirements
- [NET-001](../../network_coin/specs/NET-001.md) — Network coin puzzle requirements
