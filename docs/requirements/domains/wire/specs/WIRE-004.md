# WIRE-004 — Membership Announcement Format

> **Authoritative requirement:** [WIRE-004](../NORMATIVE.md#WIRE-004)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md) — De-Registration and Collateral Recovery

## Summary

Membership announcements are emitted by the checkpoint singleton during membership query spends. They enable validators to prove their membership status for collateral recovery. The epoch prevents replay attacks across epochs.

## Specification

### Announcement Message Format

```
announcement_message = sha256(
    "membership"            (10 bytes, UTF-8, no null terminator)
    + epoch_be              (8 bytes, big-endian u64)
    + pubkey                (48 bytes, G1 compressed)
    + is_member             (1 byte: 0x01 for true, 0x00 for false)
)
```

Total input: 67 bytes.

### Full Announcement Assertion

The registration coin asserts the full announcement:

```
AssertCoinAnnouncement(sha256(checkpoint_singleton_coin_id + announcement_message))
```

Where `checkpoint_singleton_coin_id` is the 32-byte coin ID of the current checkpoint singleton coin (not the launcher ID).

### Important: Coin ID vs Launcher ID

The announcement uses the checkpoint singleton's current **coin ID**, not its launcher ID:
- Launcher ID: Permanent identifier for the singleton lineage
- Coin ID: Identifier of the current singleton coin instance (changes each spend)

Using the wrong ID causes the registration coin spend to fail silently.

### Epoch Field Purpose

The epoch number prevents replay attacks:
- A non-membership announcement from epoch N cannot be reused after epoch N+1 if the validator re-registered
- The registration coin must match the epoch in the announcement

## Acceptance Criteria

- [ ] Announcement message is sha256 of exactly 67 bytes
- [ ] "membership" is 10-byte UTF-8 with no null terminator
- [ ] Epoch is 8-byte big-endian u64
- [ ] Pubkey is 48-byte G1 compressed
- [ ] is_member is 0x01 (member) or 0x00 (non-member)
- [ ] Full announcement uses coin ID, not launcher ID

## Implementation Notes

- **Emitter:** `puzzles/checkpoint_inner.clsp` (membership query path)
- **Asserter:** `puzzles/registration_coin.clsp`
- **Driver:** `src/membership.rs`

### Rust Implementation

```rust
pub fn compute_membership_announcement(
    epoch: u64,
    pubkey: &PublicKey,
    is_member: bool,
    checkpoint_singleton_coin_id: [u8; 32],
) -> [u8; 32] {
    let mut msg = Vec::new();
    msg.extend_from_slice(b"membership");
    msg.extend_from_slice(&epoch.to_be_bytes());
    msg.extend_from_slice(&pubkey.to_bytes());
    msg.push(if is_member { 1 } else { 0 });

    let announcement = sha256(&msg);
    sha256(&[checkpoint_singleton_coin_id.as_ref(), announcement.as_ref()].concat())
}
```

## Verification

1. Compute announcement with test values in Rust
2. Compute same announcement in CLVM simulator
3. Verify both produce identical hash
4. Test with is_member=true and is_member=false
5. Verify coin ID vs launcher ID handling

## Source Citations

- [spec-wire-format.md Lines 548-597](../../../../resources/spec-wire-format.md) — Membership Announcement Format definition and implementation
- [spec-wire-format.md Lines 680-709](../../../../resources/spec-wire-format.md) — Common Mistakes: Coin ID vs launcher ID, wrong coin ID in membership announcement
- [chip-groth16-l2-consensus.md Lines 756-822](../../../../resources/chip-groth16-l2-consensus.md) — Security: Epoch replay protection

## References

- [SEC-006](../../security/specs/SEC-006.md) — Epoch replay protection
- [REG-004](../../registration_coin/specs/REG-004.md) — Registration coin assertion
- [CHK-005](../../checkpoint/specs/CHK-005.md) — Membership query spend path
