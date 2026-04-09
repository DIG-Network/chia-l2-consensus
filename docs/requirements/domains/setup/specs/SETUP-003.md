# SETUP-003 — Project Structure

> **Authoritative requirement:** [SETUP-003](../NORMATIVE.md#SETUP-003)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [spec-consensus-crate.md](../../../../resources/spec-consensus-crate.md) — Crate Structure

## Summary

The project must follow the directory structure defined in spec-consensus-crate.md: `src/` for Rust code organized into modules, `puzzles/` for Chialisp files, and `tests/` for integration tests.

## Specification

### Directory Layout

```
chia-l2-consensus/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── lib.rs                    # Public API re-exports
│   ├── client.rs                 # ConsensusClient, main entry point
│   ├── config.rs                 # NetworkConfig, deployment parameters
│   ├── state.rs                  # NetworkState, CheckpointSingletonState
│   ├── error.rs                  # ConsensusError enum
│   ├── puzzles/
│   │   ├── mod.rs
│   │   ├── network_coin.rs       # Network coin operations
│   │   ├── registration_coin.rs  # Registration coin operations
│   │   └── checkpoint.rs         # Checkpoint singleton operations
│   ├── merkle/
│   │   ├── mod.rs
│   │   ├── sparse.rs             # SparseMerkleTree implementation
│   │   └── proof.rs              # MerkleProof type
│   ├── prover/
│   │   ├── mod.rs
│   │   ├── circuit.rs            # ConsensusCircuit definition
│   │   ├── setup.rs              # Trusted setup operations
│   │   ├── prove.rs              # Proof generation
│   │   └── serialize.rs          # CLVM serialization
│   └── indexer/
│       ├── mod.rs                # IndexerState, sync()
│       ├── chain.rs              # Raw chain queries
│       ├── validator_set.rs      # Validator set building
│       ├── reorg.rs              # Reorg handling
│       └── cache.rs              # IndexerCache
├── puzzles/
│   ├── include/                  # Chialisp include files
│   │   └── *.clib
│   ├── network_coin_inner.clsp
│   ├── registration_coin.clsp
│   └── checkpoint_inner.clsp
├── tests/
│   └── integration.rs            # End-to-end tests
└── docs/
    ├── prompt/                   # Development guidance
    ├── requirements/             # Requirements specs
    └── resources/                # Reference documents
```

### Module Responsibilities

| Module | Responsibility | Primary Types |
|--------|---------------|---------------|
| `lib.rs` | Public API exports | Re-exports ConsensusClient, types |
| `client.rs` | Orchestration | ConsensusClient |
| `config.rs` | Configuration | NetworkConfig |
| `state.rs` | State types | NetworkState, CheckpointSingletonState |
| `error.rs` | Errors | ConsensusError |
| `puzzles/` | L1 puzzle drivers | Spend bundle creation |
| `merkle/` | SMT operations | SparseMerkleTree, MerkleProof |
| `prover/` | ZK proving | ConsensusCircuit, generate_proof() |
| `indexer/` | Chain indexing | IndexerState, sync() |

### Public API Surface

Only these types should be public:
- `ConsensusClient`
- `NetworkConfig`
- `ValidatorSet`
- `SpendBundle`
- `Bytes32`
- `PublicKey`
- `ConsensusError`

## Acceptance Criteria

- [ ] All directories exist as specified
- [ ] Module structure matches spec
- [ ] Only specified types are `pub`
- [ ] Internal modules are `pub(crate)` or private
- [ ] Compiles with `cargo build`

## Implementation Notes

- Use `mod.rs` pattern for subdirectories
- Re-export public types through `lib.rs`
- Keep internal implementation details private
- Cross-cutting concerns handled within crate, not exposed

## Verification

1. Directory structure matches specification
2. `cargo doc --no-deps` shows only public API
3. `cargo build` succeeds
4. `cargo test` finds test files

## Source Citations

- [spec-consensus-crate.md Lines 113-147](../../../../resources/spec-consensus-crate.md) — Crate structure specification
- [spec-consensus-crate.md Lines 23-56](../../../../resources/spec-consensus-crate.md) — Overview and module responsibilities
- [spec-consensus-crate.md Lines 151-222](../../../../resources/spec-consensus-crate.md) — Error type (src/error.rs)
- [spec-consensus-crate.md Lines 226-315](../../../../resources/spec-consensus-crate.md) — Configuration (src/config.rs)
- [spec-consensus-crate.md Lines 319-414](../../../../resources/spec-consensus-crate.md) — State types (src/state.rs)
- [spec-consensus-crate.md Lines 414-664](../../../../resources/spec-consensus-crate.md) — Merkle module (src/merkle/)
- [spec-consensus-crate.md Lines 664-929](../../../../resources/spec-consensus-crate.md) — Serialization module (src/prover/serialize.rs)
- [spec-consensus-crate.md Lines 929-1026](../../../../resources/spec-consensus-crate.md) — Prover module (src/prover/)
- [spec-consensus-crate.md Lines 1026-1274](../../../../resources/spec-consensus-crate.md) — Puzzle modules (src/puzzles/)
- [spec-consensus-crate.md Lines 1274-1579](../../../../resources/spec-consensus-crate.md) — Indexer module (src/indexer/)
- [spec-consensus-crate.md Lines 1579-2154](../../../../resources/spec-consensus-crate.md) — Public interface ConsensusClient
- [spec-consensus-crate.md Lines 2154-2185](../../../../resources/spec-consensus-crate.md) — Public re-exports (src/lib.rs)
- [spec-consensus-crate.md Lines 2185-2293](../../../../resources/spec-consensus-crate.md) — Integration test structure

## References

- [SETUP-002](SETUP-002.md) — Cargo.toml configuration
- [SMT-001](../../smt/specs/SMT-001.md) — SMT in merkle/ module
- [CIR-001](../../circuit/specs/CIR-001.md) — Circuit in prover/ module
- [IDX-001](../../indexer/specs/IDX-001.md) — Indexer module
