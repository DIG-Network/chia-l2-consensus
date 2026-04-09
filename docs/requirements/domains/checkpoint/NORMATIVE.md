# Checkpoint Singleton — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Part 3: Checkpoint Singleton

---

## §1 Singleton Identity

<a id="CHK-001"></a>**CHK-001** The checkpoint singleton MUST be a Chia singleton that serves as the canonical on-chain source of truth for L2 state, with exactly one instance per L2 network.
> **Spec:** [`CHK-001.md`](../../../design/requirements/checkpoint/CHK-001.md)

---

## §2 State Tracking

<a id="CHK-002"></a>**CHK-002** The checkpoint singleton MUST track four state values curried into its inner puzzle on each recreation: `state_root` (32 bytes), `epoch` (u64, auto-incremented), `validator_merkle_root` (32 bytes), and `validator_count` (u64).
> **Spec:** [`CHK-002.md`](../../../design/requirements/checkpoint/CHK-002.md)

---

## §3 Spend Path 1: Checkpoint

<a id="CHK-003"></a>**CHK-003** The checkpoint spend path MUST verify a Groth16 proof demonstrating k pubkeys are members of the validator set and 2k > validator_count, then verify a BLS aggregate signature over the checkpoint message.
> **Spec:** [`CHK-003.md`](../../../design/requirements/checkpoint/CHK-003.md)

<a id="CHK-004"></a>**CHK-004** Upon successful checkpoint verification, the singleton MUST increment epoch by exactly 1, update all state values to the new values from the checkpoint message, emit a checkpoint state announcement, and recreate itself with the new state.
> **Spec:** [`CHK-004.md`](../../../design/requirements/checkpoint/CHK-004.md)

---

## §4 Spend Path 2: Membership Query

<a id="CHK-005"></a>**CHK-005** The membership query spend path MUST verify a Merkle membership or non-membership proof against the current `validator_merkle_root`, emit a membership announcement with the query result, and recreate the singleton unchanged.
> **Spec:** [`CHK-005.md`](../../../design/requirements/checkpoint/CHK-005.md)

<a id="CHK-006"></a>**CHK-006** The membership query spend path MUST be permissionless (no signature required), allowing any party to query membership status at any time without special authorization.
> **Spec:** [`CHK-006.md`](../../../design/requirements/checkpoint/CHK-006.md)

---

## §5 Verification Key

<a id="CHK-007"></a>**CHK-007** The Groth16 verification key (672 bytes) MUST be curried into the checkpoint singleton at deployment and MUST NOT change, permanently binding the singleton to a specific circuit and trusted setup.
> **Spec:** [`CHK-007.md`](../../../design/requirements/checkpoint/CHK-007.md)
