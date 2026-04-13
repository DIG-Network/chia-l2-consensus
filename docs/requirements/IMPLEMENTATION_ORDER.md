# Implementation Order

This document tracks implementation progress across all requirements, organized
by phase. Check boxes as requirements are verified.

---

## Phase 0 — Project Setup

### Setup
- [x] [SETUP-001](domains/setup/NORMATIVE.md#SETUP-001) — Rust toolchain
- [x] [SETUP-002](domains/setup/NORMATIVE.md#SETUP-002) — Cargo.toml configuration
- [x] [SETUP-003](domains/setup/NORMATIVE.md#SETUP-003) — Project structure
- [x] [SETUP-004](domains/setup/NORMATIVE.md#SETUP-004) — Core dependencies
- [x] [SETUP-005](domains/setup/NORMATIVE.md#SETUP-005) — Rue tooling
- [x] [SETUP-006](domains/setup/NORMATIVE.md#SETUP-006) — Build configuration

---

## Phase 1 — Foundation (Core Infrastructure)

### Sparse Merkle Tree
- [x] [SMT-001](domains/smt/NORMATIVE.md#SMT-001) — Fixed depth tree structure
- [x] [SMT-002](domains/smt/NORMATIVE.md#SMT-002) — Deterministic slot assignment
- [x] [SMT-003](domains/smt/NORMATIVE.md#SMT-003) — Leaf values (active/empty)
- [x] [SMT-004](domains/smt/NORMATIVE.md#SMT-004) — Proof format
- [x] [SMT-005](domains/smt/NORMATIVE.md#SMT-005) — Cross-implementation consistency (Rust + CLVM cross-verified at depth=32)
- [x] [SMT-006](domains/smt/NORMATIVE.md#SMT-006) — Empty tree optimization

### Wire Format
- [x] [WIRE-001](domains/wire/NORMATIVE.md#WIRE-001) — Checkpoint message format
- [x] [WIRE-002](domains/wire/NORMATIVE.md#WIRE-002) — Point encoding (G1/G2)
- [x] [WIRE-003](domains/wire/NORMATIVE.md#WIRE-003) — Groth16 proof format
- [x] [WIRE-004](domains/wire/NORMATIVE.md#WIRE-004) — Membership announcement
- [x] [WIRE-005](domains/wire/NORMATIVE.md#WIRE-005) — Registration message
- [x] [WIRE-006](domains/wire/NORMATIVE.md#WIRE-006) — scalar() function

---

## Phase 2 — Groth16 Circuit

### Circuit Implementation
- [x] [CIR-001](domains/circuit/NORMATIVE.md#CIR-001) — Circuit statement
- [x] [CIR-002](domains/circuit/NORMATIVE.md#CIR-002) — Merkle membership constraint (Poseidon hash)
- [x] [CIR-003](domains/circuit/NORMATIVE.md#CIR-003) — Aggregate key constraint (in-circuit non-native G1 via g1_gadget.rs; off-chain via aggregate.rs)
- [x] [CIR-004](domains/circuit/NORMATIVE.md#CIR-004) — Majority threshold constraint (64-bit decomposition)
- [x] [CIR-005](domains/circuit/NORMATIVE.md#CIR-005) — Public inputs
- [x] [CIR-006](domains/circuit/NORMATIVE.md#CIR-006) — Circuit parameters
- See [DESIGN_DECISIONS.md](domains/circuit/DESIGN_DECISIONS.md) for hash function and phasing rationale

---

## Phase 3 — On-Chain Puzzles

### Network Coin
- [x] [NET-001](domains/network_coin/NORMATIVE.md#NET-001) — Singleton identity
- [x] [NET-002](domains/network_coin/NORMATIVE.md#NET-002) — AggSigMe registration
- [x] [NET-003](domains/network_coin/NORMATIVE.md#NET-003) — Registration coin creation
- [x] [NET-004](domains/network_coin/NORMATIVE.md#NET-004) — Self-recreation
- [x] [NET-005](domains/network_coin/NORMATIVE.md#NET-005) — Pubkey memo
- [x] [NET-006](domains/network_coin/NORMATIVE.md#NET-006) — E2E simulator test

### Registration Coin
- [x] [REG-001](domains/registration_coin/NORMATIVE.md#REG-001) — Puzzle structure (18 source + 8 CLVM execution tests)
- [x] [REG-002](domains/registration_coin/NORMATIVE.md#REG-002) — Lineage verification
- [x] [REG-003](domains/registration_coin/NORMATIVE.md#REG-003) — Collateral lock
- [x] [REG-004](domains/registration_coin/NORMATIVE.md#REG-004) — Announcement assertion
- [x] [REG-005](domains/registration_coin/NORMATIVE.md#REG-005) — Collateral return
- [x] [REG-006](domains/registration_coin/NORMATIVE.md#REG-006) — Epoch replay protection
- [x] [REG-007](domains/registration_coin/NORMATIVE.md#REG-007) — E2E simulator test (2 success + 4 failure via REG-009)

### Checkpoint Singleton
- [x] [CHK-001](domains/checkpoint/NORMATIVE.md#CHK-001) — Singleton identity
- [x] [CHK-002](domains/checkpoint/NORMATIVE.md#CHK-002) — State tracking
- [x] [CHK-003](domains/checkpoint/NORMATIVE.md#CHK-003) — Groth16 + BLS verification
- [x] [CHK-004](domains/checkpoint/NORMATIVE.md#CHK-004) — State update and announcement
- [x] [CHK-005](domains/checkpoint/NORMATIVE.md#CHK-005) — Membership query
- [x] [CHK-006](domains/checkpoint/NORMATIVE.md#CHK-006) — Permissionless query
- [x] [CHK-007](domains/checkpoint/NORMATIVE.md#CHK-007) — VK immutability
- [x] [CHK-008a](domains/checkpoint/specs/CHK-008.md) — Trusted setup produces valid keys
- [x] [CHK-008b](domains/checkpoint/specs/CHK-008.md) — Proof generation (192 bytes)
- [x] [CHK-008c](domains/checkpoint/specs/CHK-008.md) — VK has 7 IC points
- [x] [CHK-008d](domains/checkpoint/specs/CHK-008.md) — Checkpoint path with real Groth16 + BLS
- [x] [CHK-008e](domains/checkpoint/specs/CHK-008.md) — Checkpoint spend accepted by simulator
- [x] [CHK-008f](domains/checkpoint/specs/CHK-008.md) — Invalid proof rejected
- [x] [CHK-008g](domains/checkpoint/specs/CHK-008.md) — Ark→ZCash format compatibility verified
- [x] [CHK-009](domains/checkpoint/NORMATIVE.md#CHK-009) — Epoch binding (proof bound to specific epoch via checkpoint_message)
- [x] [CHK-010](domains/checkpoint/NORMATIVE.md#CHK-010) — Single checkpoint per epoch (singleton + epoch hash + BLS)
- [x] [CHK-011](domains/checkpoint/NORMATIVE.md#CHK-011) — State hash binding (state_root in checkpoint_message)
- [x] [CHK-012](domains/checkpoint/NORMATIVE.md#CHK-012) — Network ID binding (network_coin_launcher_id in checkpoint_message)
- [x] [CHK-013](domains/checkpoint/NORMATIVE.md#CHK-013) — Validator attestation binding (epoch + network + state in signed message)
- [x] [CHK-014](domains/checkpoint/NORMATIVE.md#CHK-014) — Permissionless submission / forgery resistance (anyone submits, only valid majority accepted)

---

## Phase 3b — Withdraw Delay Coin

### Withdraw Delay Coin (time-locked collateral release)
- [x] [WDC-001](domains/withdraw_delay/NORMATIVE.md#WDC-001) — Puzzle structure (3 curried params, empty solution, 22 VV tests)
- [x] [WDC-002](domains/withdraw_delay/NORMATIVE.md#WDC-002) — Time lock (ASSERT_HEIGHT_RELATIVE, default 24,000 blocks / ~5 days, 16 VV tests)
- [x] [WDC-003](domains/withdraw_delay/NORMATIVE.md#WDC-003) — Fund release (CREATE_COIN at DESTINATION with AMOUNT, 16 VV tests)
- [x] [WDC-004](domains/withdraw_delay/NORMATIVE.md#WDC-004) — Registration coin integration (2 new curried params, creates delay coin, 19 VV tests)
- [x] [WDC-005](domains/withdraw_delay/NORMATIVE.md#WDC-005) — Driver and API (release_collateral() on ConsensusClient, 8 VV tests)
- [x] [WDC-006](domains/withdraw_delay/NORMATIVE.md#WDC-006) — Configuration (NetworkConfig: withdraw_delay_blocks, withdraw_delay_mod_hash, 7 VV tests)
- [x] [WDC-007](domains/withdraw_delay/NORMATIVE.md#WDC-007) — Permissionless release (no AGG_SIG, anyone can release after delay, 8 VV tests)
- [x] [WDC-008](domains/withdraw_delay/NORMATIVE.md#WDC-008) — CLVM execution tests (11 VV tests)
- [x] [WDC-009](domains/withdraw_delay/NORMATIVE.md#WDC-009) — E2E simulator test (full two-phase lifecycle, 2 VV tests)
- [x] [WDC-010](domains/withdraw_delay/NORMATIVE.md#WDC-010) — Destination hint memo (conflict-resistant hint + "DIG Network Collateral Release", 10 VV tests)

---

## Phase 4 — Off-Chain Infrastructure

### Indexer
- [x] [IDX-001](domains/indexer/NORMATIVE.md#IDX-001) — State tracking
- [x] [IDX-002](domains/indexer/NORMATIVE.md#IDX-002) — Lineage verification
- [x] [IDX-003](domains/indexer/NORMATIVE.md#IDX-003) — Merkle consistency
- [x] [IDX-004](domains/indexer/NORMATIVE.md#IDX-004) — Reorg handling
- [x] [IDX-005](domains/indexer/NORMATIVE.md#IDX-005) — Persistent cache

---

## Phase 5 — Deployment & Operations

### Deployment
- [x] [DEP-001](domains/deployment/NORMATIVE.md#DEP-001) — Trusted setup
- [x] [DEP-002](domains/deployment/NORMATIVE.md#DEP-002) — Genesis coin
- [x] [DEP-003](domains/deployment/NORMATIVE.md#DEP-003) — Initial state
- [x] [DEP-004](domains/deployment/NORMATIVE.md#DEP-004) — VK verification
- [x] [DEP-005](domains/deployment/NORMATIVE.md#DEP-005) — Artifact publication

### Validator Operations
- [x] [VAL-001](domains/validator/NORMATIVE.md#VAL-001) — Key generation
- [x] [VAL-002](domains/validator/NORMATIVE.md#VAL-002) — Registration
- [x] [VAL-003](domains/validator/NORMATIVE.md#VAL-003) — Signing protocol
- [x] [VAL-004](domains/validator/NORMATIVE.md#VAL-004) — Voluntary exit
- [x] [VAL-005](domains/validator/NORMATIVE.md#VAL-005) — Forced exit

---

## Phase 6 — Security Verification

### Security Properties
- [x] [SEC-001](domains/security/NORMATIVE.md#SEC-001) — Majority assumption documented
- [x] [SEC-002](domains/security/NORMATIVE.md#SEC-002) — Two-check completeness
- [x] [SEC-003](domains/security/NORMATIVE.md#SEC-003) — Collateral security
- [x] [SEC-004](domains/security/NORMATIVE.md#SEC-004) — Trusted setup ceremony
- [x] [SEC-005](domains/security/NORMATIVE.md#SEC-005) — Lineage enforcement
- [x] [SEC-006](domains/security/NORMATIVE.md#SEC-006) — Epoch replay protection
- [x] [SEC-007](domains/security/NORMATIVE.md#SEC-007) — CLVM vulnerability audit (9 vulns assessed, verify mitigations)
- [x] [SEC-008](domains/security/NORMATIVE.md#SEC-008) — Condition injection protection (removed from registration + checkpoint)
- [x] [SEC-009](domains/security/NORMATIVE.md#SEC-009) — Registration coin destination binding (risk mitigated by announcement)
- [x] [SEC-010](domains/security/NORMATIVE.md#SEC-010) — Comprehensive attack surface (20 vectors A-T verified)
- [x] [SEC-011](domains/security/NORMATIVE.md#SEC-011) — Phantom majority forgery resistance (CIR-003 enforced via g1_gadget.rs)

---

## Phase 7 — Crate API Conformance

### Crate API (spec-consensus-crate.md compliance)
- [x] [API-001](domains/crate_api/NORMATIVE.md#API-001) — Public API surface (minimal exports + testing module)
- [x] [API-002](domains/crate_api/NORMATIVE.md#API-002) — NetworkConfig completeness (serde, verification_key(), checkpoint_singleton_id())
- [x] [API-003](domains/crate_api/NORMATIVE.md#API-003) — State types (ValidatorInfo, ValidatorSet helpers count/contains/pubkeys)
- [x] [API-004](domains/crate_api/NORMATIVE.md#API-004) — ConsensusClient state accessors (epoch, state_root, etc.)
- [x] [API-005](domains/crate_api/NORMATIVE.md#API-005) — ConsensusClient message computation (checkpoint_message, signing_message, etc.)
- [x] [API-006](domains/crate_api/NORMATIVE.md#API-006) — Module visibility (pub(crate) + testing re-exports)
- [x] [API-008](domains/crate_api/NORMATIVE.md#API-008) — Return-not-submit pattern (all bundle methods return SpendBundle; crate never broadcasts, 10 VV tests)

---

## Phase 8 — RPC Integration (unblocked by chia-query + dig-l1-wallet)

### RPC (via chia-query and dig-l1-wallet from crates.io)
- [x] [RPC-006](domains/rpc/NORMATIVE.md#RPC-006) — Dependency version alignment (chia-query 0.2 + dig-l1-wallet 0.1 added, 9 VV tests)
- [x] [RPC-001](domains/rpc/NORMATIVE.md#RPC-001) — chia-query::ChiaQuery as blockchain query backend (connect(), hex helpers, RpcError, 13 VV tests)
- [x] [RPC-005](domains/rpc/NORMATIVE.md#RPC-005) — dig-l1-wallet coin selection for collateral funding (L1Wallet param, InsufficientFunds, 10 VV tests)
- [x] [RPC-002](domains/rpc/NORMATIVE.md#RPC-002) — Puzzle driver spend bundle construction (9 functions, correct sigs, 13 VV tests)
- [x] [RPC-003](domains/rpc/NORMATIVE.md#RPC-003) — Indexer sync algorithm (4 stubs, module structure verified, 11 VV tests)
- [x] [RPC-004](domains/rpc/NORMATIVE.md#RPC-004) — ConsensusClient operation methods (6 methods, correct sigs, 13 VV tests)

---

## Phase 9 — Test Completeness (Audit Gaps)

> Added per test coverage audit 2026-04-11. Addresses SCHEMA.md Hard Testing
> Requirements violations and missing coverage identified by systematic audit
> of all 87 requirements.

### CRITICAL — CLVM Execution Gaps (source-inspection only → must add CLVM tests)

- [x] [NET-007](domains/network_coin/NORMATIVE.md#NET-007) — CLVM execution validation for NET-001-004
- [x] [REG-008](domains/registration_coin/NORMATIVE.md#REG-008) — CLVM execution for REG-001 puzzle structure (8 CLVM tests including cross-impl hash)

### HIGH — Failure Case Gaps

- [x] [NET-008](domains/network_coin/NORMATIVE.md#NET-008) — Failure cases for NET-006 E2E (invalid lineage, no collateral, puzzle hash verification)
- [x] [REG-009](domains/registration_coin/NORMATIVE.md#REG-009) — Failure cases for REG-007 E2E (no announcement, wrong hash, is_member=true, wrong epoch)

### HIGH — Missing Test Files

- [x] [SETUP-007](domains/setup/NORMATIVE.md#SETUP-007) — Automated VV tests for SETUP-001 (5 toolchain) and SETUP-002 (9 Cargo.toml)
- [x] [API-007](domains/crate_api/NORMATIVE.md#API-007) — Dedicated VV tests for API-001 (7 tests) and API-006 (11 tests)

### MEDIUM — Strengthen Coverage

- [x] [CHK-015](domains/checkpoint/NORMATIVE.md#CHK-015) — CLVM execution for CHK-009-014 binding properties (6 tests: epoch, network ID, invalid proof)
- [x] [REG-010](domains/registration_coin/NORMATIVE.md#REG-010) — Simulator spend verification for REG-003-006 (5 tests: lock, assertion, return, epoch)

---

## Summary

| Phase | Domain | Requirements | Status |
|-------|--------|--------------|--------|
| 0 | Setup | 6 | 6/6 |
| 1 | SMT | 6 | 6/6 |
| 1 | Wire | 6 | 6/6 |
| 2 | Circuit | 6 | 6/6 |
| 3 | Network Coin | 6 | 6/6 |
| 3 | Registration Coin | 7 | 7/7 |
| 3 | Checkpoint | 14 (7 + CHK-008 + 6) | 14/14 |
| 3b | Withdraw Delay | 10 | 10/10 |
| 4 | Indexer | 5 | 5/5 |
| 5 | Deployment | 5 | 5/5 |
| 5 | Validator | 5 | 5/5 |
| 6 | Security | 11 | 11/11 |
| 7 | Crate API | 7 | 7/7 |
| 8 | RPC | 6 | 6/6 |
| 9 | Test Completeness | 8 | 8/8 |
| **Total** | | **108** | **108/108** |
