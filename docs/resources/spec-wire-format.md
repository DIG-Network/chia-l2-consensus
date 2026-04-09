# Wire Format and Serialization - Canonical Specification

## Document Relationships

**This document is foundational.** It defines every byte that crosses the
boundary between off-chain Rust and on-chain CLVM. Every encoding decision
here has downstream consequences across multiple documents.

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Implemented by** | [spec-groth16-circuit](spec-groth16-circuit.md) | Proof serialization uses `serialize_proof()` defined here |
| **Implemented by** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | All message formats used in both spend paths are defined here |
| **Implemented by** | [spec-network-coin](spec-network-coin.md) | Registration message format defined here |
| **Implemented by** | [spec-registration-coin](spec-registration-coin.md) | Membership announcement format defined here |
| **Implemented by** | [spec-consensus-crate](spec-consensus-crate.md) | The `prover/serialize.rs` module implements this spec |
| **Implemented by** | [spec-indexer](spec-indexer.md) | Indexer parses checkpoint announcements using formats defined here |
| **Depends on** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | Merkle proof siblings are serialized for CLVM consumption |
| **Referenced by** | [spec-clvm-costs](spec-clvm-costs.md) | Atom sizes affect CLVM cost calculations |
| **Referenced by** | [spec-security](spec-security.md) | Assumption 4 covers serialization consistency |
| **Referenced by** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | Groth16 On-Chain Verification section references this spec |

---

## Overview

This document defines the exact byte encoding of every value that crosses
the boundary between the off-chain Rust code and the on-chain CLVM puzzle.
This includes the Groth16 proof, the verification key, the public inputs,
and the BLS aggregate signature and pubkey. Any encoding mismatch will cause
the on-chain `bls_pairing_identity` or `bls_verify` call to fail silently or
raise an exception.

All multi-byte integers are big-endian unless stated otherwise. All elliptic
curve points use compressed encoding unless stated otherwise.

The `scalar()` function defined in this document is used in both:
- The Rue checkpoint singleton puzzle
  (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — VK Input
  Computation)
- The off-chain `compute_vk_input()` Rust function
  (→ see [spec-consensus-crate](spec-consensus-crate.md) — Checkpoint
  Submission)

---

## BLS12-381 Point Encoding

### G1 Points (48 bytes)

G1 points are encoded in compressed form following the ZCash serialization
standard used by both Arkworks and the Chia node. G1 points represent
public keys and appear in the proof as `proof.a` and `proof.c`, and as the
IC points in the verification key. The Chia node treats G1 points as
`PublicKey` atoms.

- Byte 0, bit 7 (MSB): compression flag, always 1 for compressed points
- Byte 0, bit 6: infinity flag, 1 if the point is the point at infinity
- Byte 0, bit 5: sign flag, encodes the sign of the y coordinate
- Remaining bits: the x coordinate as a 381-bit big-endian integer

Total: 48 bytes.

Arkworks `G1Affine::serialize_compressed()` produces exactly this format.
This is what the checkpoint singleton expects for:
- `proof.a` and `proof.c`
  (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Proof
  struct)
- `agg_signers`
  (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Solution
  Parameters)
- All IC points in the verification key
  (→ see this document — Verification Key Format)

### G2 Points (96 bytes)

G2 points use the same flag conventions as G1 but the field element is over
Fp2, requiring two 48-byte coordinates. G2 points represent signatures and
appear as `proof.b` in the Groth16 proof and as `agg_sig` in the checkpoint
solution. The Chia node treats G2 points as `Signature` atoms.

- First 48 bytes: the x1 coordinate with compression/infinity/sign flags
- Second 48 bytes: the x0 coordinate

Total: 96 bytes.

Arkworks `G2Affine::serialize_compressed()` produces exactly this format.
This is what the checkpoint singleton expects for:
- `proof.b`
  (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Proof
  struct)
- `agg_sig`
  (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Solution
  Parameters)
- `beta_g2`, `gamma_g2`, `delta_g2` in the verification key
  (→ see this document — Verification Key Format)

### Verification

Before currying the VK into the checkpoint puzzle or submitting a proof,
verify that each point serializes to the expected length. This check is
performed by `verify_point_sizes()` in the consensus crate
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Checkpoint Submission):

```rust
fn verify_point_sizes(proof: &ClvmProof, vk: &ClvmVerificationKey) {
    assert_eq!(proof.a.len(), 48, "proof.a must be 48 bytes");
    assert_eq!(proof.b.len(), 96, "proof.b must be 96 bytes");
    assert_eq!(proof.c.len(), 48, "proof.c must be 48 bytes");
    assert_eq!(vk.alpha_g1.len(), 48);
    assert_eq!(vk.beta_g2.len(), 96);
    assert_eq!(vk.gamma_g2.len(), 96);
    assert_eq!(vk.delta_g2.len(), 96);
    assert_eq!(vk.ic.len(), 7);
    for ic_point in &vk.ic {
        assert_eq!(ic_point.len(), 48);
    }
}
```

---

## Groth16 Proof Format

A Groth16 proof consists of three curve points. The proof is generated by the
Groth16 circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Proof Generation)
and verified on-chain by the checkpoint singleton
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
Path 1: Checkpoint). The CLVM cost of verification is covered in
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Groth16 Verification).

| Field | Type | Size | Description |
|-------|------|------|-------------|
| `a` | G1 compressed | 48 bytes | First proof element |
| `b` | G2 compressed | 96 bytes | Second proof element |
| `c` | G1 compressed | 48 bytes | Third proof element |

Total proof size: 192 bytes.

### Serialization (Rust)

This is implemented in `src/prover/serialize.rs` in the consensus crate
(→ see [spec-consensus-crate](spec-consensus-crate.md)):

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

### CLVM Representation

In the checkpoint singleton solution the proof is passed as three separate
atoms matching the `Proof` struct
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Puzzle
Source):

```
(proof_a proof_b proof_c ...)
```

Where `proof_a` and `proof_c` are 48-byte atoms and `proof_b` is a 96-byte
atom. CLVM passes these directly to `bls_pairing_identity` as G1 and G2
arguments.

---

## Verification Key Format

The verification key is produced by the trusted setup ceremony
(→ see [spec-trusted-setup](spec-trusted-setup.md) — Phase 2: Circuit-Specific
Setup) and curried permanently into the checkpoint singleton at deployment
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 2).
The number of IC points equals the number of public inputs plus one (the
constant term).

| Field | Type | Size | Description |
|-------|------|------|-------------|
| `alpha_g1` | G1 compressed | 48 bytes | |
| `beta_g2` | G2 compressed | 96 bytes | |
| `gamma_g2` | G2 compressed | 96 bytes | |
| `delta_g2` | G2 compressed | 96 bytes | |
| `ic[0]` | G1 compressed | 48 bytes | Constant term |
| `ic[1]` | G1 compressed | 48 bytes | For public input: `validator_merkle_root` |
| `ic[2]` | G1 compressed | 48 bytes | For public input: `validator_count` |
| `ic[3]` | G1 compressed | 48 bytes | For public input: `new_validator_merkle_root` |
| `ic[4]` | G1 compressed | 48 bytes | For public input: `new_validator_count` |
| `ic[5]` | G1 compressed | 48 bytes | For public input: `agg_signers` |
| `ic[6]` | G1 compressed | 48 bytes | For public input: `checkpoint_message` |

Total: 48 + 96 + 96 + 96 + (7 x 48) = 672 bytes.

### IC Point Order

The IC points must match the order public inputs are allocated in the circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Public Inputs). This
order is fixed by the Arkworks circuit definition at trusted setup time
(→ see [spec-trusted-setup](spec-trusted-setup.md)). The checkpoint singleton
Rue puzzle must use the same order when computing `vk_input`
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Puzzle
Source — vk_input computation).

### Serialization (Rust)

```rust
pub struct ClvmVerificationKey {
    pub alpha_g1: Vec<u8>,
    pub beta_g2:  Vec<u8>,
    pub gamma_g2: Vec<u8>,
    pub delta_g2: Vec<u8>,
    pub ic:       Vec<Vec<u8>>,
}

pub fn serialize_vk(
    vk: &ark_groth16::VerifyingKey<ark_bls12_381::Bls12_381>,
) -> Result<ClvmVerificationKey, SerializationError> {
    let mut alpha_g1 = Vec::new();
    let mut beta_g2  = Vec::new();
    let mut gamma_g2 = Vec::new();
    let mut delta_g2 = Vec::new();

    vk.alpha_g1.serialize_compressed(&mut alpha_g1)?;
    vk.beta_g2.serialize_compressed(&mut beta_g2)?;
    vk.gamma_g2.serialize_compressed(&mut gamma_g2)?;
    vk.delta_g2.serialize_compressed(&mut delta_g2)?;

    let ic = vk.gamma_abc_g1
        .iter()
        .map(|pt| {
            let mut buf = Vec::new();
            pt.serialize_compressed(&mut buf)?;
            Ok(buf)
        })
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(ic.len(), 7, "VK must have exactly 7 IC points for 6 public inputs");

    Ok(ClvmVerificationKey { alpha_g1, beta_g2, gamma_g2, delta_g2, ic })
}
```

### Storage Format

The VK is stored on disk as a hex-encoded JSON for easy inspection and
comparison. The SHA-256 of this file is published as the VK hash
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 6):

```json
{
  "alpha_g1": "a3050a67e4771030...",
  "beta_g2":  "a6242cc5b80eb338...",
  "gamma_g2": "b1234...",
  "delta_g2": "c5678...",
  "ic": [
    "a1111...",
    "a2222...",
    "a3333...",
    "a4444...",
    "a5555...",
    "a6666...",
    "a7777..."
  ]
}
```

---

## Public Input Encoding

Public inputs are passed to the Groth16 verifier as scalar field elements in
the BLS12-381 scalar field Fr. The circuit allocates them in a fixed order
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Public Inputs). Fr
has order:

```
r = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001
```

This is a 255-bit prime. All public inputs must be reduced modulo r before
use.

### The `scalar()` Function

The `scalar()` function converts a public input value to a field element. It
is used in both the off-chain proof generation and the on-chain Rue puzzle
to compute the linear combination of IC points
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — vk_input
computation):

```
scalar(bytes) = SHA-256(bytes) interpreted as 256-bit big-endian integer, mod r
```

In Rust:

```rust
use ark_ff::PrimeField;
use ark_bls12_381::Fr;

pub fn bytes_to_scalar(bytes: &[u8]) -> Fr {
    let hash = sha256(bytes);
    let big = num_bigint::BigUint::from_bytes_be(&hash);
    Fr::from(big)
}
```

In Rue (on-chain):

```rust
fn scalar(data: Bytes) -> Int {
    sha256_to_int(sha256(data))
}
```

Note: CLVM's `g1_multiply` accepts a scalar as a big-endian integer and
reduces it modulo the group order internally. So the Rue `scalar()` function
just needs to convert the SHA-256 output to an integer.

### Public Input Encoding Per Field

| Field | Encoding | Size passed to sha256 |
|-------|----------|----------------------|
| `validator_merkle_root` | 32 bytes raw | 32 bytes |
| `validator_count` | big-endian u64 | 8 bytes |
| `new_validator_merkle_root` | 32 bytes raw | 32 bytes |
| `new_validator_count` | big-endian u64 | 8 bytes |
| `agg_signers` | 48-byte G1 compressed | 48 bytes |
| `checkpoint_message` | 32 bytes raw | 32 bytes |

### VK Input Computation

The `vk_input` G1 point is computed identically in Rust and Rue. Any
divergence means the Groth16 verification equation evaluates to a different
value on-chain and the proof fails. This computation appears in the checkpoint
singleton puzzle
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Puzzle
Source) and in the consensus crate
(→ see [spec-consensus-crate](spec-consensus-crate.md)):

```
vk_input = ic[0]
         + scalar(validator_merkle_root)              * ic[1]
         + scalar(u64_to_be(validator_count))         * ic[2]
         + scalar(new_validator_merkle_root)           * ic[3]
         + scalar(u64_to_be(new_validator_count))      * ic[4]
         + scalar(agg_signers_bytes)                   * ic[5]
         + scalar(checkpoint_message)                  * ic[6]
```

In Rust:

```rust
pub fn compute_vk_input(
    vk: &ClvmVerificationKey,
    validator_merkle_root: [u8; 32],
    validator_count: u64,
    new_validator_merkle_root: [u8; 32],
    new_validator_count: u64,
    agg_signers: &PublicKey,
    checkpoint_message: [u8; 32],
) -> G1Affine {
    let inputs = [
        bytes_to_scalar(&validator_merkle_root),
        bytes_to_scalar(&validator_count.to_be_bytes()),
        bytes_to_scalar(&new_validator_merkle_root),
        bytes_to_scalar(&new_validator_count.to_be_bytes()),
        bytes_to_scalar(&agg_signers.to_bytes()),
        bytes_to_scalar(&checkpoint_message),
    ];

    let ic_points: Vec<G1Affine> = vk.ic.iter()
        .map(|bytes| G1Affine::deserialize_compressed(bytes.as_slice()).unwrap())
        .collect();

    let mut result = ic_points[0].into_projective();
    for (scalar, ic_point) in inputs.iter().zip(ic_points[1..].iter()) {
        result += ic_point.mul(*scalar);
    }

    result.into_affine()
}
```

---

## Checkpoint Message

The checkpoint message is what validators sign. It commits to all new state
including the new validator set root. This is the critical message that ties
the ZK proof to the BLS signature verification
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Circuit
design choice). The message format must be identical in Rust
(→ see [spec-consensus-crate](spec-consensus-crate.md) — `checkpoint_message()`),
Rue
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Puzzle
Source), and in each validator's signing code
(→ see [spec-validator-onboarding](spec-validator-onboarding.md) — Step 9):

```
checkpoint_message = sha256(
    new_state_root              (32 bytes)
    + new_validator_merkle_root (32 bytes)
    + new_validator_count_be    (8 bytes, big-endian u64)
    + new_epoch_be              (8 bytes, big-endian u64)
)
```

Total input to sha256: 80 bytes. Output: 32 bytes.

In Rust:

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

In Rue (inside the checkpoint singleton):

```rust
fn checkpoint_message(
    new_state_root: Bytes32,
    new_validator_merkle_root: Bytes32,
    new_validator_count: Int,
    new_epoch: Int,
) -> Bytes32 {
    sha256(
        new_state_root
        + new_validator_merkle_root
        + int_to_8_bytes_be(new_validator_count)
        + int_to_8_bytes_be(new_epoch)
    )
}
```

---

## BLS Signature Encoding

### Individual Signatures

Each validator signs the full AGG_SIG_ME message. This message format is used
by each validator when signing
(→ see [spec-validator-onboarding](spec-validator-onboarding.md) — Step 9)
and by the L2 integration layer when collecting signatures
(→ see [spec-l2-integration](spec-l2-integration.md) — Signature Collection):

```
signing_message = checkpoint_message + genesis_challenge + coin_id
```

Where:
- `checkpoint_message` is the 32-byte value computed above
- `genesis_challenge` is the 32-byte Chia network genesis challenge from
  `NetworkConfig.genesis_challenge`
  (→ see [spec-consensus-crate](spec-consensus-crate.md) — Configuration)
- `coin_id` is the 32-byte coin ID of the current checkpoint singleton coin
  (not the launcher ID)

Total: 96 bytes input to BLS signing. The resulting BLS signature is a G2
point, 96 bytes compressed.

### Aggregate Signature

The aggregate signature is the G2 sum of all k individual signatures. This is
a standard BLS aggregation (not rogue-key safe aggregation since all validators
sign the same message):

```
agg_sig = sig_1 + sig_2 + ... + sig_k
```

In Rust using `blst`:

```rust
use blst::min_pk::{AggregateSignature, Signature};

pub fn aggregate_signatures(sigs: &[&Signature]) -> Signature {
    AggregateSignature::aggregate(sigs, false)
        .unwrap()
        .to_signature()
}
```

The aggregate signature is a 96-byte G2 compressed point passed to `bls_verify`
in the checkpoint singleton solution
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Solution
Parameters — Checkpoint Spend Path).

### Aggregate Public Key

The aggregate signing pubkey is the G1 sum of all k signing validator pubkeys.
This is passed as `agg_signers` in the checkpoint solution and is also a
public input to the Groth16 circuit
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Public Inputs):

```
agg_signers = pk_1 + pk_2 + ... + pk_k
```

In Rust using `blst`:

```rust
use blst::min_pk::{AggregatePublicKey, PublicKey};

pub fn aggregate_pubkeys(pks: &[&PublicKey]) -> PublicKey {
    AggregatePublicKey::aggregate(pks, false)
        .unwrap()
        .to_public_key()
}
```

The aggregate pubkey is a 48-byte G1 compressed point. Its correctness is
proven by the Groth16 circuit's Constraint 2
(→ see [spec-groth16-circuit](spec-groth16-circuit.md) — Constraint 2:
Aggregate Consistency).

---

## Membership Announcement Format

The membership announcement is emitted by the checkpoint singleton membership
query spend path
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
Path 2: Membership Query) and asserted by the registration coin spend
(→ see [spec-registration-coin](spec-registration-coin.md) — What the Puzzle
Does). The epoch is included to prevent replay attacks across epochs
(→ see [spec-security](spec-security.md) — Epoch Replay Protection):

```
announcement_message = sha256(
    "membership"            (10 bytes, UTF-8, no null terminator)
    + epoch_be              (8 bytes, big-endian u64)
    + pubkey                (48 bytes, G1 compressed)
    + is_member             (1 byte: 0x01 for true, 0x00 for false)
)
```

The full announcement that the registration coin asserts is:

```
AssertCoinAnnouncement(sha256(checkpoint_singleton_coin_id + announcement_message))
```

Where `checkpoint_singleton_coin_id` is the 32-byte coin ID of the current
checkpoint singleton coin (not the launcher ID). This changes on every
checkpoint spend.

In Rust (implemented in `membership_announcement()` in the consensus crate
→ see [spec-consensus-crate](spec-consensus-crate.md) — Membership Queries):

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

---

## Checkpoint State Announcement Format

Emitted by the checkpoint singleton on every successful checkpoint spend
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
Path 1: Checkpoint). Parsed by the indexer to track state
(→ see [spec-indexer](spec-indexer.md) — Checkpoint State Updates):

```
announcement_message = sha256(
    "checkpoint"                    (10 bytes, UTF-8)
    + new_epoch_be                  (8 bytes, big-endian u64)
    + new_state_root                (32 bytes)
    + new_validator_merkle_root     (32 bytes)
    + new_validator_count_be        (8 bytes, big-endian u64)
)
```

Total input: 90 bytes.

---

## Registration Message Format

The message a validator signs when registering through the network coin
(→ see [spec-network-coin](spec-network-coin.md) — What the Puzzle Does).
This prevents unauthorized registrations on behalf of another validator:

```
registration_message = sha256(
    "register"      (8 bytes, UTF-8, no null terminator)
    + pubkey        (48 bytes, G1 compressed)
)
```

This is then signed using AGG_SIG_ME:

```
agg_sig_me_message = registration_message + genesis_challenge + network_coin_coin_id
```

---

## Test Vectors

Implementors must compute these test vectors using their implementation and
cross-check against a reference implementation before deployment
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 5):

### Checkpoint Message

```
new_state_root            = 0x0101...01 (32 bytes of 0x01)
new_validator_merkle_root = 0x0202...02 (32 bytes of 0x02)
new_validator_count       = 10 (0x000000000000000a)
new_epoch                 = 1  (0x0000000000000001)

input = 0x0101...01 + 0x0202...02 + 0x000000000000000a + 0x0000000000000001
      = 80 bytes

checkpoint_message = sha256(input) = <compute and hardcode here>
```

### Scalar Encoding

```
input = 0x0101...01 (32 bytes of 0x01)
sha256(input) = <some 32-byte hash>
interpreted as big-endian u256, mod r = <field element>
```

### Registration Message

```
pubkey = 0xb7f1d3a7... (48 bytes)
input  = "register" + pubkey = 8 + 48 = 56 bytes
registration_message = sha256(input) = <32 bytes>
```

---

## Common Mistakes

**Integer encoding**: All integers passed to sha256 as part of message
construction must be encoded as fixed-width big-endian. A u64 is always
8 bytes, zero padded on the left. Never use variable-length encoding.
This is the most common source of Rust/Rue message mismatches.

**String encoding**: String literals like "register", "membership",
"checkpoint" are UTF-8 encoded with no null terminator and no length prefix.

**Coin ID vs launcher ID**: The membership announcement uses the current
checkpoint singleton coin ID, not the launcher ID. These are different values.
The coin ID changes on every checkpoint spend. The launcher ID is fixed forever.
This mistake causes registration coin spends to fail silently because the
announcement asserter computes the wrong coin ID.

**Scalar reduction**: The `scalar()` function produces a field element mod r.
The intermediate SHA-256 hash is a 256-bit integer which may be larger than r.
In Rust you must explicitly reduce using `Fr::from(big_integer)`. In CLVM
`g1_multiply` handles this internally.

**G1 vs G2 confusion**: `agg_signers` is a G1 point (48 bytes, pubkey).
`agg_sig` is a G2 point (96 bytes, signature). Swapping these will cause
`bls_verify` to fail with no helpful error message.

**Wrong coin ID in membership announcement**: The announcement is keyed to
the checkpoint singleton's current coin ID, not its launcher ID. The coin ID
changes on every checkpoint spend. Always fetch the current coin state
(→ see [spec-indexer](spec-indexer.md) — Checkpoint State Updates) before
computing announcements.
