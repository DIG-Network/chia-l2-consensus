# Workflow — update tracking

When the task maps to a **requirement ID**:

1. **`TRACKING.yaml`** — Set `status` and add test references:
   ```yaml
   - id: SMT-001
     status: implemented  # or: verified, partial, gap
     tests:
       - tests/smt_test.rs::test_tree_depth
     notes: "Implemented with precomputed empty hashes"
   ```

2. **`VERIFICATION.md`** — Update the status column for that ID when evidence exists.

3. **`IMPLEMENTATION_ORDER.md`** — Change `- [ ]` → `- [x]` only when the requirement is **actually done** on `main`.

## Status values

| Status | Meaning |
|--------|---------|
| `gap` | Not started |
| `partial` | Some work done, not complete |
| `implemented` | Code complete, needs verification |
| `verified` | Tests pass, acceptance criteria met |

If the task is **ad-hoc** (no ID), skip this page.

Keep **NORMATIVE** prose and **dedicated spec** files aligned if acceptance criteria changed (rare — prefer new ID).

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-wf-validate.md`](dt-wf-validate.md) |
| **Next** | [`dt-wf-commit.md`](dt-wf-commit.md) |

*Back to [`tree/README.md`](README.md).*
