# WIRE-006 — scalar() Function

> **Authoritative requirement:** [WIRE-006](../NORMATIVE.md#WIRE-006)
> **Verification:** [VERIFICATION.md](../VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../../resources/chip-groth16-l2-consensus.md) — Groth16 On-Chain Verification

## Summary

The `scalar()` function converts public input values to BLS12-381 scalar field elements. It is used in both off-chain proof generation and on-chain Rue puzzle to compute the linear combination of IC points for Groth16 verification.

## Specification

### Function Definition

```
scalar(bytes) = SHA-256(bytes) interpreted as 256-bit big-endian integer, mod r
```

Where `r` is the BLS12-381 scalar field order:
```
r = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001
```

This is a 255-bit prime.

### Scalar Reduction

The SHA-256 output is a 256-bit integer which may be larger than r. The value MUST be explicitly reduced modulo r:
- In Rust: Use `Fr::from(big_integer)` which handles reduction
- In CLVM: `g1_multiply` handles reduction internally

### VK Input Computation

The `vk_input` G1 point uses scalar() for each public input:

```
vk_input = ic[0]
         + scalar(validator_merkle_root)              * ic[1]
         + scalar(u64_to_be(validator_count))         * ic[2]
         + scalar(new_validator_merkle_root)           * ic[3]
         + scalar(u64_to_be(new_validator_count))      * ic[4]
         + scalar(agg_signers_bytes)                   * ic[5]
         + scalar(checkpoint_message)                  * ic[6]
```

### Public Input Encoding

| Field | Encoding | Size passed to sha256 |
|-------|----------|----------------------|
| `validator_merkle_root` | 32 bytes raw | 32 bytes |
| `validator_count` | big-endian u64 | 8 bytes |
| `new_validator_merkle_root` | 32 bytes raw | 32 bytes |
| `new_validator_count` | big-endian u64 | 8 bytes |
| `agg_signers` | 48-byte G1 compressed | 48 bytes |
| `checkpoint_message` | 32 bytes raw | 32 bytes |

## Acceptance Criteria

- [ ] SHA-256 hash is interpreted as big-endian u256
- [ ] Result is reduced modulo r
- [ ] Rust and Rue implementations produce identical scalars
- [ ] vk_input computation matches in both implementations
- [ ] Integer fields use correct byte encoding before hashing

## Implementation Notes

- **Rust:** `src/prover/scalar.rs`
- **Rue:** `puzzles/checkpoint_inner.clsp`

### Rust Implementation

```rust
use ark_ff::PrimeField;
use ark_bls12_381::Fr;

pub fn bytes_to_scalar(bytes: &[u8]) -> Fr {
    let hash = sha256(bytes);
    let big = num_bigint::BigUint::from_bytes_be(&hash);
    Fr::from(big)
}
```

### Rue Implementation

```rust
fn scalar(data: Bytes) -> Int {
    sha256_to_int(sha256(data))
}
```

## Verification

1. Compute scalar for test bytes in Rust
2. Compute same scalar in CLVM simulator
3. Verify both produce identical field element
4. Test with values that require reduction (hash > r)
5. Test full vk_input computation matches

## Source Citations

- [spec-wire-format.md Lines 285-401](../../../../resources/spec-wire-format.md) — Public Input Encoding: scalar() function definition, VK input computation
- [spec-wire-format.md Lines 680-709](../../../../resources/spec-wire-format.md) — Common Mistakes: Scalar reduction
- [chip-groth16-l2-consensus.md Lines 68-160](../../../../resources/chip-groth16-l2-consensus.md) — Motivation: Why Groth16 and constant-cost verification

## References

- [WIRE-003](WIRE-003.md) — Groth16 proof format
- [CIR-001](../../circuit/specs/CIR-001.md) — Circuit public inputs
- [CHK-003](../../checkpoint/specs/CHK-003.md) — On-chain verification
