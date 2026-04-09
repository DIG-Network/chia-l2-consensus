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
