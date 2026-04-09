# Groth16 Circuit — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Circuit Public Inputs / Circuit Statement

---

## §1 Circuit Statement

<a id="CIR-001"></a>**CIR-001** The Groth16 circuit MUST prove: "I know k BLS pubkeys, each with a valid Merkle inclusion proof against `validator_merkle_root`, whose G1 sum equals `agg_signers`, and where 2k > `validator_count`."
> **Spec:** [`CIR-001.md`](../../../design/requirements/circuit/CIR-001.md)

---

## §2 Constraints

<a id="CIR-002"></a>**CIR-002** The circuit MUST verify Merkle membership for each of the k signing pubkeys against the current `validator_merkle_root` using the sparse Merkle tree structure defined in SMT requirements.
> **Spec:** [`CIR-002.md`](../../../design/requirements/circuit/CIR-002.md)

<a id="CIR-003"></a>**CIR-003** The circuit MUST verify that the G1 sum of the k signing pubkeys equals the public input `agg_signers`, ensuring the aggregate key corresponds exactly to the claimed signers.
> **Spec:** [`CIR-003.md`](../../../design/requirements/circuit/CIR-003.md)

<a id="CIR-004"></a>**CIR-004** The circuit MUST enforce the majority threshold constraint: `2k > validator_count`, where k is the number of signers and validator_count is a public input.
> **Spec:** [`CIR-004.md`](../../../design/requirements/circuit/CIR-004.md)

---

## §3 Public Inputs

<a id="CIR-005"></a>**CIR-005** The circuit MUST accept exactly 6 public inputs in fixed order: `validator_merkle_root`, `validator_count`, `new_validator_merkle_root`, `new_validator_count`, `agg_signers`, and `checkpoint_message`.
> **Spec:** [`CIR-005.md`](../../../design/requirements/circuit/CIR-005.md)

---

## §4 Parameters

<a id="CIR-006"></a>**CIR-006** The circuit MUST be parameterized by `MAX_SIGNERS` (maximum simultaneous signers) and `TREE_DEPTH` (Merkle tree depth), both fixed at trusted setup time.
> **Spec:** [`CIR-006.md`](../../../design/requirements/circuit/CIR-006.md)
