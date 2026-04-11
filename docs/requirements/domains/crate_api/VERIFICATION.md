# Crate API — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [API-001](NORMATIVE.md#API-001) | ✅ | Public API surface | lib.rs: minimal public exports; pub mod testing for VV tests; ~55 test files migrated |
| [API-002](NORMATIVE.md#API-002) | ✅ | NetworkConfig completeness | JSON round-trip with Bytes32 hex; verification_key() deserializes VK; checkpoint_singleton_id() matches; 6 VV tests |
| [API-003](NORMATIVE.md#API-003) | ✅ | State types | ValidatorInfo exists; ValidatorSet count/contains/pubkeys work; empty set handled; 5 VV tests |
| [API-004](NORMATIVE.md#API-004) | ✅ | Client state accessors | epoch/state_root/etc return NotDeployed before sync; set_cache_path works; 3 VV tests |
| [API-005](NORMATIVE.md#API-005) | ✅ | Client message computation | checkpoint_message/signing_message/is_active/announcement all return NotDeployed; 5 VV tests |
| [API-006](NORMATIVE.md#API-006) | ✅ | Module visibility | indexer, merkle, prover, puzzles, validator all pub(crate); testing module provides VV access |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
