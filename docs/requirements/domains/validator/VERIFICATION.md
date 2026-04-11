# Validator Operations — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [VAL-001](NORMATIVE.md#VAL-001) | ✅ | Key generation | generate_validator_keypair(); 48-byte G1 pk; sign/verify roundtrip; wrong key/msg rejected; 12 VV tests |
| [VAL-002](NORMATIVE.md#VAL-002) | ✅ | Registration | AGG_SIG_ME message (96 bytes); sign/verify roundtrip; wrong key/challenge/coin_id rejected; 11 VV tests |
| [VAL-003](NORMATIVE.md#VAL-003) | ✅ | Signing protocol | sign/verify checkpoint; aggregation (multi-sig + single + empty); wrong key/msg/coin_id; 11 VV tests |
| [VAL-004](NORMATIVE.md#VAL-004) | ✅ | Voluntary exit | is_validator_excluded(); non-membership proof; exit announcement; prepare_collateral_recovery(); 10 VV tests |
| [VAL-005](NORMATIVE.md#VAL-005) | ✅ | Forced exit | prepare_forced_exit() with ForcedExitReason; slash to governance addr; multi-exit; 8 VV tests |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
