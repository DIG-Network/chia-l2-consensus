# Workflow — write the test first

Write a **failing** test before implementation.

Before choosing the exact test shape, read the active requirement's dedicated spec file. The spec's verification guidance is the primary authority for what must be proven complete.

## Naming

`vv_req_{id}` where `{id}` is the requirement id in lowercase with hyphens to underscores:

- `SMT-001` -> `vv_req_smt_001`
- `WIRE-003` -> `vv_req_wire_003`
- `CHK-005` -> `vv_req_chk_005`

## Location

| Component | Location |
|-----------|----------|
| Rust unit tests | Next to code under `#[cfg(test)]` |
| Rust integration tests | `tests/` directory |
| Chialisp tests | `puzzles/tests/` or inline CLVM assertions |
| Cross-implementation | `tests/cross_impl/` with both Rust and CLVM verification |

## Structure

### Rust test

```rust
#[test]
fn vv_req_smt_001() {
    // REQUIREMENT: SMT-001
    // Verify tree depth is exactly 32.
    let tree = SparseMerkleTree::new();
    assert_eq!(tree.depth(), 32);
}
```

### Cross-implementation test

```rust
#[test]
fn vv_req_wire_001_cross_impl() {
    // REQUIREMENT: WIRE-001
    // Verify Rust and CLVM produce identical checkpoint message bytes.

    let rust_output = compute_checkpoint_message(&state);
    let clvm_output = run_clvm_checkpoint_message(&state);

    assert_eq!(rust_output, clvm_output, "Cross-impl mismatch");
}
```

Cite the requirement id in a comment.

## Cross-implementation consistency

For requirements that span Rust and Chialisp, the test MUST verify both implementations produce identical results:

| Domain | Cross-impl required |
|--------|---------------------|
| SMT | Yes — slot assignment, leaf values, proof verification |
| Wire | Yes — all serialization formats |
| Circuit | No — Rust only (arkworks) |
| Checkpoint | Partial — CLVM verification, Rust proof generation |
| Registration | Partial — CLVM puzzle, Rust driver |
| Indexer | No — Rust only |

## Test vectors

For wire format and hash computations, define test vectors:

```rust
// tests/vectors/wire_001.rs
pub const CHECKPOINT_MESSAGE_VECTORS: &[TestVector] = &[
    TestVector {
        input: CheckpointState { epoch: 0, validator_count: 0, ... },
        expected_bytes: hex!("0000000000000000..."),
    },
    // ...
];
```

Verify the same vectors in Chialisp:

```clojure
; puzzles/tests/wire_001_vectors.clsp
(defun test-checkpoint-message ()
  (assert (= (checkpoint-message 0 0 ...) 0x0000000000000000...))
)
```

## When to skip

Skip test-first only for:
- Documentation-only changes
- Tracking file updates
- Pure refactoring with existing test coverage

Then -> [`dt-wf-implement.md`](dt-wf-implement.md).

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-wf-gather-context.md`](dt-wf-gather-context.md) |
| **Next** | [`dt-wf-implement.md`](dt-wf-implement.md) |

*Back to [`tree/README.md`](README.md).*
