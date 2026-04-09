# Decision tree — chia-l2-consensus

One file ≈ one decision. Master flow: [`../prompt.md`](../prompt.md) and [`../start.md`](../start.md).

| File | Topic |
|------|--------|
| [`dt-paths.md`](dt-paths.md) | Path conventions under `docs/` |
| [`dt-role.md`](dt-role.md) | Operator role |
| [`dt-hard-rules.md`](dt-hard-rules.md) | Non-negotiables |
| [`dt-authoritative-sources.md`](dt-authoritative-sources.md) | Spec + requirements layout |
| [`dt-git.md`](dt-git.md) | Sync, commit, push |
| [`dt-tools.md`](dt-tools.md) | GitNexus + Repomix tools |
| **Workflow** | |
| [`dt-wf-select.md`](dt-wf-select.md) | Choose requirement or task |
| [`dt-wf-gather-context.md`](dt-wf-gather-context.md) | Specs, resource files, cross-references |
| [`dt-wf-test.md`](dt-wf-test.md) | Write VV test first (TDD) |
| [`dt-wf-implement.md`](dt-wf-implement.md) | Implement in Rust / Chialisp |
| [`dt-wf-validate.md`](dt-wf-validate.md) | Tests, cross-impl checks |
| [`dt-wf-update-tracking.md`](dt-wf-update-tracking.md) | TRACKING, VERIFICATION, order file |
| [`dt-wf-commit.md`](dt-wf-commit.md) | Commit messages |

**MCP tool index:** [`../tools/README.md`](../tools/README.md)

## Following the tree

1. Open [`../start.md`](../start.md).
2. Walk **`dt-*.md`** in workflow order; each page ends with **Continue the tree**.

**Convention chain:** `dt-paths` → `dt-role` → `dt-hard-rules` → `dt-authoritative-sources` → `dt-git` → `dt-tools` → `dt-wf-select` → `dt-wf-gather-context` → `dt-wf-test` → `dt-wf-implement` → `dt-wf-validate` → `dt-wf-update-tracking` → `dt-wf-commit`.
