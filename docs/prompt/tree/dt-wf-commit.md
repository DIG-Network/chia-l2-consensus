# Workflow — commit

**IMPORTANT:** Commit to the repo and push to GitHub after each requirement is validated. Do not batch multiple requirements into a single commit.

## Commit and push procedure

```bash
# 1. Stage all changes for this requirement
git add src/merkle/sparse.rs tests/vv_req_smt_001.rs docs/requirements/...

# 2. Commit with proper message format
git commit -m "feat(smt): implement SMT-001 fixed depth tree structure"

# 3. Push to GitHub
git push origin <branch-name>
```

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
- Batching multiple requirements before pushing

## Push requirements

After committing, **always push to GitHub**:

```bash
git push origin <branch-name>
```

This ensures:
- Progress is backed up remotely
- CI can validate the changes
- Work is visible to collaborators
- Each requirement's completion is tracked in git history

**Do not proceed to the next requirement until the current one is committed and pushed.**

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-wf-update-tracking.md`](dt-wf-update-tracking.md) |
| **Next** | Loop to [`../start.md`](../start.md) for the next task |

*Back to [`tree/README.md`](README.md) or [`../prompt.md`](../prompt.md).*
