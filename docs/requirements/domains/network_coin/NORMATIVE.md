# Network Coin — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Part 1: Network Coin

---

## §1 Singleton Identity

<a id="NET-001"></a>**NET-001** The network coin MUST be a Chia singleton with exactly one instance per L2 network, serving as the canonical registration authority for all validators.
> **Spec:** [`NET-001.md`](specs/NET-001.md)

---

## §2 Registration Flow

<a id="NET-002"></a>**NET-002** When spent for validator registration, the network coin MUST verify an `AggSigMe` condition proving the registering validator controls the claimed pubkey, where the message is `sha256("register" + pubkey)`.
> **Spec:** [`NET-002.md`](specs/NET-002.md)

<a id="NET-003"></a>**NET-003** The network coin MUST create a registration coin with puzzle hash `curry_hash(REGISTRATION_COIN_MOD_HASH, pubkey, CHECKPOINT_SINGLETON_ID)` and amount equal to `COLLATERAL_AMOUNT`.
> **Spec:** [`NET-003.md`](specs/NET-003.md)

---

## §3 Self-Recreation

<a id="NET-004"></a>**NET-004** After creating a registration coin, the network coin MUST recreate itself at `MY_PUZZLE_HASH` with `MY_AMOUNT` (1 mojo) to allow subsequent registrations.
> **Spec:** [`NET-004.md`](specs/NET-004.md)

---

## §4 Driver Convention

<a id="NET-005"></a>**NET-005** The network coin driver MUST include the validator pubkey (48 bytes) as the first memo on the `CreateCoin` condition that creates the registration coin, enabling indexer detection.
> **Spec:** [`NET-005.md`](specs/NET-005.md)

---

## §5 End-to-End Simulator Test

<a id="NET-006"></a>**NET-006** A full end-to-end simulator test MUST exercise the complete network coin lifecycle using the chia-wallet-sdk simulator: deploy the network coin as a singleton (wrapping the inner puzzle via chia-wallet-sdk), register a validator with a real BLS signature, verify the registration coin is created with the correct puzzle hash and collateral amount, and verify the network coin is recreated as a new singleton coin.
> **Spec:** [`NET-006.md`](specs/NET-006.md)

---

## §6 CLVM Execution Validation

<a id="NET-007"></a>**NET-007** Each puzzle-behaviour requirement (NET-001 through NET-004) MUST have dedicated CLVM execution tests that deserialize the compiled `.hex` artifact, curry with test parameters, run via `run_program()`, and assert exact output conditions — source-string inspection alone is insufficient per SCHEMA.md Hard Testing Requirements.
> **Spec:** [`NET-007.md`](specs/NET-007.md)

---

## §7 Failure Case Coverage

<a id="NET-008"></a>**NET-008** The E2E simulator tests (NET-006) MUST include failure-path coverage: wrong BLS signature MUST be rejected, insufficient collateral MUST be rejected, and mismatched pubkey in signature MUST cause AGG_SIG_ME failure.
> **Spec:** [`NET-008.md`](specs/NET-008.md)
