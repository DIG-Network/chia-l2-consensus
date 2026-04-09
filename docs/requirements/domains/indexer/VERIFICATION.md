# Indexer — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [IDX-001](NORMATIVE.md#IDX-001) | ❌ | State tracking | Verify all state components present after sync; query each field |
| [IDX-002](NORMATIVE.md#IDX-002) | ❌ | Lineage verification | Create fake registration coin; verify indexer ignores it |
| [IDX-003](NORMATIVE.md#IDX-003) | ❌ | Merkle consistency | Manually corrupt tree; verify StateMismatch error |
| [IDX-004](NORMATIVE.md#IDX-004) | ❌ | Reorg handling | Simulate reorg; verify rollback and re-sync |
| [IDX-005](NORMATIVE.md#IDX-005) | ❌ | Persistent cache | Kill and restart; verify fast startup from cache |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
