# Workflow — select work

1. **`git pull origin main`** on the repo.
2. Open [`IMPLEMENTATION_ORDER.md`](../../requirements/IMPLEMENTATION_ORDER.md).
3. Choose the **first** `- [ ]` in the current phase that matches your focus **or** an explicit task.
4. **Skip** every `- [x]` — those IDs are treated complete on `main`.
5. Open the **NORMATIVE** link for that ID (from the order file).
6. Open the **dedicated spec** file in `docs/requirements/domains/<domain>/specs/<ID>.md`.

## Phase priority

| Phase | Domains | Prerequisites |
|-------|---------|---------------|
| 1 | SMT, Wire | None |
| 2 | Circuit | Phase 1 |
| 3 | Network, Registration, Checkpoint | Phase 1, 2 |
| 4 | Indexer | Phase 3 |
| 5 | Deployment, Validator | Phase 1-4 |
| 6 | Security | All implementation |

Ad-hoc work (no checkbox): still follow **gather context** and **hard rules** for the touched paths.

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-git.md`](dt-git.md) |
| **Next** | [`dt-wf-gather-context.md`](dt-wf-gather-context.md) |

*Back to [`tree/README.md`](README.md).*
