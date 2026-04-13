# Security — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Security

---

## §1 Majority Assumption

<a id="SEC-001"></a>**SEC-001** The system assumes a majority of validators (>50%) are honest; if this assumption is violated, fraudulent checkpoints can be submitted and the system provides no safety guarantees.
> **Spec:** [`SEC-001.md`](../../../design/requirements/security/SEC-001.md)

---

## §2 Two-Check Completeness

<a id="SEC-002"></a>**SEC-002** The Groth16 proof verification and BLS signature verification together MUST provide complete security; neither check alone is sufficient to prevent fraudulent checkpoints.
> **Spec:** [`SEC-002.md`](../../../design/requirements/security/SEC-002.md)

---

## §3 Collateral Security

<a id="SEC-003"></a>**SEC-003** A validator MUST NOT be able to recover their collateral without a checkpoint that excludes them from the active set, as verified by a non-membership proof.
> **Spec:** [`SEC-003.md`](../../../design/requirements/security/SEC-003.md)

---

## §4 Trusted Setup

<a id="SEC-004"></a>**SEC-004** The Groth16 trusted setup MUST be performed as a multi-party computation ceremony; at least one honest participant is required for soundness. Single-party setups MUST NOT be used in production.
> **Spec:** [`SEC-004.md`](../../../design/requirements/security/SEC-004.md)

---

## §5 Lineage Enforcement

<a id="SEC-005"></a>**SEC-005** Registration coin lineage verification MUST be enforced by the indexer; coins without valid lineage MUST be excluded from the validator set regardless of puzzle hash.
> **Spec:** [`SEC-005.md`](../../../design/requirements/security/SEC-005.md)

---

## §6 Epoch Replay Protection

<a id="SEC-006"></a>**SEC-006** The epoch included in membership announcements MUST prevent replay attacks where old non-membership proofs are reused after a validator re-registers.
> **Spec:** [`SEC-006.md`](../../../design/requirements/security/SEC-006.md)

---

## §7 CLVM Vulnerability Audit

<a id="SEC-007"></a>**SEC-007** All known Chialisp/CLVM vulnerabilities from the Chia knowledge graph MUST be assessed against the three Rue puzzles. Mitigated vulnerabilities (CATbleed, AGG_SIG_UNSAFE, flash loans) MUST be verified as mitigated. Gaps (condition injection, unsigned destinations, cross-network replay) MUST have remediation requirements.
> **Spec:** [`SEC-007.md`](specs/SEC-007.md)

---

## §8 Condition Injection Protection

<a id="SEC-008"></a>**SEC-008** Passthrough `conditions` from puzzle solutions MUST be either removed, filtered to a safe whitelist, or signed via AGG_SIG_ME to prevent malicious condition injection (CREATE_COIN theft, RESERVE_FEE drain, AGG_SIG_UNSAFE injection).
> **Spec:** [`SEC-008.md`](specs/SEC-008.md)

---

## §9 Registration Coin Destination Binding

<a id="SEC-009"></a>**SEC-009** The registration coin's `collateral_destination` MUST be protected against RBF/mempool substitution attacks, either by requiring a validator signature over the destination hash or by documenting the risk as mitigated by the announcement requirement.
> **Spec:** [`SEC-009.md`](specs/SEC-009.md)

---

## §10 Comprehensive Attack Surface

<a id="SEC-010"></a>**SEC-010** All 20 identified attack vectors (A-T) MUST be verified as addressed, mitigated, or acknowledged. This includes: proof replay, cross-network replay, state forgery, minority checkpoint, fake registration, collateral theft, epoch manipulation, double checkpoint, signature subtraction, rogue key, Merkle forgery, front-running, singleton destruction, VK substitution, spam, censorship, stale proofs, announcement spoofing, and bundle splitting.
> **Spec:** [`SEC-010.md`](specs/SEC-010.md)

---

## §11 Phantom Majority Forgery Resistance

<a id="SEC-011"></a>**SEC-011** The system MUST prevent phantom majority attacks where an attacker with the proving key and a single BLS keypair forges a Groth16 proof claiming an arbitrary number of signers. CIR-003 (aggregate key constraint) MUST be enforced in the circuit, binding `agg_signers` to the G1 sum of k legitimate validator pubkeys. Production proofs MUST supply signing pubkeys to `with_public_inputs()` to activate CIR-003 enforcement.
> **Spec:** [`SEC-011.md`](specs/SEC-011.md)
