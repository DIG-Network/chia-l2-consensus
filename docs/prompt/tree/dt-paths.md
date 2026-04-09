# Paths — `docs/`

All documentation lives under **`chia-l2-consensus/docs/`**.

| Path | Meaning |
|------|---------|
| `docs/prompt/prompt.md` | Mermaid + flat outline |
| `docs/prompt/start.md` | Session start |
| `docs/prompt/tree/dt-*.md` | Decision pages |
| `docs/prompt/tools/README.md` | Tool pointers |
| `docs/requirements/` | NORMATIVE / VERIFICATION / TRACKING |
| `docs/requirements/domains/<domain>/specs/` | Per-ID requirement specs |
| `docs/requirements/IMPLEMENTATION_ORDER.md` | Phased checklist |
| `docs/resources/` | CHIP and component specifications |
| `docs/resources/chip-groth16-l2-consensus.md` | CHIP specification |
| `docs/resources/quick-reference.md` | Quick lookup tables |
| `docs/resources/spec-*.md` | Detailed component specs |

## Source code paths

| Path | Content |
|------|---------|
| `src/` | Rust implementation |
| `src/circuit/` | Groth16 circuit (arkworks) |
| `src/sparse_merkle_tree.rs` | SMT implementation |
| `src/wire/` | Wire format serialization |
| `src/indexer/` | Off-chain state tracking |
| `puzzles/` | Chialisp puzzles |
| `puzzles/network_coin_inner.clsp` | Network coin puzzle |
| `puzzles/registration_coin.clsp` | Registration coin puzzle |
| `puzzles/checkpoint_inner.clsp` | Checkpoint singleton puzzle |

---

## Continue the tree

| | |
|--|--|
| **Next** | [`dt-role.md`](dt-role.md) |

*Back to [`tree/README.md`](README.md) or [`../prompt.md`](../prompt.md).*
