# Deployment — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [DEP-001](NORMATIVE.md#DEP-001) | ❌ | Trusted setup | Ceremony transcript published; VK verified against circuit |
| [DEP-002](NORMATIVE.md#DEP-002) | ❌ | Genesis coin | Verify launcher IDs derived from genesis; circular dep resolved |
| [DEP-003](NORMATIVE.md#DEP-003) | ❌ | Initial state | Query checkpoint singleton; verify epoch=0, count=0, empty root |
| [DEP-004](NORMATIVE.md#DEP-004) | ❌ | VK verification | Decurry deployed singleton; compare to published VK |
| [DEP-005](NORMATIVE.md#DEP-005) | ❌ | Artifact publication | All artifacts accessible; hashes match |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
