//! Debug test to determine the correct environment layout for Rue-compiled puzzles.

use clvmr::reduction::Reduction;
use clvmr::{run_program, serde::node_from_bytes, Allocator, ChiaDialect, NodePtr, SExp};

fn load_hex(a: &mut Allocator, hex: &str) -> NodePtr {
    let bytes = hex::decode(hex.trim()).unwrap();
    node_from_bytes(a, &bytes).unwrap()
}

fn print_tree(a: &Allocator, node: NodePtr, depth: usize, max_depth: usize) -> String {
    if depth > max_depth {
        return "...".to_string();
    }
    match a.sexp(node) {
        SExp::Atom => {
            let bytes = a.atom(node);
            if bytes.is_empty() {
                "nil".to_string()
            } else if bytes.len() == 1 {
                format!("{}", bytes[0])
            } else {
                format!(
                    "[{}b:{}...]",
                    bytes.len(),
                    hex::encode(&bytes[..2.min(bytes.len())])
                )
            }
        }
        SExp::Pair(l, r) => {
            format!(
                "({} . {})",
                print_tree(a, l, depth + 1, max_depth),
                print_tree(a, r, depth + 1, max_depth)
            )
        }
    }
}

#[test]
fn debug_checkpoint_env_builder_output() {
    // What does the env-builder expression produce when run with our flat env?
    // outer = (a (q . body) env_builder)
    // env_builder when run with env E produces what body sees.
    // We extract env_builder and run it separately to see what's produced.

    let hex = include_str!("../puzzles/compiled/checkpoint_inner.hex");
    let mut a = Allocator::new();
    let module = load_hex(&mut a, hex);

    // Build a minimal env (flat list of 19 params, all zeros/dummy)
    // to run the env_builder part of the module
    let nil = a.nil();
    // Build a flat list of 19 pairs to represent env
    let dummy32 = a.new_atom(&[0u8; 32]).unwrap();
    let dummy48 = a.new_atom(&[0u8; 48]).unwrap();
    let dummy96 = a.new_atom(&[0u8; 96]).unwrap();
    let one_val = a.new_atom(&[1u8]).unwrap();

    // Flat env: arg1..arg19 as right-linked list
    // We'll use simple dummy atoms for all (the env builder doesn't evaluate the body)
    let make_flat = |a: &mut clvmr::Allocator| -> clvmr::NodePtr {
        let nil = a.nil();
        // 19 args, just put nil for most
        let t = a.new_pair(nil, nil).unwrap(); // conditions = (nil.nil)
        let t = a.new_pair(nil, t).unwrap(); // is_member
        let t = a.new_pair(nil, t).unwrap(); // siblings
        let t = a.new_pair(nil, t).unwrap(); // leaf_index
        let qpk = a.new_atom(&[0u8; 48]).unwrap();
        let t = a.new_pair(qpk, t).unwrap(); // query_pubkey
        let scalars_node = a.nil();
        let t = a.new_pair(scalars_node, t).unwrap(); // scalars (nil for now)
        let asig = a.new_atom(&[0u8; 96]).unwrap();
        let t = a.new_pair(asig, t).unwrap(); // agg_sig
        let asgn = a.new_atom(&[0u8; 48]).unwrap();
        let t = a.new_pair(asgn, t).unwrap(); // agg_signers
        let nvc = a.new_atom(&[1u8]).unwrap();
        let t = a.new_pair(nvc, t).unwrap(); // new_validator_count
        let nvmr = a.new_atom(&[0u8; 32]).unwrap();
        let t = a.new_pair(nvmr, t).unwrap(); // new_validator_merkle_root
        let nsr = a.new_atom(&[0u8; 32]).unwrap();
        let t = a.new_pair(nsr, t).unwrap(); // new_state_root
                                             // proof struct (a . (b . c))
        let pc = a.new_atom(&[0u8; 48]).unwrap();
        let pb = a.new_atom(&[0u8; 96]).unwrap();
        let pa = a.new_atom(&[0u8; 48]).unwrap();
        let pr = a.new_pair(pb, pc).unwrap();
        let pr = a.new_pair(pa, pr).unwrap();
        let t = a.new_pair(pr, t).unwrap(); // proof struct
        let one = a.new_atom(&[1u8]).unwrap();
        let t = a.new_pair(one, t).unwrap(); // is_checkpoint = true
                                             // state struct
        let vc = a.new_atom(&[1u8]).unwrap();
        let vmr = a.new_atom(&[0xAAu8; 32]).unwrap();
        let ep = a.new_atom(&[0u8]).unwrap();
        let sr = a.new_atom(&[0u8; 32]).unwrap();
        let st_inner = a.new_pair(vmr, vc).unwrap();
        let st_inner = a.new_pair(ep, st_inner).unwrap();
        let st = a.new_pair(sr, st_inner).unwrap();
        let t = a.new_pair(st, t).unwrap(); // STATE
        let elh = a.new_atom(&[0u8; 32]).unwrap();
        let t = a.new_pair(elh, t).unwrap(); // EMPTY_LEAF_HASH
        let td = a.new_atom(&[32u8]).unwrap();
        let t = a.new_pair(td, t).unwrap(); // TREE_DEPTH
                                            // IC struct (7 points) - build as nested cons
        let ic6 = a.new_atom(&[0u8; 48]).unwrap();
        let ic5 = a.new_atom(&[0u8; 48]).unwrap();
        let ic4 = a.new_atom(&[0u8; 48]).unwrap();
        let ic3 = a.new_atom(&[0u8; 48]).unwrap();
        let ic2 = a.new_atom(&[0u8; 48]).unwrap();
        let ic1 = a.new_atom(&[0u8; 48]).unwrap();
        let ic0 = a.new_atom(&[0u8; 48]).unwrap();
        let ics = a.new_pair(ic5, ic6).unwrap();
        let ics = a.new_pair(ic4, ics).unwrap();
        let ics = a.new_pair(ic3, ics).unwrap();
        let ics = a.new_pair(ic2, ics).unwrap();
        let ics = a.new_pair(ic1, ics).unwrap();
        let ics = a.new_pair(ic0, ics).unwrap();
        let t = a.new_pair(ics, t).unwrap(); // IC struct
                                             // VK struct (alpha . (beta . (gamma . delta)))
        let vd = a.new_atom(&[0u8; 96]).unwrap();
        let vg = a.new_atom(&[0u8; 96]).unwrap();
        let vb = a.new_atom(&[0u8; 96]).unwrap();
        let va = a.new_atom(&[0u8; 48]).unwrap();
        let vk_inner = a.new_pair(vg, vd).unwrap();
        let vk_inner = a.new_pair(vb, vk_inner).unwrap();
        let vk = a.new_pair(va, vk_inner).unwrap();
        let t = a.new_pair(vk, t).unwrap(); // VK struct
        let imh = a.new_atom(&[0u8; 32]).unwrap();
        a.new_pair(imh, t).unwrap() // INNER_MOD_HASH
    };
    let flat_env = make_flat(&mut a);

    // Extract the env_builder from the module: outer = (2 . ((1 . body) . (env_builder . nil)))
    // We want the env_builder = second arg of the outer a-call
    let env_builder = match a.sexp(module) {
        SExp::Pair(_, rest) => match a.sexp(rest) {
            SExp::Pair(_, pair2) => match a.sexp(pair2) {
                SExp::Pair(env_expr, _) => env_expr,
                _ => panic!("unexpected"),
            },
            _ => panic!("unexpected"),
        },
        _ => panic!("unexpected"),
    };

    eprintln!("\nenv_builder structure:");
    eprintln!("{}", print_tree(&a, env_builder, 0, 8));

    // Run the env_builder with our flat_env to see what the body sees
    let result = run_program(
        &mut a,
        &ChiaDialect::new(0),
        env_builder,
        flat_env,
        1_000_000,
    );
    match result {
        Ok(clvmr::reduction::Reduction(_, new_env)) => {
            eprintln!("\nBody env = run(env_builder, flat_env) -> OK");
            eprintln!("Body env structure:");
            eprintln!("{}", print_tree(&a, new_env, 0, 4));
        }
        Err(e) => {
            eprintln!("\nBody env = run(env_builder, flat_env) -> ERR: {}", e.1);
        }
    }
}

#[test]
fn debug_checkpoint_membership_path() {
    // Test the membership query path (is_checkpoint=false) which is simpler
    // If this works, the env layout is correct for the outer call
    // and the issue is specific to the checkpoint branch's nested let bindings
    use sha2::{Digest, Sha256};

    const CHK_HEX: &str = include_str!("../puzzles/compiled/checkpoint_inner.hex");

    let mut a = Allocator::new();
    let hex_bytes = hex::decode(CHK_HEX.trim()).unwrap();
    let puzzle = clvmr::serde::node_from_bytes(&mut a, &hex_bytes).unwrap();

    let nil = a.nil();
    let sha = |data: &[u8]| -> [u8; 32] { Sha256::digest(data).into() };
    let empty_leaf_hash = sha(&[0u8; 48]);

    // Build flat env with is_checkpoint=false (nil/0)
    // We'll use the membership query path with a simple Merkle proof
    // The validator merkle root = sha256(sha256(pubkey)) at depth 1 (trivial tree)
    let test_pk = [0x42u8; 48];
    let leaf_hash = sha(&test_pk); // sha256(pubkey) for a member
    let vmr = leaf_hash; // depth-1 tree: root = leaf (siblings = nil)

    // Build flat env: 19 params in order
    let imh = a.new_atom(&[0xABu8; 32]).unwrap(); // INNER_MOD_HASH
    let va = a.new_atom(&[0u8; 48]).unwrap();
    let vb = a.new_atom(&[0u8; 96]).unwrap();
    let vg = a.new_atom(&[0u8; 96]).unwrap();
    let vd = a.new_atom(&[0u8; 96]).unwrap();
    let vk = {
        let i = a.new_pair(vg, vd).unwrap();
        let i = a.new_pair(vb, i).unwrap();
        a.new_pair(va, i).unwrap()
    };
    let ic_p = a.new_atom(&[0u8; 48]).unwrap();
    let ics = {
        let ic6 = a.new_atom(&[0u8; 48]).unwrap();
        let ic5 = a.new_atom(&[0u8; 48]).unwrap();
        let ic4 = a.new_atom(&[0u8; 48]).unwrap();
        let ic3 = a.new_atom(&[0u8; 48]).unwrap();
        let ic2 = a.new_atom(&[0u8; 48]).unwrap();
        let ic1 = a.new_atom(&[0u8; 48]).unwrap();
        let ic0 = a.new_atom(&[0u8; 48]).unwrap();
        let t = a.new_pair(ic5, ic6).unwrap();
        let t = a.new_pair(ic4, t).unwrap();
        let t = a.new_pair(ic3, t).unwrap();
        let t = a.new_pair(ic2, t).unwrap();
        let t = a.new_pair(ic1, t).unwrap();
        a.new_pair(ic0, t).unwrap()
    };
    let tree_depth = a.new_atom(&[0u8]).unwrap(); // TREE_DEPTH = 0 for trivial test
    let elh = a.new_atom(empty_leaf_hash.as_slice()).unwrap();
    let state = {
        let vc = a.new_atom(&[1u8]).unwrap();
        let vmr_n = a.new_atom(vmr.as_slice()).unwrap();
        let ep = a.new_atom(&[0u8]).unwrap();
        let sr = a.new_atom(&[0u8; 32]).unwrap();
        let t = a.new_pair(vmr_n, vc).unwrap();
        let t = a.new_pair(ep, t).unwrap();
        a.new_pair(sr, t).unwrap()
    };

    // is_checkpoint = nil (false)
    let is_ckpt = a.nil();

    // proof (unused in membership path) - all zeros
    let pa = a.new_atom(&[0u8; 48]).unwrap();
    let pb = a.new_atom(&[0u8; 96]).unwrap();
    let pc = a.new_atom(&[0u8; 48]).unwrap();
    let proof = {
        let t = a.new_pair(pb, pc).unwrap();
        a.new_pair(pa, t).unwrap()
    };

    // new_state_root, new_vmr, new_vc (unused)
    let nsr = a.new_atom(&[0u8; 32]).unwrap();
    let nvmr = a.new_atom(&[0u8; 32]).unwrap();
    let nvc = a.new_atom(&[0u8]).unwrap();
    let agg_s = a.new_atom(&[0u8; 48]).unwrap();
    let agg_sig = a.new_atom(&[0u8; 96]).unwrap();

    // scalars (nil for membership path)
    let nil_scalars = a.nil();

    // query_pubkey = test_pk, leaf_index=0, siblings=nil (depth=0), is_member=true
    let qpk = a.new_atom(&test_pk).unwrap();
    let li = a.new_atom(&[0u8]).unwrap();
    let sib = a.nil(); // no siblings for depth=0 tree
    let is_member = a.new_atom(&[1u8]).unwrap(); // true

    // conditions = (nil . nil) for spread params
    let conds = a.new_pair(a.nil(), a.nil()).unwrap();

    // Build flat list right-to-left
    let t = a.new_pair(conds, a.nil()).unwrap(); // conditions
    let t = a.new_pair(is_member, t).unwrap(); // is_member
    let t = a.new_pair(sib, t).unwrap(); // siblings
    let t = a.new_pair(li, t).unwrap(); // leaf_index
    let t = a.new_pair(qpk, t).unwrap(); // query_pubkey
    let t = a.new_pair(nil_scalars, t).unwrap(); // scalars (nil = unused)
    let t = a.new_pair(agg_sig, t).unwrap(); // agg_sig
    let t = a.new_pair(agg_s, t).unwrap(); // agg_signers
    let t = a.new_pair(nvc, t).unwrap(); // new_validator_count
    let t = a.new_pair(nvmr, t).unwrap(); // new_validator_merkle_root
    let t = a.new_pair(nsr, t).unwrap(); // new_state_root
    let t = a.new_pair(proof, t).unwrap(); // proof struct
    let t = a.new_pair(is_ckpt, t).unwrap(); // is_checkpoint = false
    let t = a.new_pair(state, t).unwrap(); // STATE
    let t = a.new_pair(elh, t).unwrap(); // EMPTY_LEAF_HASH
    let t = a.new_pair(tree_depth, t).unwrap(); // TREE_DEPTH
    let t = a.new_pair(ics, t).unwrap(); // IC struct
    let t = a.new_pair(vk, t).unwrap(); // VK struct
    let env = a.new_pair(imh, t).unwrap(); // INNER_MOD_HASH

    let result = run_program(&mut a, &ChiaDialect::new(0), puzzle, env, 100_000_000);
    match result {
        Ok(clvmr::reduction::Reduction(cost, _)) => {
            eprintln!("Membership path OK (cost={})", cost);
        }
        Err(e) => {
            eprintln!("Membership path ERR: {}", e.1);
        }
    }
}

#[test]
fn debug_checkpoint_env_nesting_levels() {
    // The decompiled CLVM shows: for the checkpoint branch there are nested
    // (a (q body) (c X 1)) applies. We need to count exactly how many items
    // get prepended to env before the assertions.
    //
    // Strategy: trace through the if-branch by running it step-by-step.
    // Extract the env-builder for the checkpoint branch from the CLVM.

    // Build our standard flat env (using the same layout as passing tests)
    let mut a = clvmr::Allocator::new();
    let hex = include_str!("../puzzles/compiled/checkpoint_inner.hex");
    let hex_bytes = hex::decode(hex.trim()).unwrap();
    let module = clvmr::serde::node_from_bytes(&mut a, &hex_bytes).unwrap();

    // Build a trivial flat env for analysis
    // Using CHK-005's build_query_env style
    let nil = a.nil();
    let mk_atom = |a: &mut clvmr::Allocator, b: &[u8]| a.new_atom(b).unwrap();

    // Build flat env with 19 params
    let conds = a.new_pair(nil, nil).unwrap();
    let is_member = mk_atom(&mut a, &[1]);
    let sib = nil;
    let li = nil;
    let qpk = mk_atom(&mut a, &[0u8; 48]);
    // scalars struct (6 x 48-byte fields)
    let scalars = {
        let s6 = mk_atom(&mut a, &[0u8; 48]);
        let s5 = mk_atom(&mut a, &[0u8; 48]);
        let s4 = mk_atom(&mut a, &[0u8; 48]);
        let s3 = mk_atom(&mut a, &[0u8; 48]);
        let s2 = mk_atom(&mut a, &[0u8; 48]);
        let s1 = mk_atom(&mut a, &[0u8; 48]);
        let t = a.new_pair(s5, s6).unwrap();
        let t = a.new_pair(s4, t).unwrap();
        let t = a.new_pair(s3, t).unwrap();
        let t = a.new_pair(s2, t).unwrap();
        a.new_pair(s1, t).unwrap()
    };
    let agg_sig = mk_atom(&mut a, &[0u8; 96]);
    let agg_signers = mk_atom(&mut a, &[0u8; 48]);
    let nvc = nil;
    let nvmr = mk_atom(&mut a, &[0xBBu8; 32]);
    let nsr = mk_atom(&mut a, &[0xCCu8; 32]);
    let proof = {
        let pc = mk_atom(&mut a, &[0u8; 48]);
        let pb = mk_atom(&mut a, &[0u8; 96]);
        let pa = mk_atom(&mut a, &[0u8; 48]);
        let t = a.new_pair(pb, pc).unwrap();
        a.new_pair(pa, t).unwrap()
    };
    let is_checkpoint = mk_atom(&mut a, &[1]); // TRUE
    let state = {
        let vc = mk_atom(&mut a, &[1]);
        let vmr = mk_atom(&mut a, &[0xAAu8; 32]); // distinctive value for vmr
        let ep = nil; // epoch = 0
        let sr = mk_atom(&mut a, &[0u8; 32]);
        let t = a.new_pair(vmr, vc).unwrap();
        let t = a.new_pair(ep, t).unwrap();
        a.new_pair(sr, t).unwrap()
    };
    let elh = mk_atom(&mut a, &[0u8; 32]);
    let td = mk_atom(&mut a, &[32]); // TREE_DEPTH = 32
    let ic = {
        let i6 = mk_atom(&mut a, &[0x01u8; 48]);
        let i5 = mk_atom(&mut a, &[0x01u8; 48]);
        let i4 = mk_atom(&mut a, &[0x01u8; 48]);
        let i3 = mk_atom(&mut a, &[0x01u8; 48]);
        let i2 = mk_atom(&mut a, &[0x01u8; 48]);
        let i1 = mk_atom(&mut a, &[0x01u8; 48]);
        let i0 = mk_atom(&mut a, &[0x01u8; 48]);
        let t = a.new_pair(i5, i6).unwrap();
        let t = a.new_pair(i4, t).unwrap();
        let t = a.new_pair(i3, t).unwrap();
        let t = a.new_pair(i2, t).unwrap();
        let t = a.new_pair(i1, t).unwrap();
        a.new_pair(i0, t).unwrap()
    };
    let vk = {
        let delta = mk_atom(&mut a, &[0x01u8; 96]);
        let gamma = mk_atom(&mut a, &[0x01u8; 96]);
        let beta = mk_atom(&mut a, &[0x01u8; 96]);
        let alpha = mk_atom(&mut a, &[0x01u8; 48]);
        let t = a.new_pair(gamma, delta).unwrap();
        let t = a.new_pair(beta, t).unwrap();
        a.new_pair(alpha, t).unwrap()
    };
    let imh = mk_atom(&mut a, &[0x11u8; 32]);

    // Build flat env right-to-left (param19 first, param1 last)
    let t = a.new_pair(conds, nil).unwrap();
    let t = a.new_pair(is_member, t).unwrap();
    let t = a.new_pair(sib, t).unwrap();
    let t = a.new_pair(li, t).unwrap();
    let t = a.new_pair(qpk, t).unwrap();
    let t = a.new_pair(scalars, t).unwrap();
    let t = a.new_pair(agg_sig, t).unwrap();
    let t = a.new_pair(agg_signers, t).unwrap();
    let t = a.new_pair(nvc, t).unwrap();
    let t = a.new_pair(nvmr, t).unwrap();
    let t = a.new_pair(nsr, t).unwrap();
    let t = a.new_pair(proof, t).unwrap();
    let t = a.new_pair(is_checkpoint, t).unwrap();
    let t = a.new_pair(state, t).unwrap();
    let t = a.new_pair(elh, t).unwrap();
    let t = a.new_pair(td, t).unwrap();
    let t = a.new_pair(ic, t).unwrap();
    let t = a.new_pair(vk, t).unwrap();
    let flat_env = a.new_pair(imh, t).unwrap();

    // Run the outer env-builder to get body_env = (helpers . flat_env)
    let env_builder = match a.sexp(module) {
        SExp::Pair(_, rest) => match a.sexp(rest) {
            SExp::Pair(_, pair2) => match a.sexp(pair2) {
                SExp::Pair(env_expr, _) => env_expr,
                _ => panic!(),
            },
            _ => panic!(),
        },
        _ => panic!(),
    };

    let body_env = match run_program(
        &mut a,
        &ChiaDialect::new(0),
        env_builder,
        flat_env,
        1_000_000,
    ) {
        Ok(clvmr::reduction::Reduction(_, n)) => {
            eprintln!("body_env level1: {}", print_tree(&a, n, 0, 3));
            n
        }
        Err(e) => {
            eprintln!("ERR getting body_env: {}", e.1);
            return;
        }
    };

    // Now try to run with is_checkpoint=true env and check what paths the puzzle accesses
    // The key: run the full puzzle with flat_env and see where it fails
    let result = run_program(
        &mut a,
        &ChiaDialect::new(0),
        module,
        flat_env,
        1_000_000_000,
    );
    match result {
        Ok(_) => eprintln!("Puzzle ran OK (unexpected for dummy data)"),
        Err(e) => eprintln!("Puzzle ERR: {} (at env node {:?})", e.1, e.0),
    }

    // Test a path access: is path 5887 actually STATE.vmr in body_env?
    let prog_5887 = a
        .new_atom(
            &5887u32
                .to_be_bytes()
                .iter()
                .copied()
                .skip_while(|&b| b == 0)
                .collect::<Vec<_>>(),
        )
        .unwrap();
    match run_program(&mut a, &ChiaDialect::new(0), prog_5887, body_env, 100_000) {
        Ok(clvmr::reduction::Reduction(_, n)) => {
            let bytes = a.atom(n);
            eprintln!(
                "path 5887 in body_env = {}b: {:02x?}",
                bytes.len(),
                &bytes[..4.min(bytes.len())]
            );
        }
        Err(e) => eprintln!("path 5887 in body_env ERR: {}", e.1),
    }

    // What about after level1 checkpoint restructuring?
    // level1_env = ((new_epoch . (new_vmr . new_sr)) . body_env)
    // new_epoch = path 703 of body_env + 1 = STATE.epoch + 1 = 0 + 1 = 1
    // new_vmr = path 3071 of body_env = param10 = new_validator_merkle_root = nvmr = [0xBB;32]
    // new_sr = path 1535 of body_env = param9 = new_state_root = nsr = [0xCC;32]
    let new_epoch_val = a.new_atom(&[1]).unwrap(); // epoch 0 + 1 = 1
    let new_vmr_val = mk_atom(&mut a, &[0xBBu8; 32]);
    let new_sr_val = mk_atom(&mut a, &[0xCCu8; 32]);
    let triple = {
        let t = a.new_pair(new_vmr_val, new_sr_val).unwrap();
        a.new_pair(new_epoch_val, t).unwrap()
    };
    let level1_env = a.new_pair(triple, body_env).unwrap();
    eprintln!("level1_env: {}", print_tree(&a, level1_env, 0, 3));

    // Test path 5887 in level1_env
    match run_program(&mut a, &ChiaDialect::new(0), prog_5887, level1_env, 100_000) {
        Ok(clvmr::reduction::Reduction(_, n)) => {
            let bytes = a.atom(n);
            eprintln!(
                "path 5887 in level1_env = {}b: {:02x?}",
                bytes.len(),
                &bytes[..4.min(bytes.len())]
            );
        }
        Err(e) => eprintln!("path 5887 in level1_env ERR: {}", e.1),
    }
}

#[test]
fn debug_checkpoint_clvm_outer_structure() {
    let hex = include_str!("../puzzles/compiled/checkpoint_inner.hex");
    let mut a = Allocator::new();
    let node = load_hex(&mut a, hex);
    eprintln!("Checkpoint CLVM outer structure (depth=6):");
    eprintln!("{}", print_tree(&a, node, 0, 6));

    // Compare with registration_coin
    let reg_hex = include_str!("../puzzles/compiled/registration_coin.hex");
    let reg_node = load_hex(&mut a, reg_hex);
    eprintln!("\nRegistration coin CLVM outer structure (depth=4):");
    eprintln!("{}", print_tree(&a, reg_node, 0, 4));

    // The outermost node should be (a program env_builder)
    // For registration (no helpers): (a (q . body) 1)
    // For checkpoint (with helpers): (a (q . body) (c helpers 1))
    match a.sexp(node) {
        SExp::Pair(op, rest) => {
            let op_bytes = a.atom(op);
            eprintln!("\nCheckpoint outer op = {}", op_bytes[0]);
            // op should be 2 = 'a' (apply)
            match a.sexp(rest) {
                SExp::Pair(program, env_rest) => {
                    eprintln!("program = {}", print_tree(&a, program, 0, 2));
                    eprintln!("env_rest = {}", print_tree(&a, env_rest, 0, 3));
                }
                _ => eprintln!("rest is atom"),
            }
        }
        _ => eprintln!("outer is atom"),
    }
}

fn try_run(label: &str, hex: &str, build_env: impl FnOnce(&mut Allocator) -> NodePtr) {
    let mut a = Allocator::new();
    let puzzle = load_hex(&mut a, hex);
    let env = build_env(&mut a);
    match run_program(&mut a, &ChiaDialect::new(0), puzzle, env, 11_000_000_000) {
        Ok(Reduction(cost, _)) => println!("  {} -> OK (cost={})", label, cost),
        Err(e) => println!("  {} -> ERR: {}", label, e.1),
    }
}

#[test]
fn debug_registration_coin_env_layout() {
    let hex = include_str!("../puzzles/compiled/registration_coin.hex");
    println!("\nTesting registration_coin.rue env layouts:");

    // Layout 1: flat list (PK CKPT epoch dest amt conds)
    try_run("flat_list", hex, |a| {
        let conds = a.nil();
        let amt = a.new_atom(&[0x0F, 0x42, 0x40]).unwrap(); // 1_000_000
        let t = a.new_pair(conds, a.nil()).unwrap();
        let t = a.new_pair(amt, t).unwrap();
        let dest = a.new_atom(&[0xCC; 32]).unwrap();
        let t = a.new_pair(dest, t).unwrap();
        let epoch = a.new_atom(&[5]).unwrap();
        let t = a.new_pair(epoch, t).unwrap();
        let ckpt = a.new_atom(&[0xBB; 32]).unwrap();
        let t = a.new_pair(ckpt, t).unwrap();
        let pk = a.new_atom(&[0xAA; 48]).unwrap();
        a.new_pair(pk, t).unwrap()
    });

    // Layout 2: ((PK . CKPT) . (epoch dest amt conds))
    try_run("pair_curry", hex, |a| {
        let conds = a.nil();
        let amt = a.new_atom(&[0x0F, 0x42, 0x40]).unwrap();
        let t = a.new_pair(conds, a.nil()).unwrap();
        let t = a.new_pair(amt, t).unwrap();
        let dest = a.new_atom(&[0xCC; 32]).unwrap();
        let t = a.new_pair(dest, t).unwrap();
        let epoch = a.new_atom(&[5]).unwrap();
        let sol = a.new_pair(epoch, t).unwrap();
        let pk = a.new_atom(&[0xAA; 48]).unwrap();
        let ckpt = a.new_atom(&[0xBB; 32]).unwrap();
        let curry_pair = a.new_pair(pk, ckpt).unwrap();
        a.new_pair(curry_pair, sol).unwrap()
    });

    // Layout 3: all-params as first element: ((PK CKPT epoch dest amt conds) . ())
    try_run("all_first", hex, |a| {
        let conds = a.nil();
        let amt = a.new_atom(&[0x0F, 0x42, 0x40]).unwrap();
        let t = a.new_pair(conds, a.nil()).unwrap();
        let t = a.new_pair(amt, t).unwrap();
        let dest = a.new_atom(&[0xCC; 32]).unwrap();
        let t = a.new_pair(dest, t).unwrap();
        let epoch = a.new_atom(&[5]).unwrap();
        let t = a.new_pair(epoch, t).unwrap();
        let ckpt = a.new_atom(&[0xBB; 32]).unwrap();
        let t = a.new_pair(ckpt, t).unwrap();
        let pk = a.new_atom(&[0xAA; 48]).unwrap();
        let list = a.new_pair(pk, t).unwrap();
        let nil = a.nil();
        a.new_pair(list, nil).unwrap()
    });

    // Layout 4: Chia standard curry env: (PK . (CKPT . (epoch . (dest . (amt . (conds . ()))))))
    // Same as flat_list really — this IS the standard curry result
    // But let me try with solution as separate: (PK . (CKPT . solution))
    // where solution = (epoch dest amt conds)
    try_run("standard_curry", hex, |a| {
        let conds = a.nil();
        let amt = a.new_atom(&[0x0F, 0x42, 0x40]).unwrap();
        let nil = a.nil();
        let t = a.new_pair(conds, nil).unwrap();
        let t = a.new_pair(amt, t).unwrap();
        let dest = a.new_atom(&[0xCC; 32]).unwrap();
        let t = a.new_pair(dest, t).unwrap();
        let epoch = a.new_atom(&[5]).unwrap();
        let sol = a.new_pair(epoch, t).unwrap();
        let ckpt = a.new_atom(&[0xBB; 32]).unwrap();
        let t = a.new_pair(ckpt, sol).unwrap();
        let pk = a.new_atom(&[0xAA; 48]).unwrap();
        a.new_pair(pk, t).unwrap()
    });

    // Layout 5: Using #[clvm(list)] derive — should be same as flat_list
    {
        use clvm_traits::ToClvm;
        #[derive(ToClvm)]
        #[clvm(list)]
        struct RegParams {
            pk: Vec<u8>,
            ckpt: Vec<u8>,
            epoch: u64,
            dest: Vec<u8>,
            amt: u64,
            conds: (),
        }
        try_run("clvm_list_derive", hex, |a| {
            let params = RegParams {
                pk: vec![0xAA; 48],
                ckpt: vec![0xBB; 32],
                epoch: 5,
                dest: vec![0xCC; 32],
                amt: 1_000_000,
                conds: (),
            };
            params.to_clvm(a).unwrap()
        });
    }

    println!("Done.");
}

#[test]
fn test_path_mapping_for_reg_coin() {
    use clvmr::{Allocator, SExp};

    fn get_path(a: &Allocator, env: clvmr::NodePtr, path: u64) -> Result<clvmr::NodePtr, String> {
        if path == 1 {
            return Ok(env);
        }
        let bits = 64 - path.leading_zeros();
        let mut ops = Vec::new();
        for i in (0..bits - 1).rev() {
            ops.push(((path >> i) & 1) as u8);
        }
        let mut curr = env;
        for op in &ops {
            match a.sexp(curr) {
                SExp::Pair(l, r) => {
                    curr = if *op == 0 { l } else { r };
                }
                SExp::Atom => {
                    return Err(format!("path into atom"));
                }
            }
        }
        Ok(curr)
    }

    fn describe(a: &Allocator, node: clvmr::NodePtr) -> String {
        match a.sexp(node) {
            SExp::Atom => {
                let bytes = a.atom(node);
                format!("atom({} bytes)", bytes.len())
            }
            SExp::Pair(_, _) => "pair".to_string(),
        }
    }

    let mut a = Allocator::new();

    // Build reg coin env: (pk . (ckpt . (epoch . (dest . (amt . (conds . nil))))))
    let nil = a.nil();
    let pk = a.new_atom(&[0xAAu8; 48]).unwrap();
    let ckpt = a.new_atom(&[0xBBu8; 32]).unwrap();
    let epoch = a.new_atom(&[0x01u8]).unwrap();
    let dest = a.new_atom(&[0xCCu8; 32]).unwrap();
    let amt = a.new_atom(&[0x0F, 0x42, 0x40]).unwrap();
    let t = a.new_pair(nil, nil).unwrap();
    let t = a.new_pair(amt, t).unwrap();
    let t = a.new_pair(dest, t).unwrap();
    let t = a.new_pair(epoch, t).unwrap();
    let t = a.new_pair(ckpt, t).unwrap();
    let env = a.new_pair(pk, t).unwrap();

    for p in [1u64, 2, 3, 4, 5, 6, 7, 11, 14, 23, 30, 47, 62, 95] {
        let result = get_path(&a, env, p);
        let desc = match result {
            Ok(n) => describe(&a, n),
            Err(e) => format!("ERROR: {}", e),
        };
        eprintln!("path {}: {}", p, desc);
    }

    // Verify that path 2 = pk (48 bytes), path 6 = ckpt (32 bytes), path 14 = epoch
    let p2 = get_path(&a, env, 2).unwrap();
    let p6 = get_path(&a, env, 6).unwrap();
    let p14 = get_path(&a, env, 14).unwrap();
    eprintln!("path 2 = {}", describe(&a, p2));
    eprintln!("path 6 = {}", describe(&a, p6));
    eprintln!("path 14 = {}", describe(&a, p14));

    assert_eq!(a.atom(p2).len(), 48, "path 2 should be pk (48 bytes)");
    assert_eq!(a.atom(p6).len(), 32, "path 6 should be ckpt (32 bytes)");
}

#[test]
fn test_actual_clvm_path_5_evaluation() {
    use clvmr::reduction::Reduction;
    use clvmr::serde::node_from_bytes;

    let mut a = Allocator::new();

    // Build env: (pk . (ckpt . (epoch . (dest . (amt . (nil . nil))))))
    let nil = a.nil();
    let pk = a.new_atom(&[0xAAu8; 48]).unwrap();
    let ckpt = a.new_atom(&[0xBBu8; 32]).unwrap();
    let epoch = a.new_atom(&[0x01u8]).unwrap();
    let dest = a.new_atom(&[0xCCu8; 32]).unwrap();
    let amt = a.new_atom(&[0x0F, 0x42, 0x40]).unwrap();
    let t = a.new_pair(nil, nil).unwrap();
    let t = a.new_pair(amt, t).unwrap();
    let t = a.new_pair(dest, t).unwrap();
    let t = a.new_pair(epoch, t).unwrap();
    let t = a.new_pair(ckpt, t).unwrap();
    let env = a.new_pair(pk, t).unwrap();

    // Program: (q . 5) = return path 5
    // In CLVM: (1 . 5) = quote 5... no wait, (q . 5) = constant atom 5
    // We want: just `5` = path reference 5
    // Prog = the atom `\x05` = path 5 as program
    let prog_path5 = a.new_atom(&[5u8]).unwrap();
    let prog_path6 = a.new_atom(&[6u8]).unwrap();
    let prog_path2 = a.new_atom(&[2u8]).unwrap();

    eprintln!("\nEvaluating paths directly with reg_coin env:");

    let result2 = run_program(&mut a, &ChiaDialect::new(0), prog_path2, env, 1000);
    match result2 {
        Ok(Reduction(_, n)) => {
            eprintln!(
                "path 2 = atom({} bytes), first bytes: {:02x?}",
                a.atom(n).len(),
                &a.atom(n)[..4.min(a.atom(n).len())]
            );
        }
        Err(e) => eprintln!("path 2 = ERR: {}", e.1),
    }

    let result5 = run_program(&mut a, &ChiaDialect::new(0), prog_path5, env, 1000);
    match result5 {
        Ok(Reduction(_, n)) => {
            eprintln!(
                "path 5 = atom({} bytes), first bytes: {:02x?}",
                a.atom(n).len(),
                &a.atom(n)[..4.min(a.atom(n).len())]
            );
        }
        Err(e) => eprintln!("path 5 = ERR: {}", e.1),
    }

    let result6 = run_program(&mut a, &ChiaDialect::new(0), prog_path6, env, 1000);
    match result6 {
        Ok(Reduction(_, n)) => {
            eprintln!(
                "path 6 = atom({} bytes), first bytes: {:02x?}",
                a.atom(n).len(),
                &a.atom(n)[..4.min(a.atom(n).len())]
            );
        }
        Err(e) => eprintln!("path 6 = ERR: {}", e.1),
    }
}
