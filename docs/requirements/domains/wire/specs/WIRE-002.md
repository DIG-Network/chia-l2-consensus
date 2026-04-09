# WIRE-002 — Point Encoding (G1 and G2)

> **Authoritative requirement:** [WIRE-002](../NORMATIVE.md#WIRE-002)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md) — Groth16 On-Chain Verification

## Summary

G1 points (public keys) and G2 points (signatures) use ZCash compressed BLS12-381 format. This encoding is used throughout the system for proofs, verification keys, aggregate signatures, and aggregate public keys.

## Specification

### G1 Points (48 bytes)

G1 points represent public keys and appear in:
- `proof.a` and `proof.c` in Groth16 proofs
- IC points in the verification key
- `agg_signers` (aggregate public key)
- Individual validator pubkeys

Encoding format:
- Byte 0, bit 7 (MSB): compression flag, always 1 for compressed points
- Byte 0, bit 6: infinity flag, 1 if the point is the point at infinity
- Byte 0, bit 5: sign flag, encodes the sign of the y coordinate
- Remaining bits: the x coordinate as a 381-bit big-endian integer

Total: 48 bytes.

### G2 Points (96 bytes)

G2 points represent signatures and appear in:
- `proof.b` in Groth16 proofs
- `agg_sig` (aggregate signature)
- `beta_g2`, `gamma_g2`, `delta_g2` in the verification key

Encoding format:
- First 48 bytes: the x1 coordinate with compression/infinity/sign flags
- Second 48 bytes: the x0 coordinate

Total: 96 bytes.

### Serialization Library

Arkworks `G1Affine::serialize_compressed()` and `G2Affine::serialize_compressed()` produce exactly these formats. The Chia node treats G1 points as `PublicKey` atoms and G2 points as `Signature` atoms.

## Acceptance Criteria

- [ ] G1 points are exactly 48 bytes
- [ ] G2 points are exactly 96 bytes
- [ ] Arkworks serialization matches Chia node expectations
- [ ] Point infinity is correctly encoded
- [ ] Sign bit correctly distinguishes y-coordinate variants

## Implementation Notes

- **Library:** Arkworks (ark-bls12-381)
- **Validation:** `verify_point_sizes()` in consensus crate

### Size Verification

```rust
fn verify_point_sizes(proof: &ClvmProof, vk: &ClvmVerificationKey) {
    assert_eq!(proof.a.len(), 48, "proof.a must be 48 bytes");
    assert_eq!(proof.b.len(), 96, "proof.b must be 96 bytes");
    assert_eq!(proof.c.len(), 48, "proof.c must be 48 bytes");
    assert_eq!(vk.alpha_g1.len(), 48);
    assert_eq!(vk.beta_g2.len(), 96);
    assert_eq!(vk.gamma_g2.len(), 96);
    assert_eq!(vk.delta_g2.len(), 96);
}
```

## Verification

1. Serialize known G1 point and verify 48 bytes
2. Serialize known G2 point and verify 96 bytes
3. Round-trip test: serialize then deserialize, verify equality
4. Test point at infinity encoding
5. Verify Arkworks output matches Chia node expectations

## Source Citations

- [spec-wire-format.md Lines 46-118](../../../../resources/spec-wire-format.md) — BLS12-381 Point Encoding (G1 and G2 format details)
- [spec-wire-format.md Lines 680-709](../../../../resources/spec-wire-format.md) — Common Mistakes: G1 vs G2 confusion

## References

- [WIRE-003](WIRE-003.md) — Groth16 proof format uses G1 and G2
- [WIRE-006](WIRE-006.md) — Aggregate signature/pubkey encoding
- [CIR-001](../../circuit/specs/CIR-001.md) — Circuit uses G1 points
