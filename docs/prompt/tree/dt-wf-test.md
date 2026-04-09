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
