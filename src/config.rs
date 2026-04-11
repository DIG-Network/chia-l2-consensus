//! Network configuration for chia-l2-consensus.
//!
//! Contains all deployment-time parameters that define a specific L2 network.
//! Produced by `deploy_both_singletons()` and saved to disk as JSON.
//! Loaded on every subsequent node startup.
//!
//! Changing `collateral_amount`, `tree_depth`, or `verification_key_hex`
//! requires a full redeployment. Changing `max_signers` requires a new
//! trusted setup ceremony.
//!
//! See [spec-consensus-crate.md Lines 226-315](../docs/resources/spec-consensus-crate.md)
//! for the full NetworkConfig specification.

use chia_protocol::Bytes32;

/// All parameters that define a specific L2 network deployment.
///
/// Fixed at deployment time and never change for the life of the deployment.
/// Produced by `ConsensusClient::deploy()` and saved to disk.
/// Loaded on every subsequent node startup.
///
/// See [spec-consensus-crate.md Lines 233-297](../docs/resources/spec-consensus-crate.md).
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Launcher ID of the network coin singleton.
    /// Permanent identifier for this L2 network's registration authority.
    /// Used to derive the current network coin puzzle hash for on-chain lookups.
    ///
    /// See [spec-network-coin.md Lines 50-80](../docs/resources/spec-network-coin.md) — Deployment.
    pub network_coin_launcher_id: Bytes32,

    /// Launcher ID of the checkpoint singleton.
    /// Used to find the current checkpoint coin and derive `checkpoint_singleton_id()`.
    ///
    /// See [spec-checkpoint-singleton.md Lines 50-80](../docs/resources/spec-checkpoint-singleton.md) — Deployment.
    pub checkpoint_launcher_id: Bytes32,

    /// Tree hash of the base registration coin puzzle BEFORE currying.
    /// Every valid registration coin derives its puzzle hash from this.
    /// Used by the indexer for lineage verification (IDX-002).
    ///
    /// See [spec-registration-coin.md Lines 100-150](../docs/resources/spec-registration-coin.md) —
    /// Computing the Registration Coin Puzzle Hash.
    pub registration_coin_mod_hash: Bytes32,

    /// Tree hash of the base checkpoint inner puzzle BEFORE currying.
    /// Used to rebuild the inner puzzle with current state on each spend.
    pub checkpoint_inner_mod_hash: Bytes32,

    /// Required collateral per validator in mojos.
    /// Enforced exactly by the network coin puzzle on every registration.
    /// Cannot change without redeploying the network coin.
    ///
    /// See [spec-network-coin.md Lines 30-50](../docs/resources/spec-network-coin.md) —
    /// Curried In Parameters: COLLATERAL_AMOUNT.
    pub collateral_amount: u64,

    /// Depth of the sparse Merkle tree (default: 32).
    /// Must match TREE_DEPTH in circuit, SMT, and checkpoint singleton.
    ///
    /// See [spec-sparse-merkle-tree.md Lines 30-60](../docs/resources/spec-sparse-merkle-tree.md) —
    /// Parameters: TREE_DEPTH.
    pub tree_depth: u32,

    /// Maximum simultaneous signers the Groth16 circuit supports.
    /// Fixed at trusted setup time. Cannot increase without new ceremony.
    ///
    /// See [spec-groth16-circuit.md Lines 150-180](../docs/resources/spec-groth16-circuit.md) —
    /// Circuit Parameters: MAX_SIGNERS.
    pub max_signers: usize,

    /// Groth16 verification key from the trusted setup ceremony.
    /// Stored as hex-encoded bytes (672 bytes → 1344 hex chars).
    /// This exact value is curried into the checkpoint singleton at deployment.
    ///
    /// See [spec-wire-format.md Lines 200-250](../docs/resources/spec-wire-format.md) —
    /// Verification Key Format.
    pub verification_key_hex: String,

    /// Chia network genesis challenge (mainnet or testnet constant).
    /// Used in AGG_SIG_ME message construction for all signed conditions.
    ///
    /// See [spec-wire-format.md Lines 466-500](../docs/resources/spec-wire-format.md) —
    /// Individual Signatures.
    pub genesis_challenge: Bytes32,
}

// ============================================================================
// DEP-005: Deployment Artifacts (serializable to JSON)
// ============================================================================

/// Verification key in JSON format with hex-encoded components.
///
/// This is the published VK format that validators and auditors use to verify
/// the on-chain checkpoint singleton contains the expected VK.
///
/// See spec-wire-format.md — Verification Key Format — Storage Format.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VkJson {
    pub alpha_g1: String,
    pub beta_g2: String,
    pub gamma_g2: String,
    pub delta_g2: String,
    pub ic: Vec<String>,
}

impl VkJson {
    /// Reconstruct flat VK bytes from the JSON components.
    ///
    /// Returns the 672-byte concatenation: alpha(48) || beta(96) || gamma(96)
    /// || delta(96) || ic[0..7](7×48).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::with_capacity(672);

        let decode = |s: &str, name: &str, expected: usize| -> Result<Vec<u8>, String> {
            let trimmed = s.trim_start_matches("0x");
            let b = hex::decode(trimmed).map_err(|e| format!("{}: {}", name, e))?;
            if b.len() != expected {
                return Err(format!(
                    "{}: {} bytes, expected {}",
                    name,
                    b.len(),
                    expected
                ));
            }
            Ok(b)
        };

        bytes.extend_from_slice(&decode(&self.alpha_g1, "alpha_g1", 48)?);
        bytes.extend_from_slice(&decode(&self.beta_g2, "beta_g2", 96)?);
        bytes.extend_from_slice(&decode(&self.gamma_g2, "gamma_g2", 96)?);
        bytes.extend_from_slice(&decode(&self.delta_g2, "delta_g2", 96)?);

        for (i, ic) in self.ic.iter().enumerate() {
            bytes.extend_from_slice(&decode(ic, &format!("ic[{}]", i), 48)?);
        }

        Ok(bytes)
    }
}

/// All deployment artifacts in a single JSON-serializable structure.
///
/// Published after deployment so validators can configure their nodes and
/// auditors can verify the on-chain state matches expectations.
///
/// See spec-trusted-setup.md — What to Publish.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeploymentArtifacts {
    pub network_coin_launcher_id: String,
    pub checkpoint_launcher_id: String,
    pub registration_coin_mod_hash: String,
    pub checkpoint_inner_mod_hash: String,
    pub collateral_amount: u64,
    pub tree_depth: u32,
    pub max_signers: usize,
    pub genesis_challenge: String,
    pub vk_hash: String,
    pub verification_key: VkJson,
}

/// Format a Bytes32 as "0x" + hex.
fn bytes32_to_hex(b: &Bytes32) -> String {
    format!("0x{}", hex::encode(b.as_ref()))
}

impl DeploymentArtifacts {
    /// Build deployment artifacts from a `NetworkConfig` and VK hash.
    ///
    /// The VK components are extracted from `config.verification_key_hex`.
    pub fn from_config(config: &NetworkConfig, vk_hash: &[u8; 32]) -> Self {
        // Parse VK hex into components
        let vk_bytes = hex::decode(&config.verification_key_hex).unwrap_or_default();
        let vk_json = if vk_bytes.len() == 672 {
            VkJson {
                alpha_g1: format!("0x{}", hex::encode(&vk_bytes[0..48])),
                beta_g2: format!("0x{}", hex::encode(&vk_bytes[48..144])),
                gamma_g2: format!("0x{}", hex::encode(&vk_bytes[144..240])),
                delta_g2: format!("0x{}", hex::encode(&vk_bytes[240..336])),
                ic: (0..7)
                    .map(|i| {
                        let start = 336 + i * 48;
                        format!("0x{}", hex::encode(&vk_bytes[start..start + 48]))
                    })
                    .collect(),
            }
        } else {
            VkJson {
                alpha_g1: String::new(),
                beta_g2: String::new(),
                gamma_g2: String::new(),
                delta_g2: String::new(),
                ic: Vec::new(),
            }
        };

        Self {
            network_coin_launcher_id: bytes32_to_hex(&config.network_coin_launcher_id),
            checkpoint_launcher_id: bytes32_to_hex(&config.checkpoint_launcher_id),
            registration_coin_mod_hash: bytes32_to_hex(&config.registration_coin_mod_hash),
            checkpoint_inner_mod_hash: bytes32_to_hex(&config.checkpoint_inner_mod_hash),
            collateral_amount: config.collateral_amount,
            tree_depth: config.tree_depth,
            max_signers: config.max_signers,
            genesis_challenge: bytes32_to_hex(&config.genesis_challenge),
            vk_hash: format!("0x{}", hex::encode(vk_hash)),
            verification_key: vk_json,
        }
    }
}
