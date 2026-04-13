//! CLVM execution helpers for puzzle testing.
//!
//! Provides utilities to:
//! - Load compiled puzzle hex into a CLVM allocator
//! - Curry puzzles with parameters
//! - Run puzzles with solutions
//! - Parse output conditions

use clvm_traits::ToClvm;
use clvm_utils::CurriedProgram;
use clvmr::reduction::Reduction;
use clvmr::serde::node_from_bytes;
use clvmr::{run_program, Allocator, ChiaDialect, NodePtr};

/// Maximum CLVM cost for test execution (full block limit).
pub const MAX_COST: u64 = 11_000_000_000;

/// CLVM condition opcodes.
pub const CREATE_COIN: u8 = 51;
pub const AGG_SIG_ME: u8 = 50;
pub const ASSERT_COIN_ANNOUNCEMENT: u8 = 61;
pub const CREATE_COIN_ANNOUNCEMENT: u8 = 60;

/// Load a compiled puzzle from its hex string into the allocator.
///
/// The hex string comes from `include_str!("../puzzles/compiled/{name}.hex")`.
pub fn load_puzzle(a: &mut Allocator, hex_str: &str) -> NodePtr {
    let bytes = hex::decode(hex_str.trim()).expect("Invalid puzzle hex");
    node_from_bytes(a, &bytes).expect("Failed to deserialize puzzle CLVM")
}

/// Curry a puzzle with arguments using CurriedProgram.
///
/// `args` must implement `ToClvm`. Use a struct with `#[derive(ToClvm)]`
/// and `#[clvm(curry)]` attribute.
pub fn curry_puzzle<A: ToClvm<Allocator>>(a: &mut Allocator, puzzle: NodePtr, args: A) -> NodePtr {
    let curried = CurriedProgram {
        program: puzzle,
        args,
    };
    curried.to_clvm(a).expect("Failed to curry puzzle")
}

/// Run a curried puzzle with a solution and return (cost, output).
pub fn run_puzzle(
    a: &mut Allocator,
    puzzle: NodePtr,
    solution: NodePtr,
) -> Result<(u64, NodePtr), clvmr::reduction::EvalErr> {
    let Reduction(cost, output) = run_program(a, &ChiaDialect::new(0), puzzle, solution, MAX_COST)?;
    Ok((cost, output))
}

/// Run a curried puzzle with a solution, asserting success.
pub fn run_puzzle_ok(a: &mut Allocator, puzzle: NodePtr, solution: NodePtr) -> (u64, NodePtr) {
    run_puzzle(a, puzzle, solution).expect("CLVM execution failed")
}

/// A parsed CLVM condition.
#[derive(Debug, Clone)]
pub struct ParsedCondition {
    pub opcode: i64,
    pub args: Vec<Vec<u8>>,
}

/// Parse the output of a puzzle run into a list of conditions.
///
/// CLVM conditions are a list of (opcode arg1 arg2 ...) pairs.
pub fn parse_conditions(a: &Allocator, output: NodePtr) -> Vec<ParsedCondition> {
    let mut conditions = Vec::new();
    let mut current = output;

    loop {
        match a.sexp(current) {
            clvmr::SExp::Pair(first, rest) => {
                if let Some(cond) = parse_single_condition(a, first) {
                    conditions.push(cond);
                }
                current = rest;
            }
            clvmr::SExp::Atom => break, // nil = end of list
        }
    }

    conditions
}

fn parse_single_condition(a: &Allocator, node: NodePtr) -> Option<ParsedCondition> {
    match a.sexp(node) {
        clvmr::SExp::Pair(opcode_node, args_node) => {
            let opcode_bytes = a.atom(opcode_node);
            let opcode = match opcode_bytes.as_ref().len() {
                0 => 0i64,
                1 => opcode_bytes.as_ref()[0] as i64,
                2 => i16::from_be_bytes(opcode_bytes.as_ref().try_into().ok()?) as i64,
                _ => return None,
            };

            let mut args = Vec::new();
            let mut cur = args_node;
            loop {
                match a.sexp(cur) {
                    clvmr::SExp::Pair(arg, rest) => {
                        if let clvmr::SExp::Atom = a.sexp(arg) {
                            args.push(a.atom(arg).as_ref().to_vec());
                        } else {
                            // Nested pair - serialize as bytes
                            args.push(Vec::new());
                        }
                        cur = rest;
                    }
                    clvmr::SExp::Atom => {
                        break;
                    }
                }
            }

            Some(ParsedCondition { opcode, args })
        }
        clvmr::SExp::Atom => None,
    }
}

/// Check if conditions contain a specific opcode.
pub fn has_opcode(conditions: &[ParsedCondition], opcode: u8) -> bool {
    conditions.iter().any(|c| c.opcode == opcode as i64)
}

/// Get all conditions with a specific opcode.
pub fn conditions_with_opcode(conditions: &[ParsedCondition], opcode: u8) -> Vec<&ParsedCondition> {
    conditions
        .iter()
        .filter(|c| c.opcode == opcode as i64)
        .collect()
}

/// Build a CLVM nil node.
pub fn nil(a: &mut Allocator) -> NodePtr {
    a.nil()
}

/// Build a CLVM list from a slice of NodePtrs.
pub fn make_list(a: &mut Allocator, items: &[NodePtr]) -> NodePtr {
    let mut result = a.nil();
    for item in items.iter().rev() {
        result = a.new_pair(*item, result).expect("Failed to cons");
    }
    result
}

/// Encode a u64 as a CLVM integer atom (signed big-endian).
///
/// CLVM integers are SIGNED. If the minimal big-endian encoding has MSB set
/// (e.g., 128 = 0x80), a 0x00 prefix must be added to keep it positive.
/// Without this, 0x80 is interpreted as -128.
pub fn u64_to_clvm(a: &mut Allocator, val: u64) -> NodePtr {
    if val == 0 {
        return a.nil();
    }
    let bytes = val.to_be_bytes();
    let stripped: Vec<u8> = bytes.iter().copied().skip_while(|&b| b == 0).collect();
    // Add 0x00 prefix if MSB is set (would be interpreted as negative)
    let atom_bytes = if stripped[0] & 0x80 != 0 {
        let mut with_sign = vec![0x00];
        with_sign.extend_from_slice(&stripped);
        with_sign
    } else {
        stripped
    };
    a.new_atom(&atom_bytes).unwrap()
}

/// Build a Rue struct as a nil-terminated CLVM proper list.
/// Rue structs: (f1 . (f2 . (f3 . (f4 . nil)))) — proper list with nil terminator.
pub fn build_struct(a: &mut Allocator, fields: &[NodePtr]) -> NodePtr {
    assert!(!fields.is_empty());
    let mut result = a.nil();
    for i in (0..fields.len()).rev() {
        result = a.new_pair(fields[i], result).unwrap();
    }
    result
}

/// Build checkpoint inner puzzle env for MEMBERSHIP QUERY path.
/// Uses the new struct-based layout matching checkpoint_inner.rue.
///
/// Parameter order: INNER_MOD_HASH, VK{4}, IC{7}, TREE_DEPTH, EMPTY_LEAF_HASH,
/// STATE{4}, is_checkpoint, Proof{3}, new_sr, new_vmr, new_vc, agg_signers,
/// agg_sig, Scalars{6}, query_pubkey, leaf_index, siblings, is_member, conditions
pub fn build_checkpoint_membership_query_env(
    a: &mut Allocator,
    inner_mod_hash: &[u8],
    // VK: we need 4 fields for the struct (alpha, beta, gamma, delta)
    vk_alpha: &[u8],
    vk_beta: &[u8],
    vk_gamma: &[u8],
    vk_delta: &[u8],
    // IC: 7 fields
    ic: &[&[u8]; 7],
    tree_depth: u64,
    empty_leaf_hash: &[u8],
    // State: 4 fields
    state_root: &[u8],
    epoch: u64,
    validator_merkle_root: &[u8],
    validator_count: u64,
    // Query fields
    query_pubkey: &[u8],
    leaf_index: u64,
    siblings: &[[u8; 32]],
    is_member: bool,
) -> NodePtr {
    // Build right-to-left
    let nil = a.nil();
    let conds = a.nil();
    let t = a.new_pair(conds, nil).unwrap();

    let is_member_n = if is_member {
        a.new_atom(&[1]).unwrap()
    } else {
        a.nil()
    };
    let t = a.new_pair(is_member_n, t).unwrap();

    // Siblings as cons list
    let mut sib_list = a.nil();
    for s in siblings.iter().rev() {
        let sn = a.new_atom(s).unwrap();
        sib_list = a.new_pair(sn, sib_list).unwrap();
    }
    let t = a.new_pair(sib_list, t).unwrap();

    let li = u64_to_clvm(a, leaf_index);
    let t = a.new_pair(li, t).unwrap();
    let pk = a.new_atom(query_pubkey).unwrap();
    let t = a.new_pair(pk, t).unwrap();

    // Scalars struct (unused in query — all zeros)
    let z32 = [0u8; 32];
    let s_nodes: Vec<_> = (0..6).map(|_| a.new_atom(&z32).unwrap()).collect();
    let scalars_struct = build_struct(a, &s_nodes);
    let t = a.new_pair(scalars_struct, t).unwrap();

    // agg_sig (unused), agg_signers (unused)
    let as_n = a.new_atom(&[0u8; 96]).unwrap();
    let t = a.new_pair(as_n, t).unwrap();
    let asig_n = a.new_atom(&[0u8; 48]).unwrap();
    let t = a.new_pair(asig_n, t).unwrap();

    // new_validator_count, new_validator_merkle_root, new_state_root (unused)
    let nvc = a.nil();
    let t = a.new_pair(nvc, t).unwrap();
    let nmr = a.new_atom(&[0u8; 32]).unwrap();
    let t = a.new_pair(nmr, t).unwrap();
    let nsr = a.new_atom(&[0u8; 32]).unwrap();
    let t = a.new_pair(nsr, t).unwrap();

    // Proof struct (unused): (a . (b . c))
    let pa = a.new_atom(&[0u8; 48]).unwrap();
    let pb = a.new_atom(&[0u8; 96]).unwrap();
    let pc = a.new_atom(&[0u8; 48]).unwrap();
    let proof_struct = build_struct(a, &[pa, pb, pc]);
    let t = a.new_pair(proof_struct, t).unwrap();

    // is_checkpoint = false
    let t = a.new_pair(a.nil(), t).unwrap();

    // State struct: (state_root . (epoch . (validator_merkle_root . validator_count)))
    let sr = a.new_atom(state_root).unwrap();
    let ep = u64_to_clvm(a, epoch);
    let vmr = a.new_atom(validator_merkle_root).unwrap();
    let vc = u64_to_clvm(a, validator_count);
    let state_struct = build_struct(a, &[sr, ep, vmr, vc]);
    let t = a.new_pair(state_struct, t).unwrap();

    // EMPTY_LEAF_HASH, TREE_DEPTH
    let elh = a.new_atom(empty_leaf_hash).unwrap();
    let t = a.new_pair(elh, t).unwrap();
    let td = u64_to_clvm(a, tree_depth);
    let t = a.new_pair(td, t).unwrap();

    // IC struct: (ic0 . (ic1 . (ic2 . (ic3 . (ic4 . (ic5 . ic6))))))
    let ic_nodes: Vec<_> = ic.iter().map(|p| a.new_atom(p).unwrap()).collect();
    let ic_struct = build_struct(a, &ic_nodes);
    let t = a.new_pair(ic_struct, t).unwrap();

    // VK struct: (alpha . (beta . (gamma . delta)))
    let va = a.new_atom(vk_alpha).unwrap();
    let vb = a.new_atom(vk_beta).unwrap();
    let vg = a.new_atom(vk_gamma).unwrap();
    let vd = a.new_atom(vk_delta).unwrap();
    let vk_struct = build_struct(a, &[va, vb, vg, vd]);
    let t = a.new_pair(vk_struct, t).unwrap();

    // INNER_MOD_HASH
    let imh = a.new_atom(inner_mod_hash).unwrap();
    a.new_pair(imh, t).unwrap()
}

/// Build registration coin env as flat list (WDC-004: 4 curried + 3 solution).
/// (PK . (CKPT_ID . (WDC_MOD . (WDC_DELAY . (epoch . (dest . (amt . nil)))))))
///
/// All parameters passed as single right-linked list — Rue-compiled puzzles
/// expect this layout. Integers use CLVM signed encoding via `u64_to_clvm`.
/// Uses default WDC params ([0x55;32], 24000) for backward-compatible call sites.
pub fn build_reg_coin_env(
    a: &mut Allocator,
    pk: &[u8],
    ckpt_id: &[u8],
    epoch: u64,
    dest: &[u8],
    amt: u64,
) -> NodePtr {
    let nil = a.nil();
    let amt_n = u64_to_clvm(a, amt);
    let t = a.new_pair(amt_n, nil).unwrap();
    let dest_n = a.new_atom(dest).unwrap();
    let t = a.new_pair(dest_n, t).unwrap();
    let epoch_n = u64_to_clvm(a, epoch);
    let t = a.new_pair(epoch_n, t).unwrap();
    // WDC-004: default withdraw delay params
    let delay_n = u64_to_clvm(a, 24_000);
    let t = a.new_pair(delay_n, t).unwrap();
    let wdc_mod_n = a.new_atom(&[0x55u8; 32]).unwrap();
    let t = a.new_pair(wdc_mod_n, t).unwrap();
    let ckpt_n = a.new_atom(ckpt_id).unwrap();
    let t = a.new_pair(ckpt_n, t).unwrap();
    let pk_n = a.new_atom(pk).unwrap();
    a.new_pair(pk_n, t).unwrap()
}
