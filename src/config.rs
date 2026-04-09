//! Network configuration for chia-l2-consensus.
//!
//! See [spec-consensus-crate.md Lines 226-315](../docs/resources/spec-consensus-crate.md).

use chia_protocol::Bytes32;

/// All parameters that define a specific L2 network deployment.
///
/// Fixed at deployment time and never change for the life of the deployment.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Launcher ID of the network coin singleton.
    pub network_coin_launcher_id: Bytes32,

    /// Launcher ID of the checkpoint singleton.
    pub checkpoint_launcher_id: Bytes32,

    /// Tree hash of the base registration coin puzzle before currying.
    pub registration_coin_mod_hash: Bytes32,

    /// Tree hash of the base checkpoint inner puzzle before currying.
    pub checkpoint_inner_mod_hash: Bytes32,

    /// Required collateral per validator in mojos.
    pub collateral_amount: u64,

    /// Depth of the sparse Merkle tree.
    pub tree_depth: u32,

    /// Maximum simultaneous signers supported by the Groth16 circuit.
    pub max_signers: usize,

    /// Groth16 verification key from the trusted setup ceremony (hex-encoded).
    pub verification_key_hex: String,

    /// Chia network genesis challenge.
    pub genesis_challenge: Bytes32,
}

// TODO: Implement custom serde for NetworkConfig when needed for persistence
// The Bytes32 type from chia_protocol may need feature flags or custom serialization
