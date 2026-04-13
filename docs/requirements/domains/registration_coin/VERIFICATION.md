# Registration Coin — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [REG-001](NORMATIVE.md#REG-001) | ✅ | Curried parameters | 18 source-inspection + 8 CLVM execution: loads hex, 2 conditions, cross-impl hash at epoch 0+42, CREATE_COIN dest+amt, different pk/ckpt_id |
| [REG-002](NORMATIVE.md#REG-002) | ✅ | Lineage verification | LineageChecker tracks network coin spends; O(1) lookup rejects fakes; 12 VV tests in tests/vv_req_reg_002.rs |
| [REG-003](NORMATIVE.md#REG-003) | ✅ | Collateral lock | CLVM execution: puzzle always emits ASSERT_COIN_ANNOUNCEMENT (no skip path); hash binds to PK+CKPT_ID+epoch; non-membership hardcoded; cross-impl hash at 3 epoch values; 14 VV tests in tests/vv_req_reg_003.rs |
| [REG-004](NORMATIVE.md#REG-004) | ✅ | Announcement assertion | CLVM execution: 67-byte inner preimage verified; "membership" prefix exact; epoch 8-byte BE; is_member=0x00; 4 test vectors (zeros, 0xFF, epoch 1, realistic); hash independent of dest/amt; 18 VV tests |
| [REG-005](NORMATIVE.md#REG-005) | ✅ | Collateral return | CLVM execution: CREATE_COIN destination+amount from solution; independent of curried params; 10 VV tests |
| [REG-006](NORMATIVE.md#REG-006) | ✅ | Epoch replay protection | CLVM execution: different epochs → different hashes; cross-impl at 9 boundary epochs (0,128,255,256,max); 11 VV tests |
| [REG-007](NORMATIVE.md#REG-007) | ✅ | E2E simulator test | 2 success-path + 4 failure-path (REG-009): full lifecycle verified |
| [REG-008](NORMATIVE.md#REG-008) | ✅ | CLVM execution for REG-001 | 8 CLVM tests: loads hex, 2 conditions, cross-impl hash at epoch 0+42, CREATE_COIN dest+amt, different pk/ckpt_id |
| [REG-009](NORMATIVE.md#REG-009) | ✅ | Failure case coverage | 4 simulator tests: no announcement, wrong pubkey hash, is_member=0x01, epoch mismatch — all rejected |
| [REG-010](NORMATIVE.md#REG-010) | ✅ | Simulator spend verification | 5 tests: no-announcement rejected (REG-003), cross-coin success (REG-004), destination+amount verified (REG-005), epoch mismatch rejected (REG-006) |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
