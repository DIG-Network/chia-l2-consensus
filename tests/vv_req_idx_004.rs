//! REQUIREMENT: IDX-004 — Reorg Handling
//! (`docs/requirements/domains/indexer/NORMATIVE.md#IDX-004`).
//!
//! Spec: `docs/requirements/domains/indexer/specs/IDX-004.md`.
//!
//! Implementation: `src/indexer/reorg.rs`.
//!
//! ## Normative statement
//! On blockchain reorganization (peak < last_synced_height), the indexer MUST
//! roll back to the last safe checkpoint before the reorg point and re-index
//! forward. If no safe checkpoint exists, full re-index from genesis MUST
//! be performed. Rollback MUST truncate checkpoint history, clear registration
//! coins, and reset last_synced_height.
//!
//! ## How the tests prove the requirement
//! 1. **Reorg detected**: peak < synced = reorg; peak >= synced = not.
//! 2. **Rollback to safe checkpoint**: 3 checkpoints, reorg at 75 -> rolls
//!    back to epoch 2 (height 60), truncates history, clears registrations.
//! 3. **Full re-index when no checkpoints**: No safe point -> height=0,
//!    everything cleared.
//! 4. **Reorg before all checkpoints**: Reorg at height 20 (before all) ->
//!    no safe point.
//! 5. **State consistent after rollback**: New data can be added after rollback.
//! 6. **Spec exists**: IDX-004.md on disk.
//!
//! ## Completeness: HIGH
//! ## Gaps: Does not test against real blockchain reorg events.

use chia_protocol::Bytes32;

// ── IDX-004: Reorg detection ──────────────────────────────────────────

#[test]
fn vv_req_idx_004_reorg_detected() {
    // A reorg is detected when peak < last_synced_height.
    use chia_l2_consensus::testing::ReorgState;

    let mut state = ReorgState::new();
    state.set_last_synced_height(100);

    assert!(
        state.is_reorg(90),
        "IDX-004: peak 90 < synced 100 must detect reorg"
    );
    assert!(
        !state.is_reorg(100),
        "IDX-004: peak == synced must NOT be a reorg"
    );
    assert!(
        !state.is_reorg(110),
        "IDX-004: peak > synced must NOT be a reorg"
    );
}

// ── IDX-004: Rollback to safe checkpoint ──────────────────────────────

#[test]
fn vv_req_idx_004_rollback_to_safe_checkpoint() {
    // When a checkpoint exists before the reorg point, roll back to it.
    use chia_l2_consensus::testing::{CheckpointRecord, ReorgState};

    let mut state = ReorgState::new();
    state.set_last_synced_height(100);

    // Add checkpoint history
    state.record_checkpoint(CheckpointRecord {
        epoch: 1,
        confirmed_at_height: 30,
    });
    state.record_checkpoint(CheckpointRecord {
        epoch: 2,
        confirmed_at_height: 60,
    });
    state.record_checkpoint(CheckpointRecord {
        epoch: 3,
        confirmed_at_height: 90,
    });

    // Add some registration coins
    state.record_registration(Bytes32::from([0xAA; 32]));
    state.record_registration(Bytes32::from([0xBB; 32]));
    assert_eq!(state.registration_count(), 2);

    // Reorg to height 75 — should roll back to epoch 2 (height 60)
    let rollback = state.compute_rollback(75);
    assert!(rollback.is_some(), "IDX-004: Must find safe checkpoint");
    let safe = rollback.unwrap();
    assert_eq!(safe.epoch, 2);
    assert_eq!(safe.confirmed_at_height, 60);

    // Apply the rollback
    state.apply_rollback(&safe);

    // Checkpoint history truncated to epoch <= 2
    assert_eq!(state.checkpoint_count(), 2);
    // Registration coins cleared (will be re-indexed)
    assert_eq!(state.registration_count(), 0);
    // Last synced height set to safe checkpoint height
    assert_eq!(state.last_synced_height(), 60);
}

// ── IDX-004: Full re-index when no safe checkpoint ────────────────────

#[test]
fn vv_req_idx_004_full_reindex_no_checkpoints() {
    // When no checkpoint exists before the reorg point, full re-index.
    use chia_l2_consensus::testing::ReorgState;

    let mut state = ReorgState::new();
    state.set_last_synced_height(50);
    state.record_registration(Bytes32::from([0xCC; 32]));

    // No checkpoints recorded — reorg to any height triggers full re-index
    let rollback = state.compute_rollback(30);
    assert!(
        rollback.is_none(),
        "IDX-004: No checkpoints → no safe point"
    );

    // Apply full re-index
    state.apply_full_reindex();

    assert_eq!(state.last_synced_height(), 0);
    assert_eq!(state.checkpoint_count(), 0);
    assert_eq!(state.registration_count(), 0);
}

// ── IDX-004: Reorg before all checkpoints ─────────────────────────────

#[test]
fn vv_req_idx_004_reorg_before_all_checkpoints() {
    // Reorg to a height before ALL recorded checkpoints → full re-index.
    use chia_l2_consensus::testing::{CheckpointRecord, ReorgState};

    let mut state = ReorgState::new();
    state.set_last_synced_height(100);
    state.record_checkpoint(CheckpointRecord {
        epoch: 1,
        confirmed_at_height: 50,
    });
    state.record_checkpoint(CheckpointRecord {
        epoch: 2,
        confirmed_at_height: 80,
    });

    // Reorg to height 20 — before all checkpoints
    let rollback = state.compute_rollback(20);
    assert!(
        rollback.is_none(),
        "IDX-004: Reorg before all checkpoints → no safe point"
    );
}

// ── IDX-004: State consistent after rollback ──────────────────────────

#[test]
fn vv_req_idx_004_state_consistent_after_rollback() {
    // After rollback, adding new data should work correctly.
    use chia_l2_consensus::testing::{CheckpointRecord, ReorgState};

    let mut state = ReorgState::new();
    state.set_last_synced_height(100);
    state.record_checkpoint(CheckpointRecord {
        epoch: 1,
        confirmed_at_height: 40,
    });
    state.record_checkpoint(CheckpointRecord {
        epoch: 2,
        confirmed_at_height: 70,
    });

    // Rollback to height 50 → epoch 1
    let safe = state.compute_rollback(50).unwrap();
    state.apply_rollback(&safe);
    assert_eq!(state.last_synced_height(), 40);
    assert_eq!(state.checkpoint_count(), 1);

    // Now "re-index" by adding new data
    state.set_last_synced_height(80);
    state.record_checkpoint(CheckpointRecord {
        epoch: 2,
        confirmed_at_height: 75,
    });
    state.record_registration(Bytes32::from([0xDD; 32]));

    assert_eq!(state.checkpoint_count(), 2);
    assert_eq!(state.registration_count(), 1);
    assert_eq!(state.last_synced_height(), 80);
}

// ── Spec ───────────────────────────────────────────────────────────

#[test]
fn vv_req_idx_004_spec_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/indexer/specs/IDX-004.md").exists());
}
