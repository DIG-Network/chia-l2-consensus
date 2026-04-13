# Registration Coin — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Part 2: Registration Coin

---

## §1 Puzzle Structure

<a id="REG-001"></a>**REG-001** The registration coin puzzle MUST be curried with exactly two parameters: `VALIDATOR_PUBKEY` (48-byte BLS G1 point) and `CHECKPOINT_SINGLETON_ID` (32-byte coin ID of the checkpoint singleton).
> **Spec:** [`REG-001.md`](specs/REG-001.md)

---

## §2 Lineage Verification

<a id="REG-002"></a>**REG-002** A registration coin is valid if and only if its parent coin ID traces back to a network coin spend; the indexer MUST verify this lineage before including a validator in the active set.
> **Spec:** [`REG-002.md`](specs/REG-002.md)

---

## §3 Collateral Lock

<a id="REG-003"></a>**REG-003** The registration coin MUST hold the validator's collateral and MUST NOT be spendable until the checkpoint singleton emits a non-membership announcement confirming the validator is no longer in the active set.
> **Spec:** [`REG-003.md`](specs/REG-003.md)

---

## §4 Spend Conditions

<a id="REG-004"></a>**REG-004** When spent, the registration coin MUST assert a coin announcement from the checkpoint singleton matching the format `sha256(CHECKPOINT_SINGLETON_ID + sha256("membership" + epoch_be8 + VALIDATOR_PUBKEY + 0x00))` where `0x00` indicates non-membership.
> **Spec:** [`REG-004.md`](specs/REG-004.md)

<a id="REG-005"></a>**REG-005** Upon valid spend, the registration coin MUST create a coin at the specified `collateral_destination` puzzle hash with the full `collateral_amount`, returning funds to the exiting validator.
> **Spec:** [`REG-005.md`](specs/REG-005.md)

---

## §5 Epoch Replay Protection

<a id="REG-006"></a>**REG-006** The registration coin MUST verify the epoch in the membership announcement matches the current checkpoint epoch, preventing replay of old non-membership announcements.
> **Spec:** [`REG-006.md`](specs/REG-006.md)

---

## §6 End-to-End Simulator Test

<a id="REG-007"></a>**REG-007** A full end-to-end simulator test MUST exercise the complete registration coin lifecycle using the chia-wallet-sdk simulator: create a registration coin via network coin spend, verify collateral is locked, then execute collateral recovery by spending the checkpoint singleton (membership query path) and registration coin in the same spend bundle with cross-coin announcement matching.
> **Spec:** [`REG-007.md`](specs/REG-007.md)

---

## §7 CLVM Execution Validation

<a id="REG-008"></a>**REG-008** REG-001 (puzzle structure) MUST have dedicated CLVM execution tests that deserialize the compiled `.hex` artifact, curry with test parameters, run via `run_program()`, and assert exact output conditions — source-string inspection alone is insufficient per SCHEMA.md Hard Testing Requirements.
> **Spec:** [`REG-008.md`](specs/REG-008.md)

---

## §8 Failure Case Coverage

<a id="REG-009"></a>**REG-009** The E2E simulator tests (REG-007) MUST include failure-path coverage: spending without a checkpoint announcement MUST fail, spending with a wrong announcement hash MUST fail, spending with is_member=0x01 MUST fail, and spending with a wrong epoch MUST fail.
> **Spec:** [`REG-009.md`](specs/REG-009.md)

---

## §9 Simulator Spend Verification

<a id="REG-010"></a>**REG-010** REG-003 through REG-006 SHOULD have simulator-level spend bundle tests verifying that the CLVM-level behaviour (announcement assertion, collateral return, epoch binding) holds in a full consensus context with real coin spends.
> **Spec:** [`REG-010.md`](specs/REG-010.md)
