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

---

## §6 Epoch Binding

<a id="CHK-009"></a>**CHK-009** The checkpoint message MUST include `new_epoch` (8-byte big-endian u64) in its SHA-256 preimage, and the CLVM puzzle MUST compute `new_epoch = old_epoch + 1` internally rather than accepting it from the solution, binding each Groth16 proof to exactly one epoch value.
> **Spec:** [`CHK-009.md`](specs/CHK-009.md)

<a id="CHK-010"></a>**CHK-010** Only one checkpoint spend per epoch MUST be accepted. This is enforced by: (a) the singleton pattern — spending consumes the coin and creates a successor with epoch+1, (b) the checkpoint_message hash including the epoch — a proof for epoch N cannot verify at epoch M because the message differs, (c) the BLS signature binding — validators sign a message containing the epoch.
> **Spec:** [`CHK-010.md`](specs/CHK-010.md)

---

## §7 State Hash Binding

<a id="CHK-011"></a>**CHK-011** The checkpoint_message hash MUST include `new_state_root` as the first field of its SHA-256 preimage, binding the Groth16 proof and BLS signature to a specific L2 state transition. The CLVM puzzle MUST use the same state_root from the solution in both the checkpoint_message computation and the singleton recreation.
> **Spec:** [`CHK-011.md`](specs/CHK-011.md)

---

## §8 Network ID Binding

<a id="CHK-012"></a>**CHK-012** The checkpoint_message hash MUST include the `network_coin_launcher_id` (32 bytes) in its SHA-256 preimage, preventing proofs generated for one L2 network from being replayed on another. The network_coin_launcher_id MUST be curried into the checkpoint singleton puzzle at deployment and MUST NOT be accepted from the solution.
> **Spec:** [`CHK-012.md`](specs/CHK-012.md)

---

---

## §9 Validator Attestation Binding

<a id="CHK-013"></a>**CHK-013** Validators MUST sign a message that attests to the epoch number, network ID (network_coin_launcher_id), and state hash (new_state_root). The CLVM puzzle MUST verify via `bls_verify` that the aggregate signature is over the checkpoint_message containing all three fields. The Groth16 proof MUST prove the signers form a legitimate majority. Together, this proves a majority of validators attested to the specific epoch + network + state combination.
> **Spec:** [`CHK-013.md`](specs/CHK-013.md)

---

## §10 End-to-End Integration Test

<a id="CHK-008"></a>**CHK-008** A full end-to-end integration test MUST exercise the complete lifecycle: deploy network coin + checkpoint singleton, register validators, collect signatures, generate a real Groth16 proof, submit a checkpoint spend via chia-wallet-sdk simulator, verify state update, then execute collateral recovery via membership query + registration coin spend in the same bundle.
> **Spec:** [`CHK-008.md`](../../../design/requirements/checkpoint/CHK-008.md)
