//! Simulator helpers for spend bundle testing.
//!
//! Re-exports `chia-sdk-test::Simulator` and `chia-sdk-driver::SpendContext`.
//!
//! NOTE: The simulator internally uses an older chia-protocol version for
//! some types. When building spend bundles for the simulator, use types from
//! `chia_sdk_driver` and `chia_sdk_test` rather than mixing with
//! `chia_protocol` types from our crate's direct dependency.

pub use chia_sdk_driver::SpendContext;
pub use chia_sdk_test::Simulator;

/// Create a fresh simulator for testing.
pub fn new_sim() -> Simulator {
    Simulator::new()
}

/// A 48-byte test pubkey (all same byte). NOT a valid BLS point,
/// but sufficient for CLVM execution tests where point validity
/// is not checked by the puzzle.
pub fn test_pubkey_bytes(byte: u8) -> Vec<u8> {
    vec![byte; 48]
}
