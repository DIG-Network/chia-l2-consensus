# Hard rules

1. **Use chia-wallet-sdk first** — Before implementing ANY custom logic, check if `chia-wallet-sdk`, `chia-protocol`, `chia-puzzles`, or `clvmr` already provides the functionality. Custom implementations are only allowed when the SDK does not support the required operation.

2. **Follow crate architecture** — All implementations MUST fit within the framework defined in [`spec-consensus-crate.md`](../../resources/spec-consensus-crate.md). Public API is `ConsensusClient` and types it returns only. Internal modules (`merkle/`, `prover/`, `puzzles/`, `indexer/`) are `pub(crate)` or private. The `merkle` module is `pub` for testing purposes.

3. **On-chain puzzles MUST use Rue** — All on-chain puzzles (network_coin_inner, registration_coin, checkpoint_inner) MUST be written in [Rue](https://rue-lang.dev/), not Chialisp. Rue compiles to CLVM and is the language specified in all resource specs.

4. **Rue files MUST compile** — All `.rue` files MUST compile without errors or warnings before moving to the next requirement. Run `rue build puzzles/<file>.rue` and fix all issues. Do NOT proceed if compilation fails.

5. **Resource files are authoritative** — Specs in [`docs/resources/`](../../resources/) define the protocol. Requirement specs cite these with line numbers.

6. **Cross-implementation consistency** — Rust and Rue MUST produce identical outputs:
   - SMT slot assignment: `first_8_bytes_be(sha256(pubkey)) mod 2^32`
   - Leaf hashes: `sha256(pubkey)` for active, `sha256(zeros)` for empty
   - Merkle path: left child first in hash concatenation
   - Wire formats: exact byte layouts per spec

7. **Integer encoding** — All integers in wire formats MUST be fixed-width big-endian. Variable-length encoding is forbidden.

8. **Trusted setup immutability** — Circuit parameters (MAX_SIGNERS, TREE_DEPTH) are fixed at trusted setup. Changing them requires a new ceremony.

9. **Test vectors required** — Every hash computation and wire format must have test vectors verified in both Rust and Rue.

10. **BLS12-381 point format** — Use ZCash compressed format (48 bytes G1, 96 bytes G2) with proper infinity and sign encoding.

11. **After `git pull`** — Treat `- [x]` in [`IMPLEMENTATION_ORDER.md`](../../requirements/IMPLEMENTATION_ORDER.md) as **done**; only `- [ ]` is selectable.

## chia-wallet-sdk priority

Before writing custom code, check these crates for existing functionality:

| Crate | Provides |
|-------|----------|
| `chia-wallet-sdk` | Wallet operations, spend bundles, coin management, singleton handling |
| `chia-protocol` | Protocol types (Coin, CoinSpend, Program), serialization |
| `chia-puzzles` | Standard puzzles (singleton, CAT, DID), puzzle drivers |
| `clvm-traits` | CLVM type conversions, ToClvm/FromClvm traits |
| `clvmr` | CLVM runtime, Allocator, puzzle execution |

**Only implement custom logic when:**
- The SDK does not provide the required functionality
- The SDK implementation has a bug that cannot be worked around
- Performance requirements cannot be met with SDK abstractions

Document SDK usage or justification for custom code in implementation notes.

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-role.md`](dt-role.md) |
| **Next** | [`dt-authoritative-sources.md`](dt-authoritative-sources.md) |

*Back to [`tree/README.md`](README.md).*
