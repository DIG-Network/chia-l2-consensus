# Wire Format — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [WIRE-001](NORMATIVE.md#WIRE-001) | ✅ | Checkpoint message | Manual hash computation; verify Rust matches Rue (7 VV tests in tests/vv_req_wire_001.rs) |
| [WIRE-002](NORMATIVE.md#WIRE-002) | ✅ | Point encoding | Arkworks serialization verified (11 VV tests in tests/vv_req_wire_002.rs) |
| [WIRE-003](NORMATIVE.md#WIRE-003) | ❌ | Proof format | Verify proof is 192 bytes; A/B/C in correct order |
| [WIRE-004](NORMATIVE.md#WIRE-004) | ❌ | Membership announcement | Compare with checkpoint singleton output; test both member values |
| [WIRE-005](NORMATIVE.md#WIRE-005) | ❌ | Registration message | Verify network coin AGG_SIG_ME matches; 56-byte input |
| [WIRE-006](NORMATIVE.md#WIRE-006) | ❌ | scalar() function | Test vectors; verify mod r reduction; cross-impl check |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
