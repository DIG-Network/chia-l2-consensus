# WIRE-001 — Checkpoint Message Format

> **Authoritative requirement:** [WIRE-001](../NORMATIVE.md#WIRE-001)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md) — Groth16 On-Chain Verification

## Summary

The checkpoint message is what validators sign to attest to a state transition. It commits to all new state including the new validator set root. This is the critical message that ties the ZK proof to the BLS signature verification.

## Specification

### Message Format

```
checkpoint_message = sha256(
    new_state_root              (32 bytes)
    + new_validator_merkle_root (32 bytes)
    + new_validator_count_be    (8 bytes, big-endian u64)
    + new_epoch_be              (8 bytes, big-endian u64)
)
```

Total input to sha256: 80 bytes. Output: 32 bytes.

### Why This Structure

| Field | Purpose |
|-------|---------|
| `new_state_root` | Application-specific L2 state being checkpointed |
| `new_validator_merkle_root` | Commits to the new validator set composition |
| `new_validator_count_be` | Required for majority calculation in circuit |
| `new_epoch_be` | Monotonic counter prevents replay attacks |

### Integer Encoding

All integers MUST be encoded as fixed-width big-endian:
- `new_validator_count_be`: 8 bytes, zero-padded on the left
- `new_epoch_be`: 8 bytes, zero-padded on the left

Variable-length encoding MUST NOT be used. This is the most common source of Rust/Rue message mismatches.

### Cross-Implementation Consistency

The message format must be identical in:
- Rust (off-chain proof generation and signing)
- Rue (on-chain checkpoint singleton verification)
- Each validator's signing code

Any divergence causes BLS signature verification to fail.

## Acceptance Criteria

- [ ] Message is computed as sha256 of exactly 80 bytes
- [ ] Field order matches specification exactly
- [ ] Integers are 8-byte big-endian, zero-padded
- [ ] Rust implementation produces same hash as Rue implementation
- [ ] Test vectors pass in both implementations

## Implementation Notes

- **Primary codebase:** `src/wire/checkpoint_message.rs`
- **On-chain:** `puzzles/checkpoint_inner.clsp`
- **Dependencies:** SHA-256

### Rust Implementation

```rust
pub fn compute_checkpoint_message(
    new_state_root: [u8; 32],
    new_validator_merkle_root: [u8; 32],
    new_validator_count: u64,
    new_epoch: u64,
) -> [u8; 32] {
    let mut input = Vec::with_capacity(80);
    input.extend_from_slice(&new_state_root);
    input.extend_from_slice(&new_validator_merkle_root);
    input.extend_from_slice(&new_validator_count.to_be_bytes());
    input.extend_from_slice(&new_epoch.to_be_bytes());
    sha256(&input)
}
```

## Verification

1. Compute checkpoint message with test values in Rust
2. Compute same message in CLVM simulator
3. Verify both produce identical 32-byte hash
4. Test with edge cases (count=0, epoch=0, max u64 values)
5. Test with production-like values

## Source Citations

- [spec-wire-format.md Lines 403-463](../../../../resources/spec-wire-format.md) — Checkpoint Message format definition and implementation
- [spec-wire-format.md Lines 680-709](../../../../resources/spec-wire-format.md) — Common Mistakes: Integer encoding, string encoding
- [chip-groth16-l2-consensus.md Lines 68-160](../../../../resources/chip-groth16-l2-consensus.md) — Motivation: Why the validator set lives off-chain

## References

- [WIRE-006](WIRE-006.md) — scalar() function used with checkpoint message
- [CHK-003](../../checkpoint/specs/CHK-003.md) — On-chain verification
- [VAL-003](../../validator/specs/VAL-003.md) — Validator signing protocol
