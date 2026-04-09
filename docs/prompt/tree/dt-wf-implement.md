# Workflow — implement

## Use chia-wallet-sdk first

Before implementing ANY Chia-related functionality, check if `chia-wallet-sdk` or related crates already provide it:

```rust
// GOOD: Use SDK for singleton handling
use chia_wallet_sdk::{Singleton, SingletonLauncher};
use chia_puzzles::singleton::{SingletonStruct, SINGLETON_LAUNCHER_PUZZLE_HASH};

// GOOD: Use SDK for spend bundle creation
use chia_wallet_sdk::SpendBundle;
use chia_protocol::{Coin, CoinSpend, Program};

// GOOD: Use SDK for CLVM operations
use clvmr::{Allocator, run_program};
use clvm_traits::{ToClvm, FromClvm};
```

**Check these crates in order:**
1. `chia-wallet-sdk` — High-level wallet operations
2. `chia-puzzles` — Standard puzzle implementations
3. `chia-protocol` — Protocol types and serialization
4. `clvm-traits` — CLVM type conversions
5. `clvmr` — Low-level CLVM runtime

Only implement custom logic when SDK functionality is unavailable or insufficient. Document the justification.

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
