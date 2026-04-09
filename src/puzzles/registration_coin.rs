//! Registration coin driver.
//!
//! See [spec-registration-coin.md](../../docs/resources/spec-registration-coin.md).

use chia_protocol::Bytes32;
use chia_protocol::SpendBundle;

use crate::error::ConsensusResult;

/// Spend a registration coin to recover collateral.
pub fn spend_registration_coin(_coin_id: Bytes32) -> ConsensusResult<SpendBundle> {
    // TODO: Implement using chia-wallet-sdk
    todo!()
}

/// Compute the registration coin puzzle hash.
pub fn registration_coin_puzzle_hash(
    _checkpoint_singleton_id: Bytes32,
    _pubkey: &[u8; 48],
) -> Bytes32 {
    // TODO: Implement
    Bytes32::default()
}
