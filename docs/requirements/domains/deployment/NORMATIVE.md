# Deployment — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Deployment

---

## §1 Trusted Setup

<a id="DEP-001"></a>**DEP-001** Before deployment, a Groth16 trusted setup ceremony MUST be performed to generate the proving key and verification key bound to the circuit parameters (MAX_SIGNERS, TREE_DEPTH).
> **Spec:** [`DEP-001.md`](../../../design/requirements/deployment/DEP-001.md)

---

## §2 Genesis Coin

<a id="DEP-002"></a>**DEP-002** Deployment MUST use a genesis coin (≥2 XCH recommended) from which both singleton launcher IDs are derived, resolving circular dependencies between network coin and checkpoint singleton.
> **Spec:** [`DEP-002.md`](../../../design/requirements/deployment/DEP-002.md)

---

## §3 Initial State

<a id="DEP-003"></a>**DEP-003** The checkpoint singleton MUST be deployed with initial state: `epoch=0`, `validator_count=0`, `validator_merkle_root=EMPTY_TREE_ROOT`, and `state_root` as application-defined genesis.
> **Spec:** [`DEP-003.md`](../../../design/requirements/deployment/DEP-003.md)

---

## §4 Verification Key Verification

<a id="DEP-004"></a>**DEP-004** After deployment, operators MUST verify the VK curried into the checkpoint singleton matches the expected VK from the trusted setup ceremony by decurrying and comparing.
> **Spec:** [`DEP-004.md`](../../../design/requirements/deployment/DEP-004.md)

---

## §5 Artifact Publication

<a id="DEP-005"></a>**DEP-005** Deployment artifacts MUST be published: network_config.json, verification key (hex), VK hash, ceremony transcript, and circuit source code for independent verification.
> **Spec:** [`DEP-005.md`](../../../design/requirements/deployment/DEP-005.md)
