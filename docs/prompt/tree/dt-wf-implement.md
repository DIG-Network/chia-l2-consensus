# Workflow — implement

## Check GitNexus impact first

Before editing any **named symbol** (function, struct, method, export):

```bash
# Check if index is fresh
npx gitnexus status

# If stale, rebuild
npx gitnexus analyze
```

Then check impact:
- **`gitnexus_impact`** on the symbol (direction: upstream)
- If impact is **high/critical**, gather more context with `gitnexus_context`
- For **renames**, use `gitnexus_rename` with `dry_run: true` first

See [`dt-tools.md`](dt-tools.md) for full GitNexus workflow.

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
| SMT | Rust | `src/merkle/sparse.rs` |
| Wire | Rust | `src/prover/serialize.rs` |
| Circuit | Rust (arkworks) | `src/prover/circuit.rs` |
| Network Coin | Rue | `puzzles/network_coin_inner.rue` |
| Registration Coin | Rue | `puzzles/registration_coin.rue` |
| Checkpoint | Rue | `puzzles/checkpoint_inner.rue` |
| Indexer | Rust | `src/indexer/` |

## Rue compilation requirements

**All `.rue` files MUST compile without errors or warnings before moving on.**

```bash
# Verify all Rue puzzles compile
rue build puzzles/network_coin_inner.rue
rue build puzzles/registration_coin.rue
rue build puzzles/checkpoint_inner.rue

# Or build all at once
for f in puzzles/*.rue; do rue build "$f" || exit 1; done
```

### Fixing Rue errors

1. **Undeclared symbol** — Import the symbol or define it
2. **Type mismatch** — Use explicit casts or fix the type
3. **Expected symbol, but found type** — Use lowercase function calls, not type names

### Rue-specific patterns

```rust
// Use condition functions, not constructors
assert_coin_announcement(sha256(coin_id + message))  // GOOD
AssertCoinAnnouncement(...)  // BAD - this is a type

// Use proper types
let hash: Bytes32 = sha256(data);
let pk: PublicKey = validator_pubkey;
```

**Do not proceed to the next requirement if any Rue file fails to compile.**

## Critical consistency points

### SMT (Rust ↔ Rue)

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
