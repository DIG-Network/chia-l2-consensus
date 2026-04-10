# Groth16 Circuit — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [CIR-001](NORMATIVE.md#CIR-001) | ✅ | Circuit statement | ConsensusCircuit struct with public inputs/witnesses; 8 VV tests in tests/vv_req_cir_001.rs |
| [CIR-002](NORMATIVE.md#CIR-002) | ✅ | Merkle membership | prove_validator() and verify_for_pubkey() methods; 10 VV tests in tests/vv_req_cir_002.rs |
| [CIR-003](NORMATIVE.md#CIR-003) | ✅ | Aggregate key check | aggregate_pubkeys() and verify_aggregate(); 14 VV tests in tests/vv_req_cir_003.rs |
| [CIR-004](NORMATIVE.md#CIR-004) | ✅ | Majority threshold | is_majority() and minimum_signers(); 16 VV tests in tests/vv_req_cir_004.rs |
| [CIR-005](NORMATIVE.md#CIR-005) | ✅ | Public inputs | PUBLIC_INPUT_COUNT=6, public_input_index module; 12 VV tests in tests/vv_req_cir_005.rs |
| [CIR-006](NORMATIVE.md#CIR-006) | ✅ | Circuit parameters | MAX_SIGNERS=20,000, TREE_DEPTH=32 as constants; 13 VV tests in tests/vv_req_cir_006.rs |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
