# Withdraw Delay Coin — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [WDC-001](NORMATIVE.md#WDC-001) | ✅ | Puzzle structure | 8 source + 12 CLVM execution + 2 artifact freshness: Rue compiles, 3 curried params, empty solution, 2 conditions (82+51), values match curried, deterministic, no AGG_SIG, delay=0 valid |
| [WDC-002](NORMATIVE.md#WDC-002) | ✅ | Time lock | 16 CLVM tests: opcode 82 at delays 0,1,2,10,256,4608,24000,100000,u32::MAX; no bypass; first condition always; simulator height enforcement N/A (Chia node enforces) |
| [WDC-003](NORMATIVE.md#WDC-003) | ✅ | Fund release | 8 CLVM execution (dest×3 + amount×3 + diff) + 7 Rust curry_tree_hash cross-impl; amounts 1 mojo to 10 XCH; puzzle hash deterministic |
| [WDC-004](NORMATIVE.md#WDC-004) | ✅ | Registration coin integration | 9 source + 7 CLVM execution + 3 artifact freshness: reg coin 4 curried params, curry_tree_hash for delay coin, cross-impl hash match, net coin 6 curried params; all existing tests updated |
| [WDC-005](NORMATIVE.md#WDC-005) | ✅ | Driver and API | 8 tests: release_collateral() on client + driver, withdraw_delay_puzzle_hash(), returns SpendBundle, no broadcast, two-phase docs, exports |
| [WDC-006](NORMATIVE.md#WDC-006) | ✅ | Configuration | 7 tests: both fields exist, default 24000, JSON round-trip for delay + hash, deploy populates |
| [WDC-007](NORMATIVE.md#WDC-007) | ✅ | Permissionless release | 8 tests: no AGG_SIG_ME/UNSAFE/any variant (43-50), only opcodes 82+51, destination immutable across 4 vectors, source no AggSig |
| [WDC-008](NORMATIVE.md#WDC-008) | ✅ | CLVM execution tests | 11 tests: WDC-001 (load, 2 conds, no passthrough, diff params), WDC-002 (opcode 82, delay×5, zero), WDC-003 (dest, amt×3, cross-impl hash) |
| [WDC-009](NORMATIVE.md#WDC-009) | ✅ | E2E simulator test | 2 tests: full two-phase lifecycle (deploy → register → recovery → delay coin → release → destination coin); delay enforcement N/A in sim |
| [WDC-010](NORMATIVE.md#WDC-010) | ✅ | Destination hint memo | 10 tests: reg coin hint = sha256(ckpt+dest) cross-impl verified, diff dest/ckpt → diff hints, delay coin memo = "DIG Network Collateral Release" in CLVM |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
