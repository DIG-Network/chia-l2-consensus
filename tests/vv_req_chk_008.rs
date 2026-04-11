//! REQUIREMENT: CHK-008 — End-to-End Integration Test
//! (`docs/requirements/domains/checkpoint/NORMATIVE.md#CHK-008`).
//!
//! Spec: `docs/requirements/domains/checkpoint/specs/CHK-008.md`.
//!
//! Full lifecycle: trusted setup, deploy, register, generate Groth16 proof,
//! submit checkpoint via simulator, verify state update.
//!
//! Uses circuit with 6 public inputs allocated (no arithmetic constraints yet)
//! to produce a VK with 7 IC points. Tests the integration pipeline end-to-end
//! with real Groth16 proofs and real BLS signatures.

mod common;

use chia_l2_consensus::testing::{
    deserialize_proving_key, extract_vk_components, generate_proof, run_test_setup,
    ConsensusCircuit, GROTH16_PROOF_SIZE,
};
use clvmr::serde::node_from_bytes;
use sha2::{Digest, Sha256};

const CHK_HEX: &str = include_str!("../puzzles/compiled/checkpoint_inner.hex");
const CHK_HASH: &str = include_str!("../puzzles/compiled/checkpoint_inner.hash");

// ── Helpers ─────────────────────────────────────────────────────────

fn sha(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

fn chk_inner_mod_hash() -> [u8; 32] {
    hex::decode(CHK_HASH.trim().trim_start_matches("0x"))
        .unwrap()
        .try_into()
        .unwrap()
}

/// Build CLVM flat env for the checkpoint IS_CHECKPOINT=true path.
/// 19 parameters in exact fn main() order.
fn build_chk_path_env(
    a: &mut clvmr::Allocator,
    inner_mod_hash: &[u8; 32],
    // VK
    vk_alpha: &[u8; 48],
    vk_beta: &[u8; 96],
    vk_gamma: &[u8; 96],
    vk_delta: &[u8; 96],
    // IC points (7)
    ic: &[[u8; 48]; 7],
    tree_depth: u64,
    empty_leaf_hash: &[u8; 32],
    // Current state
    state_root: &[u8; 32],
    epoch: u64,
    validator_merkle_root: &[u8; 32],
    validator_count: u64,
    // Checkpoint solution
    proof_a: &[u8; 48],
    proof_b: &[u8; 96],
    proof_c: &[u8; 48],
    new_state_root: &[u8; 32],
    new_validator_merkle_root: &[u8; 32],
    new_validator_count: u64,
    agg_signers: &[u8; 48],
    agg_sig: &[u8; 96],
    scalars: &[[u8; 32]; 6],
) -> clvmr::NodePtr {
    use common::clvm::u64_to_clvm;

    let nil = a.nil();

    // Build right-to-left (param 19 → param 1)
    // 19. conditions (spread): (nil . nil)
    let t = a.new_pair(nil, nil).unwrap();
    // 18. is_member: false
    let t = a.new_pair(nil, t).unwrap();
    // 17. siblings: nil (unused)
    let t = a.new_pair(nil, t).unwrap();
    // 16. leaf_index: 0 (unused)
    let t = a.new_pair(nil, t).unwrap();
    // 15. query_pubkey: zeros (unused)
    let qpk = a.new_atom(&[0u8; 48]).unwrap();
    let t = a.new_pair(qpk, t).unwrap();

    // 14. scalars struct: (s1 . (s2 . (s3 . (s4 . (s5 . (s6 . nil))))))
    let s6 = a.new_atom(scalars[5].as_slice()).unwrap();
    let s5 = a.new_atom(scalars[4].as_slice()).unwrap();
    let s4 = a.new_atom(scalars[3].as_slice()).unwrap();
    let s3 = a.new_atom(scalars[2].as_slice()).unwrap();
    let s2 = a.new_atom(scalars[1].as_slice()).unwrap();
    let s1 = a.new_atom(scalars[0].as_slice()).unwrap();
    let sc = a.new_pair(s6, nil).unwrap();
    let sc = a.new_pair(s5, sc).unwrap();
    let sc = a.new_pair(s4, sc).unwrap();
    let sc = a.new_pair(s3, sc).unwrap();
    let sc = a.new_pair(s2, sc).unwrap();
    let sc = a.new_pair(s1, sc).unwrap();
    let t = a.new_pair(sc, t).unwrap();

    // 13. agg_sig
    let asig = a.new_atom(agg_sig).unwrap();
    let t = a.new_pair(asig, t).unwrap();
    // 12. agg_signers
    let asgn = a.new_atom(agg_signers).unwrap();
    let t = a.new_pair(asgn, t).unwrap();

    // 11. new_validator_count
    let nvc = u64_to_clvm(a, new_validator_count);
    let t = a.new_pair(nvc, t).unwrap();
    // 10. new_validator_merkle_root
    let nvmr = a.new_atom(new_validator_merkle_root).unwrap();
    let t = a.new_pair(nvmr, t).unwrap();
    // 9. new_state_root
    let nsr = a.new_atom(new_state_root).unwrap();
    let t = a.new_pair(nsr, t).unwrap();

    // 8. proof struct: (a . (b . (c . nil)))
    let pc = a.new_atom(proof_c).unwrap();
    let pb = a.new_atom(proof_b).unwrap();
    let pa = a.new_atom(proof_a).unwrap();
    let pr = a.new_pair(pc, nil).unwrap();
    let pr = a.new_pair(pb, pr).unwrap();
    let pr = a.new_pair(pa, pr).unwrap();
    let t = a.new_pair(pr, t).unwrap();

    // 7. is_checkpoint = true (1)
    let one = a.new_atom(&[1]).unwrap();
    let t = a.new_pair(one, t).unwrap();

    // 6. STATE struct: (state_root . (epoch . (validator_merkle_root . (validator_count . nil))))
    let sr = a.new_atom(state_root).unwrap();
    let ep = u64_to_clvm(a, epoch);
    let vmr = a.new_atom(validator_merkle_root).unwrap();
    let vc = u64_to_clvm(a, validator_count);
    let st = a.new_pair(vc, nil).unwrap();
    let st = a.new_pair(vmr, st).unwrap();
    let st = a.new_pair(ep, st).unwrap();
    let st = a.new_pair(sr, st).unwrap();
    let t = a.new_pair(st, t).unwrap();

    // 6. NETWORK_COIN_LAUNCHER_ID (CHK-012)
    let ncli = a.new_atom(&[0x00u8; 32]).unwrap();
    let t = a.new_pair(ncli, t).unwrap();

    // 5. EMPTY_LEAF_HASH
    let elh = a.new_atom(empty_leaf_hash).unwrap();
    let t = a.new_pair(elh, t).unwrap();

    // 4. TREE_DEPTH
    let td = u64_to_clvm(a, tree_depth);
    let t = a.new_pair(td, t).unwrap();

    // 3. IC struct: (ic0 . (ic1 . (ic2 . (ic3 . (ic4 . (ic5 . (ic6 . nil)))))))
    let ic6 = a.new_atom(ic[6].as_slice()).unwrap();
    let ic5 = a.new_atom(ic[5].as_slice()).unwrap();
    let ic4 = a.new_atom(ic[4].as_slice()).unwrap();
    let ic3 = a.new_atom(ic[3].as_slice()).unwrap();
    let ic2 = a.new_atom(ic[2].as_slice()).unwrap();
    let ic1 = a.new_atom(ic[1].as_slice()).unwrap();
    let ic0 = a.new_atom(ic[0].as_slice()).unwrap();
    let ics = a.new_pair(ic6, nil).unwrap();
    let ics = a.new_pair(ic5, ics).unwrap();
    let ics = a.new_pair(ic4, ics).unwrap();
    let ics = a.new_pair(ic3, ics).unwrap();
    let ics = a.new_pair(ic2, ics).unwrap();
    let ics = a.new_pair(ic1, ics).unwrap();
    let ics = a.new_pair(ic0, ics).unwrap();
    let t = a.new_pair(ics, t).unwrap();

    // 2. VK struct: (alpha . (beta . (gamma . (delta . nil))))
    let vd = a.new_atom(vk_delta).unwrap();
    let vg = a.new_atom(vk_gamma).unwrap();
    let vb = a.new_atom(vk_beta).unwrap();
    let va = a.new_atom(vk_alpha).unwrap();
    let vk = a.new_pair(vd, nil).unwrap();
    let vk = a.new_pair(vg, vk).unwrap();
    let vk = a.new_pair(vb, vk).unwrap();
    let vk = a.new_pair(va, vk).unwrap();
    let t = a.new_pair(vk, t).unwrap();

    // 1. INNER_MOD_HASH
    let imh = a.new_atom(inner_mod_hash).unwrap();
    a.new_pair(imh, t).unwrap()
}

// ── Test: Trusted setup produces valid keys ─────────────────────────

#[test]
fn vv_req_chk_008_trusted_setup() {
    let result = run_test_setup().expect("CHK-008: Setup must succeed");
    let pk_bytes: Vec<u8> = result.0;
    let vk_bytes: Vec<u8> = result.1;

    assert!(
        !pk_bytes.is_empty(),
        "CHK-008: Proving key must be non-empty"
    );
    assert!(
        !vk_bytes.is_empty(),
        "CHK-008: Verification key must be non-empty"
    );

    let _pk = deserialize_proving_key(&pk_bytes).expect("CHK-008: PK must deserialize");

    eprintln!(
        "Setup OK: PK={} bytes, VK={} bytes",
        pk_bytes.len(),
        vk_bytes.len()
    );
}

// ── Test: Proof generation produces 192-byte proof ──────────────────

#[test]
fn vv_req_chk_008_proof_generation() {
    let result = run_test_setup().expect("Setup");
    let pk_bytes: Vec<u8> = result.0;
    let pk = deserialize_proving_key(&pk_bytes).expect("PK deserialize");

    let circuit = ConsensusCircuit::with_public_inputs(
        [0xAA; 32], // validator_merkle_root
        1,          // validator_count
        [0xBB; 32], // new_validator_merkle_root
        1,          // new_validator_count
        [0xCC; 48], // agg_signers
        [0xDD; 32], // checkpoint_message
        1,          // actual_signers (majority: 2*1 > 1)
    );

    let proof_bytes = generate_proof(circuit, &pk).expect("CHK-008: Proof must generate");

    assert_eq!(
        proof_bytes.len(),
        GROTH16_PROOF_SIZE,
        "CHK-008: Proof must be exactly {} bytes",
        GROTH16_PROOF_SIZE
    );

    let proof_a = &proof_bytes[0..48];
    let proof_b = &proof_bytes[48..144];
    let proof_c = &proof_bytes[144..192];

    assert!(
        !proof_a.iter().all(|&b| b == 0),
        "CHK-008: Proof A must be non-zero"
    );
    assert!(
        !proof_b.iter().all(|&b| b == 0),
        "CHK-008: Proof B must be non-zero"
    );
    assert!(
        !proof_c.iter().all(|&b| b == 0),
        "CHK-008: Proof C must be non-zero"
    );

    eprintln!("Proof OK: {} bytes", proof_bytes.len());
}

// ── Test: VK has exactly 7 IC points (6 public inputs + 1 constant) ─

#[test]
fn vv_req_chk_008_vk_ic_points() {
    let result = run_test_setup().expect("Setup");
    let pk_bytes: Vec<u8> = result.0;
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = extract_vk_components(&pk.vk).expect("VK");

    // Circuit allocates 6 public inputs → VK has 7 IC points:
    // IC[0] = constant term, IC[1..6] = one per public input.
    assert_eq!(
        vk.ic_points.len(),
        7,
        "CHK-008: VK must have exactly 7 IC points (1 constant + 6 inputs)"
    );
    assert_eq!(vk.alpha_g1.len(), 48, "CHK-008: alpha is G1 (48 bytes)");
    assert_eq!(vk.beta_g2.len(), 96, "CHK-008: beta is G2 (96 bytes)");
    assert_eq!(vk.gamma_g2.len(), 96, "CHK-008: gamma is G2 (96 bytes)");
    assert_eq!(vk.delta_g2.len(), 96, "CHK-008: delta is G2 (96 bytes)");

    for (i, ic_point) in vk.ic_points.iter().enumerate() {
        assert_eq!(
            ic_point.len(),
            48,
            "CHK-008: IC[{}] must be G1 (48 bytes)",
            i
        );
    }

    eprintln!(
        "VK OK: alpha({}), beta({}), gamma({}), delta({}), {} IC points",
        vk.alpha_g1.len(),
        vk.beta_g2.len(),
        vk.gamma_g2.len(),
        vk.delta_g2.len(),
        vk.ic_points.len()
    );
}

// ── Test: Verify pairing directly with blst to isolate ark/CLVM compat ──

#[test]
fn vv_req_chk_008_pairing_with_blst_direct() {
    use ark_bls12_381::{Bls12_381, Fr, G1Affine, G1Projective, G2Projective};
    use ark_ec::CurveGroup;
    use ark_ff::{BigInteger, PrimeField};
    use ark_serialize::CanonicalSerialize;
    use chia_l2_consensus::testing::bytes_to_scalar;

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = &pk.vk;

    let vmr = [0xAAu8; 32];
    let vc: u64 = 3;
    let new_vmr = [0xBBu8; 32];
    let new_vc: u64 = 3;
    let agg_signers = [0xCC; 48];
    let checkpoint_msg = [0xDD; 32];

    let public_inputs: Vec<Fr> = vec![
        bytes_to_scalar(&vmr),
        bytes_to_scalar(&vc.to_be_bytes()),
        bytes_to_scalar(&new_vmr),
        bytes_to_scalar(&new_vc.to_be_bytes()),
        bytes_to_scalar(&agg_signers),
        bytes_to_scalar(&checkpoint_msg),
    ];

    // Generate proof
    let circuit = ConsensusCircuit::with_public_inputs(
        vmr,
        vc,
        new_vmr,
        new_vc,
        agg_signers,
        checkpoint_msg,
        vc as usize,
    );
    let proof_bytes = generate_proof(circuit, &pk).expect("Proof");

    // Compute vk_input using arkworks
    let vk_input_ark = {
        let mut result = G1Projective::from(vk.gamma_abc_g1[0]);
        for (i, pi) in public_inputs.iter().enumerate() {
            result += G1Projective::from(vk.gamma_abc_g1[i + 1]) * pi;
        }
        result.into_affine()
    };

    // Now verify the pairing using clvmr
    // Build a CLVM program: (bls_pairing_identity proof_a proof_b neg_alpha beta neg_vk_input gamma neg_c delta)
    let mut a = clvmr::Allocator::new();

    let proof_a_node = a.new_atom(&proof_bytes[0..48]).unwrap();
    let proof_b_node = a.new_atom(&proof_bytes[48..144]).unwrap();
    let proof_c_node = a.new_atom(&proof_bytes[144..192]).unwrap();

    let vk_comp = extract_vk_components(vk).expect("VK");
    let alpha_node = a.new_atom(&vk_comp.alpha_g1).unwrap();
    let beta_node = a.new_atom(&vk_comp.beta_g2).unwrap();
    let gamma_node = a.new_atom(&vk_comp.gamma_g2).unwrap();
    let delta_node = a.new_atom(&vk_comp.delta_g2).unwrap();

    // Serialize vk_input in ark format
    let mut vk_input_bytes = Vec::new();
    vk_input_ark
        .serialize_compressed(&mut vk_input_bytes)
        .unwrap();
    let vk_input_node = a.new_atom(&vk_input_bytes).unwrap();

    // Build env with all values and a CLVM program that uses paths
    // env = (proof_a . (proof_b . (alpha . (beta . (vk_input . (gamma . (proof_c . (delta . nil))))))))
    let nil = a.nil();
    let env = a.new_pair(delta_node, nil).unwrap();
    let env = a.new_pair(proof_c_node, env).unwrap();
    let env = a.new_pair(gamma_node, env).unwrap();
    let env = a.new_pair(vk_input_node, env).unwrap();
    let env = a.new_pair(beta_node, env).unwrap();
    let env = a.new_pair(alpha_node, env).unwrap();
    let env = a.new_pair(proof_b_node, env).unwrap();
    let env = a.new_pair(proof_a_node, env).unwrap();

    // Correct CLVM paths (LSB-first): N-th element of proper list = 3*2^(N-1) - 1
    // proof_a=2, proof_b=5, alpha=11, beta=23, vk_input=47, gamma=95, proof_c=191, delta=383
    let mk = |a: &mut clvmr::Allocator, val: u16| -> clvmr::NodePtr {
        let bytes = val.to_be_bytes();
        let stripped: Vec<u8> = bytes.iter().copied().skip_while(|&b| b == 0).collect();
        if stripped.is_empty() {
            a.nil()
        } else {
            a.new_atom(&stripped).unwrap()
        }
    };

    let op_negate = a.new_atom(&[51]).unwrap(); // g1_negate
    let op_pairing = a.new_atom(&[58]).unwrap(); // bls_pairing_identity
    let p_alpha = mk(&mut a, 11);
    let p_vkinput = mk(&mut a, 47);
    let p_proofc = mk(&mut a, 191);
    let p_delta = mk(&mut a, 383);
    let p_gamma = mk(&mut a, 95);
    let p_beta = mk(&mut a, 23);
    let p_proofb = mk(&mut a, 5);
    let p_proofa = mk(&mut a, 2);

    // (51 11) = g1_negate(alpha)
    let neg_alpha = a.new_pair(p_alpha, nil).unwrap();
    let neg_alpha = a.new_pair(op_negate, neg_alpha).unwrap();

    // (51 47) = g1_negate(vk_input)
    let neg_vk_input = a.new_pair(p_vkinput, nil).unwrap();
    let neg_vk_input = a.new_pair(op_negate, neg_vk_input).unwrap();

    // (51 191) = g1_negate(proof_c)
    let neg_c = a.new_pair(p_proofc, nil).unwrap();
    let neg_c_expr = a.new_pair(op_negate, neg_c).unwrap();

    // (58 proof_a proof_b neg_alpha beta neg_vk_input gamma neg_c delta)
    let args = a.new_pair(p_delta, nil).unwrap();
    let args = a.new_pair(neg_c_expr, args).unwrap();
    let args = a.new_pair(p_gamma, args).unwrap();
    let args = a.new_pair(neg_vk_input, args).unwrap();
    let args = a.new_pair(p_beta, args).unwrap();
    let args = a.new_pair(neg_alpha, args).unwrap();
    let args = a.new_pair(p_proofb, args).unwrap();
    let args = a.new_pair(p_proofa, args).unwrap();
    let program = a.new_pair(op_pairing, args).unwrap();

    // STEP 0: Compare IC[1]*s1 in CLVM vs arkworks
    {
        let scalar_hash = Sha256::digest(&vmr);
        let s1_bytes: Vec<u8> = scalar_hash.to_vec();
        eprintln!(
            "s1 first byte: 0x{:02x} (negative in CLVM: {})",
            s1_bytes[0],
            s1_bytes[0] >= 0x80
        );

        // CLVM computation: (50 IC1 s1) — opcode 50 = bls_g1_multiply
        let ic1_node = a.new_atom(&vk_comp.ic_points[1]).unwrap();
        let s1_node = a.new_atom(&s1_bytes).unwrap();
        let g1mul_op = a.new_atom(&[50]).unwrap();
        let g1mul_args = {
            let t = a.new_pair(s1_node, a.nil()).unwrap();
            a.new_pair(ic1_node, t).unwrap()
        };
        let g1mul_prog = a.new_pair(g1mul_op, g1mul_args).unwrap();
        // Use (q . value) to quote the literal values
        // Actually, opcode 50 will evaluate its args. Since ic1_node and s1_node are atoms,
        // they'll be treated as paths. We need to quote them.
        let q = a.new_atom(&[1]).unwrap(); // quote opcode
        let qi = a.new_pair(q, ic1_node).unwrap(); // (q . ic1_bytes)
        let qs = a.new_pair(q, s1_node).unwrap(); // (q . s1_bytes)
        let g1mul_args2 = {
            let t = a.new_pair(qs, a.nil()).unwrap();
            a.new_pair(qi, t).unwrap()
        };
        let g1mul_prog2 = a.new_pair(g1mul_op, g1mul_args2).unwrap();
        let empty_env = a.nil();
        let clvm_result = clvmr::run_program(
            &mut a,
            &clvmr::ChiaDialect::new(0),
            g1mul_prog2,
            empty_env,
            11_000_000_000,
        );
        let clvm_ic1_s1 = match &clvm_result {
            Ok(clvmr::reduction::Reduction(_, n)) => {
                let b = a.atom(*n);
                eprintln!(
                    "CLVM IC[1]*s1 = {} bytes, first 4: {:02x?}",
                    b.as_ref().len(),
                    &b.as_ref()[..4.min(b.as_ref().len())]
                );
                b.as_ref().to_vec()
            }
            Err(e) => {
                eprintln!("CLVM IC[1]*s1 FAILED: {}", e.1);
                vec![]
            }
        };

        // Arkworks computation
        let ark_ic1_s1 = {
            let scalar_fr = bytes_to_scalar(&vmr);
            let result = G1Projective::from(vk.gamma_abc_g1[1]) * scalar_fr;
            let affine = result.into_affine();
            let mut bytes = Vec::new();
            affine.serialize_compressed(&mut bytes).unwrap();
            bytes
        };
        eprintln!(
            "Ark  IC[1]*s1 = {} bytes, first 4: {:02x?}",
            ark_ic1_s1.len(),
            &ark_ic1_s1[..4.min(ark_ic1_s1.len())]
        );

        if clvm_ic1_s1 == ark_ic1_s1 {
            eprintln!("IC[1]*s1 MATCH between CLVM and arkworks");
        } else {
            eprintln!("IC[1]*s1 MISMATCH! This is the root cause of the pairing failure.");
        }

        // Compute FULL vk_input in CLVM using quoted values
        // vk_input = IC[0] + IC[1]*s1 + IC[2]*s2 + ... + IC[6]*s6
        let compute_vk_input = |a: &mut clvmr::Allocator| -> clvmr::NodePtr {
            let q = |a: &mut clvmr::Allocator, bytes: &[u8]| -> clvmr::NodePtr {
                let atom = a.new_atom(bytes).unwrap();
                let one = a.new_atom(&[1]).unwrap();
                a.new_pair(one, atom).unwrap() // (q . atom)
            };
            let g1mul = a.new_atom(&[50]).unwrap(); // bls_g1_multiply
            let padd = a.new_atom(&[29]).unwrap(); // point_add

            // Build IC[i]*s_i terms
            let scalars_bytes: [&[u8]; 6] = [
                &Sha256::digest(&vmr) as &[u8],
                &Sha256::digest(&vc.to_be_bytes()) as &[u8],
                &Sha256::digest(&new_vmr) as &[u8],
                &Sha256::digest(&new_vc.to_be_bytes()) as &[u8],
                &Sha256::digest(&agg_signers) as &[u8],
                &Sha256::digest(&checkpoint_msg) as &[u8],
            ];

            // Start with IC[0]
            let mut result = q(a, &vk_comp.ic_points[0]);

            for i in 0..6 {
                // (g1_multiply (q . IC[i+1]) (q . s[i]))
                let qi = q(a, &vk_comp.ic_points[i + 1]);
                let qs = q(a, scalars_bytes[i]);
                let nil = a.nil();
                let mul_args = a.new_pair(qs, nil).unwrap();
                let mul_args = a.new_pair(qi, mul_args).unwrap();
                let mul_expr = a.new_pair(g1mul, mul_args).unwrap();

                // (point_add prev_result mul_expr)
                let nil = a.nil();
                let add_args = a.new_pair(mul_expr, nil).unwrap();
                let add_args = a.new_pair(result, add_args).unwrap();
                result = a.new_pair(padd, add_args).unwrap();
            }
            result
        };
        let vk_input_prog = compute_vk_input(&mut a);
        let nil_env = a.nil();
        let clvm_vk_input = clvmr::run_program(
            &mut a,
            &clvmr::ChiaDialect::new(0),
            vk_input_prog,
            nil_env,
            11_000_000_000,
        );
        match &clvm_vk_input {
            Ok(clvmr::reduction::Reduction(_, n)) => {
                let clvm_bytes = a.atom(*n).as_ref().to_vec();
                eprintln!(
                    "CLVM vk_input = {} bytes, first 4: {:02x?}",
                    clvm_bytes.len(),
                    &clvm_bytes[..4.min(clvm_bytes.len())]
                );
                eprintln!(
                    "Ark  vk_input = {} bytes, first 4: {:02x?}",
                    vk_input_bytes.len(),
                    &vk_input_bytes[..4.min(vk_input_bytes.len())]
                );
                if clvm_bytes == vk_input_bytes {
                    eprintln!("FULL vk_input MATCH between CLVM and arkworks!");
                } else {
                    eprintln!("FULL vk_input MISMATCH! Puzzle computes different vk_input.");
                }
            }
            Err(e) => eprintln!("CLVM vk_input FAILED: {}", e.1),
        }
    }

    // STEP 1: Test just reading path 2 (proof_a)
    let test_read = mk(&mut a, 2); // evaluating atom 2 returns car(env) = proof_a
    let r1 = clvmr::run_program(
        &mut a,
        &clvmr::ChiaDialect::new(0),
        test_read,
        env,
        1_000_000,
    );
    match &r1 {
        Ok(clvmr::reduction::Reduction(_, n)) => {
            let b = a.atom(*n);
            eprintln!(
                "path 2 (proof_a) = {} bytes, first: {:02x?}",
                b.as_ref().len(),
                &b.as_ref()[..4.min(b.as_ref().len())]
            );
        }
        Err(e) => eprintln!("path 2 FAILED: {}", e.1),
    }

    // STEP 2: Test g1_negate(alpha) — path 11
    let negate_test = {
        let p11 = mk(&mut a, 11);
        let neg_op = a.new_atom(&[51]).unwrap();
        let arg = a.new_pair(p11, a.nil()).unwrap();
        a.new_pair(neg_op, arg).unwrap()
    };
    let r2 = clvmr::run_program(
        &mut a,
        &clvmr::ChiaDialect::new(0),
        negate_test,
        env,
        1_000_000,
    );
    match &r2 {
        Ok(clvmr::reduction::Reduction(_, n)) => {
            let b = a.atom(*n);
            eprintln!("g1_negate(alpha) = {} bytes", b.as_ref().len());
        }
        Err(e) => eprintln!("g1_negate(alpha) FAILED: {}", e.1),
    }

    // STEP 3: Full pairing
    let result = clvmr::run_program(
        &mut a,
        &clvmr::ChiaDialect::new(0),
        program,
        env,
        11_000_000_000,
    );

    match &result {
        Ok(_) => eprintln!("Direct pairing check PASSED with ark-computed vk_input"),
        Err(e) => eprintln!("Direct pairing check FAILED: {}", e.1),
    }
    assert!(
        result.is_ok(),
        "Pairing check with ark-computed vk_input must pass"
    );
}

// ── Test: Verify ark→zcash G1 conversion produces valid blst points ──

#[test]
fn vv_req_chk_008_ark_to_zcash_format() {
    use ark_bls12_381::Bls12_381;
    use ark_groth16::Groth16;
    use ark_serialize::CanonicalSerialize;
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // First, verify a proof using arkworks' OWN verifier
    let vmr = [0xAAu8; 32];
    let vc: u64 = 3;
    let new_vmr = [0xBBu8; 32];
    let new_vc: u64 = 3;
    let agg_signers = [0xCC; 48];
    let checkpoint_msg = [0xDD; 32];

    let circuit = ConsensusCircuit::with_public_inputs(
        vmr,
        vc,
        new_vmr,
        new_vc,
        agg_signers,
        checkpoint_msg,
        vc as usize,
    );
    use chia_l2_consensus::testing::bytes_to_scalar;
    let public_inputs = vec![
        bytes_to_scalar(&vmr),
        bytes_to_scalar(&vc.to_be_bytes()),
        bytes_to_scalar(&new_vmr),
        bytes_to_scalar(&new_vc.to_be_bytes()),
        bytes_to_scalar(&agg_signers),
        bytes_to_scalar(&checkpoint_msg),
    ];

    let proof_bytes = generate_proof(circuit, &pk).expect("Proof");

    // Deserialize the proof back to ark format for verification
    let proof_ark = {
        use ark_groth16::Proof;
        use ark_serialize::CanonicalDeserialize;
        // The proof bytes are in ark compressed format
        Proof::<Bls12_381>::deserialize_compressed(&proof_bytes[..])
            .expect("Proof must deserialize back to ark format")
    };

    let pvk = ark_groth16::prepare_verifying_key(&pk.vk);
    let ark_verify = Groth16::<Bls12_381>::verify_proof(&pvk, &proof_ark, &public_inputs);
    eprintln!("Ark Groth16 verify result: {:?}", ark_verify);
    assert!(
        ark_verify.is_ok() && ark_verify.unwrap(),
        "Proof must verify in arkworks' own Groth16 verifier"
    );

    // Get raw ark bytes for comparison
    let mut alpha_ark = Vec::new();
    pk.vk.alpha_g1.serialize_compressed(&mut alpha_ark).unwrap();
    eprintln!("ark alpha first 4: {:02x?}", &alpha_ark[..4]);
    eprintln!("ark alpha last 4:  {:02x?}", &alpha_ark[44..]);
    eprintln!(
        "ark alpha byte[47] flags: infinity={}, y_pos={}",
        (alpha_ark[47] >> 7) & 1,
        (alpha_ark[47] >> 6) & 1
    );

    // Also get the BLS12-381 G1 generator for reference
    use ark_bls12_381::G1Affine;
    use ark_ec::AffineRepr;
    let gen = G1Affine::generator();
    let mut gen_ark = Vec::new();
    gen.serialize_compressed(&mut gen_ark).unwrap();
    eprintln!("ark G1 generator first 4: {:02x?}", &gen_ark[..4]);
    eprintln!("ark G1 generator last 4:  {:02x?}", &gen_ark[44..]);

    // The blst G1 generator compressed (known value)
    let blst_gen = blst::min_pk::PublicKey::from_bytes(
        &hex::decode("97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb").unwrap()
    );
    eprintln!("blst G1 generator OK: {}", blst_gen.is_ok());

    let vk = extract_vk_components(&pk.vk).expect("VK");

    // VK alpha is a G1 point - verify blst can decompress it
    let alpha_result = blst::min_pk::PublicKey::uncompress(&vk.alpha_g1);
    eprintln!(
        "zcash alpha first 4: {:02x?}, len={}",
        &vk.alpha_g1[..4],
        vk.alpha_g1.len()
    );
    assert!(
        alpha_result.is_ok(),
        "VK alpha must be a valid G1 point after ark→zcash conversion: {:?}",
        alpha_result.err()
    );

    // Check each IC point
    for (i, ic) in vk.ic_points.iter().enumerate() {
        let ic_result = blst::min_pk::PublicKey::uncompress(ic);
        assert!(
            ic_result.is_ok(),
            "IC[{}] must be valid G1 after conversion: {:?}",
            i,
            ic_result.err()
        );
    }

    // Check proof points
    let circuit = ConsensusCircuit::with_public_inputs(
        [0xAA; 32], 1, [0xBB; 32], 1, [0xCC; 48], [0xDD; 32], 1,
    );
    let proof_bytes = generate_proof(circuit, &pk).expect("Proof");
    let proof_a = &proof_bytes[0..48];
    let proof_c = &proof_bytes[144..192];
    eprintln!("proof_a[0..4]: {:02x?}", &proof_a[..4]);

    let a_result = blst::min_pk::PublicKey::uncompress(proof_a);
    assert!(
        a_result.is_ok(),
        "Proof A must be valid G1: {:?}",
        a_result.err()
    );
    let c_result = blst::min_pk::PublicKey::uncompress(proof_c);
    assert!(
        c_result.is_ok(),
        "Proof C must be valid G1: {:?}",
        c_result.err()
    );

    // Check G2 points via blst Signature type
    let beta_result = blst::min_pk::Signature::uncompress(&vk.beta_g2);
    assert!(
        beta_result.is_ok(),
        "VK beta must be valid G2: {:?}",
        beta_result.err()
    );

    eprintln!("All ark→zcash converted points are valid BLS12-381 curve points");
}

// ── Test: CHK-008b/c/d — Checkpoint path with real Groth16 + BLS ────

#[test]
fn vv_req_chk_008_checkpoint_path_with_real_proof() {
    // Trusted setup (deterministic, seed=42)
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = extract_vk_components(&pk.vk).expect("VK");
    assert_eq!(vk.ic_points.len(), 7, "Need 7 IC points");

    // BLS key pair for agg_signers (use blst directly for aug-scheme signing)
    use blst::min_pk as bls;
    const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";
    let mut ikm = [0u8; 32];
    ikm[0] = 0x42;
    let bls_sk = bls::SecretKey::key_gen(&ikm, b"chk008").unwrap();
    let bls_pk_point = bls_sk.sk_to_pk();
    let agg_signers: [u8; 48] = bls_pk_point.compress();

    // Choose deterministic public inputs
    let vmr = [0xAAu8; 32]; // current validator merkle root
    let vc: u64 = 3; // current validator count
    let new_vmr = [0xBBu8; 32]; // new validator merkle root
    let new_vc: u64 = 3;
    let state_root = [0x00u8; 32];
    let epoch: u64 = 0;
    let new_epoch = epoch + 1;
    let new_sr = [0xCCu8; 32];

    // Checkpoint message = sha256(new_sr || new_vmr || new_vc_be8 || new_epoch_be8)
    let checkpoint_msg: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(new_sr);
        h.update(new_vmr);
        h.update(new_vc.to_be_bytes());
        h.update(new_epoch.to_be_bytes());
        h.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
        h.finalize().into()
    };

    // BLS signature: aug scheme prepends pk_bytes to message
    let agg_sig: [u8; 96] = bls_sk.sign(&checkpoint_msg, DST, &agg_signers).compress();

    // Groth16 proof for these exact public inputs
    let circuit = ConsensusCircuit::with_public_inputs(
        vmr,
        vc,
        new_vmr,
        new_vc,
        agg_signers,
        checkpoint_msg,
        vc as usize,
    );
    let proof_bytes = generate_proof(circuit, &pk).expect("Proof");
    let proof_a: [u8; 48] = proof_bytes[0..48].try_into().unwrap();
    let proof_b: [u8; 96] = proof_bytes[48..144].try_into().unwrap();
    let proof_c: [u8; 48] = proof_bytes[144..192].try_into().unwrap();

    // Scalars = sha256 of each public input (in order)
    let scalars: [[u8; 32]; 6] = [
        sha(&vmr),
        sha(&vc.to_be_bytes()),
        sha(&new_vmr),
        sha(&new_vc.to_be_bytes()),
        sha(&agg_signers),
        sha(&checkpoint_msg),
    ];

    // Extract VK components
    let vk_alpha: [u8; 48] = vk.alpha_g1.try_into().unwrap();
    let vk_beta: [u8; 96] = vk.beta_g2.try_into().unwrap();
    let vk_gamma: [u8; 96] = vk.gamma_g2.try_into().unwrap();
    let vk_delta: [u8; 96] = vk.delta_g2.try_into().unwrap();
    let ic: [[u8; 48]; 7] = {
        let mut arr = [[0u8; 48]; 7];
        for (i, pt) in vk.ic_points.iter().enumerate() {
            arr[i] = pt.as_slice().try_into().expect("IC[i] must be 48 bytes");
        }
        arr
    };

    let empty_leaf_hash: [u8; 32] = sha(&[0u8; 48]);
    let inner_mod_hash = chk_inner_mod_hash();

    // Build CLVM env and run puzzle
    let mut a = clvmr::Allocator::new();
    let puzzle_bytes = hex::decode(CHK_HEX.trim()).unwrap();
    let puzzle = node_from_bytes(&mut a, &puzzle_bytes).unwrap();

    let env = build_chk_path_env(
        &mut a,
        &inner_mod_hash,
        &vk_alpha,
        &vk_beta,
        &vk_gamma,
        &vk_delta,
        &ic,
        32, // TREE_DEPTH
        &empty_leaf_hash,
        &state_root,
        epoch,
        &vmr,
        vc,
        &proof_a,
        &proof_b,
        &proof_c,
        &new_sr,
        &new_vmr,
        new_vc,
        &agg_signers,
        &agg_sig,
        &scalars,
    );

    let result = clvmr::run_program(
        &mut a,
        &clvmr::ChiaDialect::new(0),
        puzzle,
        env,
        11_000_000_000,
    );

    match result {
        Ok(clvmr::reduction::Reduction(_, output)) => {
            let conditions = common::clvm::parse_conditions(&a, output);
            assert!(
                common::clvm::has_opcode(&conditions, 51),
                "CHK-008: Checkpoint path must emit CREATE_COIN"
            );
            assert!(
                common::clvm::has_opcode(&conditions, 60),
                "CHK-008: Checkpoint path must emit CREATE_COIN_ANNOUNCEMENT"
            );
            eprintln!(
                "CHK-008: Real Groth16 + BLS verification PASSED with {} conditions",
                conditions.len()
            );
        }
        Err(e) => panic!("CHK-008: Checkpoint path FAILED: {}", e.1),
    }
}

// ── Test: CHK-008e — Checkpoint spend accepted by simulator ─────────

#[test]
fn vv_req_chk_008_checkpoint_in_simulator() -> anyhow::Result<()> {
    use chia_protocol::Bytes32;
    use chia_puzzles::singleton::{SingletonArgs, SingletonSolution, SingletonStruct};
    use chia_puzzles::{EveProof, Proof};
    use chia_sdk_driver::{Launcher, Spend, SpendContext, StandardLayer};
    use chia_sdk_test::Simulator;
    use clvm_traits::ToClvm;
    use clvm_utils::CurriedProgram;

    use blst::min_pk as bls;
    const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";

    // ── Trusted setup ────────────────────────────────────────────────
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = extract_vk_components(&pk.vk).expect("VK");
    assert_eq!(vk.ic_points.len(), 7);

    // ── BLS keypair ──────────────────────────────────────────────────
    let mut ikm = [0u8; 32];
    ikm[0] = 0x43;
    let bls_sk = bls::SecretKey::key_gen(&ikm, b"sim").unwrap();
    let bls_pk_point = bls_sk.sk_to_pk();
    let agg_signers: [u8; 48] = bls_pk_point.compress();

    // ── Public inputs ────────────────────────────────────────────────
    let vmr = [0x00u8; 32];
    let vc: u64 = 1;
    let new_vmr = [0xAAu8; 32];
    let new_vc: u64 = 1;
    let state_root = [0x00u8; 32];
    let epoch: u64 = 0;
    let new_epoch = epoch + 1;
    let new_sr = [0xBBu8; 32];

    let checkpoint_msg: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(new_sr);
        h.update(new_vmr);
        h.update(new_vc.to_be_bytes());
        h.update(new_epoch.to_be_bytes());
        h.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
        h.finalize().into()
    };

    let agg_sig: [u8; 96] = bls_sk.sign(&checkpoint_msg, DST, &agg_signers).compress();

    let circuit = ConsensusCircuit::with_public_inputs(
        vmr,
        vc,
        new_vmr,
        new_vc,
        agg_signers,
        checkpoint_msg,
        vc as usize,
    );
    let proof_bytes = generate_proof(circuit, &pk).expect("Proof");
    let proof_a: [u8; 48] = proof_bytes[0..48].try_into().unwrap();
    let proof_b: [u8; 96] = proof_bytes[48..144].try_into().unwrap();
    let proof_c: [u8; 48] = proof_bytes[144..192].try_into().unwrap();

    let scalars: [[u8; 32]; 6] = [
        sha(&vmr),
        sha(&vc.to_be_bytes()),
        sha(&new_vmr),
        sha(&new_vc.to_be_bytes()),
        sha(&agg_signers),
        sha(&checkpoint_msg),
    ];

    let vk_alpha: [u8; 48] = vk.alpha_g1.try_into().unwrap();
    let vk_beta: [u8; 96] = vk.beta_g2.try_into().unwrap();
    let vk_gamma: [u8; 96] = vk.gamma_g2.try_into().unwrap();
    let vk_delta: [u8; 96] = vk.delta_g2.try_into().unwrap();
    let ic: [[u8; 48]; 7] = {
        let mut arr = [[0u8; 48]; 7];
        for (i, pt) in vk.ic_points.iter().enumerate() {
            arr[i] = pt.as_slice().try_into().unwrap();
        }
        arr
    };

    let empty_leaf_hash: [u8; 32] = sha(&[0u8; 48]);
    let inner_mod_hash = chk_inner_mod_hash();
    let inner_ph: Bytes32 = inner_mod_hash.into();

    // ── Deploy checkpoint singleton ──────────────────────────────────
    let mut sim = Simulator::new();
    let ctx = &mut SpendContext::new();
    let (p2_sk, p2_pk, _, p2_coin) = sim.new_p2(1)?;
    let launcher = Launcher::new(p2_coin.coin_id(), 1);
    let launcher_id = launcher.coin().coin_id();
    let (conds, chk_singleton) = launcher.spend(ctx, inner_ph, ())?;
    StandardLayer::new(p2_pk).spend(ctx, p2_coin, conds)?;
    sim.spend_coins(ctx.take(), &[p2_sk])?;

    assert!(
        sim.coin_state(chk_singleton.coin_id()).is_some(),
        "CHK-008: Checkpoint singleton must exist after deploy"
    );

    // ── Spend checkpoint singleton (checkpoint path) ─────────────────
    let ctx = &mut SpendContext::new();

    // Build the full checkpoint inner solution (all 19 params as flat list)
    let chk_inner_sol = build_chk_path_env(
        &mut ctx.allocator,
        &inner_mod_hash,
        &vk_alpha,
        &vk_beta,
        &vk_gamma,
        &vk_delta,
        &ic,
        32,
        &empty_leaf_hash,
        &state_root,
        epoch,
        &vmr,
        vc,
        &proof_a,
        &proof_b,
        &proof_c,
        &new_sr,
        &new_vmr,
        new_vc,
        &agg_signers,
        &agg_sig,
        &scalars,
    );

    // Build singleton outer puzzle (uncurried inner mod)
    let chk_mod = node_from_bytes(&mut ctx.allocator, &hex::decode(CHK_HEX.trim()).unwrap())?;
    let singleton_mod = ctx.singleton_top_layer()?;
    let chk_puzzle = CurriedProgram {
        program: singleton_mod,
        args: SingletonArgs {
            singleton_struct: SingletonStruct::new(launcher_id),
            inner_puzzle: chk_mod,
        },
    }
    .to_clvm(&mut ctx.allocator)?;

    let chk_sol = SingletonSolution {
        lineage_proof: Proof::Eve(EveProof {
            parent_parent_coin_info: p2_coin.coin_id(),
            parent_amount: 1,
        }),
        amount: 1,
        inner_solution: chk_inner_sol,
    }
    .to_clvm(&mut ctx.allocator)?;

    ctx.spend(chk_singleton, Spend::new(chk_puzzle, chk_sol))?;

    // No secret keys needed — checkpoint path has no AGG_SIG_ME conditions
    let result = sim.spend_coins(ctx.take(), &[]);
    assert!(
        result.is_ok(),
        "CHK-008: Checkpoint spend must succeed: {:?}",
        result.err()
    );

    // Verify singleton recreated (child coin with amount=1)
    let children = sim.children(chk_singleton.coin_id());
    let recreated = children.iter().find(|cs| cs.coin.amount == 1);
    assert!(
        recreated.is_some(),
        "CHK-008: Checkpoint singleton must be recreated after checkpoint spend"
    );

    // Verify old singleton spent
    let old = sim.coin_state(chk_singleton.coin_id()).unwrap();
    assert!(
        old.spent_height.is_some(),
        "CHK-008: Original checkpoint singleton must be spent"
    );

    eprintln!("CHK-008: Simulator checkpoint spend PASSED ✓");
    Ok(())
}

// ── Test: CHK-008 — Two-epoch end-to-end (deploy → checkpoint → checkpoint) ──

#[test]
fn vv_req_chk_008_two_epoch_e2e() -> anyhow::Result<()> {
    use chia_protocol::Bytes32;
    use chia_puzzles::singleton::{SingletonArgs, SingletonSolution, SingletonStruct};
    use chia_puzzles::{EveProof, LineageProof, Proof};
    use chia_sdk_driver::{Launcher, Spend, SpendContext, StandardLayer};
    use chia_sdk_test::Simulator;
    use clvm_traits::ToClvm;
    use clvm_utils::{
        curry_tree_hash, tree_hash, tree_hash_atom, tree_hash_pair, CurriedProgram, TreeHash,
    };

    use blst::min_pk as bls;
    const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";

    // ── Trusted setup ────────────────────────────────────────────────
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = extract_vk_components(&pk.vk).expect("VK");
    let vk_alpha: [u8; 48] = vk.alpha_g1.clone().try_into().unwrap();
    let vk_beta: [u8; 96] = vk.beta_g2.clone().try_into().unwrap();
    let vk_gamma: [u8; 96] = vk.gamma_g2.clone().try_into().unwrap();
    let vk_delta: [u8; 96] = vk.delta_g2.clone().try_into().unwrap();
    let ic: [[u8; 48]; 7] = {
        let mut arr = [[0u8; 48]; 7];
        for (i, pt) in vk.ic_points.iter().enumerate() {
            arr[i] = pt.as_slice().try_into().unwrap();
        }
        arr
    };
    let empty_leaf_hash: [u8; 32] = sha(&[0u8; 48]);
    let inner_mod_hash = chk_inner_mod_hash();
    let tree_depth: u64 = 32;

    // ── BLS keypair ──────────────────────────────────────────────────
    let mut ikm = [0u8; 32];
    ikm[0] = 0x55;
    let bls_sk = bls::SecretKey::key_gen(&ikm, b"2epoch").unwrap();
    let agg_signers: [u8; 48] = bls_sk.sk_to_pk().compress();

    // ── Deploy checkpoint singleton ──────────────────────────────────
    let mut sim = Simulator::new();
    let ctx = &mut SpendContext::new();
    let (p2_sk, p2_pk, _, p2_coin) = sim.new_p2(1)?;
    let launcher = Launcher::new(p2_coin.coin_id(), 1);
    let launcher_id = launcher.coin().coin_id();
    let inner_ph: Bytes32 = inner_mod_hash.into();
    let (conds, singleton_coin) = launcher.spend(ctx, inner_ph, ())?;
    StandardLayer::new(p2_pk).spend(ctx, p2_coin, conds)?;
    sim.spend_coins(ctx.take(), &[p2_sk])?;

    assert!(sim.coin_state(singleton_coin.coin_id()).is_some());
    eprintln!("Deployed checkpoint singleton");

    // ── Helper to build a solution for the non-curried params ───────
    let build_solution = |a: &mut clvmr::Allocator,
                          proof_a: &[u8; 48],
                          proof_b: &[u8; 96],
                          proof_c: &[u8; 48],
                          new_sr: &[u8; 32],
                          new_vmr: &[u8; 32],
                          new_vc: u64,
                          agg_signers: &[u8; 48],
                          agg_sig: &[u8; 96],
                          scalars: &[[u8; 32]; 6]|
     -> clvmr::NodePtr {
        use common::clvm::u64_to_clvm;
        let nil = a.nil();
        // Build right-to-left: params 7-19 (solution only)
        let t = a.new_pair(nil, nil).unwrap(); // conditions = (nil . nil)
        let t = a.new_pair(nil, t).unwrap(); // is_member
        let t = a.new_pair(nil, t).unwrap(); // siblings
        let t = a.new_pair(nil, t).unwrap(); // leaf_index
        let qpk = a.new_atom(&[0u8; 48]).unwrap();
        let t = a.new_pair(qpk, t).unwrap(); // query_pubkey
                                             // scalars struct (nil-terminated)
        let s6 = a.new_atom(scalars[5].as_slice()).unwrap();
        let s5 = a.new_atom(scalars[4].as_slice()).unwrap();
        let s4 = a.new_atom(scalars[3].as_slice()).unwrap();
        let s3 = a.new_atom(scalars[2].as_slice()).unwrap();
        let s2 = a.new_atom(scalars[1].as_slice()).unwrap();
        let s1 = a.new_atom(scalars[0].as_slice()).unwrap();
        let sc = a.new_pair(s6, nil).unwrap();
        let sc = a.new_pair(s5, sc).unwrap();
        let sc = a.new_pair(s4, sc).unwrap();
        let sc = a.new_pair(s3, sc).unwrap();
        let sc = a.new_pair(s2, sc).unwrap();
        let sc = a.new_pair(s1, sc).unwrap();
        let t = a.new_pair(sc, t).unwrap();
        let asig = a.new_atom(agg_sig).unwrap();
        let t = a.new_pair(asig, t).unwrap();
        let asgn = a.new_atom(agg_signers).unwrap();
        let t = a.new_pair(asgn, t).unwrap();
        let nvc = u64_to_clvm(a, new_vc);
        let t = a.new_pair(nvc, t).unwrap();
        let nvmr = a.new_atom(new_vmr).unwrap();
        let t = a.new_pair(nvmr, t).unwrap();
        let nsr = a.new_atom(new_sr).unwrap();
        let t = a.new_pair(nsr, t).unwrap();
        // proof struct (nil-terminated)
        let pc = a.new_atom(proof_c).unwrap();
        let pb = a.new_atom(proof_b).unwrap();
        let pa = a.new_atom(proof_a).unwrap();
        let pr = a.new_pair(pc, nil).unwrap();
        let pr = a.new_pair(pb, pr).unwrap();
        let pr = a.new_pair(pa, pr).unwrap();
        let t = a.new_pair(pr, t).unwrap();
        // is_checkpoint = true
        let one = a.new_atom(&[1]).unwrap();
        a.new_pair(one, t).unwrap()
    };

    // ── Helper to compute inner puzzle hash for a given state ────────
    let compute_inner_ph =
        |a: &mut clvmr::Allocator, sr: &[u8; 32], ep: u64, vmr: &[u8; 32], vc: u64| -> TreeHash {
            // Build the state struct as CLVM nodes, then tree_hash it
            let nil = a.nil();
            let sr_n = a.new_atom(sr).unwrap();
            let ep_n = common::clvm::u64_to_clvm(a, ep);
            let vmr_n = a.new_atom(vmr).unwrap();
            let vc_n = common::clvm::u64_to_clvm(a, vc);
            let state = a.new_pair(vc_n, nil).unwrap();
            let state = a.new_pair(vmr_n, state).unwrap();
            let state = a.new_pair(ep_n, state).unwrap();
            let state = a.new_pair(sr_n, state).unwrap();

            let imh_n = a.new_atom(&inner_mod_hash).unwrap();
            let va = a.new_atom(&vk_alpha).unwrap();
            let vb = a.new_atom(&vk_beta).unwrap();
            let vg = a.new_atom(&vk_gamma).unwrap();
            let vd = a.new_atom(&vk_delta).unwrap();
            let vk_node = a.new_pair(vd, nil).unwrap();
            let vk_node = a.new_pair(vg, vk_node).unwrap();
            let vk_node = a.new_pair(vb, vk_node).unwrap();
            let vk_node = a.new_pair(va, vk_node).unwrap();

            let ic_nodes: Vec<_> = ic
                .iter()
                .map(|p| a.new_atom(p.as_slice()).unwrap())
                .collect();
            let mut ic_node = nil;
            for i in (0..7).rev() {
                ic_node = a.new_pair(ic_nodes[i], ic_node).unwrap();
            }

            let td_n = common::clvm::u64_to_clvm(a, tree_depth);
            let elh_n = a.new_atom(&empty_leaf_hash).unwrap();
            let ncli_n = a.new_atom(&[0x00u8; 32]).unwrap(); // CHK-012

            curry_tree_hash(
                TreeHash::new(inner_mod_hash),
                &[
                    tree_hash(a, imh_n),
                    tree_hash(a, vk_node),
                    tree_hash(a, ic_node),
                    tree_hash(a, td_n),
                    tree_hash(a, elh_n),
                    tree_hash(a, ncli_n), // CHK-012: network_coin_launcher_id
                    tree_hash(a, state),
                ],
            )
        };

    // ── Epoch 0 state ────────────────────────────────────────────────
    let state0_root = [0x00u8; 32];
    let state0_epoch: u64 = 0;
    let state0_vmr = [0x00u8; 32];
    let state0_vc: u64 = 1;

    // ── EPOCH 0→1: First checkpoint spend ────────────────────────────
    let new_sr_1 = [0xAAu8; 32];
    let new_vmr_1 = [0xBBu8; 32];
    let new_vc_1: u64 = 1;
    let new_epoch_1 = state0_epoch + 1;

    let ckpt_msg_1: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(new_sr_1);
        h.update(new_vmr_1);
        h.update(new_vc_1.to_be_bytes());
        h.update(new_epoch_1.to_be_bytes());
        h.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
        h.finalize().into()
    };
    let agg_sig_1: [u8; 96] = bls_sk.sign(&ckpt_msg_1, DST, &agg_signers).compress();
    let circuit_1 = ConsensusCircuit::with_public_inputs(
        state0_vmr,
        state0_vc,
        new_vmr_1,
        new_vc_1,
        agg_signers,
        ckpt_msg_1,
        state0_vc as usize,
    );
    let proof_1 = generate_proof(circuit_1, &pk).expect("Proof 1");
    let scalars_1: [[u8; 32]; 6] = [
        sha(&state0_vmr),
        sha(&state0_vc.to_be_bytes()),
        sha(&new_vmr_1),
        sha(&new_vc_1.to_be_bytes()),
        sha(&agg_signers),
        sha(&ckpt_msg_1),
    ];

    // First spend uses uncurried module + all 19 params (eve proof)
    let ctx = &mut SpendContext::new();
    let chk_inner_sol_1 = build_chk_path_env(
        &mut ctx.allocator,
        &inner_mod_hash,
        &vk_alpha,
        &vk_beta,
        &vk_gamma,
        &vk_delta,
        &ic,
        tree_depth,
        &empty_leaf_hash,
        &state0_root,
        state0_epoch,
        &state0_vmr,
        state0_vc,
        &proof_1[0..48].try_into().unwrap(),
        &proof_1[48..144].try_into().unwrap(),
        &proof_1[144..192].try_into().unwrap(),
        &new_sr_1,
        &new_vmr_1,
        new_vc_1,
        &agg_signers,
        &agg_sig_1,
        &scalars_1,
    );

    let chk_mod = node_from_bytes(&mut ctx.allocator, &hex::decode(CHK_HEX.trim()).unwrap())?;
    let singleton_mod = ctx.singleton_top_layer()?;
    let chk_puzzle_1 = CurriedProgram {
        program: singleton_mod,
        args: SingletonArgs {
            singleton_struct: SingletonStruct::new(launcher_id),
            inner_puzzle: chk_mod,
        },
    }
    .to_clvm(&mut ctx.allocator)?;

    let chk_sol_1 = SingletonSolution {
        lineage_proof: Proof::Eve(EveProof {
            parent_parent_coin_info: p2_coin.coin_id(),
            parent_amount: 1,
        }),
        amount: 1,
        inner_solution: chk_inner_sol_1,
    }
    .to_clvm(&mut ctx.allocator)?;

    ctx.spend(singleton_coin, Spend::new(chk_puzzle_1, chk_sol_1))?;
    sim.spend_coins(ctx.take(), &[])?;
    eprintln!("Epoch 0→1 checkpoint PASSED");

    let children_1 = sim.children(singleton_coin.coin_id());
    let singleton_2 = children_1
        .iter()
        .find(|cs| cs.coin.amount == 1)
        .expect("Singleton must be recreated after epoch 1")
        .coin;

    // ── EPOCH 1→2: Second checkpoint spend ───────────────────────────
    let new_sr_2 = [0xCCu8; 32];
    let new_vmr_2 = [0xDDu8; 32];
    let new_vc_2: u64 = 2;
    let new_epoch_2 = new_epoch_1 + 1;

    let ckpt_msg_2: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(new_sr_2);
        h.update(new_vmr_2);
        h.update(new_vc_2.to_be_bytes());
        h.update(new_epoch_2.to_be_bytes());
        h.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
        h.finalize().into()
    };
    let agg_sig_2: [u8; 96] = bls_sk.sign(&ckpt_msg_2, DST, &agg_signers).compress();
    let circuit_2 = ConsensusCircuit::with_public_inputs(
        new_vmr_1,
        new_vc_1,
        new_vmr_2,
        new_vc_2,
        agg_signers,
        ckpt_msg_2,
        new_vc_1 as usize,
    );
    let proof_2 = generate_proof(circuit_2, &pk).expect("Proof 2");
    let scalars_2: [[u8; 32]; 6] = [
        sha(&new_vmr_1),
        sha(&new_vc_1.to_be_bytes()),
        sha(&new_vmr_2),
        sha(&new_vc_2.to_be_bytes()),
        sha(&agg_signers),
        sha(&ckpt_msg_2),
    ];

    // Second spend: inner puzzle = curried module (with epoch 1 state)
    let ctx = &mut SpendContext::new();

    // Curry the module with epoch-1 state
    let chk_mod_2 = node_from_bytes(&mut ctx.allocator, &hex::decode(CHK_HEX.trim()).unwrap())?;
    let nil = ctx.allocator.nil();

    // Build curried args as CLVM nodes (nil-terminated structs)
    let imh_n = ctx.allocator.new_atom(&inner_mod_hash).unwrap();
    let va = ctx.allocator.new_atom(&vk_alpha).unwrap();
    let vb = ctx.allocator.new_atom(&vk_beta).unwrap();
    let vg = ctx.allocator.new_atom(&vk_gamma).unwrap();
    let vd = ctx.allocator.new_atom(&vk_delta).unwrap();
    let vk_n = ctx.allocator.new_pair(vd, nil).unwrap();
    let vk_n = ctx.allocator.new_pair(vg, vk_n).unwrap();
    let vk_n = ctx.allocator.new_pair(vb, vk_n).unwrap();
    let vk_n = ctx.allocator.new_pair(va, vk_n).unwrap();

    let ic_ns: Vec<_> = ic
        .iter()
        .map(|p| ctx.allocator.new_atom(p.as_slice()).unwrap())
        .collect();
    let mut ic_n = nil;
    for i in (0..7).rev() {
        ic_n = ctx.allocator.new_pair(ic_ns[i], ic_n).unwrap();
    }

    let td_n = common::clvm::u64_to_clvm(&mut ctx.allocator, tree_depth);
    let elh_n = ctx.allocator.new_atom(&empty_leaf_hash).unwrap();
    let ncli_n = ctx.allocator.new_atom(&[0x00u8; 32]).unwrap(); // CHK-012

    // Epoch 1 state
    let sr_n = ctx.allocator.new_atom(&new_sr_1).unwrap();
    let ep_n = common::clvm::u64_to_clvm(&mut ctx.allocator, new_epoch_1);
    let vmr_n = ctx.allocator.new_atom(&new_vmr_1).unwrap();
    let vc_n = common::clvm::u64_to_clvm(&mut ctx.allocator, new_vc_1);
    let state_n = ctx.allocator.new_pair(vc_n, nil).unwrap();
    let state_n = ctx.allocator.new_pair(vmr_n, state_n).unwrap();
    let state_n = ctx.allocator.new_pair(ep_n, state_n).unwrap();
    let state_n = ctx.allocator.new_pair(sr_n, state_n).unwrap();

    // Curry: (a (q . module) (c (q . imh) (c (q . vk) (c (q . ic) (c (q . td) (c (q . elh) (c (q . state) 1)))))))
    // Build manually because CurriedProgram with &[NodePtr] doesn't use curry pattern
    let curried_inner = {
        let a = &mut ctx.allocator;
        let one = a.new_atom(&[1]).unwrap(); // path 1 = solution
        let q_op = a.new_atom(&[1]).unwrap(); // quote
        let a_op = a.new_atom(&[2]).unwrap(); // apply
        let c_op = a.new_atom(&[4]).unwrap(); // cons

        // Build from inside out: (c (q . argN) 1), then (c (q . argN-1) prev), ...
        let curry_args = [imh_n, vk_n, ic_n, td_n, elh_n, ncli_n, state_n];
        let mut env_builder = one; // start with 1 (= solution)
        for &arg in curry_args.iter().rev() {
            let quoted_arg = a.new_pair(q_op, arg).unwrap(); // (q . arg)
            let pair = a.new_pair(env_builder, nil).unwrap(); // (prev . nil)
            let pair = a.new_pair(quoted_arg, pair).unwrap(); // ((q . arg) prev . nil)
            env_builder = a.new_pair(c_op, pair).unwrap(); // (c (q . arg) prev)
        }
        // (a (q . module) env_builder)
        let quoted_mod = a.new_pair(q_op, chk_mod_2).unwrap(); // (q . module)
        let outer = a.new_pair(env_builder, nil).unwrap();
        let outer = a.new_pair(quoted_mod, outer).unwrap();
        a.new_pair(a_op, outer).unwrap()
    };

    // Build solution with just the 13 non-curried params
    let inner_sol_2 = build_solution(
        &mut ctx.allocator,
        &proof_2[0..48].try_into().unwrap(),
        &proof_2[48..144].try_into().unwrap(),
        &proof_2[144..192].try_into().unwrap(),
        &new_sr_2,
        &new_vmr_2,
        new_vc_2,
        &agg_signers,
        &agg_sig_2,
        &scalars_2,
    );

    // Wrap in singleton
    let singleton_mod_2 = ctx.singleton_top_layer()?;
    let chk_puzzle_2 = CurriedProgram {
        program: singleton_mod_2,
        args: SingletonArgs {
            singleton_struct: SingletonStruct::new(launcher_id),
            inner_puzzle: curried_inner,
        },
    }
    .to_clvm(&mut ctx.allocator)?;

    // Lineage proof: parent was the first singleton (uncurried inner)
    let chk_sol_2 = SingletonSolution {
        lineage_proof: Proof::Lineage(LineageProof {
            parent_parent_coin_info: singleton_coin.parent_coin_info,
            parent_inner_puzzle_hash: inner_ph,
            parent_amount: 1,
        }),
        amount: 1,
        inner_solution: inner_sol_2,
    }
    .to_clvm(&mut ctx.allocator)?;

    ctx.spend(singleton_2, Spend::new(chk_puzzle_2, chk_sol_2))?;

    let result_2 = sim.spend_coins(ctx.take(), &[]);
    assert!(
        result_2.is_ok(),
        "Epoch 1→2 checkpoint must succeed: {:?}",
        result_2.err()
    );
    eprintln!("Epoch 1→2 checkpoint PASSED");

    // Verify singleton chain
    let children_2 = sim.children(singleton_2.coin_id());
    let singleton_3 = children_2.iter().find(|cs| cs.coin.amount == 1);
    assert!(
        singleton_3.is_some(),
        "Singleton must be recreated after epoch 2"
    );

    let old_1 = sim.coin_state(singleton_coin.coin_id()).unwrap();
    let old_2 = sim.coin_state(singleton_2.coin_id()).unwrap();
    assert!(
        old_1.spent_height.is_some(),
        "First singleton must be spent"
    );
    assert!(
        old_2.spent_height.is_some(),
        "Second singleton must be spent"
    );

    eprintln!("Two-epoch E2E: deploy → epoch 1 → epoch 2 PASSED ✓");
    Ok(())
}

// ── Test: CHK-008g — Invalid proof is rejected ───────────────────────

#[test]
fn vv_req_chk_008_invalid_proof_rejected() {
    // Build env with WRONG proof bytes (all zeros/non-curve points)
    // The bls_pairing_identity check should fail.

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = extract_vk_components(&pk.vk).expect("VK");

    use blst::min_pk as bls;
    const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";
    let mut ikm = [0u8; 32];
    ikm[0] = 0x44;
    let bls_sk = bls::SecretKey::key_gen(&ikm, b"reject").unwrap();
    let agg_signers: [u8; 48] = bls_sk.sk_to_pk().compress();

    let vmr = [0x00u8; 32];
    let vc: u64 = 1;
    let new_vmr = [0xCCu8; 32];
    let new_vc: u64 = 1;
    let state_root = [0x00u8; 32];
    let epoch: u64 = 0;
    let new_epoch = epoch + 1;
    let new_sr = [0xDDu8; 32];

    let checkpoint_msg: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(new_sr);
        h.update(new_vmr);
        h.update(new_vc.to_be_bytes());
        h.update(new_epoch.to_be_bytes());
        h.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
        h.finalize().into()
    };

    let agg_sig: [u8; 96] = bls_sk.sign(&checkpoint_msg, DST, &agg_signers).compress();

    let scalars: [[u8; 32]; 6] = [
        sha(&vmr),
        sha(&vc.to_be_bytes()),
        sha(&new_vmr),
        sha(&new_vc.to_be_bytes()),
        sha(&agg_signers),
        sha(&checkpoint_msg),
    ];

    let vk_alpha: [u8; 48] = vk.alpha_g1.try_into().unwrap();
    let vk_beta: [u8; 96] = vk.beta_g2.try_into().unwrap();
    let vk_gamma: [u8; 96] = vk.gamma_g2.try_into().unwrap();
    let vk_delta: [u8; 96] = vk.delta_g2.try_into().unwrap();
    let ic: [[u8; 48]; 7] = {
        let mut arr = [[0u8; 48]; 7];
        for (i, pt) in vk.ic_points.iter().enumerate() {
            arr[i] = pt.as_slice().try_into().unwrap();
        }
        arr
    };

    // Use INVALID proof (generate proof for WRONG inputs, then use with correct scalars)
    // This simulates a proof forgery attempt: valid proof format but wrong inputs
    let wrong_circuit = ConsensusCircuit::with_public_inputs(
        [0xFF; 32], 99, [0xFF; 32], 99, [0xFF; 48], [0xFF; 32], 99,
    );
    let wrong_proof_bytes = generate_proof(wrong_circuit, &pk).expect("Wrong proof");
    let proof_a: [u8; 48] = wrong_proof_bytes[0..48].try_into().unwrap();
    let proof_b: [u8; 96] = wrong_proof_bytes[48..144].try_into().unwrap();
    let proof_c: [u8; 48] = wrong_proof_bytes[144..192].try_into().unwrap();

    let empty_leaf_hash: [u8; 32] = sha(&[0u8; 48]);
    let inner_mod_hash = chk_inner_mod_hash();

    let mut a = clvmr::Allocator::new();
    let puzzle_bytes = hex::decode(CHK_HEX.trim()).unwrap();
    let puzzle = node_from_bytes(&mut a, &puzzle_bytes).unwrap();

    let env = build_chk_path_env(
        &mut a,
        &inner_mod_hash,
        &vk_alpha,
        &vk_beta,
        &vk_gamma,
        &vk_delta,
        &ic,
        32,
        &empty_leaf_hash,
        &state_root,
        epoch,
        &vmr,
        vc,
        &proof_a,
        &proof_b,
        &proof_c,
        &new_sr,
        &new_vmr,
        new_vc,
        &agg_signers,
        &agg_sig,
        &scalars,
    );

    let result = clvmr::run_program(
        &mut a,
        &clvmr::ChiaDialect::new(0),
        puzzle,
        env,
        11_000_000_000,
    );

    assert!(
        result.is_err(),
        "CHK-008: Checkpoint with wrong proof must be rejected by CLVM"
    );
    eprintln!(
        "CHK-008: Invalid proof correctly rejected: {}",
        result.unwrap_err().1
    );
}

// ── Diagnostic: trace what the puzzle sees at each pairing argument path ──
// NOTE: CLVM path numbers hardcoded for 6-param curry. After CHK-012 added
// NETWORK_COIN_LAUNCHER_ID as 7th curried param, all paths shift. This test
// needs path recalculation when re-enabled.

#[test]
#[ignore = "CLVM paths need recalculation after CHK-012 added 7th curried parameter"]
fn vv_req_chk_008_trace_puzzle_pairing_args() {
    use clvmr::{run_program, serde::node_from_bytes, Allocator, ChiaDialect, NodePtr, SExp};

    // ── Same setup as the failing test ──────────────────────────────
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = extract_vk_components(&pk.vk).expect("VK");

    use blst::min_pk as bls;
    const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";
    let mut ikm = [0u8; 32];
    ikm[0] = 0x42;
    let bls_sk = bls::SecretKey::key_gen(&ikm, b"chk008").unwrap();
    let agg_signers: [u8; 48] = bls_sk.sk_to_pk().compress();

    let vmr = [0xAAu8; 32];
    let vc: u64 = 3;
    let new_vmr = [0xBBu8; 32];
    let new_vc: u64 = 3;
    let state_root = [0x00u8; 32];
    let epoch: u64 = 0;
    let new_sr = [0xCCu8; 32];
    let new_epoch = epoch + 1;

    let checkpoint_msg: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(new_sr);
        h.update(new_vmr);
        h.update(new_vc.to_be_bytes());
        h.update(new_epoch.to_be_bytes());
        h.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
        h.finalize().into()
    };
    let agg_sig: [u8; 96] = bls_sk.sign(&checkpoint_msg, DST, &agg_signers).compress();

    let circuit = ConsensusCircuit::with_public_inputs(
        vmr,
        vc,
        new_vmr,
        new_vc,
        agg_signers,
        checkpoint_msg,
        vc as usize,
    );
    let proof_bytes = generate_proof(circuit, &pk).expect("Proof");
    let proof_a: [u8; 48] = proof_bytes[0..48].try_into().unwrap();
    let proof_b: [u8; 96] = proof_bytes[48..144].try_into().unwrap();
    let proof_c: [u8; 48] = proof_bytes[144..192].try_into().unwrap();

    let scalars: [[u8; 32]; 6] = [
        sha(&vmr),
        sha(&vc.to_be_bytes()),
        sha(&new_vmr),
        sha(&new_vc.to_be_bytes()),
        sha(&agg_signers),
        sha(&checkpoint_msg),
    ];

    let vk_alpha: [u8; 48] = vk.alpha_g1.try_into().unwrap();
    let vk_beta: [u8; 96] = vk.beta_g2.try_into().unwrap();
    let vk_gamma: [u8; 96] = vk.gamma_g2.try_into().unwrap();
    let vk_delta: [u8; 96] = vk.delta_g2.try_into().unwrap();
    let ic: [[u8; 48]; 7] = {
        let mut arr = [[0u8; 48]; 7];
        for (i, pt) in vk.ic_points.iter().enumerate() {
            arr[i] = pt.as_slice().try_into().unwrap();
        }
        arr
    };
    let empty_leaf_hash: [u8; 32] = sha(&[0u8; 48]);
    let inner_mod_hash = chk_inner_mod_hash();

    // ── Build env and load puzzle ──────────────────────────────────
    let mut a = Allocator::new();
    let puzzle_bytes = hex::decode(CHK_HEX.trim()).unwrap();
    let module = node_from_bytes(&mut a, &puzzle_bytes).unwrap();

    let flat_env = build_chk_path_env(
        &mut a,
        &inner_mod_hash,
        &vk_alpha,
        &vk_beta,
        &vk_gamma,
        &vk_delta,
        &ic,
        32,
        &empty_leaf_hash,
        &state_root,
        epoch,
        &vmr,
        vc,
        &proof_a,
        &proof_b,
        &proof_c,
        &new_sr,
        &new_vmr,
        new_vc,
        &agg_signers,
        &agg_sig,
        &scalars,
    );

    // ── Extract outer env_builder and run it to get body_env ──────
    let (body_code, outer_env_builder) = match a.sexp(module) {
        SExp::Pair(_, rest) => match a.sexp(rest) {
            SExp::Pair(quoted_body, eb_pair) => {
                let body = match a.sexp(quoted_body) {
                    SExp::Pair(_, b) => b,
                    _ => panic!("expected (q . body)"),
                };
                let eb = match a.sexp(eb_pair) {
                    SExp::Pair(eb, _) => eb,
                    _ => panic!("expected (eb . nil)"),
                };
                (body, eb)
            }
            _ => panic!("unexpected"),
        },
        _ => panic!("unexpected"),
    };

    let body_env = match run_program(
        &mut a,
        &ChiaDialect::new(0),
        outer_env_builder,
        flat_env,
        1_000_000,
    ) {
        Ok(clvmr::reduction::Reduction(_, n)) => n,
        Err(e) => panic!("outer env_builder failed: {}", e.1),
    };
    eprintln!("body_env OK");

    // ── Helper: evaluate a path in an env ─────────────────────────
    let eval_path = |a: &mut Allocator, path: i64, env: NodePtr| -> Result<Vec<u8>, String> {
        let path_bytes = if path >= 0 {
            let p = path as u64;
            let be = p.to_be_bytes();
            be.iter()
                .copied()
                .skip_while(|&b| b == 0)
                .collect::<Vec<u8>>()
        } else {
            // Negative path: compute two's complement bytes
            let p = path;
            let be = p.to_be_bytes();
            // Strip leading 0xFF bytes (sign extension) but keep at least 1 byte
            let stripped: Vec<u8> = be.iter().copied().skip_while(|&b| b == 0xFF).collect();
            if stripped.is_empty() || stripped[0] & 0x80 == 0 {
                let mut with_sign = vec![0xFF];
                with_sign.extend_from_slice(&stripped);
                with_sign
            } else {
                stripped
            }
        };
        if path_bytes.is_empty() {
            return Err("empty path".to_string());
        }
        let path_node = a.new_atom(&path_bytes).unwrap();
        match run_program(a, &ChiaDialect::new(0), path_node, env, 100_000) {
            Ok(clvmr::reduction::Reduction(_, n)) => {
                match a.sexp(n) {
                    SExp::Atom => Ok(a.atom(n).as_ref().to_vec()),
                    SExp::Pair(_, _) => Ok(vec![0xFF, 0xFF]), // marker for pair
                }
            }
            Err(e) => Err(e.1.to_string()),
        }
    };

    // ── Build level1_env using the checkpoint branch env_builder ──
    // level1 env_builder: (c (c (+ 703 (q . 1)) (c 3071 1535)) 1)
    // This runs in body_env context
    // Instead of extracting from CLVM, reconstruct manually:
    // new_epoch = STATE.epoch + 1
    // 703 in body_env = STATE.epoch, 3071 = new_vmr (param 10), 1535 = new_sr (param 9)
    let epoch_val = eval_path(&mut a, 703, body_env);
    eprintln!(
        "path 703 (STATE.epoch) in body_env: {:?}",
        epoch_val.as_ref().map(|v| v.len())
    );
    let new_vmr_val = eval_path(&mut a, 3071, body_env);
    eprintln!(
        "path 3071 (new_vmr) in body_env: {:?}",
        new_vmr_val.as_ref().map(|v| v.len())
    );
    let new_sr_val = eval_path(&mut a, 1535, body_env);
    eprintln!(
        "path 1535 (new_sr) in body_env: {:?}",
        new_sr_val.as_ref().map(|v| v.len())
    );

    // Build level1_env = ((new_epoch . (new_vmr . new_sr)) . body_env)
    let new_epoch_node = {
        let ep = eval_path(&mut a, 703, body_env).unwrap();
        let ep_num = if ep.is_empty() {
            0i64
        } else {
            let mut val = 0i64;
            for &b in &ep {
                val = (val << 8) | b as i64;
            }
            val
        };
        let new_ep = ep_num + 1;
        let be = new_ep.to_be_bytes();
        let stripped: Vec<u8> = be.iter().copied().skip_while(|&b| b == 0).collect();
        if stripped.is_empty() {
            a.nil()
        } else {
            a.new_atom(&stripped).unwrap()
        }
    };
    let new_vmr_node = {
        let v = eval_path(&mut a, 3071, body_env).unwrap();
        a.new_atom(&v).unwrap()
    };
    let new_sr_node = {
        let v = eval_path(&mut a, 1535, body_env).unwrap();
        a.new_atom(&v).unwrap()
    };
    let triple = {
        let t = a.new_pair(new_vmr_node, new_sr_node).unwrap();
        a.new_pair(new_epoch_node, t).unwrap()
    };
    let level1_env = a.new_pair(triple, body_env).unwrap();
    eprintln!("level1_env OK");

    // ── Build level2_env = (checkpoint_message . level1_env) ──────
    // checkpoint_message = sha256(new_sr || new_vmr || int_to_8_bytes_be(new_vc) || int_to_8_bytes_be(new_epoch))
    // Computed from paths in level1_env: 14=new_sr_from_level1? Actually:
    // In level1_env, path 14 and 10 come from the env_builder expression.
    // Let me just use the known checkpoint_msg value.
    let ckpt_msg_node = a.new_atom(&checkpoint_msg).unwrap();
    let level2_env = a.new_pair(ckpt_msg_node, level1_env).unwrap();
    eprintln!("level2_env OK");

    // ── Now evaluate the paths used by bls_pairing_identity ───────
    let expected: Vec<(&str, i64, &[u8])> = vec![
        ("proof.a", 5119, &proof_a),
        ("proof.b", 11263, &proof_b),
        ("VK.alpha", 79, &vk_alpha),
        ("VK.beta", -81, &vk_beta),
        ("VK.gamma", 367, &vk_gamma),
        ("proof.c", 23551, &proof_c),
        ("VK.delta", 751, &vk_delta),
        ("IC[0]", -97, &ic[0]),
        ("IC[1]", 351, &ic[1]),
        ("scalars.s1", 0x04ffff, &scalars[0]),
    ];

    let mut all_match = true;
    for (name, path, expected_bytes) in &expected {
        match eval_path(&mut a, *path, level2_env) {
            Ok(actual) => {
                if actual == *expected_bytes {
                    eprintln!("  {} (path {}) = {} bytes ✓", name, path, actual.len());
                } else {
                    eprintln!(
                        "  {} (path {}) = {} bytes MISMATCH! expected {} bytes",
                        name,
                        path,
                        actual.len(),
                        expected_bytes.len()
                    );
                    eprintln!("    actual:   {:02x?}", &actual[..8.min(actual.len())]);
                    eprintln!(
                        "    expected: {:02x?}",
                        &expected_bytes[..8.min(expected_bytes.len())]
                    );
                    all_match = false;
                }
            }
            Err(e) => {
                eprintln!("  {} (path {}) ERROR: {}", name, path, e);
                all_match = false;
            }
        }
    }

    if all_match {
        eprintln!("\nAll paths match expected values in manual level2_env.");
    } else {
        eprintln!("\nPath mismatches found!");
    }

    // ── Now build the EXACT bls_pairing_identity call the puzzle makes ──
    // Using the puzzle's path numbers against our level2_env
    // Program: (58 5119 11263 (51 79) 175 (51 (29 (29 (29 (29 (29 (29 -97 (50 351 327679)) (50 735 720895)) (50 1503 1507327)) (50 3039 3080191)) (50 6111 6225919)) (50 12255 12517375))) 367 (51 23551) 751)
    //
    // This is complex. Let me build it step by step.

    let mk_path = |a: &mut Allocator, val: i64| -> NodePtr {
        if val >= 0 {
            let p = val as u64;
            let be = p.to_be_bytes();
            let stripped: Vec<u8> = be.iter().copied().skip_while(|&b| b == 0).collect();
            if stripped.is_empty() {
                a.nil()
            } else {
                a.new_atom(&stripped).unwrap()
            }
        } else {
            let be = val.to_be_bytes();
            // For negative CLVM atoms: strip leading 0xFF but keep sign bit
            let mut start = 0;
            while start < 7 && be[start] == 0xFF && be[start + 1] & 0x80 != 0 {
                start += 1;
            }
            a.new_atom(&be[start..]).unwrap()
        }
    };

    let nil = a.nil();
    let op58 = a.new_atom(&[58]).unwrap(); // bls_pairing_identity
    let op51 = a.new_atom(&[51]).unwrap(); // g1_negate
    let op50 = a.new_atom(&[50]).unwrap(); // g1_multiply
    let op29 = a.new_atom(&[29]).unwrap(); // point_add

    // Build vk_input = IC[0] + sum(IC[i]*s[i])
    // IC paths: -97, 351, 735, 1503, 3039, 6111, 12255
    // scalar paths: 0x04ffff, 0x0affff, 0x16ffff, 0x2effff, 0x5effff, 0xbeffff
    let ic_paths: [i64; 7] = [-97, 351, 735, 1503, 3039, 6111, 12255];
    let s_paths: [i64; 6] = [0x04ffff, 0x0affff, 0x16ffff, 0x2effff, 0x5effff, 0xbeffff];

    // Start with IC[0]
    let mut vk_input_expr = mk_path(&mut a, ic_paths[0]);
    for i in 0..6 {
        // (50 IC[i+1] s[i]) = g1_multiply
        let ic_p = mk_path(&mut a, ic_paths[i + 1]);
        let s_p = mk_path(&mut a, s_paths[i]);
        let mul_args = a.new_pair(s_p, nil).unwrap();
        let mul_args = a.new_pair(ic_p, mul_args).unwrap();
        let mul = a.new_pair(op50, mul_args).unwrap();
        // (29 prev mul) = point_add
        let add_args = a.new_pair(mul, nil).unwrap();
        let add_args = a.new_pair(vk_input_expr, add_args).unwrap();
        vk_input_expr = a.new_pair(op29, add_args).unwrap();
    }

    // (51 vk_input_expr) = g1_negate
    let neg_vk = a.new_pair(vk_input_expr, nil).unwrap();
    let neg_vk = a.new_pair(op51, neg_vk).unwrap();

    // (51 79) = -alpha
    let p79 = mk_path(&mut a, 79);
    let neg_alpha = a.new_pair(p79, nil).unwrap();
    let neg_alpha = a.new_pair(op51, neg_alpha).unwrap();

    // (51 23551) = -proof.c
    let p23551 = mk_path(&mut a, 23551);
    let neg_c = a.new_pair(p23551, nil).unwrap();
    let neg_c = a.new_pair(op51, neg_c).unwrap();

    // Full: (58 5119 11263 neg_alpha 175 neg_vk 367 neg_c 751)
    let p5119 = mk_path(&mut a, 5119);
    let p11263 = mk_path(&mut a, 11263);
    let p175 = mk_path(&mut a, -81); // VK.beta = path 175 unsigned
    let p367 = mk_path(&mut a, 367);
    let p751 = mk_path(&mut a, 751);

    let pairing_args = a.new_pair(p751, nil).unwrap();
    let pairing_args = a.new_pair(neg_c, pairing_args).unwrap();
    let pairing_args = a.new_pair(p367, pairing_args).unwrap();
    let pairing_args = a.new_pair(neg_vk, pairing_args).unwrap();
    let pairing_args = a.new_pair(p175, pairing_args).unwrap();
    let pairing_args = a.new_pair(neg_alpha, pairing_args).unwrap();
    let pairing_args = a.new_pair(p11263, pairing_args).unwrap();
    let pairing_args = a.new_pair(p5119, pairing_args).unwrap();
    let pairing_prog = a.new_pair(op58, pairing_args).unwrap();

    let pairing_result = run_program(
        &mut a,
        &ChiaDialect::new(0),
        pairing_prog,
        level2_env,
        11_000_000_000,
    );
    match &pairing_result {
        Ok(_) => eprintln!("\nPairing with puzzle paths against manual level2_env: PASSED!"),
        Err(e) => eprintln!(
            "\nPairing with puzzle paths against manual level2_env: FAILED: {}",
            e.1
        ),
    }

    // ── Now get the REAL level2_env by running the puzzle's env_builders ──
    // Extract checkpoint THEN branch from compiled CLVM:
    // module = (a (q BODY) outer_eb)
    // BODY = (a (i 383 THEN ELSE) 1)
    // THEN = (q (a INNER1 eb1)) where eb1 = level1 env_builder
    // INNER1 = (q (a INNER2 eb2)) where eb2 = level2 env_builder

    // Navigate: body_code = BODY, already extracted above
    // BODY = (2 (i 383 THEN ELSE) 1) → second element is (i 383 THEN ELSE)
    let if_expr = match a.sexp(body_code) {
        SExp::Pair(_, rest) => match a.sexp(rest) {
            SExp::Pair(if_node, _) => if_node,
            _ => panic!("expected (if_expr . rest)"),
        },
        _ => panic!("expected pair"),
    };
    // if_expr = (i 383 THEN ELSE) → THEN is 3rd element = cdr(cdr(if_expr)).car
    let then_quoted = match a.sexp(if_expr) {
        SExp::Pair(_, rest) => match a.sexp(rest) {
            // skip 'i'... actually (3 383 THEN ELSE)
            SExp::Pair(_, rest2) => match a.sexp(rest2) {
                SExp::Pair(then_q, _) => then_q, // THEN (quoted)
                _ => panic!("expected THEN"),
            },
            _ => panic!("expected (383 THEN ELSE)"),
        },
        _ => panic!("expected if pair"),
    };
    // then_quoted = (q . (a INNER1 eb1)) → cdr = (a INNER1 eb1)
    let then_body = match a.sexp(then_quoted) {
        SExp::Pair(_, body) => body,
        _ => panic!("expected (q . body)"),
    };
    // then_body = (a INNER1 eb1) = (2 . (INNER1 . (eb1 . nil)))
    // eb1 is the 3rd element = cdr(cdr(then_body)).car
    let (inner1_quoted, eb1) = match a.sexp(then_body) {
        SExp::Pair(_, rest) => match a.sexp(rest) {
            SExp::Pair(inner1, rest2) => {
                let eb = match a.sexp(rest2) {
                    SExp::Pair(e, _) => e,
                    _ => panic!("expected (eb1 . nil)"),
                };
                (inner1, eb)
            }
            _ => panic!(""),
        },
        _ => panic!(""),
    };

    // Run eb1 in body_env to get REAL level1_env
    let real_level1 = match run_program(&mut a, &ChiaDialect::new(0), eb1, body_env, 1_000_000) {
        Ok(clvmr::reduction::Reduction(_, n)) => n,
        Err(e) => panic!("eb1 failed: {}", e.1),
    };
    eprintln!("Real level1_env OK");

    // inner1_quoted = (q . (a INNER2 eb2))
    let inner1_body = match a.sexp(inner1_quoted) {
        SExp::Pair(_, body) => body,
        _ => panic!("expected (q . body)"),
    };
    let (_inner2_quoted, eb2) = match a.sexp(inner1_body) {
        SExp::Pair(_, rest) => match a.sexp(rest) {
            SExp::Pair(inner2, rest2) => {
                let eb = match a.sexp(rest2) {
                    SExp::Pair(e, _) => e,
                    _ => panic!("expected (eb2 . nil)"),
                };
                (inner2, eb)
            }
            _ => panic!(""),
        },
        _ => panic!(""),
    };

    // Run eb2 in real_level1 to get REAL level2_env
    let real_level2 = match run_program(&mut a, &ChiaDialect::new(0), eb2, real_level1, 1_000_000) {
        Ok(clvmr::reduction::Reduction(_, n)) => n,
        Err(e) => panic!("eb2 failed: {}", e.1),
    };
    eprintln!("Real level2_env OK");

    // Compare checkpoint_message: path 2 in level2_env = car = ckpt_msg
    let real_ckpt = eval_path(&mut a, 2, real_level2).unwrap();
    let manual_ckpt = eval_path(&mut a, 2, level2_env).unwrap();
    if real_ckpt == manual_ckpt {
        eprintln!("checkpoint_msg MATCHES between real and manual level2_env ✓");
    } else {
        eprintln!("checkpoint_msg MISMATCH!");
        eprintln!(
            "  real:   {} bytes {:02x?}",
            real_ckpt.len(),
            &real_ckpt[..8.min(real_ckpt.len())]
        );
        eprintln!(
            "  manual: {} bytes {:02x?}",
            manual_ckpt.len(),
            &manual_ckpt[..8.min(manual_ckpt.len())]
        );
    }

    // Now run the pairing against the REAL level2_env
    let real_pairing = run_program(
        &mut a,
        &ChiaDialect::new(0),
        pairing_prog,
        real_level2,
        11_000_000_000,
    );
    match &real_pairing {
        Ok(_) => eprintln!("Pairing with puzzle paths against REAL level2_env: PASSED!"),
        Err(e) => eprintln!(
            "Pairing with puzzle paths against REAL level2_env: FAILED: {}",
            e.1
        ),
    }

    // ── Run INNER2 body directly in real level2_env ──
    let inner2_body = match a.sexp(_inner2_quoted) {
        SExp::Pair(_, body) => body,
        _ => panic!("expected (q . body)"),
    };
    let inner2_direct = run_program(
        &mut a,
        &ChiaDialect::new(0),
        inner2_body,
        real_level2,
        11_000_000_000,
    );
    match &inner2_direct {
        Ok(_) => eprintln!("INNER2 body in real level2_env: PASSED!"),
        Err(e) => eprintln!("INNER2 body in real level2_env: FAILED: {}", e.1),
    }

    // ── Run INNER1 body (= a INNER2 eb2) in real level1_env ──
    let inner1_body_direct = match a.sexp(inner1_quoted) {
        SExp::Pair(_, body) => body,
        _ => panic!("expected (q . body)"),
    };
    let inner1_direct = run_program(
        &mut a,
        &ChiaDialect::new(0),
        inner1_body_direct,
        real_level1,
        11_000_000_000,
    );
    match &inner1_direct {
        Ok(_) => eprintln!("INNER1 body in real level1_env: PASSED!"),
        Err(e) => eprintln!("INNER1 body in real level1_env: FAILED: {}", e.1),
    }

    // ── Run the inner1 code (a INNER1 eb1) in body_env ──
    // This is what the puzzle actually does after the is_checkpoint check
    let inner1_result = run_program(
        &mut a,
        &ChiaDialect::new(0),
        then_body,
        body_env,
        11_000_000_000,
    );
    match &inner1_result {
        Ok(clvmr::reduction::Reduction(_, _)) => {
            eprintln!("Running (a INNER1 eb1) in body_env: PASSED!");
        }
        Err(e) => {
            eprintln!("Running (a INNER1 eb1) in body_env: FAILED: {}", e.1);
        }
    }

    // ── Run the full puzzle with flat_env ──
    let full_result = run_program(
        &mut a,
        &ChiaDialect::new(0),
        module,
        flat_env,
        11_000_000_000,
    );
    match &full_result {
        Ok(_) => eprintln!("Full puzzle execution: PASSED!"),
        Err(e) => eprintln!("Full puzzle execution: FAILED: {}", e.1),
    }
}

// ── CIR-004 E2E: Majority proof generates and verifies on-chain ──────

#[test]
fn vv_req_cir_004_majority_proof_verified_on_chain() {
    // CIR-004: A proof generated with majority signers (k=3, n=5)
    // is accepted by the on-chain CLVM puzzle.

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk = extract_vk_components(&pk.vk).expect("VK");

    use blst::min_pk as bls;
    const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";
    let mut ikm = [0u8; 32];
    ikm[0] = 0x77;
    let bls_sk = bls::SecretKey::key_gen(&ikm, b"majority").unwrap();
    let agg_signers: [u8; 48] = bls_sk.sk_to_pk().compress();

    let vmr = [0x11u8; 32];
    let vc: u64 = 5; // 5 validators total
    let new_vmr = [0x22u8; 32];
    let new_vc: u64 = 5;
    let state_root = [0x00u8; 32];
    let epoch: u64 = 0;
    let new_sr = [0x33u8; 32];
    let new_epoch = epoch + 1;

    let checkpoint_msg: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(new_sr);
        h.update(new_vmr);
        h.update(new_vc.to_be_bytes());
        h.update(new_epoch.to_be_bytes());
        h.update([0x00u8; 32]); // CHK-012: network_coin_launcher_id
        h.finalize().into()
    };
    let agg_sig: [u8; 96] = bls_sk.sign(&checkpoint_msg, DST, &agg_signers).compress();

    // MAJORITY: k=3 signers out of n=5 validators (2*3=6 > 5) ✓
    let circuit = ConsensusCircuit::with_public_inputs(
        vmr,
        vc,
        new_vmr,
        new_vc,
        agg_signers,
        checkpoint_msg,
        3, // actual_signers = 3 (majority of 5)
    );
    let proof_bytes = generate_proof(circuit, &pk).expect("Majority proof must generate");

    let scalars: [[u8; 32]; 6] = [
        sha(&vmr),
        sha(&vc.to_be_bytes()),
        sha(&new_vmr),
        sha(&new_vc.to_be_bytes()),
        sha(&agg_signers),
        sha(&checkpoint_msg),
    ];

    let vk_alpha: [u8; 48] = vk.alpha_g1.try_into().unwrap();
    let vk_beta: [u8; 96] = vk.beta_g2.try_into().unwrap();
    let vk_gamma: [u8; 96] = vk.gamma_g2.try_into().unwrap();
    let vk_delta: [u8; 96] = vk.delta_g2.try_into().unwrap();
    let ic: [[u8; 48]; 7] = {
        let mut arr = [[0u8; 48]; 7];
        for (i, pt) in vk.ic_points.iter().enumerate() {
            arr[i] = pt.as_slice().try_into().unwrap();
        }
        arr
    };
    let empty_leaf_hash: [u8; 32] = sha(&[0u8; 48]);
    let inner_mod_hash = chk_inner_mod_hash();

    let mut a = clvmr::Allocator::new();
    let puzzle = node_from_bytes(&mut a, &hex::decode(CHK_HEX.trim()).unwrap()).unwrap();
    let env = build_chk_path_env(
        &mut a,
        &inner_mod_hash,
        &vk_alpha,
        &vk_beta,
        &vk_gamma,
        &vk_delta,
        &ic,
        32,
        &empty_leaf_hash,
        &state_root,
        epoch,
        &vmr,
        vc,
        &proof_bytes[0..48].try_into().unwrap(),
        &proof_bytes[48..144].try_into().unwrap(),
        &proof_bytes[144..192].try_into().unwrap(),
        &new_sr,
        &new_vmr,
        new_vc,
        &agg_signers,
        &agg_sig,
        &scalars,
    );

    let result = clvmr::run_program(
        &mut a,
        &clvmr::ChiaDialect::new(0),
        puzzle,
        env,
        11_000_000_000,
    );
    assert!(
        result.is_ok(),
        "CIR-004: Majority proof (k=3, n=5) must be accepted on-chain: {:?}",
        result.err().map(|e| e.1)
    );
    eprintln!("CIR-004: Majority proof (k=3, n=5) verified on-chain ✓");
}

#[test]
fn vv_req_cir_004_minority_proof_fails() {
    // CIR-004: A circuit with minority signers (k=2, n=5) is unsatisfiable.
    // The arkworks prover panics on unsatisfied constraints (assertion in prover.rs).

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    // MINORITY: k=2 signers out of n=5 validators (2*2=4 ≤ 5) ✗
    let circuit = ConsensusCircuit::with_public_inputs(
        [0x11; 32], 5, [0x22; 32], 5, [0xCC; 48], [0xDD; 32], 2, // minority
    );

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        generate_proof(circuit, &pk)
    }));
    assert!(
        result.is_err(),
        "CIR-004: Minority proof (k=2, n=5) must fail (unsatisfied constraints)"
    );
    eprintln!("CIR-004: Minority proof (k=2, n=5) correctly rejected ✓");
}

#[test]
fn vv_req_cir_004_boundary_majority() {
    // CIR-004 edge cases: exact boundaries for majority threshold.
    // Groth16 proof generation doesn't check satisfiability — the proof bytes
    // are produced regardless, but they only VERIFY for valid witnesses.

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");

    let try_prove = |label: &str, vc: u64, k: usize, expect_valid: bool| {
        let circuit =
            ConsensusCircuit::with_public_inputs([0; 32], vc, [0; 32], vc, [0; 48], [0; 32], k);
        // Arkworks panics on unsatisfied constraints, so catch panics
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            generate_proof(circuit, &pk)
        }));
        let ok = result.is_ok() && result.unwrap().is_ok();
        assert_eq!(
            ok, expect_valid,
            "CIR-004 {}: expected valid={}, got {}",
            label, expect_valid, ok
        );
        eprintln!(
            "  {} (k={}, n={}): {}",
            label,
            k,
            vc,
            if ok { "VALID" } else { "REJECTED" }
        );
    };

    try_prove("minimum majority", 100, 51, true); // 2*51=102 > 100
    try_prove("not strict majority", 100, 50, false); // 2*50=100 ≤ 100
    try_prove("single validator", 1, 1, true); // 2*1=2 > 1
    try_prove("no signers", 1, 0, false); // 2*0=0 ≤ 1
    try_prove("two of three", 3, 2, true); // 2*2=4 > 3
    try_prove("one of three", 3, 1, false); // 2*1=2 ≤ 3

    eprintln!("CIR-004: All boundary cases verified ✓");
}
