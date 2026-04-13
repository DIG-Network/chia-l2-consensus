# Network Coin — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [NET-001](NORMATIVE.md#NET-001) | ✅ | Singleton identity | 8 source-inspection + 5 CLVM execution tests: loads hex, executes, 3 conditions, recreation amt=1, deterministic, different pubkeys |
| [NET-002](NORMATIVE.md#NET-002) | ✅ | AggSigMe registration | 12 source/format + 6 CLVM execution: AGG_SIG_ME emitted, pubkey matches solution, message = sha256("register"+pk) cross-impl verified |
| [NET-003](NORMATIVE.md#NET-003) | ✅ | Registration coin creation | 13 source + 7 CLVM execution: amount==COLLATERAL, puzzle hash cross-impl with Rust curry_tree_hash, different pk/ckpt_id differ |
| [NET-004](NORMATIVE.md#NET-004) | ✅ | Self-recreation | 10 source + 5 CLVM execution: recreation amt=1, hash=INNER_MOD_HASH, real hash verified, always emitted, independent of pubkey |
| [NET-005](NORMATIVE.md#NET-005) | ✅ | Pubkey memo | Driver convention; source + format tests sufficient |
| [NET-006](NORMATIVE.md#NET-006) | ✅ | E2E simulator test | 4 success-path + 3 failure/verification: invalid lineage rejected, no collateral rejected, recreated puzzle hash verified |
| [NET-007](NORMATIVE.md#NET-007) | ✅ | CLVM execution validation | 23 CLVM tests across NET-001(5), NET-002(6), NET-003(7), NET-004(5) |
| [NET-008](NORMATIVE.md#NET-008) | ✅ | Failure case coverage | Invalid lineage rejected, insufficient collateral rejected, recreated puzzle hash verified; note: AGG_SIG_ME not enforced by chia-sdk-test simulator |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
