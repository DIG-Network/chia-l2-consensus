//! Golden vectors for every consensus-visible derivation in this crate.
//!
//! These pin the *computed bytes* of the derivations that decide what this
//! validator will accept on chain: singleton coin ids and the registration
//! coin's curried puzzle hash. A change to any of these values is a fork,
//! not a refactor, so they are pinned rather than recomputed.
//!
//! The expected values are NOT captured from this crate's own output. They
//! were derived from an independent implementation of the two underlying
//! algorithms as specified:
//!
//! - coin id  = `sha256(parent_coin_info || puzzle_hash || minimal_be(amount))`
//! - currying = the CLVM `(a (q . prog) args)` tree hash, where an atom hashes
//!   as `sha256(0x01 || bytes)` and a pair as `sha256(0x02 || left || right)`
//!
//! An independent oracle is what makes these load-bearing: a fixture blessed
//! from the implementation under test would agree with that implementation
//! even when both are wrong.

use chia_l2_consensus::testing::{derive_launcher_id, registration_coin_puzzle_hash};
use chia_protocol::Bytes32;

/// Fixture inputs are fixed byte patterns, chosen here rather than derived
/// from the crate, so the vectors stay stable across dependency uplifts.
const LAUNCHER_ID: [u8; 32] = [0x11; 32];
const PARENT_COIN: [u8; 32] = [0x22; 32];
const REG_MOD_HASH: [u8; 32] = [0x33; 32];

/// A 48-byte G1 pubkey pattern. Arbitrary bytes: currying hashes the atom, it
/// does not interpret the key, so no valid-point requirement applies here.
fn pubkey() -> [u8; 48] {
    let mut pk = [0u8; 48];
    for (i, b) in pk.iter_mut().enumerate() {
        *b = i as u8;
    }
    pk
}

/// The singleton launcher puzzle hash is a consensus constant. It moved from
/// `chia_puzzles::singleton::SINGLETON_LAUNCHER_PUZZLE_HASH` (a `TreeHash`) to
/// `chia_puzzles::SINGLETON_LAUNCHER_HASH` (a `[u8; 32]`) when chia-puzzles
/// split, so this pins the value across that move.
#[test]
fn golden_singleton_launcher_hash_is_unchanged() {
    assert_eq!(
        hex::encode(chia_puzzles::SINGLETON_LAUNCHER_HASH),
        "eff07522495060c066f66f32acc2a77e3a3e737aca8baea4d1a64ea4cdc13da9"
    );
}

/// `derive_launcher_id` must reproduce the coin id of the launcher coin.
/// Amount 1 exercises the single-byte minimal encoding.
#[test]
fn golden_derive_launcher_id_amount_one() {
    assert_eq!(
        hex::encode(derive_launcher_id(Bytes32::new(PARENT_COIN), 1)),
        "e9fb9d0439f55099c232c41c179232c96a2fe58453ec0abc49caa723a384607f"
    );
}

/// A large amount exercises the multi-byte branch of the minimal big-endian
/// amount encoding, which the single-byte vector above cannot reach.
#[test]
fn golden_derive_launcher_id_large_amount() {
    assert_eq!(
        hex::encode(derive_launcher_id(Bytes32::new(PARENT_COIN), 13_370_000_000_000)),
        "7003c54e9b95f497b906e88899466bbea92bacd1e47cda52b156ccd3d6332d6d"
    );
}

/// The registration coin's puzzle hash is the value a validator's coin is
/// matched against on chain, so it is the single most fork-sensitive
/// derivation in the crate.
#[test]
fn golden_registration_coin_puzzle_hash() {
    let checkpoint_singleton_id = derive_launcher_id(Bytes32::new(LAUNCHER_ID), 1);
    assert_eq!(
        hex::encode(checkpoint_singleton_id),
        "573d41c79e81a15e499d6ee1994c4ef939443f3d4eeee67f9132634ce2131c58",
        "the checkpoint singleton id feeds the curry below; pin it too"
    );

    assert_eq!(
        hex::encode(registration_coin_puzzle_hash(
            Bytes32::new(REG_MOD_HASH),
            &pubkey(),
            checkpoint_singleton_id,
        )),
        "9c9130c9e91c7cffd4cd085ab10e8f8ac316c748e7cc8518870bc7141275eea9"
    );
}
