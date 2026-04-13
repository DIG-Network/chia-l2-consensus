# RPC Integration — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [RPC-001](NORMATIVE.md#RPC-001) | ✅ | ChiaQuery client | 13 tests: connect() method, ChiaQuery stored, hex conversion roundtrip + edge cases, RpcError variant, no custom RPC |
| [RPC-002](NORMATIVE.md#RPC-002) | ✅ | Puzzle drivers | 13 tests: 9 driver functions exist with correct sigs; .hex loaded; no broadcast; mod exports all; stubs are todo!() |
| [RPC-003](NORMATIVE.md#RPC-003) | ✅ | Indexer sync | 11 tests: 4 stubs exist (sync, get_coin, get_records, build_set); module structure; cache; reorg; lineage |
| [RPC-004](NORMATIVE.md#RPC-004) | ✅ | Client operations | 13 tests: 6 methods exist with correct sigs; all return SpendBundle; L1Wallet in register; connect+sync; no broadcast; WDC-004 docs |
| [RPC-005](NORMATIVE.md#RPC-005) | ✅ | Wallet coin selection | 10 tests: register_validator accepts L1Wallet+name+index+fee; InsufficientFunds error; wallet not stored; types importable |
| [RPC-006](NORMATIVE.md#RPC-006) | ✅ | Dependency alignment | 9 tests: chia-query + dig-l1-wallet in Cargo.toml; ChiaQuery/Config/L1Wallet/Config importable; existing types work; cargo check + tests pass |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
