# Security — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [SEC-001](NORMATIVE.md#SEC-001) | ✅ | Majority assumption | Strict 2k>n verified; boundary tests; exhaustive minimum_signers 0-200; circuit rejects minority/half; 9 VV tests |
| [SEC-002](NORMATIVE.md#SEC-002) | ✅ | Two-check completeness | Both opcodes (58+59) in compiled CLVM; all 5 BLS operators present; shared agg_signers; 9 VV tests |
| [SEC-003](NORMATIVE.md#SEC-003) | ✅ | Collateral security | ASSERT_COIN_ANNOUNCEMENT hardcoded; non-membership 0x00 hardcoded; no bypass; active locked; 9 VV tests |
| [SEC-004](NORMATIVE.md#SEC-004) | ✅ | Trusted setup | Source warns single-party; MPC in spec+CHIP; VK valid; invalid rejected; deterministic; 9 VV tests |
| [SEC-005](NORMATIVE.md#SEC-005) | ✅ | Lineage enforcement | Invalid parent rejected; puzzle hash alone insufficient; wrong collateral rejected; O(1) HashSet; 10 VV tests |
| [SEC-006](NORMATIVE.md#SEC-006) | ✅ | Epoch replay | Epoch in inner hash + coin ID in outer hash; full replay scenario; adjacent/boundary epochs; 11 VV tests |

| [SEC-007](NORMATIVE.md#SEC-007) | ✅ | CLVM vulnerability audit | 9 vulns verified: V1-V3 mitigated, V4-V6 tracked, V7 N/A, V8 by-design, V9 CHK-012; 13 VV tests |
| [SEC-008](NORMATIVE.md#SEC-008) | ✅ | Condition injection | Removed from registration + checkpoint; network coin keeps (AggSigMe protected); 9 VV tests |
| [SEC-009](NORMATIVE.md#SEC-009) | ✅ | Destination binding | Risk mitigated by announcement requirement; RBF impractical; 8 VV tests |
| [SEC-010](NORMATIVE.md#SEC-010) | ✅ | Attack surface | 20 attack vectors verified across all puzzles; 16 addressed, 3 trade-offs, 1 by-design; 20 VV tests |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
