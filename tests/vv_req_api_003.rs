//! REQUIREMENT: API-003 — State Types
//! (`docs/requirements/domains/crate_api/NORMATIVE.md#API-003`).
//!
//! Verifies ValidatorInfo, ValidatorSet helpers, and type consistency.

use chia_l2_consensus::{CheckpointSingletonState, ValidatorInfo, ValidatorSet};
use chia_protocol::{Bytes32, Coin};

#[test]
fn vv_req_api_003_validator_info_exists() {
    let info = ValidatorInfo {
        pubkey: [0xAA; 48],
        registration_coin: Coin::new(Bytes32::default(), Bytes32::default(), 1000),
    };
    assert_eq!(info.pubkey, [0xAA; 48]);
    assert_eq!(info.registration_coin.amount, 1000);
}

#[test]
fn vv_req_api_003_validator_set_count() {
    let set = ValidatorSet {
        validators: vec![
            chia_l2_consensus::testing::Validator {
                pubkey: vec![0xAA; 48],
                slot: 0,
                registration_coin_id: Bytes32::default(),
            },
            chia_l2_consensus::testing::Validator {
                pubkey: vec![0xBB; 48],
                slot: 1,
                registration_coin_id: Bytes32::default(),
            },
        ],
        epoch: 5,
        merkle_root: Bytes32::default(),
    };
    assert_eq!(
        set.count(),
        2,
        "API-003: count() must return validator count"
    );
}

#[test]
fn vv_req_api_003_validator_set_contains() {
    let set = ValidatorSet {
        validators: vec![chia_l2_consensus::testing::Validator {
            pubkey: vec![0xAA; 48],
            slot: 0,
            registration_coin_id: Bytes32::default(),
        }],
        epoch: 5,
        merkle_root: Bytes32::default(),
    };
    assert!(
        set.contains(&[0xAA; 48]),
        "API-003: contains() must find existing pubkey"
    );
    assert!(
        !set.contains(&[0xBB; 48]),
        "API-003: contains() must not find missing pubkey"
    );
}

#[test]
fn vv_req_api_003_validator_set_pubkeys() {
    let set = ValidatorSet {
        validators: vec![
            chia_l2_consensus::testing::Validator {
                pubkey: vec![0xAA; 48],
                slot: 0,
                registration_coin_id: Bytes32::default(),
            },
            chia_l2_consensus::testing::Validator {
                pubkey: vec![0xBB; 48],
                slot: 1,
                registration_coin_id: Bytes32::default(),
            },
        ],
        epoch: 5,
        merkle_root: Bytes32::default(),
    };
    let pks = set.pubkeys();
    assert_eq!(pks.len(), 2, "API-003: pubkeys() must return all pubkeys");
    assert_eq!(pks[0], vec![0xAA; 48]);
    assert_eq!(pks[1], vec![0xBB; 48]);
}

#[test]
fn vv_req_api_003_empty_validator_set() {
    let set = ValidatorSet {
        validators: vec![],
        epoch: 0,
        merkle_root: Bytes32::default(),
    };
    assert_eq!(set.count(), 0);
    assert!(!set.contains(&[0xAA; 48]));
    assert!(set.pubkeys().is_empty());
}
