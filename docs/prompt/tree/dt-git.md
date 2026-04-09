# Git — sync, commit, push

Use `main` as the default branch.

## Sync

1. **Fetch** — `git fetch origin`
2. **Pull** — `git pull origin main`

## Commit

- **Message format** — `type(scope): imperative subject`
- **Types** — `feat`, `fix`, `docs`, `chore`, `refactor`, `test`
- **Scopes** — `circuit`, `smt`, `wire`, `checkpoint`, `registration`, `network`, `indexer`, `puzzles`, `docs`

## Push

`git push origin main`

## Conflicts

- Open conflicted files; remove `<<<<<<<` / `=======` / `>>>>>>>`.
- Re-run tests for affected components.
- Never push conflict markers.

**Rejected push:** `git pull --rebase origin main`, resolve, push again.

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-authoritative-sources.md`](dt-authoritative-sources.md) |
| **Next** | [`dt-wf-select.md`](dt-wf-select.md) |

*Back to [`tree/README.md`](README.md).*
