# Groth16 Circuit — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [CIR-001](NORMATIVE.md#CIR-001) | ❌ | Circuit statement | Generate proof with valid witnesses; verify on-chain; test edge cases |
| [CIR-002](NORMATIVE.md#CIR-002) | ❌ | Merkle membership | Proofs with invalid paths fail; valid paths succeed; cross-impl test Rust=Rue |
| [CIR-003](NORMATIVE.md#CIR-003) | ❌ | Aggregate key check | Wrong agg_signers fails; correct sum succeeds; single-key test |
| [CIR-004](NORMATIVE.md#CIR-004) | ❌ | Majority threshold | k=50, count=100 fails; k=51, count=100 succeeds; boundary tests |
| [CIR-005](NORMATIVE.md#CIR-005) | ❌ | Public inputs | Verify 6 inputs in correct order; VK has 7 IC points |
| [CIR-006](NORMATIVE.md#CIR-006) | ❌ | Circuit parameters | MAX_SIGNERS and TREE_DEPTH match between circuit, VK, and checkpoint singleton |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
