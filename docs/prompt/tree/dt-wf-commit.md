# Workflow — commit

## Message format

`type(scope): imperative subject`

**Types:** `feat`, `fix`, `docs`, `chore`, `refactor`, `test`

**Scopes:**
- `smt` — Sparse Merkle tree
- `wire` — Wire format serialization
- `circuit` — Groth16 circuit
- `checkpoint` — Checkpoint singleton puzzle
- `registration` — Registration coin puzzle
- `network` — Network coin puzzle
- `indexer` — Off-chain indexer
- `puzzles` — General Chialisp work
- `docs` — Documentation
- `requirements` — Requirements files

## Examples

```
feat(smt): implement slot assignment from pubkey hash
fix(wire): correct big-endian encoding for epoch field
test(circuit): add cross-impl test for VK input computation
docs(requirements): add source citations to CHK-001
chore(deps): update arkworks to 0.4.2
```

## What to include

- All files for one logical change
- Related test updates
- Tracking file updates (TRACKING.yaml, VERIFICATION.md)

## What to avoid

- Mixing unrelated requirement IDs in one commit
- Large refactors bundled with feature work
- Incomplete implementations (prefer `partial` status + separate commit)

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-wf-update-tracking.md`](dt-wf-update-tracking.md) |
| **Next** | Loop to [`../start.md`](../start.md) for the next task |

*Back to [`tree/README.md`](README.md) or [`../prompt.md`](../prompt.md).*
