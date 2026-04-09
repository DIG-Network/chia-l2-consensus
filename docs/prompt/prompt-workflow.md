# chia-l2-consensus — implementation workflow

The step cycle lives in **`tree/dt-wf-*.md`** files.

| Step | Page |
|------|------|
| Select | [`tree/dt-wf-select.md`](tree/dt-wf-select.md) |
| Gather context | [`tree/dt-wf-gather-context.md`](tree/dt-wf-gather-context.md) |
| Test first | [`tree/dt-wf-test.md`](tree/dt-wf-test.md) |
| Implement | [`tree/dt-wf-implement.md`](tree/dt-wf-implement.md) |
| Validate | [`tree/dt-wf-validate.md`](tree/dt-wf-validate.md) |
| Update tracking | [`tree/dt-wf-update-tracking.md`](tree/dt-wf-update-tracking.md) |
| Commit | [`tree/dt-wf-commit.md`](tree/dt-wf-commit.md) |

**Master outline:** [`prompt.md`](prompt.md)

## Implementation phases

| Phase | Domain | Focus |
|-------|--------|-------|
| 1 | SMT, Wire | Foundation data structures and serialization |
| 2 | Circuit | Groth16 circuit implementation |
| 3 | Network, Registration, Checkpoint | On-chain Chialisp puzzles |
| 4 | Indexer | Off-chain state tracking |
| 5 | Deployment, Validator | Operations and lifecycle |
| 6 | Security | Security verification |
