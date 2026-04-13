//! REQUIREMENT: DEP-005 — Artifact Publication
//! (`docs/requirements/domains/deployment/NORMATIVE.md#DEP-005`).
//!
//! Spec: `docs/requirements/domains/deployment/specs/DEP-005.md`.
//!
//! Implementation: `src/config.rs`.
//!
//! ## Normative statement
//! Deployment artifacts MUST serialize to JSON with all required fields
//! (launcher IDs, mod hashes, collateral, tree_depth, max_signers,
//! genesis_challenge, vk_hash, verification_key). Bytes32 fields MUST use
//! 0x-prefix hex encoding. The VK JSON MUST include alpha_g1, beta_g2,
//! gamma_g2, delta_g2, and 7 IC points with correct byte sizes.
//!
//! ## How the tests prove the requirement
//! 1. **Serializes to JSON**: Non-empty JSON with key fields.
//! 2. **All required fields**: 10 fields verified present.
//! 3. **0x prefix**: All hex fields start with "0x".
//! 4. **Bytes32 = 66 chars**: "0x" + 64 hex chars.
//! 5. **VK JSON structure**: Correct sizes for all VK components.
//! 6. **VK hash matches content**: Reconstructed VK bytes match hash.
//! 7. **JSON roundtrip**: Serialize -> deserialize -> compare succeeds.
//! 8. **Numeric fields**: collateral, tree_depth, max_signers correct.
//!
//! ## Completeness: HIGH
//! ## Gaps: Does not test publishing to a real distribution channel.

use chia_l2_consensus::testing::{
    compute_vk_hash, deploy_both_singletons, deserialize_proving_key, extract_vk_components,
    run_test_setup, verify_vk_hash, vk_to_bytes,
};
use chia_l2_consensus::DeploymentArtifacts;
use chia_sdk_driver::SpendContext;
use chia_sdk_test::Simulator;

// ── Helper: create a deployment and get artifacts ───────────────────

fn setup_artifacts() -> DeploymentArtifacts {
    let mut sim = Simulator::new();
    let ctx = &mut SpendContext::new();
    let (sk1, pk1, _, net_coin) = sim.new_p2(1).unwrap();
    let (sk2, pk2, _, chk_coin) = sim.new_p2(1).unwrap();

    let (pk_bytes, _) = run_test_setup().unwrap();
    let pk = deserialize_proving_key(&pk_bytes).unwrap();
    let vk_components = extract_vk_components(&pk.vk).unwrap();

    let genesis_challenge = chia_protocol::Bytes32::from([0x42; 32]);

    let (_, _, config) = deploy_both_singletons(
        ctx,
        net_coin,
        pk1,
        chk_coin,
        pk2,
        &vk_components,
        10_000_000_000_000,
        32,
        genesis_challenge,
    )
    .unwrap();

    sim.spend_coins(ctx.take(), &[sk1, sk2]).unwrap();

    let vk_hash = compute_vk_hash(&pk.vk).unwrap();

    DeploymentArtifacts::from_config(&config, &vk_hash)
}

// ── Artifacts serialize to valid JSON ───────────────────────────────

#[test]
fn vv_req_dep_005_serializes_to_json() {
    let artifacts = setup_artifacts();
    let json = serde_json::to_string_pretty(&artifacts).expect("DEP-005: Must serialize to JSON");

    assert!(!json.is_empty(), "DEP-005: JSON must not be empty");
    assert!(
        json.contains("network_coin_launcher_id"),
        "DEP-005: Must contain network_coin_launcher_id"
    );
    assert!(
        json.contains("checkpoint_launcher_id"),
        "DEP-005: Must contain checkpoint_launcher_id"
    );
    assert!(json.contains("vk_hash"), "DEP-005: Must contain vk_hash");
}

// ── All required fields present ─────────────────────────────────────

#[test]
fn vv_req_dep_005_all_fields_present() {
    let artifacts = setup_artifacts();
    let json = serde_json::to_string(&artifacts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();

    let required_fields = [
        "network_coin_launcher_id",
        "checkpoint_launcher_id",
        "registration_coin_mod_hash",
        "checkpoint_inner_mod_hash",
        "collateral_amount",
        "tree_depth",
        "max_signers",
        "genesis_challenge",
        "vk_hash",
        "verification_key",
    ];

    for field in &required_fields {
        assert!(
            v.get(field).is_some(),
            "DEP-005: Required field '{}' missing from artifacts JSON",
            field
        );
    }
}

// ── Hex fields have 0x prefix ───────────────────────────────────────

#[test]
fn vv_req_dep_005_hex_fields_have_prefix() {
    let artifacts = setup_artifacts();

    assert!(
        artifacts.network_coin_launcher_id.starts_with("0x"),
        "DEP-005: network_coin_launcher_id must start with 0x"
    );
    assert!(
        artifacts.checkpoint_launcher_id.starts_with("0x"),
        "DEP-005: checkpoint_launcher_id must start with 0x"
    );
    assert!(
        artifacts.registration_coin_mod_hash.starts_with("0x"),
        "DEP-005: registration_coin_mod_hash must start with 0x"
    );
    assert!(
        artifacts.checkpoint_inner_mod_hash.starts_with("0x"),
        "DEP-005: checkpoint_inner_mod_hash must start with 0x"
    );
    assert!(
        artifacts.genesis_challenge.starts_with("0x"),
        "DEP-005: genesis_challenge must start with 0x"
    );
    assert!(
        artifacts.vk_hash.starts_with("0x"),
        "DEP-005: vk_hash must start with 0x"
    );
}

// ── Bytes32 fields are 66 chars (0x + 64 hex) ──────────────────────

#[test]
fn vv_req_dep_005_bytes32_fields_correct_length() {
    let artifacts = setup_artifacts();

    let bytes32_fields = [
        (
            "network_coin_launcher_id",
            &artifacts.network_coin_launcher_id,
        ),
        ("checkpoint_launcher_id", &artifacts.checkpoint_launcher_id),
        (
            "registration_coin_mod_hash",
            &artifacts.registration_coin_mod_hash,
        ),
        (
            "checkpoint_inner_mod_hash",
            &artifacts.checkpoint_inner_mod_hash,
        ),
        ("genesis_challenge", &artifacts.genesis_challenge),
        ("vk_hash", &artifacts.vk_hash),
    ];

    for (name, value) in &bytes32_fields {
        assert_eq!(
            value.len(),
            66, // "0x" + 64 hex chars = 66
            "DEP-005: {} must be 66 chars (0x + 64 hex), got {}",
            name,
            value.len()
        );
    }
}

// ── VK JSON has correct structure ───────────────────────────────────

#[test]
fn vv_req_dep_005_vk_json_structure() {
    let artifacts = setup_artifacts();
    let vk = &artifacts.verification_key;

    assert!(
        vk.alpha_g1.starts_with("0x"),
        "DEP-005: alpha_g1 must be hex"
    );
    assert!(vk.beta_g2.starts_with("0x"), "DEP-005: beta_g2 must be hex");
    assert!(
        vk.gamma_g2.starts_with("0x"),
        "DEP-005: gamma_g2 must be hex"
    );
    assert!(
        vk.delta_g2.starts_with("0x"),
        "DEP-005: delta_g2 must be hex"
    );

    assert_eq!(
        vk.alpha_g1.len(),
        2 + 96,
        "DEP-005: alpha_g1 must be 0x + 96 hex (48 bytes)"
    );
    assert_eq!(
        vk.beta_g2.len(),
        2 + 192,
        "DEP-005: beta_g2 must be 0x + 192 hex (96 bytes)"
    );
    assert_eq!(
        vk.gamma_g2.len(),
        2 + 192,
        "DEP-005: gamma_g2 must be 0x + 192 hex (96 bytes)"
    );
    assert_eq!(
        vk.delta_g2.len(),
        2 + 192,
        "DEP-005: delta_g2 must be 0x + 192 hex (96 bytes)"
    );

    assert_eq!(vk.ic.len(), 7, "DEP-005: VK must have 7 IC points");
    for (i, ic) in vk.ic.iter().enumerate() {
        assert!(ic.starts_with("0x"), "DEP-005: IC[{}] must be hex", i);
        assert_eq!(
            ic.len(),
            2 + 96,
            "DEP-005: IC[{}] must be 0x + 96 hex (48 bytes)",
            i
        );
    }
}

// ── VK hash in artifacts matches VK content ─────────────────────────

#[test]
fn vv_req_dep_005_vk_hash_matches_vk_content() {
    let (pk_bytes, _) = run_test_setup().unwrap();
    let pk = deserialize_proving_key(&pk_bytes).unwrap();

    let vk_bytes = vk_to_bytes(&pk.vk).unwrap();
    let vk_hash = compute_vk_hash(&pk.vk).unwrap();
    let config = chia_l2_consensus::NetworkConfig {
        network_coin_launcher_id: chia_protocol::Bytes32::default(),
        checkpoint_launcher_id: chia_protocol::Bytes32::default(),
        registration_coin_mod_hash: chia_protocol::Bytes32::default(),
        checkpoint_inner_mod_hash: chia_protocol::Bytes32::default(),
        collateral_amount: 0,
        tree_depth: 32,
        max_signers: 20_000,
        verification_key_hex: hex::encode(&vk_bytes),
        genesis_challenge: chia_protocol::Bytes32::default(),
        withdraw_delay_blocks: 24_000,
        withdraw_delay_mod_hash: chia_protocol::Bytes32::default(),
    };

    let artifacts = DeploymentArtifacts::from_config(&config, &vk_hash);

    // Reconstruct VK bytes from VkJson
    let reconstructed = artifacts
        .verification_key
        .to_bytes()
        .expect("DEP-005: VkJson to_bytes");

    assert!(
        verify_vk_hash(&reconstructed, &vk_hash),
        "DEP-005: VK hash from artifacts must match reconstructed VK bytes"
    );
}

// ── Round-trip: serialize → deserialize → compare ───────────────────

#[test]
fn vv_req_dep_005_json_roundtrip() {
    let artifacts = setup_artifacts();

    let json = serde_json::to_string_pretty(&artifacts).unwrap();
    let deserialized: DeploymentArtifacts = serde_json::from_str(&json).unwrap();

    assert_eq!(
        artifacts.network_coin_launcher_id,
        deserialized.network_coin_launcher_id
    );
    assert_eq!(
        artifacts.checkpoint_launcher_id,
        deserialized.checkpoint_launcher_id
    );
    assert_eq!(artifacts.collateral_amount, deserialized.collateral_amount);
    assert_eq!(artifacts.tree_depth, deserialized.tree_depth);
    assert_eq!(artifacts.max_signers, deserialized.max_signers);
    assert_eq!(artifacts.vk_hash, deserialized.vk_hash);
    assert_eq!(
        artifacts.verification_key.ic.len(),
        deserialized.verification_key.ic.len()
    );
}

// ── Numeric fields are correct ──────────────────────────────────────

#[test]
fn vv_req_dep_005_numeric_fields() {
    let artifacts = setup_artifacts();

    assert_eq!(
        artifacts.collateral_amount, 10_000_000_000_000,
        "DEP-005: collateral_amount"
    );
    assert_eq!(artifacts.tree_depth, 32, "DEP-005: tree_depth");
    assert_eq!(artifacts.max_signers, 20_000, "DEP-005: max_signers");
}
