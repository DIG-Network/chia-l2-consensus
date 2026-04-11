//! REQUIREMENT: IDX-005 — Persistent Cache
//! (`docs/requirements/domains/indexer/NORMATIVE.md#IDX-005`).
//!
//! Spec: `docs/requirements/domains/indexer/specs/IDX-005.md`.
//!
//! Implementation: `src/indexer/cache.rs`.
//!
//! ## Normative statement
//! The indexer MUST maintain a persistent JSON cache using atomic writes
//! (write to tmp file, then rename) for crash-safe restarts. The cache MUST
//! support save/load roundtrip, handle missing files (return None), detect
//! corrupted files (return Err), and work in memory-only mode.
//!
//! ## How the tests prove the requirement
//! 1. **Save and load roundtrip**: Height and checkpoint records survive.
//! 2. **Missing file returns None**: No panic, no error.
//! 3. **JSON format**: Human-readable, valid JSON with field names.
//! 4. **Atomic write**: No .tmp file remains after save; overwrite works.
//! 5. **Corrupted file returns error**: Invalid JSON detected.
//! 6. **In-memory mode**: Works without disk access.
//! 7. **Spec exists**: IDX-005.md on disk.
//!
//! ## Completeness: HIGH
//! ## Gaps: Does not test crash mid-write recovery (OS-level atomic rename).

use std::path::Path;

// ── IDX-005: Round-trip save and load ─────────────────────────────────

#[test]
fn vv_req_idx_005_save_and_load() {
    use chia_l2_consensus::testing::IndexerCache;

    let dir = std::env::temp_dir().join("idx005_save_load");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cache.json");

    let mut cache = IndexerCache::in_memory();
    cache.set_last_synced_height(42);
    cache.add_checkpoint_record(5, 100);
    cache.add_checkpoint_record(6, 120);

    cache
        .save(path.to_str().unwrap())
        .expect("save must succeed");
    assert!(path.exists(), "IDX-005: Cache file must be created");

    let loaded = IndexerCache::load(path.to_str().unwrap())
        .expect("load must succeed")
        .expect("file exists so must return Some");

    assert_eq!(loaded.last_synced_height(), 42);
    assert_eq!(loaded.checkpoint_count(), 2);

    let _ = std::fs::remove_dir_all(&dir);
}

// ── IDX-005: Load returns None when file missing ──────────────────────

#[test]
fn vv_req_idx_005_load_missing_file() {
    use chia_l2_consensus::testing::IndexerCache;

    let result = IndexerCache::load("/tmp/nonexistent_idx005_cache.json");
    assert!(result.is_ok());
    assert!(
        result.unwrap().is_none(),
        "IDX-005: Missing file must return None"
    );
}

// ── IDX-005: JSON format human-readable ───────────────────────────────

#[test]
fn vv_req_idx_005_json_format() {
    use chia_l2_consensus::testing::IndexerCache;

    let dir = std::env::temp_dir().join("idx005_json");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cache.json");

    let mut cache = IndexerCache::in_memory();
    cache.set_last_synced_height(99);
    cache.save(path.to_str().unwrap()).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(
        contents.contains("last_synced_height"),
        "IDX-005: JSON must contain field names"
    );
    assert!(
        contents.contains("99"),
        "IDX-005: JSON must contain the height value"
    );
    // Must be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("must be valid JSON");
    assert!(parsed.is_object());

    let _ = std::fs::remove_dir_all(&dir);
}

// ── IDX-005: Atomic write (tmp file then rename) ──────────────────────

#[test]
fn vv_req_idx_005_atomic_write() {
    use chia_l2_consensus::testing::IndexerCache;

    let dir = std::env::temp_dir().join("idx005_atomic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cache.json");

    let mut cache = IndexerCache::in_memory();
    cache.set_last_synced_height(10);
    cache.save(path.to_str().unwrap()).unwrap();

    // Overwrite with new data — should not leave tmp files
    cache.set_last_synced_height(20);
    cache.save(path.to_str().unwrap()).unwrap();

    let tmp_path = dir.join("cache.json.tmp");
    assert!(
        !tmp_path.exists(),
        "IDX-005: Temp file must not remain after save"
    );

    let loaded = IndexerCache::load(path.to_str().unwrap()).unwrap().unwrap();
    assert_eq!(loaded.last_synced_height(), 20);

    let _ = std::fs::remove_dir_all(&dir);
}

// ── IDX-005: Corrupted file handled gracefully ────────────────────────

#[test]
fn vv_req_idx_005_corrupted_file() {
    use chia_l2_consensus::testing::IndexerCache;

    let dir = std::env::temp_dir().join("idx005_corrupt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cache.json");

    std::fs::write(&path, "NOT VALID JSON {{{").unwrap();

    let result = IndexerCache::load(path.to_str().unwrap());
    assert!(result.is_err(), "IDX-005: Corrupted file must return error");

    let _ = std::fs::remove_dir_all(&dir);
}

// ── IDX-005: In-memory cache works without disk ───────────────────────

#[test]
fn vv_req_idx_005_in_memory() {
    use chia_l2_consensus::testing::IndexerCache;

    let mut cache = IndexerCache::in_memory();
    assert_eq!(cache.last_synced_height(), 0);
    assert_eq!(cache.checkpoint_count(), 0);

    cache.set_last_synced_height(50);
    cache.add_checkpoint_record(1, 25);

    assert_eq!(cache.last_synced_height(), 50);
    assert_eq!(cache.checkpoint_count(), 1);
}

// ── Spec ───────────────────────────────────────────────────────────

#[test]
fn vv_req_idx_005_spec_exists() {
    assert!(Path::new("docs/requirements/domains/indexer/specs/IDX-005.md").exists());
}
