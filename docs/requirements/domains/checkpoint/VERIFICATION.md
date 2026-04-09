# Checkpoint Singleton — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [CHK-001](NORMATIVE.md#CHK-001) | ❌ | Singleton identity | Deploy checkpoint singleton; verify singleton wrapper; attempt duplicate creation fails |
| [CHK-002](NORMATIVE.md#CHK-002) | ❌ | State tracking | Decurry puzzle; verify 4 state values present with correct types and sizes |
| [CHK-003](NORMATIVE.md#CHK-003) | ❌ | Groth16 + BLS verify | Submit valid checkpoint → success; invalid proof → failure; minority signers → failure |
| [CHK-004](NORMATIVE.md#CHK-004) | ❌ | State update | After checkpoint, verify epoch+1, state values match solution, announcement emitted |
| [CHK-005](NORMATIVE.md#CHK-005) | ❌ | Membership query | Query with valid proof → correct announcement; invalid proof → failure; singleton unchanged |
| [CHK-006](NORMATIVE.md#CHK-006) | ❌ | Permissionless query | Submit membership query without signature → succeeds; no AGG_SIG conditions |
| [CHK-007](NORMATIVE.md#CHK-007) | ❌ | VK immutability | Decurry deployed puzzle; VK matches expected 672 bytes; VK unchangeable across spends |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
