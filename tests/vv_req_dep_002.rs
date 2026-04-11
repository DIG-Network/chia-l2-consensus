//! REQUIREMENT: DEP-002 — Genesis Coin
//! (`docs/requirements/domains/deployment/NORMATIVE.md#DEP-002`).
//!
//! Spec: `docs/requirements/domains/deployment/specs/DEP-002.md`.
//!
//! Implementation: `src/puzzles/deploy.rs`.
//!
//! Verifies that both singletons (network coin + checkpoint) can be deployed
//! atomically from funding coins, that launcher IDs are predictable before
//! spending, and that the resulting NetworkConfig is self-consistent.

use chia_l2_consensus::testing::{deploy_both_singletons, derive_launcher_id};
use chia_sdk_driver::SpendContext;
use chia_sdk_test::Simulator;

// ── Launcher ID derivation is deterministic ────────────────────────

#[test]
fn vv_req_dep_002_launcher_id_deterministic() {
    use chia_protocol::Bytes32;

    let parent_id = Bytes32::from([0xAA; 32]);
    let id1 = derive_launcher_id(parent_id, 1);
    let id2 = derive_launcher_id(parent_id, 1);

    assert_eq!(id1, id2, "DEP-002: Launcher ID must be deterministic");
}

// ── Different parents produce different launcher IDs ────────────────

#[test]
fn vv_req_dep_002_different_parents_different_ids() {
    use chia_protocol::Bytes32;

    let parent_a = Bytes32::from([0xAA; 32]);
    let parent_b = Bytes32::from([0xBB; 32]);

    let id_a = derive_launcher_id(parent_a, 1);
    let id_b = derive_launcher_id(parent_b, 1);

    assert_ne!(
        id_a, id_b,
        "DEP-002: Different parents must produce different launcher IDs"
    );
}

// ── Launcher ID matches chia-wallet-sdk Launcher API ────────────────

#[test]
fn vv_req_dep_002_launcher_id_matches_sdk() {
    use chia_sdk_driver::Launcher;

    let mut sim = Simulator::new();
    let (_, _, _, p2_coin) = sim.new_p2(1).expect("P2 coin");

    // Derive using our function
    let derived_id = derive_launcher_id(p2_coin.coin_id(), 1);

    // Derive using SDK Launcher
    let launcher = Launcher::new(p2_coin.coin_id(), 1);
    let sdk_id = launcher.coin().coin_id();

    assert_eq!(
        derived_id, sdk_id,
        "DEP-002: derive_launcher_id must match chia-wallet-sdk Launcher"
    );
}

// ── Deploy both singletons atomically ───────────────────────────────

#[test]
fn vv_req_dep_002_deploy_both_singletons() -> anyhow::Result<()> {
    use chia_l2_consensus::testing::{
        deserialize_proving_key, extract_vk_components, run_test_setup,
    };

    let mut sim = Simulator::new();
    let ctx = &mut SpendContext::new();

    // Two funding coins (simulates genesis split)
    let (sk1, pk1, _, net_coin) = sim.new_p2(1)?;
    let (sk2, pk2, _, chk_coin) = sim.new_p2(1)?;

    // Trusted setup for VK
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk_components = extract_vk_components(&pk.vk).expect("VK");

    let genesis_challenge = chia_protocol::Bytes32::from([0x00; 32]);

    let (net_singleton, chk_singleton, config) = deploy_both_singletons(
        ctx,
        net_coin,
        pk1,
        chk_coin,
        pk2,
        &vk_components,
        10_000_000_000_000, // 10 XCH collateral
        32,                 // tree depth
        genesis_challenge,
    )?;

    sim.spend_coins(ctx.take(), &[sk1, sk2])?;

    // Both singletons must exist
    assert!(
        sim.coin_state(net_singleton.coin_id()).is_some(),
        "DEP-002: Network coin singleton must exist after deployment"
    );
    assert!(
        sim.coin_state(chk_singleton.coin_id()).is_some(),
        "DEP-002: Checkpoint singleton must exist after deployment"
    );

    // Config has correct launcher IDs
    assert_eq!(
        config.network_coin_launcher_id,
        derive_launcher_id(net_coin.coin_id(), 1),
        "DEP-002: Config network coin launcher ID must match derivation"
    );
    assert_eq!(
        config.checkpoint_launcher_id,
        derive_launcher_id(chk_coin.coin_id(), 1),
        "DEP-002: Config checkpoint launcher ID must match derivation"
    );

    Ok(())
}

// ── Both singletons created in same spend bundle (atomicity) ────────

#[test]
fn vv_req_dep_002_both_created_in_same_block() -> anyhow::Result<()> {
    use chia_l2_consensus::testing::{
        deserialize_proving_key, extract_vk_components, run_test_setup,
    };

    let mut sim = Simulator::new();
    let ctx = &mut SpendContext::new();

    let (sk1, pk1, _, net_coin) = sim.new_p2(1)?;
    let (sk2, pk2, _, chk_coin) = sim.new_p2(1)?;

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk_components = extract_vk_components(&pk.vk).expect("VK");

    let genesis_challenge = chia_protocol::Bytes32::from([0x00; 32]);

    // Before deploy: neither singleton exists
    let net_launcher_id = derive_launcher_id(net_coin.coin_id(), 1);
    let chk_launcher_id = derive_launcher_id(chk_coin.coin_id(), 1);
    assert!(
        sim.coin_state(net_launcher_id).is_none(),
        "DEP-002: Network launcher must not exist before deploy"
    );
    assert!(
        sim.coin_state(chk_launcher_id).is_none(),
        "DEP-002: Checkpoint launcher must not exist before deploy"
    );

    let (net_singleton, chk_singleton, _) = deploy_both_singletons(
        ctx,
        net_coin,
        pk1,
        chk_coin,
        pk2,
        &vk_components,
        10_000_000_000_000,
        32,
        genesis_challenge,
    )?;

    // Single submit creates both atomically
    sim.spend_coins(ctx.take(), &[sk1, sk2])?;

    // After deploy: both singletons exist
    assert!(
        sim.coin_state(net_singleton.coin_id()).is_some(),
        "DEP-002: Network singleton must exist after atomic deploy"
    );
    assert!(
        sim.coin_state(chk_singleton.coin_id()).is_some(),
        "DEP-002: Checkpoint singleton must exist after atomic deploy"
    );

    // Both funding coins are spent
    let net_state = sim.coin_state(net_coin.coin_id()).unwrap();
    assert!(
        net_state.spent_height.is_some(),
        "DEP-002: Network funding coin must be spent"
    );
    let chk_state = sim.coin_state(chk_coin.coin_id()).unwrap();
    assert!(
        chk_state.spent_height.is_some(),
        "DEP-002: Checkpoint funding coin must be spent"
    );

    Ok(())
}

// ── NetworkConfig has all required fields populated ─────────────────

#[test]
fn vv_req_dep_002_config_fields_populated() -> anyhow::Result<()> {
    use chia_l2_consensus::testing::{
        deserialize_proving_key, extract_vk_components, run_test_setup,
    };

    let mut sim = Simulator::new();
    let ctx = &mut SpendContext::new();

    let (sk1, pk1, _, net_coin) = sim.new_p2(1)?;
    let (sk2, pk2, _, chk_coin) = sim.new_p2(1)?;

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk_components = extract_vk_components(&pk.vk).expect("VK");

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
    )?;

    sim.spend_coins(ctx.take(), &[sk1, sk2])?;

    // Verify all fields
    assert_ne!(
        config.network_coin_launcher_id,
        chia_protocol::Bytes32::default(),
        "DEP-002: network_coin_launcher_id must be non-zero"
    );
    assert_ne!(
        config.checkpoint_launcher_id,
        chia_protocol::Bytes32::default(),
        "DEP-002: checkpoint_launcher_id must be non-zero"
    );
    assert_ne!(
        config.network_coin_launcher_id, config.checkpoint_launcher_id,
        "DEP-002: Both launcher IDs must be different"
    );
    assert_ne!(
        config.registration_coin_mod_hash,
        chia_protocol::Bytes32::default(),
        "DEP-002: registration_coin_mod_hash must be non-zero"
    );
    assert_ne!(
        config.checkpoint_inner_mod_hash,
        chia_protocol::Bytes32::default(),
        "DEP-002: checkpoint_inner_mod_hash must be non-zero"
    );
    assert_eq!(
        config.collateral_amount, 10_000_000_000_000,
        "DEP-002: collateral_amount must match deployment param"
    );
    assert_eq!(
        config.tree_depth, 32,
        "DEP-002: tree_depth must match deployment param"
    );
    assert_eq!(
        config.genesis_challenge, genesis_challenge,
        "DEP-002: genesis_challenge must match deployment param"
    );
    assert!(
        !config.verification_key_hex.is_empty(),
        "DEP-002: verification_key_hex must be non-empty"
    );

    Ok(())
}

// ── Launcher IDs are predictable before deployment ──────────────────

#[test]
fn vv_req_dep_002_ids_predictable_before_deploy() -> anyhow::Result<()> {
    use chia_l2_consensus::testing::{
        deserialize_proving_key, extract_vk_components, run_test_setup,
    };

    let mut sim = Simulator::new();

    let (sk1, pk1, _, net_coin) = sim.new_p2(1)?;
    let (sk2, pk2, _, chk_coin) = sim.new_p2(1)?;

    // Predict IDs BEFORE deployment
    let predicted_net_id = derive_launcher_id(net_coin.coin_id(), 1);
    let predicted_chk_id = derive_launcher_id(chk_coin.coin_id(), 1);

    // Now deploy
    let ctx = &mut SpendContext::new();
    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk_components = extract_vk_components(&pk.vk).expect("VK");

    let (_, _, config) = deploy_both_singletons(
        ctx,
        net_coin,
        pk1,
        chk_coin,
        pk2,
        &vk_components,
        10_000_000_000_000,
        32,
        chia_protocol::Bytes32::from([0x00; 32]),
    )?;

    sim.spend_coins(ctx.take(), &[sk1, sk2])?;

    assert_eq!(
        config.network_coin_launcher_id, predicted_net_id,
        "DEP-002: Network coin launcher ID must be predictable before deploy"
    );
    assert_eq!(
        config.checkpoint_launcher_id, predicted_chk_id,
        "DEP-002: Checkpoint launcher ID must be predictable before deploy"
    );

    Ok(())
}

// ── Both singletons have amount 1 (singleton convention) ────────────

#[test]
fn vv_req_dep_002_singletons_have_amount_1() -> anyhow::Result<()> {
    use chia_l2_consensus::testing::{
        deserialize_proving_key, extract_vk_components, run_test_setup,
    };

    let mut sim = Simulator::new();
    let ctx = &mut SpendContext::new();

    let (sk1, pk1, _, net_coin) = sim.new_p2(1)?;
    let (sk2, pk2, _, chk_coin) = sim.new_p2(1)?;

    let (pk_bytes, _) = run_test_setup().expect("Setup");
    let pk = deserialize_proving_key(&pk_bytes).expect("PK");
    let vk_components = extract_vk_components(&pk.vk).expect("VK");

    let (net_singleton, chk_singleton, _) = deploy_both_singletons(
        ctx,
        net_coin,
        pk1,
        chk_coin,
        pk2,
        &vk_components,
        10_000_000_000_000,
        32,
        chia_protocol::Bytes32::from([0x00; 32]),
    )?;

    sim.spend_coins(ctx.take(), &[sk1, sk2])?;

    assert_eq!(
        net_singleton.amount, 1,
        "DEP-002: Network coin singleton must have amount 1"
    );
    assert_eq!(
        chk_singleton.amount, 1,
        "DEP-002: Checkpoint singleton must have amount 1"
    );

    Ok(())
}
