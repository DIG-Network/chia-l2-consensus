# SETUP-005 — Rue Tooling

> **Authoritative requirement:** [SETUP-005](../NORMATIVE.md#SETUP-005)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md)

## Summary

Developers must have Rue tooling installed (`rue` compiler from [rue-lang.dev](https://rue-lang.dev/)) for compiling puzzles. Compiled CLVM output goes to `puzzles/compiled/`.

## Specification

### Required Tools

| Tool | Purpose | Installation |
|------|---------|--------------|
| `rue` | Compile Rue to CLVM bytecode | See [rue-lang.dev](https://rue-lang.dev/) |

### Installation

```bash
# Install Rue compiler (see https://rue-lang.dev/ for current instructions)
# The rue compiler is available via cargo or as a binary release

# Verify installation
rue --version
```

### Project Puzzle Files

| File | Purpose |
|------|---------|
| `puzzles/network_coin_inner.rue` | Network coin inner puzzle |
| `puzzles/registration_coin.rue` | Registration coin puzzle |
| `puzzles/checkpoint_inner.rue` | Checkpoint singleton inner puzzle |
| `puzzles/compiled/` | Compiled CLVM output directory |

### Compilation

```bash
# Compile puzzle to CLVM
rue build puzzles/checkpoint_inner.rue -o puzzles/compiled/checkpoint_inner.clvm

# Compile all puzzles
rue build puzzles/*.rue -o puzzles/compiled/
```

## Acceptance Criteria

- [ ] `rue --version` returns version info
- [ ] Puzzles compile to CLVM
- [ ] `puzzles/compiled/` contains compiled output
- [ ] Compiled puzzles can be executed with test solutions

## Implementation Notes

- Rue is a typed language that compiles to CLVM
- Rue syntax resembles Rust (structs, functions, types)
- All on-chain puzzles MUST be written in Rue per dt-hard-rules.md
- Cross-implementation consistency requires Rust and Rue to produce identical outputs

### Development Workflow

```bash
# Compile puzzle
rue build puzzles/checkpoint_inner.rue -o puzzles/compiled/checkpoint_inner.clvm

# Test with chia-dev-tools brun
brun "$(cat puzzles/compiled/checkpoint_inner.clvm)" "(solution args here)"
```

### Cross-Implementation Testing

Ensure CLVM execution matches Rust implementation:

```bash
# Run puzzle and capture output
OUTPUT=$(brun "$(cat puzzles/compiled/checkpoint_inner.clvm)" "(solution)")

# Compare with Rust test
cargo test clvm_parity -- --nocapture
```

## Verification

1. `rue --version` succeeds
2. `rue build puzzles/checkpoint_inner.rue` compiles
3. Compiled output exists in `puzzles/compiled/`
4. `brun` executes compiled puzzle with test solution
5. Tree hash matches expected value

## Source Citations

- [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md) — Puzzle specifications
- [spec-checkpoint-singleton.md](../../../../resources/spec-checkpoint-singleton.md) — Checkpoint puzzle (Rue source)
- [spec-registration-coin.md](../../../../resources/spec-registration-coin.md) — Registration puzzle (Rue source)

## References

- [SETUP-001](SETUP-001.md) — Rust toolchain (for CLVM parity tests)
- [CHK-001](../../checkpoint/specs/CHK-001.md) — Checkpoint puzzle requirements
- [REG-001](../../registration_coin/specs/REG-001.md) — Registration puzzle requirements
- [NET-001](../../network_coin/specs/NET-001.md) — Network coin puzzle requirements
