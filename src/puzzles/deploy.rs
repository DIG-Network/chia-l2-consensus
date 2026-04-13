//! Deployment functions for creating both singletons atomically.
//!
//! DEP-002: Both launcher IDs derived from genesis coin(s).
//!
//! Uses chia-wallet-sdk `Launcher` for singleton creation. Both
//! singletons are deployed in a single spend bundle for atomicity.
//!
//! See [spec-deployment-runbook.md](../../docs/resources/spec-deployment-runbook.md).

use chia_protocol::{Bytes32, Coin};
use chia_puzzles::singleton::SINGLETON_LAUNCHER_PUZZLE_HASH;
use chia_sdk_driver::{Launcher, SpendContext, StandardLayer};

use crate::config::NetworkConfig;
use crate::error::{ConsensusError, ConsensusResult};
use crate::prover::setup::VkComponents;
use crate::puzzles::checkpoint::CHECKPOINT_INNER_MOD_HASH_HEX;
use crate::puzzles::registration_coin::REGISTRATION_COIN_MOD_HASH_HEX;

/// Derive the singleton launcher ID from a parent coin ID and amount.
///
/// The launcher coin's ID is `sha256(parent_coin_id + SINGLETON_LAUNCHER_PUZZLE_HASH + amount)`.
/// This lets you predict the launcher ID before actually spending the parent coin.
///
/// This is the permanent identifier for the singleton — it never changes
/// across spends of the singleton.
pub fn derive_launcher_id(parent_coin_id: Bytes32, amount: u64) -> Bytes32 {
    let launcher_coin = Coin::new(
        parent_coin_id,
        SINGLETON_LAUNCHER_PUZZLE_HASH.into(),
        amount,
    );
    launcher_coin.coin_id()
}

/// Parse a hex-encoded mod hash from a compiled puzzle artifact.
///
/// The `.hash` files produced by `rue build --hash` contain a `0x`-prefixed
/// hex string. This helper strips the prefix and parses to Bytes32.
fn parse_mod_hash(hex_str: &str) -> ConsensusResult<Bytes32> {
    let trimmed = hex_str.trim().trim_start_matches("0x");
    let bytes = hex::decode(trimmed)
        .map_err(|e| ConsensusError::SerializationError(format!("Invalid mod hash hex: {}", e)))?;
    if bytes.len() != 32 {
        return Err(ConsensusError::SerializationError(format!(
            "Mod hash must be 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr.into())
}

/// Deploy both singletons (network coin + checkpoint) atomically.
///
/// Takes two P2 funding coins (each 1 mojo for singleton), creates a launcher
/// from each, and spends both launchers in the same `SpendContext`.
///
/// The caller must then submit `ctx.take()` with both secret keys to the
/// simulator or Chia node.
///
/// # Arguments
///
/// * `ctx` — Spend context accumulating coin spends
/// * `net_funding_coin` — P2 coin funding the network coin launcher (≥1 mojo)
/// * `net_pk` — Public key controlling `net_funding_coin`
/// * `chk_funding_coin` — P2 coin funding the checkpoint launcher (≥1 mojo)
/// * `chk_pk` — Public key controlling `chk_funding_coin`
/// * `vk_components` — Verification key components from trusted setup
/// * `collateral_amount` — Required collateral per validator in mojos
/// * `tree_depth` — Sparse Merkle tree depth
/// * `genesis_challenge` — Chia network genesis challenge
///
/// # Returns
///
/// `(network_coin_singleton, checkpoint_singleton, NetworkConfig)`
#[allow(clippy::too_many_arguments)]
pub fn deploy_both_singletons(
    ctx: &mut SpendContext,
    net_funding_coin: Coin,
    net_pk: chia_bls::PublicKey,
    chk_funding_coin: Coin,
    chk_pk: chia_bls::PublicKey,
    vk_components: &VkComponents,
    collateral_amount: u64,
    tree_depth: u32,
    genesis_challenge: Bytes32,
) -> ConsensusResult<(Coin, Coin, NetworkConfig)> {
    // Parse compiled puzzle mod hashes
    let registration_coin_mod_hash = parse_mod_hash(REGISTRATION_COIN_MOD_HASH_HEX)?;
    let checkpoint_inner_mod_hash = parse_mod_hash(CHECKPOINT_INNER_MOD_HASH_HEX)?;

    // For inner puzzle hashes, use the mod hashes directly.
    // In a full implementation, these would be curried with deployment params.
    // For now, the inner_puzzle_hash passed to the launcher determines the
    // singleton puzzle hash.
    let net_inner_ph: Bytes32 = registration_coin_mod_hash;
    let chk_inner_ph: Bytes32 = checkpoint_inner_mod_hash;

    // --- Network coin launcher ---
    let net_launcher = Launcher::new(net_funding_coin.coin_id(), 1);
    let net_launcher_id = net_launcher.coin().coin_id();
    let (net_conds, net_singleton) = net_launcher
        .spend(ctx, net_inner_ph, ())
        .map_err(|e| ConsensusError::ProvingError(format!("Network launcher spend: {}", e)))?;
    StandardLayer::new(net_pk)
        .spend(ctx, net_funding_coin, net_conds)
        .map_err(|e| ConsensusError::ProvingError(format!("Network P2 spend: {}", e)))?;

    // --- Checkpoint launcher ---
    let chk_launcher = Launcher::new(chk_funding_coin.coin_id(), 1);
    let chk_launcher_id = chk_launcher.coin().coin_id();
    let (chk_conds, chk_singleton) = chk_launcher
        .spend(ctx, chk_inner_ph, ())
        .map_err(|e| ConsensusError::ProvingError(format!("Checkpoint launcher spend: {}", e)))?;
    StandardLayer::new(chk_pk)
        .spend(ctx, chk_funding_coin, chk_conds)
        .map_err(|e| ConsensusError::ProvingError(format!("Checkpoint P2 spend: {}", e)))?;

    // --- Build VK hex string ---
    let mut vk_bytes = Vec::new();
    vk_bytes.extend_from_slice(&vk_components.alpha_g1);
    vk_bytes.extend_from_slice(&vk_components.beta_g2);
    vk_bytes.extend_from_slice(&vk_components.gamma_g2);
    vk_bytes.extend_from_slice(&vk_components.delta_g2);
    for ic in &vk_components.ic_points {
        vk_bytes.extend_from_slice(ic);
    }
    let verification_key_hex = hex::encode(&vk_bytes);

    // --- Build NetworkConfig ---
    // WDC-006: Parse withdraw delay coin mod hash from compiled artifact
    let withdraw_delay_mod_hash =
        parse_mod_hash(crate::puzzles::withdraw_delay::WITHDRAW_DELAY_COIN_MOD_HASH_HEX)?;

    let config = NetworkConfig {
        network_coin_launcher_id: net_launcher_id,
        checkpoint_launcher_id: chk_launcher_id,
        registration_coin_mod_hash,
        checkpoint_inner_mod_hash,
        collateral_amount,
        tree_depth,
        max_signers: crate::prover::circuit::MAX_SIGNERS,
        verification_key_hex,
        genesis_challenge,
        withdraw_delay_blocks: crate::puzzles::withdraw_delay::DEFAULT_WITHDRAW_DELAY_BLOCKS,
        withdraw_delay_mod_hash,
    };

    Ok((net_singleton, chk_singleton, config))
}
