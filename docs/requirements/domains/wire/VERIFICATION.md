# Wire Format — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [WIRE-001](NORMATIVE.md#WIRE-001) | ✅ | Checkpoint message | Manual hash computation; verify Rust matches Rue (7 VV tests in tests/vv_req_wire_001.rs) |
| [WIRE-002](NORMATIVE.md#WIRE-002) | ✅ | Point encoding | Arkworks serialization verified (11 VV tests in tests/vv_req_wire_002.rs) |
| [WIRE-003](NORMATIVE.md#WIRE-003) | ✅ | Proof format | ClvmProof struct with a/b/c fields; 8 VV tests in tests/vv_req_wire_003.rs |
| [WIRE-004](NORMATIVE.md#WIRE-004) | ✅ | Membership announcement | 12 VV tests in tests/vv_req_wire_004.rs; tests both member values |
| [WIRE-005](NORMATIVE.md#WIRE-005) | ✅ | Registration message | 9 VV tests in tests/vv_req_wire_005.rs; 56-byte input verified |
| [WIRE-006](NORMATIVE.md#WIRE-006) | ❌ | scalar() function | Test vectors; verify mod r reduction; cross-impl check |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
