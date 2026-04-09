# Validator Operations — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — De-Registration and Collateral Recovery

---

## §1 Key Generation

<a id="VAL-001"></a>**VAL-001** Validators MUST generate a BLS12-381 keypair and securely back up the secret key before registration; loss of the secret key results in permanent loss of signing capability and complicates collateral recovery.
> **Spec:** [`VAL-001.md`](../../../design/requirements/validator/VAL-001.md)

---

## §2 Registration

<a id="VAL-002"></a>**VAL-002** To register, a validator MUST spend the network coin with their secret key, providing collateral ≥ COLLATERAL_AMOUNT, and wait for indexer confirmation before participating in consensus.
> **Spec:** [`VAL-002.md`](../../../design/requirements/validator/VAL-002.md)

---

## §3 Signing Protocol

<a id="VAL-003"></a>**VAL-003** When signing checkpoints, validators MUST sign the message `checkpoint_message || genesis_challenge || checkpoint_singleton_coin_id` using their BLS secret key; signatures MUST be provided only for valid checkpoint messages.
> **Spec:** [`VAL-003.md`](../../../design/requirements/validator/VAL-003.md)

---

## §4 Voluntary Exit

<a id="VAL-004"></a>**VAL-004** To exit voluntarily, a validator MUST signal intent at the L2 level, wait for a checkpoint that excludes them, then recover collateral using a membership query and registration coin spend in the same bundle.
> **Spec:** [`VAL-004.md`](../../../design/requirements/validator/VAL-004.md)

---

## §5 Forced Exit

<a id="VAL-005"></a>**VAL-005** A validator MAY be force-exited by majority vote; the next checkpoint excludes them and the same collateral recovery process applies, potentially with slashing to a governance address.
> **Spec:** [`VAL-005.md`](../../../design/requirements/validator/VAL-005.md)
