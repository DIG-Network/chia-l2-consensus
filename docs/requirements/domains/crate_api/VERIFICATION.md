# Crate API — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [API-001](NORMATIVE.md#API-001) | ✅ | Public API surface | lib.rs: minimal public exports; pub mod testing for VV tests; ~55 test files migrated; full suite passes |
| [API-002](NORMATIVE.md#API-002) | ❌ | NetworkConfig completeness | serde round-trip; `verification_key()` returns valid VK; `checkpoint_singleton_id()` derives correctly |
| [API-003](NORMATIVE.md#API-003) | ❌ | State types | `NetworkState` compiles; `ValidatorInfo` has pubkey+coin; `ValidatorSet` helpers work |
| [API-004](NORMATIVE.md#API-004) | ❌ | Client state accessors | All 7 accessor methods return correct values from synced state |
| [API-005](NORMATIVE.md#API-005) | ❌ | Client message computation | 5 message methods produce correct bytes matching wire format specs |
| [API-006](NORMATIVE.md#API-006) | ✅ | Module visibility | indexer, merkle, prover, puzzles, validator all pub(crate); testing module provides VV access |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
