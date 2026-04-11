//! Debug: test checkpoint env layouts with both spend paths.

use clvmr::reduction::Reduction;
use clvmr::{run_program, serde::node_from_bytes, Allocator, ChiaDialect, NodePtr};
use sha2::{Digest, Sha256};

fn load(a: &mut Allocator) -> NodePtr {
    let hex = include_str!("../puzzles/compiled/checkpoint_inner.hex");
    node_from_bytes(a, &hex::decode(hex.trim()).unwrap()).unwrap()
}

fn u64n(a: &mut Allocator, v: u64) -> NodePtr {
    if v == 0 {
        return a.nil();
    }
    let b = v.to_be_bytes();
    let s: Vec<u8> = b.iter().copied().skip_while(|&x| x == 0).collect();
    if !s.is_empty() && s[0] & 0x80 != 0 {
        let mut r = vec![0u8];
        r.extend(&s);
        a.new_atom(&r).unwrap()
    } else {
        a.new_atom(&s).unwrap()
    }
}

fn st(a: &mut Allocator, fields: &[NodePtr]) -> NodePtr {
    let mut r = fields[fields.len() - 1];
    for i in (0..fields.len() - 1).rev() {
        r = a.new_pair(fields[i], r).unwrap();
    }
    r
}

fn flat(a: &mut Allocator, items: &[NodePtr]) -> NodePtr {
    let mut t = a.nil();
    for i in items.iter().rev() {
        t = a.new_pair(*i, t).unwrap();
    }
    t
}

#[test]
fn debug_chk_both_paths() {
    let el: [u8; 32] = Sha256::digest(&[0u8; 48]).into();
    let pk = [0xAA; 48];
    let active: [u8; 32] = Sha256::digest(&pk).into();
    let mut rp = Vec::new();
    rp.extend(&active);
    rp.extend(&el);
    let root: [u8; 32] = Sha256::digest(&rp).into();

    println!("\n=== Checkpoint path (is_checkpoint=true), struct layout ===");
    {
        let mut a = Allocator::new();
        let puzzle = load(&mut a);
        let nil = a.nil();

        // Compute checkpoint msg for scalar verification
        let new_sr = [0xCC; 32];
        let new_vmr = [0xDD; 32];
        let new_vc: u64 = 5;
        let epoch: u64 = 3;
        let new_epoch = epoch + 1;
        let mut cm_pre = Vec::new();
        cm_pre.extend(&new_sr);
        cm_pre.extend(&new_vmr);
        cm_pre.extend(&new_vc.to_be_bytes());
        cm_pre.extend(&new_epoch.to_be_bytes());
        let cm: [u8; 32] = Sha256::digest(&cm_pre).into();

        // Compute correct scalars
        let vmr = [0xBB; 32];
        let vc: u64 = 3;
        let agg_s = [0xEE; 48];
        let s1: [u8; 32] = Sha256::digest(&vmr).into();
        let s2: [u8; 32] = Sha256::digest(&vc.to_be_bytes()).into();
        let s3: [u8; 32] = Sha256::digest(&new_vmr).into();
        let s4: [u8; 32] = Sha256::digest(&new_vc.to_be_bytes()).into();
        let s5: [u8; 32] = Sha256::digest(&agg_s).into();
        let s6: [u8; 32] = Sha256::digest(&cm).into();

        // Build VK, IC, State, Proof, Scalars as structs
        let vk_a = a.new_atom(&[0x01; 48]).unwrap();
        let vk_b = a.new_atom(&[0x01; 96]).unwrap();
        let vk_g = a.new_atom(&[0x01; 96]).unwrap();
        let vk_d = a.new_atom(&[0x01; 96]).unwrap();
        let vk_s = st(&mut a, &[vk_a, vk_b, vk_g, vk_d]);

        let ic_n: Vec<NodePtr> = (0..7).map(|_| a.new_atom(&[0x01; 48]).unwrap()).collect();
        let ic_s = st(&mut a, &ic_n);

        let sr_n = a.new_atom(&[0xAA; 32]).unwrap();
        let ep_n = u64n(&mut a, epoch);
        let vmr_n = a.new_atom(&vmr).unwrap();
        let vc_n = u64n(&mut a, vc);
        let state_s = st(&mut a, &[sr_n, ep_n, vmr_n, vc_n]);

        let pa = a.new_atom(&[0x10; 48]).unwrap();
        let pb = a.new_atom(&[0x20; 96]).unwrap();
        let pc = a.new_atom(&[0x30; 48]).unwrap();
        let proof_s = st(&mut a, &[pa, pb, pc]);

        let sn: Vec<NodePtr> = [s1, s2, s3, s4, s5, s6]
            .iter()
            .map(|s| a.new_atom(s).unwrap())
            .collect();
        let scalars_s = st(&mut a, &sn);

        let imh = a.new_atom(&[0x11; 32]).unwrap();
        let td = u64n(&mut a, 32);
        let elh = a.new_atom(&el).unwrap();
        let one = a.new_atom(&[1]).unwrap(); // is_checkpoint = true
        let nsr = a.new_atom(&new_sr).unwrap();
        let nmr = a.new_atom(&new_vmr).unwrap();
        let nvc = u64n(&mut a, new_vc);
        let asig = a.new_atom(&agg_s).unwrap();
        let as_n = a.new_atom(&[0xFF; 96]).unwrap();
        // query fields (unused)
        let qpk = a.new_atom(&[0u8; 48]).unwrap();

        let sib_a = a.new_atom(&el).unwrap();
        let sib_node = a.new_pair(sib_a, nil).unwrap();

        let env = flat(
            &mut a,
            &[
                imh, vk_s, ic_s, td, elh, state_s, one, // is_checkpoint
                proof_s, nsr, nmr, nvc, asig, as_n, scalars_s, qpk, nil, sib_node, nil,
                nil, // query fields + conditions
            ],
        );

        match run_program(&mut a, &ChiaDialect::new(0), puzzle, env, 11_000_000_000) {
            Ok(Reduction(cost, _)) => println!("  checkpoint -> OK cost={}", cost),
            Err(e) => println!("  checkpoint -> ERR: {}", e.1),
        }
    }

    println!("\n=== Membership query (is_checkpoint=false), struct layout ===");
    {
        let mut a = Allocator::new();
        let puzzle = load(&mut a);
        let nil = a.nil();

        let imh = a.new_atom(&[0x11; 32]).unwrap();
        let vk_a = a.new_atom(&[0x01; 48]).unwrap();
        let vk_b = a.new_atom(&[0x01; 96]).unwrap();
        let vk_g = a.new_atom(&[0x01; 96]).unwrap();
        let vk_d = a.new_atom(&[0x01; 96]).unwrap();
        let vk_s = st(&mut a, &[vk_a, vk_b, vk_g, vk_d]);
        let ic_n: Vec<NodePtr> = (0..7).map(|_| a.new_atom(&[0x01; 48]).unwrap()).collect();
        let ic_s = st(&mut a, &ic_n);
        let td = u64n(&mut a, 1);
        let elh = a.new_atom(&el).unwrap();
        let sr_n = a.new_atom(&[0xAA; 32]).unwrap();
        let ep_n = u64n(&mut a, 5);
        let vmr_n = a.new_atom(&root).unwrap();
        let vc_n = u64n(&mut a, 1);
        let state_s = st(&mut a, &[sr_n, ep_n, vmr_n, vc_n]);
        // is_checkpoint = false (nil)
        let pa = a.new_atom(&[0u8; 48]).unwrap();
        let pb = a.new_atom(&[0u8; 96]).unwrap();
        let pc = a.new_atom(&[0u8; 48]).unwrap();
        let proof_s = st(&mut a, &[pa, pb, pc]);
        let nsr = a.new_atom(&[0u8; 32]).unwrap();
        let nmr = a.new_atom(&[0u8; 32]).unwrap();
        let asig = a.new_atom(&[0u8; 48]).unwrap();
        let as_n = a.new_atom(&[0u8; 96]).unwrap();
        let sn: Vec<NodePtr> = (0..6).map(|_| a.new_atom(&[0u8; 32]).unwrap()).collect();
        let scalars_s = st(&mut a, &sn);
        let qpk = a.new_atom(&pk).unwrap();
        let sib_a = a.new_atom(&el).unwrap();
        let sib_node = a.new_pair(sib_a, nil).unwrap();
        let is_mem = a.new_atom(&[1]).unwrap();

        let env = flat(
            &mut a,
            &[
                imh, vk_s, ic_s, td, elh, state_s, nil, // is_checkpoint = false
                proof_s, nsr, nmr, nil, asig, as_n, scalars_s, qpk, nil, sib_node, is_mem,
                nil, // query + conditions
            ],
        );

        match run_program(&mut a, &ChiaDialect::new(0), puzzle, env, 11_000_000_000) {
            Ok(Reduction(cost, _)) => println!("  query -> OK cost={}", cost),
            Err(e) => println!("  query -> ERR: {}", e.1),
        }
    }

    println!("Done.");
}
