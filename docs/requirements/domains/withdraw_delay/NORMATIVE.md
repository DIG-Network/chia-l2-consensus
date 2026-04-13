# Withdraw Delay Coin — Normative Requirements

> **Master spec:** [spec-withdraw-delay-coin.md](../../../resources/spec-withdraw-delay-coin.md)

---

## §1 Puzzle Structure

<a id="WDC-001"></a>**WDC-001** The withdraw delay coin puzzle MUST be curried with exactly three parameters: `DESTINATION` (Bytes32 — puzzle hash for fund release), `AMOUNT` (Int — collateral in mojos), and `WITHDRAW_DELAY_BLOCKS` (Int — L1 block delay). The solution MUST be empty with no passthrough conditions.
> **Spec:** [`WDC-001.md`](specs/WDC-001.md)

---

## §2 Time Lock

<a id="WDC-002"></a>**WDC-002** The withdraw delay coin MUST emit `ASSERT_HEIGHT_RELATIVE(WITHDRAW_DELAY_BLOCKS)` to enforce that the configured number of L1 blocks have passed since coin creation before funds can be released.
> **Spec:** [`WDC-002.md`](specs/WDC-002.md)

---

## §3 Fund Release

<a id="WDC-003"></a>**WDC-003** Upon successful spend (after delay), the withdraw delay coin MUST create a coin at `DESTINATION` with `AMOUNT`, releasing the full collateral to the address the exiting validator specified at recovery time.
> **Spec:** [`WDC-003.md`](specs/WDC-003.md)

---

## §4 Registration Coin Integration

<a id="WDC-004"></a>**WDC-004** The registration coin puzzle MUST be updated to create a withdraw delay coin instead of sending collateral directly to the destination. The registration coin MUST compute the withdraw delay coin puzzle hash on-chain using `curry_hash(WITHDRAW_DELAY_MOD_HASH, destination, amount, WITHDRAW_DELAY_BLOCKS)` and MUST have `WITHDRAW_DELAY_MOD_HASH` and `WITHDRAW_DELAY_BLOCKS` as additional curried parameters.
> **Spec:** [`WDC-004.md`](specs/WDC-004.md)

---

## §5 Driver and API

<a id="WDC-005"></a>**WDC-005** The crate MUST provide a `release_collateral()` method on `ConsensusClient` that builds a `SpendBundle` to spend a withdraw delay coin after the delay period has elapsed. The method MUST return the bundle without broadcasting it (per API-008).
> **Spec:** [`WDC-005.md`](specs/WDC-005.md)

---

## §6 Configuration

<a id="WDC-006"></a>**WDC-006** `NetworkConfig` MUST include `withdraw_delay_blocks` (u64) and `withdraw_delay_mod_hash` (Bytes32) fields. The delay value MUST be fixed at deployment time and curried into all registration coin puzzles.
> **Spec:** [`WDC-006.md`](specs/WDC-006.md)

---

## §7 Permissionless Release

<a id="WDC-007"></a>**WDC-007** The withdraw delay coin MUST be permissionless to spend (no AGG_SIG conditions). After the delay period, any party MUST be able to release the funds to the curried destination. This ensures validators don't need to be online at the exact moment the delay expires.
> **Spec:** [`WDC-007.md`](specs/WDC-007.md)

---

## §8 CLVM Execution Tests

<a id="WDC-008"></a>**WDC-008** WDC-001 through WDC-003 MUST have dedicated CLVM execution tests that deserialize the compiled `.hex` artifact, curry with test parameters, run via `run_program()`, and assert exact output conditions per SCHEMA.md Hard Testing Requirements.
> **Spec:** [`WDC-008.md`](specs/WDC-008.md)

---

## §9 E2E Simulator Test

<a id="WDC-009"></a>**WDC-009** A full end-to-end simulator test MUST exercise the complete two-phase collateral recovery lifecycle: register a validator, exit via checkpoint, spend registration coin to create withdraw delay coin, advance blocks past the delay period, then spend the withdraw delay coin to release funds to the destination. Failure cases (spending before delay expires) MUST also be tested.
> **Spec:** [`WDC-009.md`](specs/WDC-009.md)

---

## §10 Destination Hint Memo

<a id="WDC-010"></a>**WDC-010** The registration coin puzzle MUST include `sha256(CHECKPOINT_SINGLETON_ID + collateral_destination)` as a conflict-resistant memo (hint) on the `CreateCoin` condition that creates the withdraw delay coin. Using the checkpoint singleton ID as a network prefix prevents cross-network hint collisions. The indexer computes the same hash to query via `get_coin_records_by_hint()`. The withdraw delay coin MUST include the memo `"DIG Network Collateral Release"` on its `CreateCoin` output for transaction identification.
> **Spec:** [`WDC-010.md`](specs/WDC-010.md)
