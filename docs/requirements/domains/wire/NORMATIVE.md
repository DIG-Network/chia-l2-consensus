# Wire Format — Normative Requirements

> **Master spec:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) — Byte formats
> **Quick reference:** [quick-reference.md](../../../resources/quick-reference.md) — Tables 2-4

---

## §1 Checkpoint Message

<a id="WIRE-001"></a>**WIRE-001** The checkpoint message MUST be computed as `sha256(new_state_root || new_validator_merkle_root || new_validator_count_be8 || new_epoch_be8)` where integers are 8-byte big-endian.
> **Spec:** [`WIRE-001.md`](../../../design/requirements/wire/WIRE-001.md)

---

## §2 Point Encoding

<a id="WIRE-002"></a>**WIRE-002** G1 points (pubkeys) MUST be 48-byte ZCash compressed BLS12-381 format. G2 points (signatures) MUST be 96-byte ZCash compressed format.
> **Spec:** [`WIRE-002.md`](../../../design/requirements/wire/WIRE-002.md)

---

## §3 Groth16 Proof

<a id="WIRE-003"></a>**WIRE-003** The Groth16 proof MUST be exactly 192 bytes: A (G1, 48 bytes) + B (G2, 96 bytes) + C (G1, 48 bytes), in that order.
> **Spec:** [`WIRE-003.md`](../../../design/requirements/wire/WIRE-003.md)

---

## §4 Membership Announcement

<a id="WIRE-004"></a>**WIRE-004** Membership announcements MUST be formatted as `sha256("membership" || epoch_be8 || pubkey || is_member_byte)` where is_member_byte is 0x01 for member, 0x00 for non-member.
> **Spec:** [`WIRE-004.md`](../../../design/requirements/wire/WIRE-004.md)

---

## §5 Registration Message

<a id="WIRE-005"></a>**WIRE-005** The registration message for AGG_SIG_ME MUST be `sha256("register" || pubkey)` where "register" is 8-byte UTF-8 and pubkey is 48-byte compressed G1.
> **Spec:** [`WIRE-005.md`](../../../design/requirements/wire/WIRE-005.md)

---

## §6 scalar() Function

<a id="WIRE-006"></a>**WIRE-006** The `scalar(bytes)` function for circuit public inputs MUST compute `sha256(bytes)` interpreted as big-endian u256, reduced modulo the BLS12-381 scalar field order r.
> **Spec:** [`WIRE-006.md`](../../../design/requirements/wire/WIRE-006.md)
