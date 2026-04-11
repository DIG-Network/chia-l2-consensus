# RPC Integration — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [RPC-001](NORMATIVE.md#RPC-001) | ❌ | RPC client | Connect to Chia node; query coins; submit bundles |
| [RPC-002](NORMATIVE.md#RPC-002) | ❌ | Puzzle drivers | Build spend bundles for all 3 puzzles; submit to testnet |
| [RPC-003](NORMATIVE.md#RPC-003) | ❌ | Indexer sync | Full sync against testnet; verify Merkle consistency |
| [RPC-004](NORMATIVE.md#RPC-004) | ❌ | Client operations | E2E: deploy, register, checkpoint, recover via client API |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
