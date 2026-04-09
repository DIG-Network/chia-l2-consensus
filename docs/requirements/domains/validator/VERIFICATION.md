# Validator Operations — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [VAL-001](NORMATIVE.md#VAL-001) | ❌ | Key generation | Document key generation; verify backup/restore works |
| [VAL-002](NORMATIVE.md#VAL-002) | ❌ | Registration | Complete registration flow; verify indexer detects new validator |
| [VAL-003](NORMATIVE.md#VAL-003) | ❌ | Signing protocol | Sign checkpoint; verify signature accepted; test invalid message rejection |
| [VAL-004](NORMATIVE.md#VAL-004) | ❌ | Voluntary exit | Complete exit flow; verify collateral returned to validator |
| [VAL-005](NORMATIVE.md#VAL-005) | ❌ | Forced exit | Simulate majority vote; verify validator excluded; test slashing path |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
