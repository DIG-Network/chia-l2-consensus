# Groth16 Circuit - Technical Specification

## Document Relationships

| Relationship | Document | Nature |
|-------------|----------|--------|
| **Depends on** | [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) | Merkle path verification gadget encodes that spec as R1CS constraints. TREE_DEPTH must match. |
| **Depends on** | [spec-wire-format](spec-wire-format.md) | Public input encoding, scalar() function, proof serialization, and G1/G2 point format |
| **Depends on** | [spec-trusted-setup](spec-trusted-setup.md) | Trusted setup ceremony produces the proving key and VK. MAX_SIGNERS and TREE_DEPTH fixed at ceremony time. |
| **Enables** | [spec-checkpoint-singleton](spec-checkpoint-singleton.md) | Produces the (A, B, C) proof that bls_pairing_identity verifies on-chain |
| **Enables** | [spec-consensus-crate](spec-consensus-crate.md) | The prover/circuit.rs module implements this spec |
| **Referenced by** | [spec-clvm-costs](spec-clvm-costs.md) | Constraint count estimates determine proof generation time |
| **Referenced by** | [spec-security](spec-security.md) | Assumption 4 covers circuit correctness; Assumption 2 covers trusted setup soundness |
| **Referenced by** | [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) | Circuit Statement and Circuit Structure sections of the CHIP |

---

## Overview

This document specifies the Groth16 circuit that proves L2 validator consensus.
The circuit is implemented in Rust using the Arkworks library targeting
BLS12-381. Read
(→ see [spec-wire-format](spec-wire-format.md)) and
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)) before this
document as both are prerequisites.

The circuit proves that a checkpoint submitter knows k validator pubkeys that
are members of the registered validator set, that k is a majority of the total
registered validators, and that those pubkeys aggregate to the claimed
`agg_signers` G1 point. It does not prove BLS signature validity. That is
handled separately on-chain by `bls_verify` inside the checkpoint singleton
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend Path
1: Checkpoint). The CHIP explains why this split is the right design choice
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Rationale:
Circuit design choice).

---

## What the Circuit Proves

Given public inputs `(validator_merkle_root, validator_count,
new_validator_merkle_root, new_validator_count, agg_signers,
checkpoint_message)` and private witness `(signing_pubkeys, merkle_proofs)`,
the circuit proves:

1. Each pubkey in `signing_pubkeys` has a valid Merkle membership proof against
   `validator_merkle_root` using the tree structure defined in
   (→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md))
2. The G1 sum of `signing_pubkeys` equals `agg_signers`
3. `2 * len(signing_pubkeys) > validator_count` (majority threshold)

The circuit does not prove anything about `new_validator_merkle_root`,
`new_validator_count`, or `checkpoint_message` directly. Those values are
included as public inputs so they are committed to in the proof and attested
to by the majority BLS signature that `bls_verify` checks on-chain. This is
what ties the ZK proof to the BLS signature and gives the system its complete
security
(→ see [spec-security](spec-security.md) — Assumption 3).

---

## Circuit Parameters

These are fixed at trusted setup time and cannot change without a new ceremony
(→ see [spec-trusted-setup](spec-trusted-setup.md) — When to Rerun the
Ceremony):

| Parameter | Description |
|-----------|-------------|
| `MAX_SIGNERS` | Maximum number of signing validators the circuit supports. The actual k can be anything from majority threshold up to MAX_SIGNERS. |
| `TREE_DEPTH` | Depth of the sparse Merkle tree. Must match the TREE_DEPTH curried into the checkpoint singleton (→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Curried In Parameters) and the depth used by the off-chain tree (→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Parameters). |

`MAX_SIGNERS` bounds the circuit size. If a checkpoint needs more than
`MAX_SIGNERS` simultaneous signers a new trusted setup is required. The
constraint count and proof generation time scale with `MAX_SIGNERS`
(→ see this document — Constraint Count Estimates and
[spec-clvm-costs](spec-clvm-costs.md)).

---

## Dependencies

```toml
[dependencies]
ark-groth16           = "0.4"
ark-bls12-381         = "0.4"
ark-r1cs-std          = "0.4"
ark-relations         = "0.4"
ark-ff                = "0.4"
ark-ec                = "0.4"
ark-std               = "0.4"
ark-serialize         = "0.4"
ark-crypto-primitives = { version = "0.4", features = ["crh"] }
sha2                  = "0.10"
```

---

## Public Inputs

Public inputs are allocated as `new_input` in Arkworks. They are committed to
in the proof and visible to the verifier. They must be allocated in this exact
order as this determines the IC point assignment in the verification key
(→ see [spec-wire-format](spec-wire-format.md) — Verification Key Format —
IC Point Order). Changing this order requires a new trusted setup.

The encoding of each public input to a field element uses the `scalar()`
function defined in
(→ see [spec-wire-format](spec-wire-format.md) — The scalar() Function):

```rust
// Order matters - must match IC point order in the VK
// ic[0] = constant term (allocated automatically by Arkworks)
// ic[1] = validator_merkle_root
// ic[2] = validator_count
// ic[3] = new_validator_merkle_root
// ic[4] = new_validator_count
// ic[5] = agg_signers
// ic[6] = checkpoint_message

let validator_merkle_root_var = UInt8::new_input_vec(
    ns!(cs, "validator_merkle_root"),
    &self.validator_merkle_root,
)?;

let validator_count_var = UInt64::new_input(
    ns!(cs, "validator_count"),
    self.validator_count,
)?;

let new_validator_merkle_root_var = UInt8::new_input_vec(
    ns!(cs, "new_validator_merkle_root"),
    &self.new_validator_merkle_root,
)?;

let _new_validator_count_var = UInt64::new_input(
    ns!(cs, "new_validator_count"),
    self.new_validator_count,
)?;

let agg_signers_var = UInt8::new_input_vec(
    ns!(cs, "agg_signers"),
    &self.agg_signers,
)?;

let _checkpoint_message_var = UInt8::new_input_vec(
    ns!(cs, "checkpoint_message"),
    &self.checkpoint_message,
)?;
```

---

## Private Witness

Private witness values are allocated as `new_witness`. They are known only to
the prover and are never revealed on-chain. The `signing_pubkeys` and
`merkle_proofs` are generated by the off-chain prover in the consensus crate
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Checkpoint
Submission). The Merkle proofs are produced using the tree defined in
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Proof
Generation):

```rust
let mut pubkey_vars  = Vec::new();
let mut proof_vars   = Vec::new();
let mut is_active    = Vec::new();

for i in 0..self.max_signers {
    let pk = if i < self.signing_pubkeys.len() {
        self.signing_pubkeys[i]
    } else {
        [0u8; 48]
    };

    let proof = if i < self.merkle_proofs.len() {
        self.merkle_proofs[i].clone()
    } else {
        MerkleProof::empty(self.tree_depth)
    };

    let active = i < self.actual_signers;

    let pk_var = UInt8::new_witness_vec(ns!(cs, "pk_{}", i), &pk)?;
    let active_var = Boolean::new_witness(ns!(cs, "active_{}", i), || Ok(active))?;
    let index_var = UInt64::new_witness(
        ns!(cs, "index_{}", i),
        proof.leaf_index,
    )?;
    let sibling_vars: Vec<_> = proof.siblings.iter().enumerate().map(|(j, s)| {
        UInt8::new_witness_vec(ns!(cs, "sib_{}_{}", i, j), s)
    }).collect::<Result<_, _>>()?;

    pubkey_vars.push(pk_var);
    proof_vars.push((index_var, sibling_vars));
    is_active.push(active_var);
}
```

---

## Constraint 1: Merkle Membership

For each of the k signing pubkeys, prove it hashes to a leaf that is a valid
member of `validator_merkle_root`. The leaf value is `sha256(pubkey)` exactly
as defined in
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Leaf Values).
The sibling ordering in path verification must match the canonical ordering in
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Sibling
Ordering): left child always comes first.

### SHA-256 Gadget

```rust
use ark_crypto_primitives::crh::sha256::constraints::{
    DigestVar, Sha256Gadget,
};

fn hash_pubkey_to_leaf(
    cs: ConstraintSystemRef<Fr>,
    pubkey_var: &[UInt8<Fr>],
) -> Result<DigestVar<Fr>, SynthesisError> {
    Sha256Gadget::digest(pubkey_var)
}
```

### Merkle Path Verification Gadget

This gadget implements the same logic as `verify_merkle_path` in the Rue
checkpoint singleton
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Puzzle
Source) and the Rust `verify_path` function
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md) — Proof
Verification). All three must be bit-for-bit equivalent:

```rust
fn verify_merkle_path_gadget(
    cs: ConstraintSystemRef<Fr>,
    leaf: &DigestVar<Fr>,
    leaf_index: &UInt64<Fr>,
    siblings: &[DigestVar<Fr>],
    root: &[UInt8<Fr>],
    depth: u32,
    is_active: &Boolean<Fr>,
) -> Result<(), SynthesisError> {

    let mut current = leaf.clone();
    let mut index_bits = leaf_index.to_bits_le()?;

    for (level, sibling) in siblings.iter().enumerate() {
        let is_right = &index_bits[level];

        // Left child first in concatenation - matches spec-sparse-merkle-tree
        let left  = DigestVar::conditionally_select(is_right, sibling, &current)?;
        let right = DigestVar::conditionally_select(is_right, &current, sibling)?;

        let mut combined = left.0.clone();
        combined.extend_from_slice(&right.0);
        current = Sha256Gadget::digest(&combined)?;
    }

    // For active slots: enforce root match
    // For inactive slots: no constraint (slot uses zero pubkey with empty proof)
    let matches_root = current.0.iter().zip(root.iter())
        .map(|(c, r)| c.is_eq(r))
        .collect::<Result<Vec<_>, _>>()?;

    let all_match = Boolean::kary_and(&matches_root)?;
    let valid = is_active.not().or(&all_match)?;
    valid.enforce_equal(&Boolean::TRUE)?;

    Ok(())
}
```

---

## Constraint 2: Aggregate Consistency

Prove that the G1 sum of the k active signing pubkeys equals `agg_signers`.
This is the most expensive constraint group. Each G1 point addition in R1CS
over BLS12-381 costs approximately 10,000 constraints because you are
emulating BLS12-381 field arithmetic inside a BLS12-381 constraint system.
This is the cost that BLS12-377 pairing support would eliminate
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — A note
on future improvements):

```rust
fn verify_g1_aggregate(
    cs: ConstraintSystemRef<Fr>,
    pubkey_vars: &[Vec<UInt8<Fr>>],
    is_active: &[Boolean<Fr>],
    expected_agg: &[UInt8<Fr>],
) -> Result<(), SynthesisError> {

    let mut g1_vars: Vec<G1Var<ark_bls12_381::Config>> = pubkey_vars
        .iter()
        .map(|pk_bytes| g1_from_bytes_gadget(cs.clone(), pk_bytes))
        .collect::<Result<_, _>>()?;

    let identity = G1Var::zero();
    for (g1, active) in g1_vars.iter_mut().zip(is_active.iter()) {
        *g1 = G1Var::conditionally_select(active, g1, &identity)?;
    }

    let mut sum = G1Var::zero();
    for g1 in &g1_vars {
        sum = sum.add(g1)?;
    }

    let computed_bytes = g1_to_bytes_gadget(&sum)?;
    for (computed, expected) in computed_bytes.iter().zip(expected_agg.iter()) {
        computed.enforce_equal(expected)?;
    }

    Ok(())
}
```

The `agg_signers` expected value is computed off-chain using
`aggregate_pubkeys()` as defined in
(→ see [spec-wire-format](spec-wire-format.md) — Aggregate Public Key) before
being passed as a public input.

---

## Constraint 3: Majority Threshold

Prove that `2 * k > validator_count` where k is the number of active signers.
`validator_count` comes from the checkpoint singleton state
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Singleton
State) and is a runtime public input rather than a fixed circuit parameter.
This is why the circuit does not need to be redeployed when validators join or
leave
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — Rationale:
Why count is a runtime input not a circuit parameter):

```rust
fn verify_majority(
    cs: ConstraintSystemRef<Fr>,
    is_active: &[Boolean<Fr>],
    validator_count: &UInt64<Fr>,
) -> Result<(), SynthesisError> {

    let mut k = UInt64::constant(0);
    for active in is_active {
        let one = UInt64::constant(1);
        let contribution = UInt64::conditionally_select(active, &one, &UInt64::constant(0))?;
        k = k.wrapping_add(&contribution);
    }

    let two_k = k.wrapping_add(&k);
    two_k.enforce_cmp(validator_count, std::cmp::Ordering::Greater, false)?;

    Ok(())
}
```

---

## Full Circuit Implementation

```rust
// src/prover/circuit.rs

use ark_bls12_381::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::prelude::*;

pub struct ConsensusCircuit {
    // Private witness
    pub signing_pubkeys:  Vec<[u8; 48]>,
    pub merkle_proofs:    Vec<MerkleProof>,   // defined in spec-sparse-merkle-tree
    pub actual_signers:   usize,

    // Public inputs (encoding per spec-wire-format — Public Input Encoding)
    pub validator_merkle_root:     [u8; 32],
    pub validator_count:           u64,
    pub new_validator_merkle_root: [u8; 32],
    pub new_validator_count:       u64,
    pub agg_signers:               [u8; 48],  // G1 compressed per spec-wire-format
    pub checkpoint_message:        [u8; 32],  // per spec-wire-format — Checkpoint Message

    // Circuit parameters (fixed at trusted setup time)
    pub max_signers: usize,
    pub tree_depth:  u32,   // must match spec-sparse-merkle-tree TREE_DEPTH
}

impl ConsensusCircuit {
    /// Create a blank circuit for trusted setup.
    /// The blank circuit must have the same structure as a real circuit
    /// with MAX_SIGNERS active signers. Constraint count must be identical
    /// between blank and real circuits (→ see spec-trusted-setup).
    pub fn blank(max_signers: usize, tree_depth: u32) -> Self {
        Self {
            signing_pubkeys:  vec![[0u8; 48]; max_signers],
            merkle_proofs:    vec![MerkleProof::empty(tree_depth); max_signers],
            actual_signers:   0,
            validator_merkle_root:     [0u8; 32],
            validator_count:           0,
            new_validator_merkle_root: [0u8; 32],
            new_validator_count:       0,
            agg_signers:               [0u8; 48],
            checkpoint_message:        [0u8; 32],
            max_signers,
            tree_depth,
        }
    }
}

impl ConstraintSynthesizer<Fr> for ConsensusCircuit {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<Fr>,
    ) -> Result<(), SynthesisError> {

        // --- Public inputs (order determines IC assignment in VK) ---
        // Encoding per spec-wire-format — Public Input Encoding Per Field

        let validator_merkle_root_var = UInt8::new_input_vec(
            ns!(cs, "validator_merkle_root"),
            &self.validator_merkle_root,
        )?;

        let validator_count_var = UInt64::new_input(
            ns!(cs, "validator_count"),
            self.validator_count,
        )?;

        let new_validator_merkle_root_var = UInt8::new_input_vec(
            ns!(cs, "new_validator_merkle_root"),
            &self.new_validator_merkle_root,
        )?;

        let _new_validator_count_var = UInt64::new_input(
            ns!(cs, "new_validator_count"),
            self.new_validator_count,
        )?;

        let agg_signers_var = UInt8::new_input_vec(
            ns!(cs, "agg_signers"),
            &self.agg_signers,
        )?;

        let _checkpoint_message_var = UInt8::new_input_vec(
            ns!(cs, "checkpoint_message"),
            &self.checkpoint_message,
        )?;

        // --- Private witness ---

        let mut pubkey_vars  = Vec::new();
        let mut proof_vars   = Vec::new();
        let mut is_active    = Vec::new();

        for i in 0..self.max_signers {
            let pk = self.signing_pubkeys.get(i).copied().unwrap_or([0u8; 48]);
            let proof = self.merkle_proofs.get(i)
                .cloned()
                .unwrap_or_else(|| MerkleProof::empty(self.tree_depth));
            let active = i < self.actual_signers;

            let pk_var = UInt8::new_witness_vec(ns!(cs, "pk_{}", i), &pk)?;
            let active_var = Boolean::new_witness(ns!(cs, "active_{}", i), || Ok(active))?;
            let index_var = UInt64::new_witness(ns!(cs, "index_{}", i), proof.leaf_index)?;
            let sibling_vars: Vec<_> = proof.siblings.iter().enumerate()
                .map(|(j, s)| UInt8::new_witness_vec(ns!(cs, "sib_{}_{}", i, j), s))
                .collect::<Result<_, _>>()?;

            pubkey_vars.push(pk_var);
            proof_vars.push((index_var, sibling_vars));
            is_active.push(active_var);
        }

        // --- Constraint 1: Merkle membership ---
        // Uses exact leaf/sibling ordering from spec-sparse-merkle-tree

        for i in 0..self.max_signers {
            let leaf = hash_pubkey_to_leaf_gadget(cs.clone(), &pubkey_vars[i])?;
            let (index_var, sibling_vars) = &proof_vars[i];

            verify_merkle_path_gadget(
                cs.clone(),
                &leaf,
                index_var,
                sibling_vars,
                &validator_merkle_root_var,
                self.tree_depth,
                &is_active[i],
            )?;
        }

        // --- Constraint 2: G1 aggregate consistency ---
        // agg_signers encoding per spec-wire-format — Aggregate Public Key

        verify_g1_aggregate(
            cs.clone(),
            &pubkey_vars,
            &is_active,
            &agg_signers_var,
        )?;

        // --- Constraint 3: Majority threshold ---

        verify_majority(
            cs.clone(),
            &is_active,
            &validator_count_var,
        )?;

        Ok(())
    }
}
```

---

## Trusted Setup

The trusted setup is run once before deployment
(→ see [spec-trusted-setup](spec-trusted-setup.md) — Phase 2). The blank
circuit is used. The proving key goes to checkpoint submitters
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 6). The
verification key is curried into the checkpoint singleton
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Curried
In Parameters):

```rust
pub fn run_setup(
    max_signers: usize,
    tree_depth:  u32,
    pk_path:     &str,
    vk_path:     &str,
) -> Result<(), SetupError> {

    let mut rng = ark_std::rand::thread_rng();
    let blank = ConsensusCircuit::blank(max_signers, tree_depth);

    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(blank, &mut rng)
        .map_err(|e| SetupError::ProvingSystemError(e.to_string()))?;

    // VK must have exactly 7 IC points: constant + 6 public inputs
    assert_eq!(vk.gamma_abc_g1.len(), 7,
        "VK must have 7 IC points matching the public input order in this spec");

    let mut pk_bytes = Vec::new();
    pk.serialize_uncompressed(&mut pk_bytes)?;
    std::fs::write(pk_path, &pk_bytes)?;

    let mut vk_bytes = Vec::new();
    vk.serialize_uncompressed(&mut vk_bytes)?;
    std::fs::write(vk_path, &vk_bytes)?;

    Ok(())
}
```

---

## Proof Generation

Called by the consensus crate's `build_checkpoint()` method
(→ see [spec-consensus-crate](spec-consensus-crate.md) — Checkpoint
Submission). Runs in `spawn_blocking` because it takes 2-15 minutes:

```rust
pub fn generate_proof(
    circuit: ConsensusCircuit,
    pk: &ProvingKey<Bls12_381>,
) -> Result<ark_groth16::Proof<Bls12_381>, ProverError> {

    let mut rng = ark_std::rand::thread_rng();
    Groth16::<Bls12_381>::prove(pk, circuit, &mut rng)
        .map_err(|e| ProverError::ProvingSystemError(e.to_string()))
}
```

The proof output is serialized using `serialize_proof()` from
(→ see [spec-wire-format](spec-wire-format.md) — Groth16 Proof Format —
Serialization) before being passed to the checkpoint singleton solution.

---

## On-Chain Verification

The Groth16 verification equation implemented in the checkpoint singleton Rue
puzzle
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md) — Spend
Path 1: Checkpoint):

```
e(A, B) * e(-alpha_g1, beta_g2) * e(-vk_input, gamma_g2) * e(-C, delta_g2) = 1
```

The `vk_input` linear combination is computed using `scalar()` from
(→ see [spec-wire-format](spec-wire-format.md) — VK Input Computation).
The CLVM cost of this verification is covered in
(→ see [spec-clvm-costs](spec-clvm-costs.md) — Spend Path 2: Checkpoint —
Groth16 Verification).

---

## Constraint Count Estimates

These estimates determine proof generation time. They feed into the cost
analysis in
(→ see [spec-clvm-costs](spec-clvm-costs.md)) and the performance discussion
in the CHIP
(→ see [chip-groth16-l2-consensus](chip-groth16-l2-consensus.md) — A note
on future improvements).

| Operation | Constraints (approx) |
|-----------|----------------------|
| SHA-256 hash (one pubkey to leaf) | 25,000 |
| Merkle path verification (depth 32) | 800,000 |
| G1 point decompression | 50,000 |
| G1 point addition | 10,000 |
| Majority check | 500 |

Per signer slot at MAX_SIGNERS = 10, TREE_DEPTH = 32:
- Merkle: 10 * 800,000 = 8,000,000 constraints
- SHA-256: 10 * 25,000 = 250,000 constraints
- G1 ops: 10 * 60,000 = 600,000 constraints
- **Total: approximately 8,850,000 constraints**

Proof generation time: 5-15 minutes on a modern server.
With BLS12-377 support: approximately 50,000 total constraints, under 10 seconds.

---

## Important Notes

**Blank circuit for setup**: The trusted setup uses a blank circuit with all
zeros. The blank circuit must have the same structure and constraint count as a
real circuit with MAX_SIGNERS active signers. Any code change that alters the
constraint count requires a new trusted setup
(→ see [spec-trusted-setup](spec-trusted-setup.md) — When to Rerun the
Ceremony).

**Public input order is fixed**: The order in which public inputs are allocated
determines the IC point mapping in the VK
(→ see [spec-wire-format](spec-wire-format.md) — IC Point Order). Never
reorder public inputs between the setup and proving phases.

**Deterministic proof generation**: Groth16 proofs are randomized. Two calls
to `generate_proof` with identical inputs produce different proofs that both
verify correctly. This is expected.

**TREE_DEPTH must match everywhere**: The TREE_DEPTH in this circuit must
match the TREE_DEPTH in the sparse Merkle tree spec
(→ see [spec-sparse-merkle-tree](spec-sparse-merkle-tree.md)), the TREE_DEPTH
curried into the checkpoint singleton
(→ see [spec-checkpoint-singleton](spec-checkpoint-singleton.md)), and the
TREE_DEPTH used by the indexer
(→ see [spec-indexer](spec-indexer.md)). They are set independently and must
be manually verified to match. The deployment runbook covers this check
(→ see [spec-deployment-runbook](spec-deployment-runbook.md) — Step 3).
