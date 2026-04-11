# Registration Coin — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [REG-001](NORMATIVE.md#REG-001) | ✅ | Curried parameters | Puzzle compiles; 2 curried params verified; .hex/.hash artifacts match live build; 18 VV tests in tests/vv_req_reg_001.rs |
| [REG-002](NORMATIVE.md#REG-002) | ✅ | Lineage verification | LineageChecker tracks network coin spends; O(1) lookup rejects fakes; 12 VV tests in tests/vv_req_reg_002.rs |
| [REG-003](NORMATIVE.md#REG-003) | ✅ | Collateral lock | CLVM execution: puzzle always emits ASSERT_COIN_ANNOUNCEMENT (no skip path); hash binds to PK+CKPT_ID+epoch; non-membership hardcoded; cross-impl hash at 3 epoch values; 14 VV tests in tests/vv_req_reg_003.rs |
| [REG-004](NORMATIVE.md#REG-004) | ✅ | Announcement assertion | CLVM execution: 67-byte inner preimage verified; "membership" prefix exact; epoch 8-byte BE; is_member=0x00; 4 test vectors (zeros, 0xFF, epoch 1, realistic); hash independent of dest/amt; 18 VV tests |
| [REG-005](NORMATIVE.md#REG-005) | ✅ | Collateral return | CLVM execution: CREATE_COIN destination+amount from solution; independent of curried params; 10 VV tests |
| [REG-006](NORMATIVE.md#REG-006) | ✅ | Epoch replay protection | CLVM execution: different epochs → different hashes; cross-impl at 9 boundary epochs (0,128,255,256,max); 11 VV tests |
| [REG-007](NORMATIVE.md#REG-007) | ✅ | E2E simulator test | Cross-coin bundle: checkpoint query + registration coin spend; collateral recovered; 2 VV tests |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
