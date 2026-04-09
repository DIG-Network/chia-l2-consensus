# Tools — GitNexus and Repomix

Development tools for dependency analysis and context gathering. These tools help ensure safe code changes and provide comprehensive context.

## GitNexus — Dependency Analysis

**Source:** [nxpatterns/gitnexus](https://github.com/nxpatterns/gitnexus)

GitNexus builds and queries a dependency graph of the codebase. Use it to understand impact before making changes.

### Commands

```bash
# Check if index is fresh (run first!)
npx gitnexus status

# Build/update the dependency index
npx gitnexus analyze        # Incremental update
npx gitnexus analyze -f     # Full rebuild
```

### Before Changing Code

For edits that touch **named symbols** (functions, structs, methods, exports):

1. **Check impact:** `gitnexus_impact` on the symbol (direction: upstream)
2. **If high impact:** Add `gitnexus_context` and/or `gitnexus_query` before editing
3. **For renames:** Use `gitnexus_rename` with `dry_run: true` first — never raw find-replace

### After Changing Code

1. Run `npx gitnexus analyze` to update the index
2. Optionally `gitnexus_detect_changes` before commit to verify scope

### One-Line Loop

```
status → (analyze if needed) → impact → edit → analyze → (detect_changes → commit)
```

---

## Repomix — Context Packing

**Source:** [yamadashy/repomix](https://github.com/yamadashy/repomix)

Repomix packs codebase context into XML files for comprehensive understanding before making changes.

### Before Any Code Change (Required)

1. **Pick scope:** The smallest directory containing all files you'll edit
2. **Pack:** Run from repo root, output to `.repomix/`:
   ```bash
   npx repomix@latest src/merkle -o .repomix/pack-merkle.xml
   npx repomix@latest src/prover -o .repomix/pack-prover.xml
   npx repomix@latest puzzles -o .repomix/pack-puzzles.xml
   ```
3. **Load:** Read all relevant packs before proposing or applying patches

### Common Scopes

| Scope | Command |
|-------|---------|
| SMT module | `npx repomix@latest src/merkle -o .repomix/pack-merkle.xml` |
| Prover | `npx repomix@latest src/prover -o .repomix/pack-prover.xml` |
| Puzzles | `npx repomix@latest puzzles -o .repomix/pack-puzzles.xml` |
| Indexer | `npx repomix@latest src/indexer -o .repomix/pack-indexer.xml` |
| Tests | `npx repomix@latest tests -o .repomix/pack-tests.xml` |
| Full src | `npx repomix@latest src -o .repomix/pack-src.xml` |

### Hygiene

- `.repomix/` is gitignored (local context, not source of truth)
- Repomix respects `.gitignore` / `.repomixignore`
- Do not bypass ignores to include secrets

### One-Liner

```
npx repomix@latest <scope> -o .repomix/pack-<scope>.xml → read packs → edit
```

---

## Integration with Workflow

| Workflow Step | Tool Usage |
|---------------|------------|
| **Gather context** | Pack relevant scope with Repomix, read the pack |
| **Before editing** | Run `gitnexus_impact` on symbols you'll change |
| **After editing** | Run `npx gitnexus analyze` to update index |
| **Before commit** | Optionally `gitnexus_detect_changes` to verify scope |

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-git.md`](dt-git.md) |
| **Next** | [`dt-wf-select.md`](dt-wf-select.md) |

*Back to [`tree/README.md`](README.md).*
