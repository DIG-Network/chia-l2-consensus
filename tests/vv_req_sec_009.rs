//! REQUIREMENT: SEC-009 — Registration Coin Destination Binding
//! (`docs/requirements/domains/security/NORMATIVE.md#SEC-009`).
//!
//! Spec: `docs/requirements/domains/security/specs/SEC-009.md`.
//!
//! The collateral_destination in the registration coin solution is unsigned.
//! This is mitigated by the ASSERT_COIN_ANNOUNCEMENT requirement: a farmer
//! attacking via RBF must reconstruct the non-membership announcement from
//! the checkpoint singleton in the same spend bundle, which requires knowing
//! the specific checkpoint coin ID (changes every epoch).

const REG_SRC: &str = include_str!("../puzzles/registration_coin.rue");
const REG_HEX: &str = include_str!("../puzzles/compiled/registration_coin.hex");

// ── Announcement requirement is mandatory (the mitigation) ──────────

#[test]
fn vv_req_sec_009_announcement_is_mandatory() {
    // The non-membership announcement assertion is the security mechanism
    // that makes destination manipulation impractical
    assert!(
        REG_SRC.contains("AssertCoinAnnouncement"),
        "SEC-009: Registration coin MUST require AssertCoinAnnouncement"
    );
}

#[test]
fn vv_req_sec_009_announcement_hardcoded_not_optional() {
    // The assertion is unconditional — always emitted, no bypass path
    let count = REG_SRC.matches("AssertCoinAnnouncement").count();
    assert_eq!(
        count, 1,
        "SEC-009: Exactly one AssertCoinAnnouncement (unconditional, no bypass)"
    );
}

// ── Announcement binds to specific checkpoint coin ID ───────────────

#[test]
fn vv_req_sec_009_bound_to_checkpoint_coin_id() {
    // CHECKPOINT_SINGLETON_ID is curried (not from solution)
    // This means the announcement must come from a SPECIFIC checkpoint coin
    assert!(
        REG_SRC.contains("CHECKPOINT_SINGLETON_ID: Bytes32"),
        "SEC-009: CHECKPOINT_SINGLETON_ID must be curried parameter"
    );
}

#[test]
fn vv_req_sec_009_checkpoint_id_in_announcement_hash() {
    // The full announcement hash includes the checkpoint coin ID
    // sha256(CHECKPOINT_SINGLETON_ID + inner_announcement)
    assert!(
        REG_SRC.contains("checkpoint_bytes + expected_announcement"),
        "SEC-009: Announcement hash must include checkpoint coin ID"
    );
}

// ── Destination is from solution (acknowledged design) ──────────────

#[test]
fn vv_req_sec_009_destination_from_solution() {
    // This is the acknowledged design trade-off
    assert!(
        REG_SRC.contains("collateral_destination: Bytes32"),
        "SEC-009: collateral_destination is from solution (acknowledged)"
    );
}

// ── No conditions passthrough (SEC-008 mitigates injection) ─────────

#[test]
fn vv_req_sec_009_no_extra_create_coin_possible() {
    // SEC-008 removed conditions passthrough, so attacker can't inject
    // additional CREATE_COIN to redirect value
    assert!(
        !REG_SRC.contains("...conditions"),
        "SEC-009: No conditions passthrough means no extra CREATE_COIN injection"
    );
}

// ── Risk documentation present in spec ──────────────────────────────

#[test]
fn vv_req_sec_009_risk_documented() {
    let spec = include_str!("../docs/requirements/domains/security/specs/SEC-009.md");

    assert!(
        spec.contains("RBF") || spec.contains("Replace-By-Fee") || spec.contains("mempool"),
        "SEC-009: Spec must document the RBF attack scenario"
    );
    assert!(
        spec.contains("mitigated") || spec.contains("impractical"),
        "SEC-009: Spec must document the mitigation"
    );
}

// ── Compiled puzzle has ASSERT_COIN_ANNOUNCEMENT opcode ─────────────

#[test]
fn vv_req_sec_009_compiled_has_assertion() {
    // ASSERT_COIN_ANNOUNCEMENT = opcode 61 = 0x3d, appears as ff013d (quoted atom)
    assert!(
        REG_HEX.contains("013d"),
        "SEC-009: Compiled puzzle must contain ASSERT_COIN_ANNOUNCEMENT (opcode 61)"
    );
}
