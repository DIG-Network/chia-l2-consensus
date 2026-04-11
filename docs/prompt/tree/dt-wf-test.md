# Workflow — write the test first

Write a **failing** test before implementation.

Before choosing the exact test shape, read the active requirement's dedicated spec file. The spec's verification guidance is the primary authority for what must be proven complete.

## One file per requirement

Each requirement gets its own dedicated test file:

```
tests/
├── vv_req_smt_001.rs
├── vv_req_smt_002.rs
├── vv_req_wire_001.rs
├── vv_req_chk_001.rs
└── ...
```

## Naming

File: `tests/vv_req_{id}.rs` where `{id}` is lowercase with underscores:

- `SMT-001` → `tests/vv_req_smt_001.rs`
- `WIRE-003` → `tests/vv_req_wire_003.rs`
- `CHK-005` → `tests/vv_req_chk_005.rs`

## File structure

Each VV test file follows this structure:

```rust
//! REQUIREMENT: SMT-001 — Fixed depth tree structure
//! (`docs/requirements/domains/smt/NORMATIVE.md#SMT-001`).
//!
//! Spec: `docs/requirements/domains/smt/specs/SMT-001.md`.
//!
//! Implementation: `src/merkle/sparse.rs`.

use crate::merkle::SparseMerkleTree;

#[test]
fn vv_req_smt_001_tree_depth_is_32() {
    let tree = SparseMerkleTree::new();
    assert_eq!(tree.depth(), 32, "SMT-001: depth must be exactly 32");
}

#[test]
fn vv_req_smt_001_depth_matches_slot_bits() {
    // Additional test for same requirement
    let tree = SparseMerkleTree::new();
    assert_eq!(tree.depth(), 32);
    assert_eq!(tree.max_slot(), u32::MAX);
}
```

Key elements:
1. **Module doc comment** — Cite requirement ID, NORMATIVE link, spec file, implementation file
2. **Test function naming** — `vv_req_{id}_{description}`
3. **Assertion messages** — Include requirement ID

## Cross-implementation tests

For requirements spanning Rust and Chialisp, create cross-impl verification:

```rust
//! REQUIREMENT: WIRE-001 — Checkpoint message format
//! (`docs/requirements/domains/wire/NORMATIVE.md#WIRE-001`).
//!
//! Cross-implementation test: verifies Rust and CLVM produce identical bytes.

use crate::wire::compute_checkpoint_message;
use crate::test_utils::run_clvm;

#[test]
fn vv_req_wire_001_cross_impl_checkpoint_message() {
    let state = CheckpointState { epoch: 1, validator_count: 10, ... };

    let rust_output = compute_checkpoint_message(&state);
    let clvm_output = run_clvm("checkpoint-message", &state);

    assert_eq!(rust_output, clvm_output, "WIRE-001: cross-impl mismatch");
}
```

## Cross-implementation matrix

| Domain | Cross-impl required | Notes |
|--------|---------------------|-------|
| SMT | Yes | Slot assignment, leaf values, proof verification |
| Wire | Yes | All serialization formats |
| Circuit | No | Rust only (arkworks) |
| Checkpoint | Partial | CLVM verification, Rust proof generation |
| Registration | Partial | CLVM puzzle, Rust driver |
| Indexer | No | Rust only |

## Test vectors

For wire format and hash computations, define vectors in a shared module:

```rust
// tests/vectors/mod.rs
pub mod wire_vectors;
pub mod smt_vectors;

// tests/vectors/wire_vectors.rs
pub const CHECKPOINT_MESSAGE_VECTORS: &[TestVector] = &[
    TestVector {
        name: "empty_state",
        input: CheckpointState { epoch: 0, validator_count: 0, ... },
        expected: hex!("0000000000000000..."),
    },
];
```

Use vectors in requirement tests:

```rust
//! REQUIREMENT: WIRE-001
use crate::vectors::wire_vectors::CHECKPOINT_MESSAGE_VECTORS;

#[test]
fn vv_req_wire_001_test_vectors() {
    for v in CHECKPOINT_MESSAGE_VECTORS {
        let result = compute_checkpoint_message(&v.input);
        assert_eq!(result, v.expected, "WIRE-001: vector '{}' failed", v.name);
    }
}
```

## HARD REQUIREMENT: CLVM Execution + Simulator for Puzzle Domains

For **all puzzle-domain requirements** (NET, REG, CHK), source-inspection tests
alone are **never sufficient**. Every VV test file MUST include:

### Tier 1: CLVM Execution Tests (MUST)

Deserialize the compiled `.hex`, curry with parameters, run with a solution,
and assert the exact output conditions:

```rust
use clvmr::{Allocator, ChiaDialect, run_program, serde::node_from_bytes};
use clvm_traits::{ToClvm, FromClvm};
use clvm_utils::CurriedProgram;

#[test]
fn vv_req_reg_001_clvm_produces_assert_announcement() {
    let mut a = Allocator::new();

    // 1. Deserialize compiled puzzle hex
    let puzzle_bytes = hex::decode(include_str!("../puzzles/compiled/registration_coin.hex")).unwrap();
    let mod_ptr = node_from_bytes(&mut a, &puzzle_bytes).unwrap();

    // 2. Curry with test params
    let curried = CurriedProgram {
        program: mod_ptr,
        args: RegistrationCoinArgs {
            validator_pubkey: test_pubkey(),
            checkpoint_singleton_id: test_checkpoint_id(),
        },
    }.to_clvm(&mut a).unwrap();

    // 3. Build solution
    let solution = RegistrationCoinSolution {
        epoch: 5u64,
        collateral_destination: dest_hash(),
        collateral_amount: 1_000_000u64,
        conditions: vec![],
    }.to_clvm(&mut a).unwrap();

    // 4. Run CLVM
    let result = run_program(&mut a, &ChiaDialect::new(0), curried, solution, 11_000_000_000);
    assert!(result.is_ok(), "CLVM execution must succeed");

    // 5. Parse and assert conditions
    let conditions = parse_conditions(&a, result.unwrap().1);
    assert!(conditions.contains_opcode(61)); // ASSERT_COIN_ANNOUNCEMENT
    assert!(conditions.contains_opcode(51)); // CREATE_COIN
}
```

### Tier 2: Simulator Spend Tests (MUST)

Use `chia-sdk-test::Simulator` to test full spend bundle lifecycle:

```rust
use chia_sdk_test::Simulator;
use chia_sdk_driver::SpendContext;

#[test]
fn vv_req_reg_001_simulator_spend_lifecycle() {
    let mut sim = Simulator::new();

    // 1. Create coins
    let coin = sim.new_coin(puzzle_hash, 1_000_000);

    // 2. Build spend bundle via SpendContext
    let ctx = &mut SpendContext::new();
    // ... build coin spends ...

    // 3. Submit to simulator
    let result = sim.spend_coins(ctx.take(), &[sk]);

    // 4. Verify resulting coin states
    assert!(result.is_ok());
    let children = sim.children(coin.coin_id());
    assert_eq!(children.len(), expected);
}
```

### Tier 3: Failure Case Tests (MUST)

Every puzzle test MUST verify rejection of invalid inputs:

- Wrong pubkey / checkpoint ID → CLVM failure or wrong conditions
- Invalid epoch → announcement hash mismatch → simulator rejects
- Missing announcement → `AssertCoinAnnouncement` fails
- Wrong collateral amount → coin amount mismatch

### Required Permutation Matrix

| Dimension | Min test cases |
|-----------|---------------|
| Valid spend (happy path) | 1+ per spend path |
| Wrong curried param | 1+ per curried param |
| Wrong solution field | 1+ per solution field |
| Edge values | 0, max, boundary |
| Cross-impl hash check | 1+ per hash computation |

## When to skip test-first

Skip only for:
- Documentation-only changes
- Tracking file updates
- Pure refactoring with existing test coverage

Then → [`dt-wf-implement.md`](dt-wf-implement.md).

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-wf-gather-context.md`](dt-wf-gather-context.md) |
| **Next** | [`dt-wf-implement.md`](dt-wf-implement.md) |

*Back to [`tree/README.md`](README.md).*
