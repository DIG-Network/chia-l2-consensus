# Circuit Design Decisions

This document records design decisions for implementing CIR-002, CIR-003, and CIR-004
as real R1CS constraints. These decisions must be finalized before implementation.

---

## Decision 1: Hash Function for In-Circuit Merkle Proofs (CIR-002)

### Problem

CIR-002 requires verifying Merkle inclusion proofs inside the Groth16 circuit.
The on-chain Merkle tree (used by CHK-005 membership queries) uses SHA-256.
SHA-256 in R1CS costs ~25,000 constraints per invocation.

Per-signer cost with SHA-256:
- Leaf hash: 1 × SHA-256 = 25,000 constraints
- Path verification: TREE_DEPTH × SHA-256 = 32 × 25,000 = 800,000 constraints
- **Total per signer: ~830,000 constraints**

For MAX_SIGNERS = 100: ~83 million constraints (borderline feasible).
For MAX_SIGNERS = 1,000: ~830 million constraints (impractical).
For MAX_SIGNERS = 20,000: ~16 billion constraints (impossible).

### Options

**Option A: Poseidon hash (recommended)**
- ZK-friendly algebraic hash: ~300 constraints per invocation
- Per-signer cost: ~10,000 constraints (vs 830,000 for SHA-256)
- MAX_SIGNERS = 1,000 → ~10 million constraints (very feasible)
- Trade-off: the circuit's Merkle tree uses Poseidon, NOT SHA-256
- The on-chain membership query tree (CHK-005) remains SHA-256
- Two separate trees exist for the same validator set

**Option B: SHA-256 in circuit**
- Compatible with on-chain tree (single tree)
- MAX_SIGNERS limited to ~50-100 (constraint budget)
- Not practical for production validator set sizes

**Option C: No in-circuit Merkle proofs (placeholder)**
- Skip CIR-002 constraints entirely
- Rely on BLS signature verification for validator authentication
- Simplest to implement but weakest security model
- Acceptable for initial E2E testing of CIR-004

### Proposed Decision

**Phase 1**: Option C — implement CIR-004 (majority) with no Merkle constraints.
The prover provides `actual_signers` as a witness; the circuit enforces 2k > n.
Security relies on BLS signature binding (honest aggregate key = honest signers).

**Phase 2**: Option A — add Poseidon Merkle proofs. This requires:
- `ark-crypto-primitives` Poseidon gadget (or `light-poseidon`)
- Off-chain Poseidon Merkle tree builder
- `validator_merkle_root` public input becomes a Poseidon root
- On-chain SHA-256 tree (CHK-005) coexists as a separate structure

### Impact on Other Requirements

If Poseidon is adopted:
- **SMT-001/002/003**: Add Poseidon variant of the sparse Merkle tree
- **CHK-002**: `validator_merkle_root` in checkpoint state refers to the Poseidon root
- **CHK-005**: On-chain membership queries continue using a SHA-256 tree (separate state field)
- **WIRE-006**: `scalar()` function unchanged (operates on public input bytes, not tree hashes)

---

## Decision 2: G1 Aggregation in Circuit (CIR-003)

### Problem

CIR-003 requires verifying `pk₁ + pk₂ + ... + pkₖ = agg_signers` inside the circuit.
BLS12-381 G1 points use coordinates in Fq (base field, 381 bits).
The circuit operates over Fr (scalar field, 255 bits).
G1 arithmetic in Fr requires non-native field emulation.

Cost per G1 addition (non-native): ~50,000 constraints.
For MAX_SIGNERS = 100: ~5 million constraints.
For MAX_SIGNERS = 1,000: ~50 million constraints.

### Options

**Option A: Non-native G1 arithmetic (standard approach)**
- Use `ark-r1cs-std` non-native field gadgets
- Provably correct but expensive per-signer
- For MAX_SIGNERS = 20,000 requires recursive proving (see Decision 4)

**Option B: Scalar commitment scheme (optimization)**
- Instead of G1 addition in-circuit, commit to pubkeys via hashing
- Circuit verifies: `hash(pk₁, ..., pkₖ) = commitment`
- Off-chain: verify G1 sum matches agg_signers
- Weaker in-circuit guarantee but much cheaper
- Requires careful security analysis

**Option C: Skip G1 aggregation (placeholder)**
- No CIR-003 constraints
- Security relies on: the prover who knows the proving key IS the
  checkpoint operator, and they have no incentive to fake the aggregate
- Acceptable for initial E2E testing

### Proposed Decision

**Phase 1**: Option C — no G1 aggregation constraints. Focus on CIR-004.

**Phase 3**: Option A — non-native G1 arithmetic for full 20,000-signer support.
This requires:
- `ark-r1cs-std` BLS12-381 G1 gadget
- Recursive proving strategy (see Decision 4) to handle constraint volume

---

## Decision 3: Majority Threshold Implementation (CIR-004)

### Problem

CIR-004 requires: `2k > validator_count` where k is the actual signer count.

The current public inputs use `bytes_to_scalar(sha256(validator_count_be8))` — a hashed
representation. The circuit cannot extract the raw `validator_count` from this hash
without a hash preimage gadget.

### Options

**Option A: Private witness with range check (recommended)**
- `k` and `validator_count` are private witnesses
- Circuit enforces: `2k - validator_count - 1 >= 0` via bit decomposition
- `validator_count` is bound to the public input indirectly:
  the on-chain puzzle verifies `sha256(vc_be8) == scalar`, and the VK input
  ties the circuit's public inputs to the on-chain values
- The binding is: the prover MUST use the same `validator_count` that matches
  the public input hash, or the Groth16 proof won't verify on-chain

**Option B: Hash preimage in circuit**
- Verify `sha256(vc_witness_be8) == public_input` inside the circuit
- Then use `vc_witness` for the majority check
- Requires SHA-256 gadget (~25,000 extra constraints)
- Strongest binding but expensive

### Proposed Decision

**Option A** for Phase 1. The security argument:
1. The on-chain puzzle checks `sha256(vc_be8) == scalars.s2` (scalar assertion)
2. The circuit's public input for validator_count = `bytes_to_scalar(sha256(vc_be8))`
3. The VK input computation uses this public input with IC[2]
4. If the prover uses a DIFFERENT validator_count, the proof's public inputs
   won't match the on-chain scalars → `bls_pairing_identity` fails
5. Therefore the prover is forced to use the correct validator_count

The private witness `validator_count` in the circuit must match the public input
or the proof is invalid. This provides sufficient binding.

---

## Decision 4: MAX_SIGNERS = 20,000 (fixed requirement)

### Requirement

`MAX_SIGNERS = 20,000` is a hard requirement. The L2 must support validator sets
of this size. The circuit parameters and proving infrastructure must accommodate it.

### Constraint Budget

With Phase 1 (CIR-004 only): MAX_SIGNERS doesn't affect constraint count.
The circuit has ~200 constraints regardless of MAX_SIGNERS.

With Phase 2 (CIR-002 Poseidon) and Phase 3 (CIR-003 G1):
- Per-signer: ~10,000 (Poseidon Merkle) + ~50,000 (non-native G1 add) = ~60,000
- For MAX_SIGNERS = 20,000: ~1.2 billion constraints

### Proving Strategy for 20,000 Signers

A monolithic Groth16 circuit with 1.2 billion constraints is impractical as a single
proof on commodity hardware. Strategies to achieve MAX_SIGNERS = 20,000:

1. **Recursive SNARKs (Nova/IVC)**: Split the 20,000 signers into batches (e.g., 100
   signers per step). Each step produces an incrementally verifiable proof. The final
   proof is compressed to a single Groth16 proof for on-chain verification. This is
   the most promising approach.

2. **Parallelized proving**: Distribute constraint generation across multiple cores/machines.
   The Groth16 FFT and MSM operations are parallelizable. With a 64-core machine,
   proving time for 1.2B constraints could be reduced from hours to minutes.

3. **Optimized G1 gadgets**: Use specialized BLS12-381 G1 gadgets that minimize
   non-native field overhead (e.g., CycleFold, or custom limb decomposition).

4. **Hybrid approach**: Use Poseidon Merkle proofs inside the SNARK but verify G1
   aggregation via a BLS multi-signature scheme OUTSIDE the circuit. The circuit would
   only prove Merkle membership and majority; the on-chain `bls_verify` already checks
   the aggregate signature.

### Decision

MAX_SIGNERS = 20,000 is maintained across all phases. Phase 2/3 implementation will
evaluate the proving strategies above and select the one that meets the time budget
(target: < 5 minutes proving time for a full 20,000-signer checkpoint).

---

## Decision 5: Implementation Phasing

### Phase 1: Majority Threshold (this PR)

Implement CIR-004 only:
- Private witness: `actual_signers` (k)
- Private witness: `validator_count` (n) — bound to public input via hash
- Constraint: `2k - n - 1 >= 0` via 64-bit decomposition
- ~200 constraints added to circuit

E2E tests:
- Proof with k=3, n=5 (majority) → succeeds, verified on-chain
- Proof with k=2, n=5 (minority) → proof generation fails
- Proof with k=1, n=1 (minimum) → succeeds
- Two-epoch test with majority constraint

### Phase 2: Poseidon Merkle Proofs (future)

Implement CIR-002 with Poseidon:
- Add `ark-crypto-primitives` or `light-poseidon` dependency
- Build Poseidon sparse Merkle tree
- In-circuit Merkle path verification
- Dual tree: Poseidon (circuit) + SHA-256 (on-chain queries)

### Phase 3: G1 Aggregation (future)

Implement CIR-003:
- Add non-native BLS12-381 G1 gadgets
- In-circuit pubkey aggregation
- Tune MAX_SIGNERS based on proving benchmarks

---

## Dependencies

| Phase | New Dependencies |
|-------|-----------------|
| 1 | None (ark-r1cs-std already included) |
| 2 | `ark-crypto-primitives` or `light-poseidon` |
| 3 | `ark-r1cs-std` non-native field (already included) |
