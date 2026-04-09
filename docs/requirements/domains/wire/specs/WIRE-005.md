# WIRE-005 — Registration Message Format

> **Authoritative requirement:** [WIRE-005](../NORMATIVE.md#WIRE-005)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md) — Part 1: Network Coin

## Summary

The registration message is what a validator signs when registering through the network coin. This proves ownership of the BLS key and prevents unauthorized registrations on behalf of another validator.

## Specification

### Message Format

```
registration_message = sha256(
    "register"      (8 bytes, UTF-8, no null terminator)
    + pubkey        (48 bytes, G1 compressed)
)
```

Total input: 56 bytes. Output: 32 bytes.

### AGG_SIG_ME Format

The registration message is then signed using AGG_SIG_ME:

```
agg_sig_me_message = registration_message + genesis_challenge + network_coin_coin_id
```

Where:
- `registration_message`: 32-byte hash computed above
- `genesis_challenge`: 32-byte Chia network genesis challenge
- `network_coin_coin_id`: 32-byte coin ID of the network coin being spent

Total AGG_SIG_ME input: 96 bytes.

### String Encoding

The literal "register" is:
- UTF-8 encoded
- No null terminator
- No length prefix
- Exactly 8 bytes

## Acceptance Criteria

- [ ] Registration message is sha256 of exactly 56 bytes
- [ ] "register" is 8-byte UTF-8 with no null terminator
- [ ] Pubkey is 48-byte G1 compressed
- [ ] AGG_SIG_ME includes genesis_challenge and coin_id
- [ ] Signature verification passes on-chain

## Implementation Notes

- **Verifier:** `puzzles/network_coin_inner.clsp`
- **Signer:** `src/validator/register.rs`
- **Dependencies:** BLS signing, SHA-256

### Rust Implementation

```rust
pub fn compute_registration_message(pubkey: &PublicKey) -> [u8; 32] {
    let mut input = Vec::with_capacity(56);
    input.extend_from_slice(b"register");
    input.extend_from_slice(&pubkey.to_bytes());
    sha256(&input)
}

pub fn compute_agg_sig_me_message(
    registration_message: [u8; 32],
    genesis_challenge: [u8; 32],
    network_coin_coin_id: [u8; 32],
) -> Vec<u8> {
    let mut msg = Vec::with_capacity(96);
    msg.extend_from_slice(&registration_message);
    msg.extend_from_slice(&genesis_challenge);
    msg.extend_from_slice(&network_coin_coin_id);
    msg
}
```

## Verification

1. Compute registration message with test pubkey in Rust
2. Compute same message in CLVM simulator
3. Verify both produce identical hash
4. Generate signature with test key
5. Verify signature passes on-chain AGG_SIG_ME check

## Source Citations

- [spec-wire-format.md Lines 620-639](../../../../resources/spec-wire-format.md) — Registration Message Format definition
- [spec-wire-format.md Lines 680-709](../../../../resources/spec-wire-format.md) — Common Mistakes: String encoding
- [spec-wire-format.md Lines 466-544](../../../../resources/spec-wire-format.md) — BLS Signature Encoding for AGG_SIG_ME

## References

- [VAL-002](../../validator/specs/VAL-002.md) — Validator registration process
- [NET-003](../../network_coin/specs/NET-003.md) — Network coin registration spend
- [SEC-005](../../security/specs/SEC-005.md) — Lineage enforcement
