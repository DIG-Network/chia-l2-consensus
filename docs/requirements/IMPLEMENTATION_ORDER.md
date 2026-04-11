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
- [x] [SMT-005](domains/smt/NORMATIVE.md#SMT-005) — Cross-implementation consistency
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
- [x] [CIR-003](domains/circuit/NORMATIVE.md#CIR-003) — Aggregate key constraint (off-chain complete; in-circuit deferred to Phase 3: non-native G1)
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
- [x] [NET-006](domains/network_coin/NORMATIVE.md#NET-006) — E2E simulator test (singleton + registration)

### Registration Coin
- [x] [REG-001](domains/registration_coin/NORMATIVE.md#REG-001) — Puzzle structure
- [x] [REG-002](domains/registration_coin/NORMATIVE.md#REG-002) — Lineage verification
- [x] [REG-003](domains/registration_coin/NORMATIVE.md#REG-003) — Collateral lock
- [x] [REG-004](domains/registration_coin/NORMATIVE.md#REG-004) — Announcement assertion
- [x] [REG-005](domains/registration_coin/NORMATIVE.md#REG-005) — Collateral return
- [x] [REG-006](domains/registration_coin/NORMATIVE.md#REG-006) — Epoch replay protection
- [x] [REG-007](domains/registration_coin/NORMATIVE.md#REG-007) — E2E simulator test (collateral lifecycle)

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

---

## Phase 7 — Crate API Conformance

### Crate API (spec-consensus-crate.md compliance)
- [x] [API-001](domains/crate_api/NORMATIVE.md#API-001) — Public API surface (minimal exports + testing module)
- [x] [API-002](domains/crate_api/NORMATIVE.md#API-002) — NetworkConfig completeness (serde, verification_key(), checkpoint_singleton_id())
- [x] [API-003](domains/crate_api/NORMATIVE.md#API-003) — State types (ValidatorInfo, ValidatorSet helpers count/contains/pubkeys)
- [x] [API-004](domains/crate_api/NORMATIVE.md#API-004) — ConsensusClient state accessors (epoch, state_root, etc.)
- [x] [API-005](domains/crate_api/NORMATIVE.md#API-005) — ConsensusClient message computation (checkpoint_message, signing_message, etc.)
- [x] [API-006](domains/crate_api/NORMATIVE.md#API-006) — Module visibility (pub(crate) + testing re-exports)

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
| 3 | Checkpoint | 12 (7+7sub+4) | 12/12 |
| 4 | Indexer | 5 | 5/5 |
| 5 | Deployment | 5 | 5/5 |
| 5 | Validator | 5 | 5/5 |
| 6 | Security | 9 | 9/9 |
| 7 | Crate API | 6 | 6/6 |
| **Total** | | **79** | **79/79** |
