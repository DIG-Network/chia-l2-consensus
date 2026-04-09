//! REQUIREMENT: SETUP-004 — Core dependencies
//! (`docs/requirements/domains/setup/NORMATIVE.md#SETUP-004`).
//!
//! Spec: `docs/requirements/domains/setup/specs/SETUP-004.md`.
//!
//! Verifies that all core dependencies are available and importable.

#[test]
fn vv_req_setup_004_chia_dependencies() {
    // Verify Chia SDK crates are available
    use chia_protocol::{Bytes32, Coin, CoinSpend, SpendBundle};
    use clvmr::Allocator;

    // Verify types are constructible
    let _allocator = Allocator::new();

    // Verify Bytes32 is a 32-byte array wrapper
    let bytes: Bytes32 = Bytes32::from([0u8; 32]);
    assert_eq!(bytes.as_ref().len(), 32);

    // Verify protocol types exist (compile-time check)
    fn _check_coin(_c: Coin) {}
    fn _check_coin_spend(_cs: CoinSpend) {}
    fn _check_spend_bundle(_sb: SpendBundle) {}
}

#[test]
fn vv_req_setup_004_arkworks_dependencies() {
    // Verify arkworks crates are available
    use ark_groth16::Groth16;
    use ark_bls12_381::{Bls12_381, Fr, G1Projective, G2Projective};
    use ark_relations::r1cs::ConstraintSystem;
    use ark_ff::PrimeField;
    use ark_ec::CurveGroup;
    use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};

    // Verify basic field operations work
    let cs = ConstraintSystem::<Fr>::new_ref();
    assert!(cs.is_satisfied().unwrap());

    // Verify curve types are accessible
    fn _assert_curve_group<T: CurveGroup>() {}
    _assert_curve_group::<G1Projective>();
    _assert_curve_group::<G2Projective>();

    // Verify Groth16 type is accessible
    fn _assert_groth16<E: ark_ec::pairing::Pairing>() -> std::marker::PhantomData<Groth16<E>> {
        std::marker::PhantomData
    }
    let _ = _assert_groth16::<Bls12_381>();

    // Verify serialization traits
    fn _assert_serialize<T: CanonicalSerialize + CanonicalDeserialize>() {}
    _assert_serialize::<Fr>();

    // Verify field trait
    fn _assert_prime_field<F: PrimeField>() {}
    _assert_prime_field::<Fr>();
}

#[test]
fn vv_req_setup_004_blst_dependency() {
    // Verify blst is available for BLS signature aggregation
    use blst::min_pk::{SecretKey, PublicKey, Signature, AggregateSignature};

    // Verify key generation works
    let ikm = [0u8; 32];
    let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
    let pk: PublicKey = sk.sk_to_pk();

    // Verify signing works
    let msg = b"test message";
    let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
    let sig: Signature = sk.sign(msg, dst, &[]);

    // Verify signature verification works
    let result = sig.verify(true, msg, dst, &[], &pk, true);
    assert_eq!(result, blst::BLST_ERROR::BLST_SUCCESS);

    // Verify aggregate signature type exists
    fn _assert_agg_sig(_agg: AggregateSignature) {}
}

#[test]
fn vv_req_setup_004_utility_dependencies() {
    // Verify utility crates are available
    use sha2::{Sha256, Digest};
    use hex::{encode, decode};
    use num_bigint::BigUint;
    use serde::{Serialize, Deserialize};
    use thiserror::Error;
    use anyhow::{Result, anyhow};

    // Verify SHA-256 works
    let mut hasher = Sha256::new();
    hasher.update(b"test");
    let result = hasher.finalize();
    assert_eq!(result.len(), 32);

    // Verify hex encoding works
    let encoded = encode(&[0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(encoded, "deadbeef");

    // Verify hex decoding works
    let decoded = decode("deadbeef").unwrap();
    assert_eq!(decoded, vec![0xde, 0xad, 0xbe, 0xef]);

    // Verify bigint works
    let big = BigUint::from(123456789u64);
    assert!(big > BigUint::from(0u64));

    // Verify serde traits exist
    #[derive(Serialize, Deserialize)]
    struct TestStruct { value: u32 }
    let json = serde_json::to_string(&TestStruct { value: 42 }).unwrap();
    assert!(json.contains("42"));

    // Verify thiserror works
    #[derive(Error, Debug)]
    #[error("test error")]
    struct TestError;
    let _err: TestError = TestError;

    // Verify anyhow works
    fn _returns_result() -> Result<()> {
        Err(anyhow!("test error"))
    }
}

#[tokio::test]
async fn vv_req_setup_004_async_dependencies() {
    // Verify tokio and futures are available
    use futures::future::ready;
    use tokio::time::{sleep, Duration};

    // Verify async works
    let result = ready(42).await;
    assert_eq!(result, 42);

    // Verify tokio timer works (just a quick check)
    sleep(Duration::from_millis(1)).await;
}

#[test]
fn vv_req_setup_004_ark_crypto_primitives() {
    // Verify ark-crypto-primitives with crh feature
    use ark_crypto_primitives::crh::{CRHScheme, sha256::Sha256 as ArkSha256};

    // CRH scheme should be available for circuit hash gadgets
    fn _check_crh<H: CRHScheme>() {}
    _check_crh::<ArkSha256>();
}
