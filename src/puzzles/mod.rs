//! L1 puzzle driver modules — Rust code that builds CLVM spend bundles.
//!
//! Each puzzle is authored in Rue (`puzzles/*.rue`), compiled to CLVM hex
//! (`puzzles/compiled/*.hex`), and embedded in the driver via `include_str!`.
//!
//! ## Puzzle Inventory
//!
//! | Puzzle | Rue Source | Driver | Spec |
//! |--------|-----------|--------|------|
//! | Network Coin | `puzzles/network_coin_inner.rue` | `network_coin.rs` | [spec-network-coin.md](../../docs/resources/spec-network-coin.md) |
//! | Registration Coin | `puzzles/registration_coin.rue` | `registration_coin.rs` | [spec-registration-coin.md](../../docs/resources/spec-registration-coin.md) |
//! | Checkpoint | `puzzles/checkpoint_inner.rue` | `checkpoint.rs` | [spec-checkpoint-singleton.md](../../docs/resources/spec-checkpoint-singleton.md) |
//! | Deployment | N/A (uses chia-wallet-sdk Launcher) | `deploy.rs` | [spec-deployment-runbook.md](../../docs/resources/spec-deployment-runbook.md) |
//!
//! See [spec-consensus-crate.md Lines 1026-1274](../../docs/resources/spec-consensus-crate.md)
//! for the full puzzle driver specification.

mod checkpoint;
mod deploy;
mod network_coin;
mod registration_coin;
mod withdraw_delay;

pub use checkpoint::*;
pub use deploy::*;
pub use network_coin::*;
pub use registration_coin::*;
pub use withdraw_delay::*;
