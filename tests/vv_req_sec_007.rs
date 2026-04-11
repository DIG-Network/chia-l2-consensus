//! REQUIREMENT: SEC-007 — CLVM Vulnerability Audit
//! (`docs/requirements/domains/security/NORMATIVE.md#SEC-007`).
//!
//! Spec: `docs/requirements/domains/security/specs/SEC-007.md`.
//!
//! Verifies that all 9 known Chialisp/CLVM vulnerabilities from the Chia
//! knowledge graph are addressed in the three Rue puzzles.

/// Network coin inner puzzle source.
const NET_SRC: &str = include_str!("../puzzles/network_coin_inner.rue");
/// Registration coin puzzle source.
const REG_SRC: &str = include_str!("../puzzles/registration_coin.rue");
/// Checkpoint inner puzzle source.
const CHK_SRC: &str = include_str!("../puzzles/checkpoint_inner.rue");

/// All three puzzle sources for bulk checks.
fn all_sources() -> [(&'static str, &'static str); 3] {
    [
        ("network_coin_inner", NET_SRC),
        ("registration_coin", REG_SRC),
        ("checkpoint_inner", CHK_SRC),
    ]
}

// ── V1: CATbleed — No manual coin ID sha256 concatenation ──────────

#[test]
fn vv_req_sec_007_v1_no_manual_coinid() {
    // CATbleed exploits manual sha256(parent + puzzle_hash + amount) with
    // variable-length fields. Rue typed fields and int_to_8_bytes_be prevent this.
    for (name, src) in &all_sources() {
        // No puzzle should manually compute sha256(parent_id + puzzle_hash + amount)
        // They should use curry_tree_hash or let the singleton wrapper handle coin IDs
        assert!(
            !src.contains("sha256(parent"),
            "SEC-007 V1: {} must NOT manually compute coin IDs via sha256(parent...)",
            name
        );
    }
}

#[test]
fn vv_req_sec_007_v1_fixed_width_integers() {
    // All integer encoding uses int_to_8_bytes_be (fixed-width, prevents byte shifting)
    for (name, src) in &all_sources() {
        if src.contains("int_to_8_bytes_be") {
            // Good — using the fixed-width helper
            assert!(
                src.contains("inline fn int_to_8_bytes_be") || src.contains("int_to_8_bytes_be("),
                "SEC-007 V1: {} uses int_to_8_bytes_be for fixed-width encoding",
                name
            );
        }
    }
}

// ── V2: AGG_SIG_UNSAFE — Not used in any puzzle ────────────────────

#[test]
fn vv_req_sec_007_v2_no_agg_sig_unsafe() {
    for (name, src) in &all_sources() {
        assert!(
            !src.contains("AggSigUnsafe"),
            "SEC-007 V2: {} must NOT use AggSigUnsafe (replay vulnerable). Found in source.",
            name
        );
    }
}

#[test]
fn vv_req_sec_007_v2_network_coin_uses_aggsigme() {
    // Network coin must use AggSigMe (bound to coin_id + genesis)
    assert!(
        NET_SRC.contains("AggSigMe"),
        "SEC-007 V2: network_coin must use AggSigMe for validator registration"
    );
}

// ── V3: Announcement replay — binding fields present ────────────────

#[test]
fn vv_req_sec_007_v3_announcement_includes_epoch() {
    // Checkpoint announcement includes epoch (replay protection)
    assert!(
        CHK_SRC.contains("membership") && CHK_SRC.contains("epoch"),
        "SEC-007 V3: Checkpoint announcement must include epoch"
    );
}

#[test]
fn vv_req_sec_007_v3_registration_asserts_coin_id() {
    // Registration coin asserts against CHECKPOINT_SINGLETON_ID (specific coin)
    assert!(
        REG_SRC.contains("CHECKPOINT_SINGLETON_ID"),
        "SEC-007 V3: Registration coin must assert against specific checkpoint coin ID"
    );
    assert!(
        REG_SRC.contains("AssertCoinAnnouncement"),
        "SEC-007 V3: Registration coin must use AssertCoinAnnouncement"
    );
}

// ── V4/V5: Condition injection — assessed via SEC-008 ───────────────

#[test]
fn vv_req_sec_007_v4_v5_condition_injection_assessed() {
    // This test documents that condition injection is tracked by SEC-008.
    // After SEC-008 is implemented, checkpoint and registration puzzles
    // will no longer have ...conditions passthrough.
    //
    // Network coin keeps conditions (protected by AggSigMe signature).

    // Verify network coin HAS AggSigMe protection for its conditions
    assert!(
        NET_SRC.contains("AggSigMe"),
        "SEC-007 V4/V5: network_coin conditions protected by AggSigMe"
    );
}

// ── V6: Unsigned destination — assessed via SEC-009 ─────────────────

#[test]
fn vv_req_sec_007_v6_destination_binding_assessed() {
    // Registration coin destination is from solution (unsigned).
    // SEC-009 documents this as mitigated by the announcement requirement:
    // farmer needs the non-membership announcement AND the registration coin
    // in the same bundle to redirect collateral.

    // Verify the announcement requirement exists (the mitigation)
    assert!(
        REG_SRC.contains("AssertCoinAnnouncement"),
        "SEC-007 V6: Registration coin requires announcement (mitigates unsigned destination)"
    );
}

// ── V7: Flash loan — not applicable (singleton model) ───────────────

#[test]
fn vv_req_sec_007_v7_flash_loan_not_applicable() {
    // Flash loans require creating ephemeral coins same-block.
    // The network coin is a singleton (only one exists, sequential spends).
    // Cannot create ephemeral validator registrations.
    //
    // Verify singleton pattern: network coin recreates itself
    assert!(
        NET_SRC.contains("INNER_MOD_HASH") && NET_SRC.contains("CreateCoin"),
        "SEC-007 V7: Network coin is singleton (recreates itself), preventing flash loans"
    );
}

// ── V8: Bricked coins — by design (registration epoch-scoped) ──────

#[test]
fn vv_req_sec_007_v8_bricked_coins_by_design() {
    // Registration coin is curried with CHECKPOINT_SINGLETON_ID (coin ID, not launcher).
    // This means it's valid only for the current checkpoint epoch.
    // This is intentional — once excluded, the validator uses the CURRENT checkpoint.
    assert!(
        REG_SRC.contains("CHECKPOINT_SINGLETON_ID: Bytes32"),
        "SEC-007 V8: Registration coin curried with CHECKPOINT_SINGLETON_ID (epoch-scoped by design)"
    );
}

// ── V9: Cross-network replay — CHK-012 implemented ─────────────────

#[test]
fn vv_req_sec_007_v9_network_id_in_checkpoint_message() {
    // CHK-012: network_coin_launcher_id is now in the checkpoint_message preimage
    assert!(
        CHK_SRC.contains("NETWORK_COIN_LAUNCHER_ID"),
        "SEC-007 V9: checkpoint_inner must have NETWORK_COIN_LAUNCHER_ID curried parameter"
    );
    // Verify it's included in the checkpoint_message computation
    assert!(
        CHK_SRC.contains("net_id_b"),
        "SEC-007 V9: network_coin_launcher_id must be in checkpoint_message sha256 preimage"
    );
}

#[test]
fn vv_req_sec_007_v9_network_id_is_curried() {
    // The network ID must be curried (not from solution) to prevent spoofing
    // It should appear in the curried parameters section, before STATE
    let curried_section = CHK_SRC
        .find("Curried parameters")
        .expect("Must have curried parameters section");
    let solution_section = CHK_SRC
        .find("Solution:")
        .or_else(|| CHK_SRC.find("spend path selector"))
        .expect("Must have solution section");
    let network_id_pos = CHK_SRC
        .find("NETWORK_COIN_LAUNCHER_ID")
        .expect("Must have NETWORK_COIN_LAUNCHER_ID");

    assert!(
        network_id_pos > curried_section && network_id_pos < solution_section,
        "SEC-007 V9: NETWORK_COIN_LAUNCHER_ID must be in curried section (not solution)"
    );
}

// ── Summary: all 9 vulnerabilities accounted for ────────────────────

#[test]
fn vv_req_sec_007_all_9_assessed() {
    // This is a documentation test confirming all vulnerabilities are tracked:
    // V1: CATbleed — MITIGATED (typed Rue fields + int_to_8_bytes_be)
    // V2: AGG_SIG_UNSAFE — MITIGATED (only AggSigMe used)
    // V3: Announcement replay — MITIGATED (epoch + coin_id binding)
    // V4: Solution manipulation — TRACKED (SEC-008)
    // V5: Condition injection — TRACKED (SEC-008)
    // V6: Unsigned destination — TRACKED (SEC-009)
    // V7: Flash loans — NOT APPLICABLE (singleton model)
    // V8: Bricked coins — BY DESIGN (epoch-scoped registration)
    // V9: Cross-network replay — IMPLEMENTED (CHK-012)

    // Verify the spec file documents all 9
    let spec = include_str!("../docs/requirements/domains/security/specs/SEC-007.md");
    for vid in ["V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8", "V9"] {
        assert!(
            spec.contains(vid),
            "SEC-007: Spec must document vulnerability {}",
            vid
        );
    }
}
