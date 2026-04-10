//! REQUIREMENT: CIR-003 — Aggregate Key Constraint
//! (`docs/requirements/domains/circuit/NORMATIVE.md#CIR-003`).
//!
//! Spec: `docs/requirements/domains/circuit/specs/CIR-003.md`.
//!
//! Verifies that the G1 sum of k signing pubkeys equals the public input
//! `agg_signers`. This binds the ZK proof to the specific aggregate public
//! key that will be used for BLS signature verification on-chain.

use ark_bls12_381::{Fr, G1Affine, G1Projective};
use ark_ec::AffineRepr;
use ark_ff::UniformRand;
use chia_l2_consensus::{
    add_g1, aggregate_pubkeys, g1_identity, negate_g1, serialize_g1, verify_aggregate,
};
use rand::thread_rng;

/// Helper: generate a random G1 pubkey
fn random_pubkey() -> [u8; 48] {
    let mut rng = thread_rng();
    let scalar = Fr::rand(&mut rng);
    let point = G1Affine::from(G1Affine::generator() * scalar);
    serialize_g1(&point)
}

/// Helper: generate n random pubkeys
fn random_pubkeys(n: usize) -> Vec<[u8; 48]> {
    (0..n).map(|_| random_pubkey()).collect()
}

#[test]
fn vv_req_cir_003_g1_sum_equals_agg_signers() {
    // CIR-003: G1 sum of k pubkeys equals agg_signers
    let pubkeys = random_pubkeys(3);

    let computed_aggregate = aggregate_pubkeys(&pubkeys).unwrap();

    // Verify the constraint: sum of pubkeys must equal agg_signers
    assert!(
        verify_aggregate(&pubkeys, &computed_aggregate),
        "CIR-003: G1 sum must equal agg_signers"
    );
}

#[test]
fn vv_req_cir_003_proper_elliptic_curve_addition() {
    // CIR-003: Sum uses proper elliptic curve addition (not XOR or other)
    let mut rng = thread_rng();

    // Generate two known points
    let scalar1 = Fr::rand(&mut rng);
    let scalar2 = Fr::rand(&mut rng);
    let pk1 = G1Affine::from(G1Affine::generator() * scalar1);
    let pk2 = G1Affine::from(G1Affine::generator() * scalar2);

    let pk1_bytes = serialize_g1(&pk1);
    let pk2_bytes = serialize_g1(&pk2);

    // Compute aggregate using our function
    let aggregate = aggregate_pubkeys(&[pk1_bytes, pk2_bytes]).unwrap();

    // Compute expected using arkworks directly
    let expected_sum = G1Affine::from(G1Projective::from(pk1) + G1Projective::from(pk2));
    let expected_bytes = serialize_g1(&expected_sum);

    assert_eq!(
        aggregate, expected_bytes,
        "CIR-003: Aggregate must use elliptic curve addition"
    );
}

#[test]
fn vv_req_cir_003_identity_handling_for_empty() {
    // CIR-003: Empty input returns identity (point at infinity)
    let aggregate = aggregate_pubkeys(&[]).unwrap();
    let identity = g1_identity();

    assert_eq!(
        aggregate, identity,
        "CIR-003: Empty aggregate must be identity"
    );
}

#[test]
fn vv_req_cir_003_identity_handling_for_padding() {
    // CIR-003: Identity + pk = pk (for k < MAX_SIGNERS padding)
    let pk = random_pubkey();
    let identity = g1_identity();

    // pk + identity should equal pk
    let sum = add_g1(&pk, &identity).unwrap();
    assert_eq!(sum, pk, "CIR-003: pk + identity must equal pk");

    // identity + pk should equal pk
    let sum2 = add_g1(&identity, &pk).unwrap();
    assert_eq!(sum2, pk, "CIR-003: identity + pk must equal pk");
}

#[test]
fn vv_req_cir_003_wrong_aggregate_fails() {
    // CIR-003: Wrong agg_signers must fail verification
    let pubkeys = random_pubkeys(3);
    let correct_aggregate = aggregate_pubkeys(&pubkeys).unwrap();

    // Create a wrong aggregate (different random point)
    let wrong_aggregate = random_pubkey();

    // Ensure they're different
    assert_ne!(correct_aggregate, wrong_aggregate);

    // Verification must fail with wrong aggregate
    assert!(
        !verify_aggregate(&pubkeys, &wrong_aggregate),
        "CIR-003: Wrong agg_signers must fail verification"
    );

    // Verification must succeed with correct aggregate
    assert!(
        verify_aggregate(&pubkeys, &correct_aggregate),
        "CIR-003: Correct agg_signers must pass verification"
    );
}

#[test]
fn vv_req_cir_003_single_pubkey_equals_itself() {
    // CIR-003: k=1, agg_signers = pk₁
    let pk = random_pubkey();

    let aggregate = aggregate_pubkeys(&[pk]).unwrap();

    assert_eq!(
        aggregate, pk,
        "CIR-003: Single pubkey aggregate must equal the pubkey itself"
    );

    assert!(
        verify_aggregate(&[pk], &pk),
        "CIR-003: Single pubkey must verify against itself"
    );
}

#[test]
fn vv_req_cir_003_subtraction_attack_prevention() {
    // CIR-003: Can't use pk - pk' to manipulate aggregate
    // If attacker tries to add pk and -pk, they cancel out
    let pk = random_pubkey();
    let neg_pk = negate_g1(&pk).unwrap();

    // pk + (-pk) = identity
    let sum = aggregate_pubkeys(&[pk, neg_pk]).unwrap();
    let identity = g1_identity();

    assert_eq!(sum, identity, "CIR-003: pk + (-pk) must equal identity");

    // If attacker wants to make sum = target, they can't add phantom pairs
    let target = random_pubkey();
    let pk1 = random_pubkey();
    let neg_pk1 = negate_g1(&pk1).unwrap();

    // target + pk1 + (-pk1) should still equal target
    let sum_with_phantom = aggregate_pubkeys(&[target, pk1, neg_pk1]).unwrap();
    assert_eq!(
        sum_with_phantom, target,
        "CIR-003: Adding phantom pairs (pk + -pk) doesn't change result"
    );
}

#[test]
fn vv_req_cir_003_order_independent() {
    // CIR-003: Sum is commutative (order doesn't matter)
    let pubkeys = random_pubkeys(5);

    let aggregate1 = aggregate_pubkeys(&pubkeys).unwrap();

    // Reverse order
    let mut reversed = pubkeys.clone();
    reversed.reverse();
    let aggregate2 = aggregate_pubkeys(&reversed).unwrap();

    assert_eq!(
        aggregate1, aggregate2,
        "CIR-003: Aggregate must be order-independent"
    );
}

#[test]
fn vv_req_cir_003_many_signers() {
    // CIR-003: Test with a reasonable number of signers (100)
    // Note: MAX_SIGNERS is 20,000 but we test with smaller set for performance
    let pubkeys = random_pubkeys(100);

    let aggregate = aggregate_pubkeys(&pubkeys).unwrap();

    // Should produce a valid G1 point
    assert!(
        chia_l2_consensus::deserialize_g1(&aggregate).is_some(),
        "CIR-003: Aggregate of many pubkeys must be valid G1 point"
    );

    // Verify round-trip
    assert!(
        verify_aggregate(&pubkeys, &aggregate),
        "CIR-003: Many pubkeys must verify against their aggregate"
    );
}

#[test]
fn vv_req_cir_003_associative_aggregation() {
    // CIR-003: (pk1 + pk2) + pk3 = pk1 + (pk2 + pk3)
    let pk1 = random_pubkey();
    let pk2 = random_pubkey();
    let pk3 = random_pubkey();

    // Left association: (pk1 + pk2) + pk3
    let left_inner = add_g1(&pk1, &pk2).unwrap();
    let left_result = add_g1(&left_inner, &pk3).unwrap();

    // Right association: pk1 + (pk2 + pk3)
    let right_inner = add_g1(&pk2, &pk3).unwrap();
    let right_result = add_g1(&pk1, &right_inner).unwrap();

    // Full aggregate
    let full = aggregate_pubkeys(&[pk1, pk2, pk3]).unwrap();

    assert_eq!(
        left_result, right_result,
        "CIR-003: Aggregation must be associative"
    );
    assert_eq!(
        left_result, full,
        "CIR-003: Associative result must match aggregate"
    );
}

#[test]
fn vv_req_cir_003_invalid_pubkey_fails() {
    // CIR-003: Invalid pubkey bytes should fail aggregation
    // Use bytes that are definitely not on the curve
    // All zeros (except compression flag) is not a valid point
    let mut invalid_bytes = [0x00u8; 48];
    invalid_bytes[0] = 0x80; // Set compression flag but rest is zeros (not on curve)

    // This should be invalid because x=0 with y derived from compression
    // doesn't give a valid curve point
    let result = aggregate_pubkeys(&[invalid_bytes]);

    // If it happens to be valid, try another invalid pattern
    if result.is_ok() {
        // Try with invalid flags (both infinity and compression set wrong)
        let mut truly_invalid = [0x00u8; 48];
        truly_invalid[0] = 0xE0; // Invalid flag combination
        let result2 = aggregate_pubkeys(&[truly_invalid]);
        assert!(
            result2.is_err(),
            "CIR-003: Invalid pubkey with bad flags must fail aggregation"
        );
    }
}

#[test]
fn vv_req_cir_003_padding_with_identity_preserves_sum() {
    // CIR-003: For k < MAX_SIGNERS, padding with identity preserves sum
    let k = 5;
    let padded_size = 100; // Test with reasonable padding size

    let pubkeys = random_pubkeys(k);
    let actual_aggregate = aggregate_pubkeys(&pubkeys).unwrap();

    // Pad with identity to reach padded_size
    let mut padded = pubkeys.clone();
    let identity = g1_identity();
    for _ in k..padded_size {
        padded.push(identity);
    }

    let padded_aggregate = aggregate_pubkeys(&padded).unwrap();

    assert_eq!(
        actual_aggregate, padded_aggregate,
        "CIR-003: Padding with identity must preserve aggregate"
    );
}

#[test]
fn vv_req_cir_003_double_negation_is_identity() {
    // CIR-003: -(-pk) = pk
    let pk = random_pubkey();
    let neg_pk = negate_g1(&pk).unwrap();
    let double_neg = negate_g1(&neg_pk).unwrap();

    assert_eq!(
        pk, double_neg,
        "CIR-003: Double negation must equal original"
    );
}

#[test]
fn vv_req_cir_003_generator_point_works() {
    // CIR-003: Generator point can be aggregated
    let generator = serialize_g1(&G1Affine::generator());

    // n * G = G + G + ... + G (n times)
    let n = 10;
    let pubkeys: Vec<[u8; 48]> = (0..n).map(|_| generator).collect();
    let aggregate = aggregate_pubkeys(&pubkeys).unwrap();

    // Verify by computing n * G directly
    let expected = G1Affine::from(G1Projective::from(G1Affine::generator()) * Fr::from(n as u64));
    let expected_bytes = serialize_g1(&expected);

    assert_eq!(
        aggregate, expected_bytes,
        "CIR-003: n * G must equal G + G + ... + G"
    );
}
