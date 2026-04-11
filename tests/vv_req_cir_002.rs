//! REQUIREMENT: CIR-002 — Merkle Membership Constraint
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-002`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-002.md`.
//!
//! Implementation: `src/prover/circuit.rs`, `src/merkle/poseidon.rs`.
//!
//! ## Normative Statement
//!
//! For each of the k signing pubkeys, the circuit verifies a Poseidon Merkle
//! inclusion proof demonstrating the pubkey exists in the validator set. This
//! constraint ensures only registered validators can be counted toward the
//! signing majority. Uses ZK-friendly Poseidon hash (~300 constraints per hash
//! vs ~25,000 for SHA-256).
//!
//! ## How These Tests Prove the Requirement
//!
//! The tests operate at two levels: (1) off-chain Poseidon tree correctness
//! (insert, prove, verify) and (2) in-circuit proof generation via Groth16.
//! Valid Merkle proofs produce a satisfiable circuit; corrupted siblings or
//! wrong roots cause the circuit to be unsatisfiable. This directly verifies
//! the constraint `verify_merkle_path(leaf, proof, index, TREE_DEPTH) = root`.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] Each signer's pubkey has Merkle proof verified (valid proofs accepted)
//! - [x] Leaf computed via Poseidon hash of pubkey (poseidon_leaf)
//! - [x] All k proofs must verify against same root (k=2 test)
//! - [x] Invalid proof -> proof generation fails (corrupted sibling)
//! - [x] Root mismatch -> proof generation fails (wrong root)
//! - [ ] Path verification uses correct sibling ordering (implicitly tested
//!       via off-chain verify, but not explicitly checked bit-by-bit)
//!
//! ## Gaps
//!
//! - Sibling ordering convention (left-first vs right-first) is implicitly
//!   tested but not isolated. A cross-implementation ordering test would
//!   strengthen this.
//! - Tests use TEST_DEPTH=4 (16 slots) for speed; TREE_DEPTH=32 is not
//!   exercised here due to proof generation cost.

use ark_bls12_381::{Bls12_381, Fr};
use ark_groth16::Groth16;
use chia_l2_consensus::testing::poseidon::{poseidon_config, poseidon_leaf, PoseidonMerkleTree};
use chia_l2_consensus::testing::ConsensusCircuit;

/// Small tree depth for fast tests (2^4 = 16 slots).
const TEST_DEPTH: u32 = 4;

/// Helper: build a Poseidon tree with given pubkeys and return proofs.
fn build_tree_and_proofs(
    pubkeys: &[[u8; 48]],
) -> (PoseidonMerkleTree, Fr, Vec<(Fr, Vec<Fr>, u64)>) {
    let config = poseidon_config();
    let mut tree = PoseidonMerkleTree::new(config.clone(), TEST_DEPTH);

    let mut slots = Vec::new();
    for pk in pubkeys {
        let slot = tree.insert_validator(pk);
        slots.push(slot);
    }

    let root = tree.root();
    let proofs: Vec<_> = pubkeys
        .iter()
        .zip(slots.iter())
        .map(|(pk, &slot)| {
            let leaf = poseidon_leaf(&config, pk);
            let proof = tree.prove(slot);
            (leaf, proof.siblings, slot)
        })
        .collect();

    (tree, root, proofs)
}

/// Helper: run trusted setup for a circuit with given max_signers.
fn setup_for(max_signers: usize) -> ark_groth16::ProvingKey<Bls12_381> {
    // Setup must be satisfiable: use empty tree root + valid padding proofs.
    let config = poseidon_config();
    let empty_tree = PoseidonMerkleTree::new(config.clone(), TEST_DEPTH);
    let empty_root = empty_tree.root();

    // Build valid padding proofs (empty leaf proofs against the empty tree)
    let mut padding_proofs = Vec::new();
    let empty_leaf = chia_l2_consensus::testing::poseidon::poseidon_empty_leaf(&config);
    for i in 0..max_signers {
        let proof = empty_tree.prove(i as u64);
        padding_proofs.push((empty_leaf, proof.siblings, i as u64));
    }

    let setup_circuit = ConsensusCircuit::with_merkle_proofs(
        [0; 32],
        0,
        [0; 32],
        0,
        [0; 48],
        [0; 32],
        1, // dummy majority (2*1 > 0)
        config,
        empty_root,
        padding_proofs,
        max_signers,
        TEST_DEPTH,
    );
    use ark_std::rand::SeedableRng;
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(42);
    Groth16::<Bls12_381>::generate_random_parameters_with_reduction(setup_circuit, &mut rng)
        .expect("setup")
}

/// Helper: try to generate a proof. Returns true if satisfiable.
fn try_prove(circuit: ConsensusCircuit, params: &ark_groth16::ProvingKey<Bls12_381>) -> bool {
    use ark_std::rand::SeedableRng;
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(1337);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Groth16::<Bls12_381>::create_random_proof_with_reduction(circuit, params, &mut rng)
    }));
    result.is_ok() && result.unwrap().is_ok()
}

// ── CIR-002: Off-chain Poseidon tree basics ───────────────────────────

// Verifies the off-chain Poseidon Merkle tree: insert a pubkey, compute its
// leaf hash, generate a proof, and verify it against the tree root. This is
// the foundational correctness check -- if the off-chain tree is wrong, all
// in-circuit proofs will fail. A passing result means the Poseidon hash,
// tree structure, and proof format are self-consistent.
#[test]
fn vv_req_cir_002_poseidon_tree_insert_and_verify() {
    let config = poseidon_config();
    let pk = [0xAAu8; 48];
    let mut tree = PoseidonMerkleTree::new(config.clone(), TEST_DEPTH);
    let slot = tree.insert_validator(&pk);

    let leaf = poseidon_leaf(&config, &pk);
    let proof = tree.prove(slot);
    assert!(
        tree.verify(leaf, &proof),
        "CIR-002: Valid proof must verify"
    );
}

// Verifies that a proof generated for pubkey A does NOT verify when checked
// against a different pubkey B's leaf hash. This proves the Merkle tree
// binds proofs to specific leaf values, preventing a validator from using
// another validator's proof.
#[test]
fn vv_req_cir_002_poseidon_tree_wrong_leaf_fails() {
    let config = poseidon_config();
    let pk = [0xAAu8; 48];
    let mut tree = PoseidonMerkleTree::new(config.clone(), TEST_DEPTH);
    let slot = tree.insert_validator(&pk);

    let wrong_leaf = poseidon_leaf(&config, &[0xBB; 48]);
    let proof = tree.prove(slot);
    assert!(
        !tree.verify(wrong_leaf, &proof),
        "CIR-002: Wrong leaf must not verify"
    );
}

// ── CIR-002: Circuit with valid Merkle proofs ─────────────────────────

// Verifies that a circuit with k=2 valid Poseidon Merkle proofs is
// satisfiable: Groth16 proof generation succeeds. This is the positive
// end-to-end test -- the circuit's R1CS constraints accept valid proofs.
// A passing result means the in-circuit Merkle verification matches the
// off-chain tree, which is the core of CIR-002.
#[test]
fn vv_req_cir_002_valid_merkle_proofs_accepted() {
    let pk1 = [0xAAu8; 48];
    let pk2 = [0xBBu8; 48];
    let (_, root, proofs) = build_tree_and_proofs(&[pk1, pk2]);

    eprintln!("Root: {:?}", root);
    for (i, (leaf, siblings, slot)) in proofs.iter().enumerate() {
        eprintln!(
            "Proof {}: leaf={:?}, slot={}, siblings={}",
            i,
            leaf,
            slot,
            siblings.len()
        );
    }

    let params = setup_for(2);

    let circuit = ConsensusCircuit::with_merkle_proofs(
        [0; 32],
        2,
        [0; 32],
        2,
        [0; 48],
        [0; 32],
        2,
        poseidon_config(),
        root,
        proofs,
        2,
        TEST_DEPTH,
    );

    assert!(
        try_prove(circuit, &params),
        "CIR-002: Valid Merkle proofs (k=2) must produce valid proof"
    );
    eprintln!("CIR-002: Valid Merkle proofs (k=2) accepted");
}

// ── CIR-002: Invalid Merkle proof rejected ────────────────────────────

// Verifies that corrupting the first sibling in a Merkle proof makes the
// circuit unsatisfiable (proof generation fails). This is the negative
// soundness test: an attacker cannot forge a Merkle proof by tampering
// with sibling hashes.
#[test]
fn vv_req_cir_002_invalid_merkle_proof_rejected() {
    let pk1 = [0xAAu8; 48];
    let (_, root, mut proofs) = build_tree_and_proofs(&[pk1]);

    // Corrupt the first sibling
    if !proofs[0].1.is_empty() {
        proofs[0].1[0] = Fr::from(9999u64);
    }

    let params = setup_for(1);

    let circuit = ConsensusCircuit::with_merkle_proofs(
        [0; 32],
        1,
        [0; 32],
        1,
        [0; 48],
        [0; 32],
        1,
        poseidon_config(),
        root,
        proofs,
        1,
        TEST_DEPTH,
    );

    assert!(
        !try_prove(circuit, &params),
        "CIR-002: Corrupted Merkle proof must fail"
    );
    eprintln!("CIR-002: Invalid Merkle proof correctly rejected");
}

// ── CIR-002: Wrong root rejected ──────────────────────────────────────

// Verifies that a valid Merkle proof checked against the wrong root makes
// the circuit unsatisfiable. This prevents an attacker from presenting
// proofs generated against a different (e.g., outdated or fabricated)
// validator set.
#[test]
fn vv_req_cir_002_wrong_root_rejected() {
    let pk1 = [0xAAu8; 48];
    let (_, _real_root, proofs) = build_tree_and_proofs(&[pk1]);

    let wrong_root = Fr::from(12345u64);
    let params = setup_for(1);

    let circuit = ConsensusCircuit::with_merkle_proofs(
        [0; 32],
        1,
        [0; 32],
        1,
        [0; 48],
        [0; 32],
        1,
        poseidon_config(),
        wrong_root,
        proofs,
        1,
        TEST_DEPTH,
    );

    assert!(
        !try_prove(circuit, &params),
        "CIR-002: Wrong Merkle root must fail"
    );
    eprintln!("CIR-002: Wrong root correctly rejected");
}

// ── Spec ───────────────────────────────────────────────────────────

// Confirms the CIR-002 specification file exists on disk. This is a
// traceability check ensuring the requirement document is present.
#[test]
fn vv_req_cir_002_spec_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/circuit/specs/CIR-002.md").exists());
}
