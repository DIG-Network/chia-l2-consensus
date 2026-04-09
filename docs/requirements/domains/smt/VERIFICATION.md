# Sparse Merkle Tree — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [SMT-001](NORMATIVE.md#SMT-001) | ✅ | Fixed depth tree | Tree operations use TREE_DEPTH consistently; roots computed correctly |
| [SMT-002](NORMATIVE.md#SMT-002) | ✅ | Deterministic slots | Same pubkey → same slot across runs; collision detection works |
| [SMT-003](NORMATIVE.md#SMT-003) | ✅ | Leaf values | sha256(pubkey) for active; sha256(zeros) for empty; test vectors |
| [SMT-004](NORMATIVE.md#SMT-004) | ✅ | Proof format | Proofs have TREE_DEPTH siblings; sibling ordering matches spec |
| [SMT-005](NORMATIVE.md#SMT-005) | ❌ | Cross-impl consistency | CI test: Rust root == Chialisp root for same inputs |
| [SMT-006](NORMATIVE.md#SMT-006) | ❌ | Empty tree optimization | Empty subtrees use precomputed hashes; known empty root constant |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
