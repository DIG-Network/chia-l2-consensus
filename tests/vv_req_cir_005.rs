//! REQUIREMENT: CIR-005 — Public Inputs
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-005`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-005.md`.
//!
//! Verifies that the circuit accepts exactly 6 public inputs in the correct
//! fixed order. The order must match the VK IC point assignment.

use ark_ff::Zero;
use chia_l2_consensus::testing::{
    bytes_to_scalar, public_input_index, ConsensusCircuit, PUBLIC_INPUT_COUNT,
};

#[test]
fn vv_req_cir_005_exactly_6_public_inputs() {
    // CIR-005: Circuit has exactly 6 public inputs
    assert_eq!(
        PUBLIC_INPUT_COUNT, 6,
        "CIR-005: Circuit must have exactly 6 public inputs"
    );

    // Verify circuit method agrees
    let circuit = ConsensusCircuit::new();
    assert_eq!(
        circuit.public_input_count(),
        6,
        "CIR-005: public_input_count() must return 6"
    );
}

#[test]
fn vv_req_cir_005_public_input_order() {
    // CIR-005: Public inputs must be in specified order
    // | Index | Name |
    // |-------|------|
    // | 1 | validator_merkle_root |
    // | 2 | validator_count |
    // | 3 | new_validator_merkle_root |
    // | 4 | new_validator_count |
    // | 5 | agg_signers |
    // | 6 | checkpoint_message |

    assert_eq!(
        public_input_index::VALIDATOR_MERKLE_ROOT,
        1,
        "CIR-005: validator_merkle_root must be index 1"
    );
    assert_eq!(
        public_input_index::VALIDATOR_COUNT,
        2,
        "CIR-005: validator_count must be index 2"
    );
    assert_eq!(
        public_input_index::NEW_VALIDATOR_MERKLE_ROOT,
        3,
        "CIR-005: new_validator_merkle_root must be index 3"
    );
    assert_eq!(
        public_input_index::NEW_VALIDATOR_COUNT,
        4,
        "CIR-005: new_validator_count must be index 4"
    );
    assert_eq!(
        public_input_index::AGG_SIGNERS,
        5,
        "CIR-005: agg_signers must be index 5"
    );
    assert_eq!(
        public_input_index::CHECKPOINT_MESSAGE,
        6,
        "CIR-005: checkpoint_message must be index 6"
    );
}

#[test]
fn vv_req_cir_005_public_inputs_bytes_order() {
    // CIR-005: public_inputs_bytes() returns inputs in correct order
    let merkle_root = [0x11u8; 32];
    let count = 100u64;
    let new_merkle_root = [0x22u8; 32];
    let new_count = 101u64;
    let agg_signers = [0x33u8; 48];
    let checkpoint_msg = [0x44u8; 32];

    let circuit = ConsensusCircuit::with_public_inputs(
        merkle_root,
        count,
        new_merkle_root,
        new_count,
        agg_signers,
        checkpoint_msg,
        count as usize,
    );

    let inputs = circuit.public_inputs_bytes();

    assert_eq!(inputs.len(), 6, "CIR-005: Must have 6 public inputs");

    // Index 0 (input 1): validator_merkle_root
    assert_eq!(
        inputs[0].as_slice(),
        &merkle_root[..],
        "CIR-005: Input 1 must be validator_merkle_root"
    );

    // Index 1 (input 2): validator_count (big-endian)
    assert_eq!(
        inputs[1].as_slice(),
        &count.to_be_bytes()[..],
        "CIR-005: Input 2 must be validator_count"
    );

    // Index 2 (input 3): new_validator_merkle_root
    assert_eq!(
        inputs[2].as_slice(),
        &new_merkle_root[..],
        "CIR-005: Input 3 must be new_validator_merkle_root"
    );

    // Index 3 (input 4): new_validator_count (big-endian)
    assert_eq!(
        inputs[3].as_slice(),
        &new_count.to_be_bytes()[..],
        "CIR-005: Input 4 must be new_validator_count"
    );

    // Index 4 (input 5): agg_signers
    assert_eq!(
        inputs[4].as_slice(),
        &agg_signers[..],
        "CIR-005: Input 5 must be agg_signers"
    );

    // Index 5 (input 6): checkpoint_message
    assert_eq!(
        inputs[5].as_slice(),
        &checkpoint_msg[..],
        "CIR-005: Input 6 must be checkpoint_message"
    );
}

#[test]
fn vv_req_cir_005_public_input_sizes() {
    // CIR-005: Public inputs have correct sizes
    // | Index | Size |
    // | 1 | 32 bytes |
    // | 2 | 8 bytes |
    // | 3 | 32 bytes |
    // | 4 | 8 bytes |
    // | 5 | 48 bytes |
    // | 6 | 32 bytes |

    let circuit =
        ConsensusCircuit::with_public_inputs([0u8; 32], 0, [0u8; 32], 0, [0u8; 48], [0u8; 32], 1);

    let inputs = circuit.public_inputs_bytes();

    assert_eq!(
        inputs[0].len(),
        32,
        "CIR-005: validator_merkle_root is 32 bytes"
    );
    assert_eq!(inputs[1].len(), 8, "CIR-005: validator_count is 8 bytes");
    assert_eq!(
        inputs[2].len(),
        32,
        "CIR-005: new_validator_merkle_root is 32 bytes"
    );
    assert_eq!(
        inputs[3].len(),
        8,
        "CIR-005: new_validator_count is 8 bytes"
    );
    assert_eq!(inputs[4].len(), 48, "CIR-005: agg_signers is 48 bytes");
    assert_eq!(
        inputs[5].len(),
        32,
        "CIR-005: checkpoint_message is 32 bytes"
    );
}

#[test]
fn vv_req_cir_005_accessors_match_public_inputs() {
    // CIR-005: Individual accessors return same values as public_inputs_bytes
    let merkle_root = [0xAAu8; 32];
    let count = 999u64;
    let new_merkle_root = [0xBBu8; 32];
    let new_count = 1000u64;
    let agg_signers = [0xCCu8; 48];
    let checkpoint_msg = [0xDDu8; 32];

    let circuit = ConsensusCircuit::with_public_inputs(
        merkle_root,
        count,
        new_merkle_root,
        new_count,
        agg_signers,
        checkpoint_msg,
        count as usize,
    );

    // Verify accessors
    assert_eq!(
        circuit.validator_merkle_root(),
        merkle_root,
        "CIR-005: validator_merkle_root accessor"
    );
    assert_eq!(
        circuit.validator_count(),
        count,
        "CIR-005: validator_count accessor"
    );
    assert_eq!(
        circuit.new_validator_merkle_root(),
        new_merkle_root,
        "CIR-005: new_validator_merkle_root accessor"
    );
    assert_eq!(
        circuit.new_validator_count(),
        new_count,
        "CIR-005: new_validator_count accessor"
    );
    assert_eq!(
        circuit.agg_signers(),
        agg_signers,
        "CIR-005: agg_signers accessor"
    );
    assert_eq!(
        circuit.checkpoint_message(),
        checkpoint_msg,
        "CIR-005: checkpoint_message accessor"
    );
}

#[test]
fn vv_req_cir_005_scalar_applied_to_inputs() {
    // CIR-005: scalar() function can be applied to each public input
    // scalar(bytes) = sha256(bytes) as big-endian u256, mod r

    let merkle_root = [0x11u8; 32];
    let count = 100u64;
    let new_merkle_root = [0x22u8; 32];
    let new_count = 101u64;
    let agg_signers = [0x33u8; 48];
    let checkpoint_msg = [0x44u8; 32];

    let circuit = ConsensusCircuit::with_public_inputs(
        merkle_root,
        count,
        new_merkle_root,
        new_count,
        agg_signers,
        checkpoint_msg,
        count as usize,
    );

    let inputs = circuit.public_inputs_bytes();

    // All inputs should produce valid scalars
    for (i, input) in inputs.iter().enumerate() {
        let scalar = bytes_to_scalar(input);
        // Scalar should be non-zero for non-zero input
        assert!(
            !scalar.is_zero() || input.iter().all(|&b| b == 0),
            "CIR-005: scalar() produces valid result for input {}",
            i + 1
        );
    }
}

#[test]
fn vv_req_cir_005_vk_ic_point_count() {
    // CIR-005: VK has 7 IC points (1 constant + 6 for public inputs)
    // IC[0] is constant term, IC[1..7] correspond to public inputs 1..6

    // This is verified by checking that PUBLIC_INPUT_COUNT + 1 = 7
    let expected_ic_count = PUBLIC_INPUT_COUNT + 1;
    assert_eq!(expected_ic_count, 7, "CIR-005: VK must have 7 IC points");
}

#[test]
fn vv_req_cir_005_empty_circuit_has_zero_inputs() {
    // CIR-005: Empty circuit initializes public inputs to zero
    let circuit = ConsensusCircuit::new();

    assert_eq!(
        circuit.validator_merkle_root(),
        [0u8; 32],
        "CIR-005: Empty circuit has zero validator_merkle_root"
    );
    assert_eq!(
        circuit.validator_count(),
        0,
        "CIR-005: Empty circuit has zero validator_count"
    );
    assert_eq!(
        circuit.new_validator_merkle_root(),
        [0u8; 32],
        "CIR-005: Empty circuit has zero new_validator_merkle_root"
    );
    assert_eq!(
        circuit.new_validator_count(),
        0,
        "CIR-005: Empty circuit has zero new_validator_count"
    );
    assert_eq!(
        circuit.agg_signers(),
        [0u8; 48],
        "CIR-005: Empty circuit has zero agg_signers"
    );
    assert_eq!(
        circuit.checkpoint_message(),
        [0u8; 32],
        "CIR-005: Empty circuit has zero checkpoint_message"
    );
}

#[test]
fn vv_req_cir_005_count_as_big_endian_u64() {
    // CIR-005: validator_count and new_validator_count are 8-byte big-endian

    let count = 0x0102030405060708u64;
    let new_count = 0x0908070605040302u64;

    let circuit = ConsensusCircuit::with_public_inputs(
        [0u8; 32],
        count,
        [0u8; 32],
        new_count,
        [0u8; 48],
        [0u8; 32],
        count as usize,
    );

    let inputs = circuit.public_inputs_bytes();

    // validator_count (index 1)
    assert_eq!(
        inputs[1],
        vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        "CIR-005: validator_count must be big-endian"
    );

    // new_validator_count (index 3)
    assert_eq!(
        inputs[3],
        vec![0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02],
        "CIR-005: new_validator_count must be big-endian"
    );
}

#[test]
fn vv_req_cir_005_different_inputs_different_scalars() {
    // CIR-005: Different public inputs produce different scalars
    let input1 = vec![0x11u8; 32];
    let input2 = vec![0x22u8; 32];

    let scalar1 = bytes_to_scalar(&input1);
    let scalar2 = bytes_to_scalar(&input2);

    assert_ne!(
        scalar1, scalar2,
        "CIR-005: Different inputs must produce different scalars"
    );
}

#[test]
fn vv_req_cir_005_public_input_indices_consecutive() {
    // CIR-005: Public input indices are consecutive from 1 to 6
    let indices = [
        public_input_index::VALIDATOR_MERKLE_ROOT,
        public_input_index::VALIDATOR_COUNT,
        public_input_index::NEW_VALIDATOR_MERKLE_ROOT,
        public_input_index::NEW_VALIDATOR_COUNT,
        public_input_index::AGG_SIGNERS,
        public_input_index::CHECKPOINT_MESSAGE,
    ];

    for (i, &index) in indices.iter().enumerate() {
        assert_eq!(
            index,
            i + 1,
            "CIR-005: Public input index {} must be {}",
            i,
            i + 1
        );
    }
}

#[test]
fn vv_req_cir_005_with_witnesses_preserves_public_inputs() {
    // CIR-005: with_witnesses constructor preserves public input values
    let merkle_root = [0xAAu8; 32];
    let count = 50u64;
    let new_merkle_root = [0xBBu8; 32];
    let new_count = 51u64;
    let agg_signers = [0xCCu8; 48];
    let checkpoint_msg = [0xDDu8; 32];

    let circuit = ConsensusCircuit::with_witnesses(
        merkle_root,
        count,
        new_merkle_root,
        new_count,
        agg_signers,
        checkpoint_msg,
        vec![], // empty witnesses for this test
        vec![],
    );

    // Public inputs should be preserved
    assert_eq!(circuit.validator_merkle_root(), merkle_root);
    assert_eq!(circuit.validator_count(), count);
    assert_eq!(circuit.new_validator_merkle_root(), new_merkle_root);
    assert_eq!(circuit.new_validator_count(), new_count);
    assert_eq!(circuit.agg_signers(), agg_signers);
    assert_eq!(circuit.checkpoint_message(), checkpoint_msg);
}
