# WIRE-003 — Groth16 Proof Format

> **Authoritative requirement:** [WIRE-003](../NORMATIVE.md#WIRE-003)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md) — Groth16 On-Chain Verification

## Summary

The Groth16 proof consists of exactly three curve points totaling 192 bytes. This constant-size proof is verified on-chain regardless of validator set size.

## Specification

### Proof Structure

| Field | Type | Size | Description |
|-------|------|------|-------------|
| `a` | G1 compressed | 48 bytes | First proof element |
| `b` | G2 compressed | 96 bytes | Second proof element |
| `c` | G1 compressed | 48 bytes | Third proof element |

Total proof size: 192 bytes.

### Field Order

The proof MUST be serialized in exactly this order: A, B, C. This order is fixed by the Groth16 specification and matches Arkworks output.

### CLVM Representation

In the checkpoint singleton solution, the proof is passed as three separate atoms:

```
(proof_a proof_b proof_c ...)
```

Where `proof_a` and `proof_c` are 48-byte atoms and `proof_b` is a 96-byte atom. CLVM passes these directly to `bls_pairing_identity` as G1 and G2 arguments.

## Acceptance Criteria

- [ ] Proof is exactly 192 bytes total
- [ ] `a` is 48-byte G1 compressed
- [ ] `b` is 96-byte G2 compressed
- [ ] `c` is 48-byte G1 compressed
- [ ] Field order is A, B, C
- [ ] CLVM receives three separate atoms

## Implementation Notes

- **Library:** Arkworks (ark-groth16)
- **Codebase:** `src/prover/serialize.rs`

### Serialization

```rust
use ark_serialize::CanonicalSerialize;

pub struct ClvmProof {
    pub a: Vec<u8>,  // 48 bytes
    pub b: Vec<u8>,  // 96 bytes
    pub c: Vec<u8>,  // 48 bytes
}

pub fn serialize_proof(
    proof: &ark_groth16::Proof<ark_bls12_381::Bls12_381>,
) -> Result<ClvmProof, SerializationError> {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut c = Vec::new();

    proof.a.serialize_compressed(&mut a)?;
    proof.b.serialize_compressed(&mut b)?;
    proof.c.serialize_compressed(&mut c)?;

    Ok(ClvmProof { a, b, c })
}
```

## Verification

1. Generate valid proof from circuit
2. Serialize and verify total size is 192 bytes
3. Verify field sizes: 48 + 96 + 48
4. Pass to CLVM simulator and verify acceptance
5. Verify on-chain `bls_pairing_identity` accepts the format

## Source Citations

- [spec-wire-format.md Lines 122-183](../../../../resources/spec-wire-format.md) — Groth16 Proof Format definition and serialization
- [spec-wire-format.md Lines 46-118](../../../../resources/spec-wire-format.md) — BLS12-381 Point Encoding (G1, G2 formats)

## References

- [WIRE-002](WIRE-002.md) — Point encoding used in proof
- [CIR-001](../../circuit/specs/CIR-001.md) — Circuit that generates proofs
- [CHK-003](../../checkpoint/specs/CHK-003.md) — On-chain proof verification
