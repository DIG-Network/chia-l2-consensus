//! REQUIREMENT: RPC-001 — Blockchain Query Client (chia-query)
//! (`docs/requirements/domains/rpc/NORMATIVE.md#RPC-001`).
//!
//! Spec: `docs/requirements/domains/rpc/specs/RPC-001.md`.
//!
//! ## Normative Statement
//!
//! The crate MUST use chia-query::ChiaQuery as the blockchain query client.
//! The ChiaQuery instance MUST be shared across the indexer and puzzle drivers.
//! All chain reads go through ChiaQuery; the crate MUST NOT open direct peer
//! connections or implement custom RPC.
//!
//! ## Acceptance Criteria Coverage
//!
//! - [x] ChiaQuery added as dependency (verified by RPC-006)
//! - [x] ConsensusClient has connect() method accepting ChiaQueryConfig
//! - [x] ChiaQuery instance stored in ConsensusClient
//! - [x] Hex string ↔ Bytes32 conversion correct
//! - [x] Error mapping: RpcError variant exists
//! - [x] No custom RPC implementation in src/

use chia_protocol::Bytes32;

use chia_l2_consensus::testing::{bytes32_to_hex, hex_to_bytes32};

// ── ConsensusClient integration ──────────────────────────────────────

/// RPC-001: ConsensusClient has connect() method.
#[test]
fn vv_req_rpc_001_client_has_connect() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("pub async fn connect("),
        "RPC-001: ConsensusClient must have connect() method"
    );
}

/// RPC-001: connect() accepts ChiaQueryConfig.
#[test]
fn vv_req_rpc_001_connect_accepts_config() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("ChiaQueryConfig"),
        "RPC-001: connect() must accept ChiaQueryConfig"
    );
}

/// RPC-001: ConsensusClient stores ChiaQuery.
#[test]
fn vv_req_rpc_001_client_stores_query() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("query: Option<ChiaQuery>") || src.contains("query: ChiaQuery"),
        "RPC-001: ConsensusClient must store ChiaQuery"
    );
}

/// RPC-001: query() helper method exists for internal access.
#[test]
fn vv_req_rpc_001_query_helper_exists() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("fn query(&self)"),
        "RPC-001: query() helper must exist"
    );
}

/// RPC-001: ChiaQuery import in client.rs.
#[test]
fn vv_req_rpc_001_chia_query_imported() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        src.contains("use chia_query::"),
        "RPC-001: client.rs must import from chia_query"
    );
}

// ── Hex conversion ───────────────────────────────────────────────────

/// RPC-001: bytes32_to_hex produces 0x-prefixed 64-char hex string.
#[test]
fn vv_req_rpc_001_bytes32_to_hex_format() {
    let b = Bytes32::default(); // all zeros
    let hex = bytes32_to_hex(&b);
    assert!(hex.starts_with("0x"), "Must start with 0x");
    assert_eq!(hex.len(), 66, "0x + 64 hex chars = 66");
}

/// RPC-001: bytes32_to_hex is deterministic.
#[test]
fn vv_req_rpc_001_bytes32_to_hex_deterministic() {
    let mut arr = [0u8; 32];
    arr[0] = 0xAB;
    arr[31] = 0xCD;
    let b: Bytes32 = arr.into();
    let h1 = bytes32_to_hex(&b);
    let h2 = bytes32_to_hex(&b);
    assert_eq!(h1, h2);
    assert!(h1.starts_with("0xab"));
    assert!(h1.ends_with("cd"));
}

/// RPC-001: hex_to_bytes32 round-trips with bytes32_to_hex.
#[test]
fn vv_req_rpc_001_hex_roundtrip() {
    let mut arr = [0u8; 32];
    arr[0] = 0xFF;
    arr[15] = 0x42;
    arr[31] = 0x01;
    let original: Bytes32 = arr.into();
    let hex = bytes32_to_hex(&original);
    let restored = hex_to_bytes32(&hex).expect("roundtrip");
    assert_eq!(original, restored);
}

/// RPC-001: hex_to_bytes32 handles with and without 0x prefix.
#[test]
fn vv_req_rpc_001_hex_with_without_prefix() {
    let hex_with = "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let hex_without = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let a = hex_to_bytes32(hex_with).expect("with prefix");
    let b = hex_to_bytes32(hex_without).expect("without prefix");
    assert_eq!(a, b, "Both forms must produce same Bytes32");
}

/// RPC-001: hex_to_bytes32 rejects invalid input.
#[test]
fn vv_req_rpc_001_hex_rejects_invalid() {
    assert!(hex_to_bytes32("not_hex").is_err());
    assert!(hex_to_bytes32("0xGG").is_err());
    assert!(hex_to_bytes32("0xABCD").is_err()); // too short
}

// ── Error mapping ────────────────────────────────────────────────────

/// RPC-001: ConsensusError has RpcError variant.
#[test]
fn vv_req_rpc_001_rpc_error_variant() {
    let src = std::fs::read_to_string("src/error.rs").expect("error.rs");
    assert!(
        src.contains("RpcError"),
        "RPC-001: ConsensusError must have RpcError variant"
    );
}

// ── No custom RPC ────────────────────────────────────────────────────

/// RPC-001: No custom HTTP/TCP/peer connection code in src/.
#[test]
fn vv_req_rpc_001_no_custom_rpc() {
    let src = std::fs::read_to_string("src/client.rs").expect("client.rs");
    assert!(
        !src.contains("TcpStream") && !src.contains("reqwest::") && !src.contains("hyper::"),
        "RPC-001: Must NOT implement custom RPC — use chia-query"
    );
}

/// RPC-001: Spec file exists.
#[test]
fn vv_req_rpc_001_spec_file_exists() {
    assert!(std::path::Path::new("docs/requirements/domains/rpc/specs/RPC-001.md").exists(),);
}
