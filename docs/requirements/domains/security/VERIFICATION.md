# Security — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [SEC-001](NORMATIVE.md#SEC-001) | ❌ | Majority assumption | Document assumption; test minority rejection |
| [SEC-002](NORMATIVE.md#SEC-002) | ❌ | Two-check completeness | Test proof-only attack fails; test sig-only attack fails |
| [SEC-003](NORMATIVE.md#SEC-003) | ❌ | Collateral security | Active validator cannot recover; only non-members can |
| [SEC-004](NORMATIVE.md#SEC-004) | ❌ | Trusted setup | MPC ceremony documentation; reject single-party in prod |
| [SEC-005](NORMATIVE.md#SEC-005) | ❌ | Lineage enforcement | Fake registration coin ignored by indexer |
| [SEC-006](NORMATIVE.md#SEC-006) | ❌ | Epoch replay | Re-registered validator cannot use old announcement |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
