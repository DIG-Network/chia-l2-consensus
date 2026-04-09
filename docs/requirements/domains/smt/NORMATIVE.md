# Sparse Merkle Tree — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Sparse Merkle Tree

---

## §1 Tree Structure

<a id="SMT-001"></a>**SMT-001** The validator set MUST be stored in a sparse Merkle tree with fixed depth `TREE_DEPTH` (default 32), supporting 2^TREE_DEPTH validator slots.
> **Spec:** [`SMT-001.md`](../../../design/requirements/smt/SMT-001.md)

---

## §2 Slot Assignment

<a id="SMT-002"></a>**SMT-002** Each validator's slot MUST be computed deterministically as `first_8_bytes_be(sha256(pubkey)) mod 2^TREE_DEPTH`, ensuring consistent placement across implementations.
> **Spec:** [`SMT-002.md`](../../../design/requirements/smt/SMT-002.md)

---

## §3 Leaf Values

<a id="SMT-003"></a>**SMT-003** Active validator leaves MUST have value `sha256(pubkey)` (48-byte compressed G1 input). Empty slots MUST have value `EMPTY_LEAF_HASH = sha256(0x00 × 48)`.
> **Spec:** [`SMT-003.md`](../../../design/requirements/smt/SMT-003.md)

---

## §4 Proof Format

<a id="SMT-004"></a>**SMT-004** Merkle proofs MUST consist of exactly `TREE_DEPTH` sibling hashes, with sibling ordering following the convention: left child at index % 2 == 0, concatenation always as `sha256(left || right)`.
> **Spec:** [`SMT-004.md`](../../../design/requirements/smt/SMT-004.md)

---

## §5 Cross-Implementation Consistency

<a id="SMT-005"></a>**SMT-005** The Rust off-chain implementation and Chialisp on-chain verification MUST produce identical Merkle roots for the same validator set; CI tests MUST verify this property.
> **Spec:** [`SMT-005.md`](../../../design/requirements/smt/SMT-005.md)

---

## §6 Empty Tree Optimization

<a id="SMT-006"></a>**SMT-006** Empty subtrees MUST use precomputed hashes at each level to avoid redundant computation. The empty tree root at depth 32 MUST be a known constant.
> **Spec:** [`SMT-006.md`](../../../design/requirements/smt/SMT-006.md)
