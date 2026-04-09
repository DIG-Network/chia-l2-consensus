# chia-l2-consensus — start here

Compact entrypoint. Use linked **`tree/dt-*.md`** pages for detail.

## Session start

1. **`git checkout main`** → **`git pull origin main`** on the repo.
2. **Check GitNexus status** — Run `npx gitnexus status`. If stale, run `npx gitnexus analyze`.
3. **Pick work** — next `- [ ]` in [`IMPLEMENTATION_ORDER.md`](../requirements/IMPLEMENTATION_ORDER.md), or a scoped task (circuit, puzzle, wire format, indexer).
4. **Pack context with Repomix** — `npx repomix@latest <scope> -o .repomix/pack-<scope>.xml` for the module you'll edit.
5. **Read spec files** — Resource files in [`docs/resources/`](../resources/) contain authoritative specifications with line numbers cited in requirement specs.
6. **Read requirement spec** — Each requirement has a dedicated spec file in [`docs/requirements/domains/<domain>/specs/`](../requirements/domains/).

## Hard requirements

- **Use chia-wallet-sdk first** — Check `chia-wallet-sdk`, `chia-protocol`, `chia-puzzles`, `clvmr` before implementing custom logic.
- **On-chain puzzles MUST use Rue** — All puzzles in `puzzles/*.rue` use [Rue](https://rue-lang.dev/), not Chialisp.
- **Rue files MUST compile** — Run `rue build puzzles/<file>.rue` and fix all errors/warnings before proceeding.
- **Resource files are authoritative** — [`docs/resources/`](../resources/) contains the CHIP and spec files. Requirement specs cite these with line numbers.
- **Cross-implementation consistency** — Rust and Rue MUST produce identical results for shared logic (SMT, wire format, hashes).
- **No trusted setup changes** — Circuit parameters (MAX_SIGNERS, TREE_DEPTH) require new trusted setup ceremony to change.
- **Test vectors** — All wire formats and hash computations must have test vectors verified in both Rust and Rue.
- **VV tests per requirement** — Each requirement gets a dedicated test file `tests/vv_req_{id}.rs`.

## Decision tree (short)

| Step | Page |
|------|------|
| Paths + role | [`tree/dt-paths.md`](tree/dt-paths.md), [`tree/dt-role.md`](tree/dt-role.md) |
| Rules + sources | [`tree/dt-hard-rules.md`](tree/dt-hard-rules.md), [`tree/dt-authoritative-sources.md`](tree/dt-authoritative-sources.md) |
| Git | [`tree/dt-git.md`](tree/dt-git.md) |
| Tools | [`tree/dt-tools.md`](tree/dt-tools.md) (GitNexus + Repomix) |
| Select | [`tree/dt-wf-select.md`](tree/dt-wf-select.md) |
| Context | [`tree/dt-wf-gather-context.md`](tree/dt-wf-gather-context.md) |
| Test first | [`tree/dt-wf-test.md`](tree/dt-wf-test.md) |
| Implement | [`tree/dt-wf-implement.md`](tree/dt-wf-implement.md) |
| Validate | [`tree/dt-wf-validate.md`](tree/dt-wf-validate.md) |
| Tracking | [`tree/dt-wf-update-tracking.md`](tree/dt-wf-update-tracking.md) |
| Commit | [`tree/dt-wf-commit.md`](tree/dt-wf-commit.md) |

**Full index:** [`tree/README.md`](tree/README.md) · **Workflow index:** [`prompt-workflow.md`](prompt-workflow.md)

## Tech stack

| Component | Technology |
|-----------|------------|
| Off-chain logic | Rust (arkworks for ZK, clvmr for CLVM) |
| On-chain puzzles | Rue (compiles to CLVM) |
| ZK proofs | Groth16 on BLS12-381 |
| Signatures | BLS aggregate signatures |
| Data structure | Sparse Merkle Tree (depth 32) |
| Blockchain | Chia (coin set model) |
