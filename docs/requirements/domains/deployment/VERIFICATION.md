# Deployment — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [DEP-001](NORMATIVE.md#DEP-001) | ✅ | Trusted setup | run_test_setup() produces valid PK/VK; VK validated (7 IC, 672 bytes); hash deterministic; proof round-trips; 12 VV tests |
| [DEP-002](NORMATIVE.md#DEP-002) | ✅ | Genesis coin | derive_launcher_id() matches SDK; deploy_both_singletons() atomic; config populated; 8 VV tests |
| [DEP-003](NORMATIVE.md#DEP-003) | ✅ | Initial state | initial_checkpoint_state(): epoch=0, count=0, merkle_root=EMPTY_TREE_ROOT; 9 VV tests |
| [DEP-004](NORMATIVE.md#DEP-004) | ✅ | VK verification | verify_vk_hash(), validate_vk_bytes(), extract_vk_components_from_bytes(); hash match/mismatch/corruption; 10 VV tests |
| [DEP-005](NORMATIVE.md#DEP-005) | ✅ | Artifact publication | DeploymentArtifacts JSON with all fields; VkJson structure; round-trip; hash match; 8 VV tests |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
