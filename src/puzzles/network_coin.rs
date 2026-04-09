//! Network coin singleton driver.
//!
//! See [spec-network-coin.md](../../docs/resources/spec-network-coin.md).

use chia_protocol::Bytes32;
use chia_protocol::SpendBundle;

use crate::config::NetworkConfig;
use crate::error::ConsensusResult;

/// Deploy a new network coin singleton.
pub fn deploy_network_coin(_config: &NetworkConfig) -> ConsensusResult<SpendBundle> {
    // TODO: Implement using chia-wallet-sdk
    todo!()
}

/// Register a validator through the network coin.
pub fn register_validator(
    _config: &NetworkConfig,
    _pubkey: &[u8; 48],
) -> ConsensusResult<SpendBundle> {
    // TODO: Implement using chia-wallet-sdk
    todo!()
}

/// Fetch the current network coin state.
pub fn fetch_network_coin_state(_launcher_id: Bytes32) -> ConsensusResult<()> {
    // TODO: Implement
    todo!()
}
