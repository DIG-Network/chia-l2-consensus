# Registration Coin — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [REG-001](NORMATIVE.md#REG-001) | ❌ | Curried parameters | Decurry puzzle; verify exactly 2 params: 48-byte pubkey + 32-byte singleton ID |
| [REG-002](NORMATIVE.md#REG-002) | ❌ | Lineage verification | Create coin without network coin parent; verify indexer rejects it |
| [REG-003](NORMATIVE.md#REG-003) | ❌ | Collateral lock | Attempt spend without announcement; verify rejection; hold amount equals COLLATERAL_AMOUNT |
| [REG-004](NORMATIVE.md#REG-004) | ❌ | Announcement assertion | Spend with valid non-membership announcement succeeds; wrong format rejected |
| [REG-005](NORMATIVE.md#REG-005) | ❌ | Collateral return | After spend, coin at destination has full collateral amount |
| [REG-006](NORMATIVE.md#REG-006) | ❌ | Epoch replay protection | Attempt spend with stale epoch announcement; verify rejection |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
