# Workflow — implement

## Implementation by domain

| Domain | Language | Location |
|--------|----------|----------|
| SMT | Rust | `src/sparse_merkle_tree.rs` |
| Wire | Rust | `src/wire/` |
| Circuit | Rust (arkworks) | `src/circuit/` |
| Network Coin | Chialisp | `puzzles/network_coin_inner.clsp` |
| Registration Coin | Chialisp | `puzzles/registration_coin.clsp` |
| Checkpoint | Chialisp | `puzzles/checkpoint_inner.clsp` |
| Indexer | Rust | `src/indexer/` |

## Critical consistency points

### SMT (Rust ↔ Chialisp)

- Slot: `first_8_bytes_be(sha256(pubkey)) mod 2^32`
- Active leaf: `sha256(pubkey)`
- Empty leaf: `sha256([0u8; 48])`
- Parent hash: `sha256(left || right)` — left child always first

### Wire format

- Integers: fixed-width big-endian (8 bytes for u64)
- G1 points: 48 bytes ZCash compressed
- G2 points: 96 bytes ZCash compressed
- No variable-length encoding

### Circuit

- Public inputs in exact order (6 inputs)
- VK has 7 IC points (1 constant + 6 inputs)
- `scalar()` = `sha256(bytes) mod r`

## Smallest change principle

- Match the **spec file** acceptance criteria exactly
- Do not add features beyond the requirement
- Do not refactor unrelated code

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-wf-test.md`](dt-wf-test.md) |
| **Next** | [`dt-wf-validate.md`](dt-wf-validate.md) |

*Back to [`tree/README.md`](README.md).*
