# Registration Coin — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Part 2: Registration Coin

---

## §1 Puzzle Structure

<a id="REG-001"></a>**REG-001** The registration coin puzzle MUST be curried with exactly two parameters: `VALIDATOR_PUBKEY` (48-byte BLS G1 point) and `CHECKPOINT_SINGLETON_ID` (32-byte coin ID of the checkpoint singleton).
> **Spec:** [`REG-001.md`](../../../design/requirements/registration_coin/REG-001.md)

---

## §2 Lineage Verification

<a id="REG-002"></a>**REG-002** A registration coin is valid if and only if its parent coin ID traces back to a network coin spend; the indexer MUST verify this lineage before including a validator in the active set.
> **Spec:** [`REG-002.md`](../../../design/requirements/registration_coin/REG-002.md)

---

## §3 Collateral Lock

<a id="REG-003"></a>**REG-003** The registration coin MUST hold the validator's collateral and MUST NOT be spendable until the checkpoint singleton emits a non-membership announcement confirming the validator is no longer in the active set.
> **Spec:** [`REG-003.md`](../../../design/requirements/registration_coin/REG-003.md)

---

## §4 Spend Conditions

<a id="REG-004"></a>**REG-004** When spent, the registration coin MUST assert a coin announcement from the checkpoint singleton matching the format `sha256(CHECKPOINT_SINGLETON_ID + sha256("membership" + epoch_be8 + VALIDATOR_PUBKEY + 0x00))` where `0x00` indicates non-membership.
> **Spec:** [`REG-004.md`](../../../design/requirements/registration_coin/REG-004.md)

<a id="REG-005"></a>**REG-005** Upon valid spend, the registration coin MUST create a coin at the specified `collateral_destination` puzzle hash with the full `collateral_amount`, returning funds to the exiting validator.
> **Spec:** [`REG-005.md`](../../../design/requirements/registration_coin/REG-005.md)

---

## §5 Epoch Replay Protection

<a id="REG-006"></a>**REG-006** The registration coin MUST verify the epoch in the membership announcement matches the current checkpoint epoch, preventing replay of old non-membership announcements.
> **Spec:** [`REG-006.md`](../../../design/requirements/registration_coin/REG-006.md)
