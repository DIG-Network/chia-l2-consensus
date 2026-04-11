# Checkpoint Singleton — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [CHK-001](NORMATIVE.md#CHK-001) | ✅ | Singleton identity | Rue compiles; .hex/.hash artifacts; loads into CLVM; 5 tests |
| [CHK-002](NORMATIVE.md#CHK-002) | ✅ | State tracking | Source has 4 state params + TREE_DEPTH; 2 tests |
| [CHK-003](NORMATIVE.md#CHK-003) | ✅ | Groth16 + BLS verify | Full implementation: bls_pairing_identity, bls_verify, g1_multiply, g1_negate in compiled CLVM; scalar verification via assert; VK input with 7 IC points; pairing equation matches spec; 15 VV tests in vv_req_chk_003.rs |
| [CHK-004](NORMATIVE.md#CHK-004) | ✅ | State update | CLVM execution: checkpoint path produces CREATE_COIN + announcement; 1 test |
| [CHK-005](NORMATIVE.md#CHK-005) | ✅ | Membership query | CLVM execution: depth=0 membership/non-membership verified; announcement cross-impl hash match; 3 VV tests |
| [CHK-006](NORMATIVE.md#CHK-006) | ✅ | Permissionless query | CLVM execution: no AGG_SIG_ME/UNSAFE in membership query output; 1 VV test |
| [CHK-007](NORMATIVE.md#CHK-007) | ✅ | VK immutability | VK_HASH curried in; included in curry_tree_hash recreation; 2 tests |

| [CHK-008](NORMATIVE.md#CHK-008) | ❌ | E2E integration test | Full lifecycle: deploy, register, checkpoint with real Groth16 proof, collateral recovery — all via simulator |

| [CHK-009](NORMATIVE.md#CHK-009) | ✅ | Epoch binding | Epoch in checkpoint_message; puzzle computes new_epoch=old+1; scalar s6 changes; proof differs; 9 VV tests |
| [CHK-010](NORMATIVE.md#CHK-010) | ✅ | Single checkpoint per epoch | Signature epoch mismatch; aggregate epoch-bound; singleton recreation; replay prevented; 7 VV tests |

| [CHK-011](NORMATIVE.md#CHK-011) | ✅ | State hash binding | state_root is first field; different roots → different messages/scalars/proofs; Rue uses same root for msg+recreation; 8 VV tests |
| [CHK-012](NORMATIVE.md#CHK-012) | ✅ | Network ID binding | network_coin_launcher_id curried in puzzle; 112-byte preimage; different networks → different messages; 7 VV tests |
| [CHK-013](NORMATIVE.md#CHK-013) | ✅ | Validator attestation | Validators sign epoch+network+state; bls_verify + Groth16 prove majority attestation; 11 VV tests |
| [CHK-014](NORMATIVE.md#CHK-014) | ✅ | Permissionless/forgery | No AGG_SIG (anyone submits); both checks reject forgery; minority rejected; internal computation; 11 VV tests |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
