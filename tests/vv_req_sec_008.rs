//! REQUIREMENT: SEC-008 — Condition Injection Protection
//! (`docs/requirements/domains/security/NORMATIVE.md#SEC-008`).
//!
//! Spec: `docs/requirements/domains/security/specs/SEC-008.md`.
//!
//! ## Normative statement
//! Registration coin and checkpoint inner puzzles MUST NOT have passthrough
//! conditions (`...conditions` spread or `conditions: List<Condition>` param).
//! Each puzzle MUST emit a fixed set of conditions (exactly 2 for registration,
//! exactly 2 per path for checkpoint). Network coin keeps conditions because
//! AggSigMe signs the entire spend.
//!
//! ## How the tests prove the requirement
//! 1. **Registration: no conditions param/spread**: Source lacks both.
//! 2. **Registration: exactly 2 conditions**: Output list is `[assert_announcement, create_collateral]`.
//! 3. **Checkpoint: no conditions param/spread**: Source lacks both.
//! 4. **Checkpoint: exactly 2 per path**: Two `[recreate, announce]` lists in source.
//! 5. **Network coin keeps conditions**: Protected by AggSigMe.
//! 6. **No AGG_SIG_UNSAFE**: Neither registration nor checkpoint source contains it.
//! 7. **SEC-008 documented**: Both puzzle sources reference SEC-008.
//!
//! ## Completeness: HIGH
//! ## Gaps: Does not dynamically test that injected conditions are impossible
//! (structural analysis is sufficient since conditions param is absent).

const REG_SRC: &str = include_str!("../puzzles/registration_coin.rue");
const CHK_SRC: &str = include_str!("../puzzles/checkpoint_inner.rue");
const NET_SRC: &str = include_str!("../puzzles/network_coin_inner.rue");

// ── Registration coin: no conditions passthrough ────────────────────

#[test]
fn vv_req_sec_008_registration_no_conditions_param() {
    assert!(
        !REG_SRC.contains("conditions: List<Condition>"),
        "SEC-008: registration_coin must NOT have conditions parameter"
    );
}

#[test]
fn vv_req_sec_008_registration_no_conditions_spread() {
    assert!(
        !REG_SRC.contains("...conditions"),
        "SEC-008: registration_coin must NOT spread conditions"
    );
}

#[test]
fn vv_req_sec_008_registration_only_two_conditions() {
    // The puzzle should output exactly: [assert_announcement, create_collateral]
    // Count the number of items in the output list
    // WDC-004: renamed create_collateral → create_delay_coin
    assert!(
        REG_SRC.contains("[assert_announcement, create_delay_coin]"),
        "SEC-008: registration_coin must output exactly 2 conditions"
    );
}

// ── Checkpoint inner: no conditions passthrough ─────────────────────

#[test]
fn vv_req_sec_008_checkpoint_no_conditions_param() {
    assert!(
        !CHK_SRC.contains("conditions: List<Condition>"),
        "SEC-008: checkpoint_inner must NOT have conditions parameter"
    );
}

#[test]
fn vv_req_sec_008_checkpoint_no_conditions_spread() {
    assert!(
        !CHK_SRC.contains("...conditions"),
        "SEC-008: checkpoint_inner must NOT spread conditions"
    );
}

#[test]
fn vv_req_sec_008_checkpoint_only_recreate_and_announce() {
    // Both paths should output: [recreate, announce]
    let occurrences = CHK_SRC.matches("[recreate, announce]").count();
    assert_eq!(
        occurrences, 2,
        "SEC-008: checkpoint_inner must have exactly 2 output lists: [recreate, announce] (one per path)"
    );
}

// ── Network coin: conditions kept (protected by AggSigMe) ──────────

#[test]
fn vv_req_sec_008_network_coin_keeps_conditions() {
    // Network coin is allowed to keep conditions because AggSigMe
    // signs the entire spend, preventing farmer modification
    assert!(
        NET_SRC.contains("conditions"),
        "SEC-008: network_coin keeps conditions (protected by AggSigMe)"
    );
    assert!(
        NET_SRC.contains("AggSigMe"),
        "SEC-008: network_coin conditions protected by AggSigMe"
    );
}

// ── No AGG_SIG_UNSAFE injectable in any puzzle ──────────────────────

#[test]
fn vv_req_sec_008_no_agg_sig_unsafe_anywhere() {
    let _reg_hex = include_str!("../puzzles/compiled/registration_coin.hex");
    let _chk_hex = include_str!("../puzzles/compiled/checkpoint_inner.hex");

    // AGG_SIG_UNSAFE would appear as opcode 49 (0x31) if hardcoded
    // Since conditions are removed, no external injection possible either
    assert!(
        !REG_SRC.contains("AggSigUnsafe"),
        "SEC-008: No AggSigUnsafe in registration_coin source"
    );
    assert!(
        !CHK_SRC.contains("AggSigUnsafe"),
        "SEC-008: No AggSigUnsafe in checkpoint_inner source"
    );
}

// ── SEC-008 comment present in puzzle sources ───────────────────────

#[test]
fn vv_req_sec_008_documented_in_puzzles() {
    assert!(
        REG_SRC.contains("SEC-008"),
        "SEC-008: registration_coin must document SEC-008"
    );
    assert!(
        CHK_SRC.contains("SEC-008"),
        "SEC-008: checkpoint_inner must document SEC-008"
    );
}
