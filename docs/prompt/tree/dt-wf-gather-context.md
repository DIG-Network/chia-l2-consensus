# Workflow — gather context

## 0. Pack with Repomix (first!)

Before reading files manually, pack the relevant scope:

```bash
# From repo root - pack the module you'll work on
npx repomix@latest src/merkle -o .repomix/pack-merkle.xml
npx repomix@latest src/prover -o .repomix/pack-prover.xml
npx repomix@latest puzzles -o .repomix/pack-puzzles.xml
```

Read the packed context before proceeding. See [`dt-tools.md`](dt-tools.md) for full Repomix workflow.

## 1. Requirements trace (when a REQ-ID applies)

1. **`NORMATIVE.md`** — Read the rule; note the link to the spec file.
2. **Dedicated spec** — `docs/requirements/domains/<domain>/specs/<ID>.md` — Primary authority for intent and acceptance.
3. **Source Citations** — Each spec file cites resource files with line numbers.
4. **`VERIFICATION.md`** — Row for this ID (how to prove done).
5. **`TRACKING.yaml`** — `status`, `spec_ref`, `tests`.

## 2. Resource files

Read the cited sections in [`docs/resources/`](../../resources/):

| File | Key sections |
|------|--------------|
| `spec-sparse-merkle-tree.md` | Slot assignment, leaf values, proof format |
| `spec-wire-format.md` | Point encoding, proof format, messages |
| `spec-groth16-circuit.md` | Constraints, public inputs |
| `spec-checkpoint-singleton.md` | Spend paths, announcements |
| `spec-registration-coin.md` | Puzzle parameters, collateral |
| `chip-groth16-l2-consensus.md` | Design rationale, security |

## 3. Cross-references

Check related requirements:
- SMT requirements affect Circuit (Merkle verification)
- Wire requirements affect Checkpoint (on-chain parsing)
- Registration affects Checkpoint (announcements)

## 4. Existing code

Review existing implementations:
- `src/` for Rust patterns
- `puzzles/` for Chialisp patterns
- `tests/` for test vector format

**Authority order:** NORMATIVE → dedicated spec → Source Citations → resource files → existing code.

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-wf-select.md`](dt-wf-select.md) |
| **Next** | [`dt-wf-test.md`](dt-wf-test.md) |

*Back to [`tree/README.md`](README.md).*
