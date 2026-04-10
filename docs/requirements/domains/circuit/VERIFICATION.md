# Groth16 Circuit — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [CIR-001](NORMATIVE.md#CIR-001) | ✅ | Circuit statement | ConsensusCircuit struct with public inputs/witnesses; 8 VV tests in tests/vv_req_cir_001.rs |
| [CIR-002](NORMATIVE.md#CIR-002) | ✅ | Merkle membership | prove_validator() and verify_for_pubkey() methods; 10 VV tests in tests/vv_req_cir_002.rs |
| [CIR-003](NORMATIVE.md#CIR-003) | ✅ | Aggregate key check | aggregate_pubkeys() and verify_aggregate(); 14 VV tests in tests/vv_req_cir_003.rs |
| [CIR-004](NORMATIVE.md#CIR-004) | ❌ | Majority threshold | k=50, count=100 fails; k=51, count=100 succeeds; boundary tests |
| [CIR-005](NORMATIVE.md#CIR-005) | ❌ | Public inputs | Verify 6 inputs in correct order; VK has 7 IC points |
| [CIR-006](NORMATIVE.md#CIR-006) | ❌ | Circuit parameters | MAX_SIGNERS and TREE_DEPTH match between circuit, VK, and checkpoint singleton |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
