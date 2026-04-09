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
- [ ] [WIRE-001](domains/wire/NORMATIVE.md#WIRE-001) — Checkpoint message format
- [ ] [WIRE-002](domains/wire/NORMATIVE.md#WIRE-002) — Point encoding (G1/G2)
- [ ] [WIRE-003](domains/wire/NORMATIVE.md#WIRE-003) — Groth16 proof format
- [ ] [WIRE-004](domains/wire/NORMATIVE.md#WIRE-004) — Membership announcement
- [ ] [WIRE-005](domains/wire/NORMATIVE.md#WIRE-005) — Registration message
- [ ] [WIRE-006](domains/wire/NORMATIVE.md#WIRE-006) — scalar() function

---

## Phase 2 — Groth16 Circuit

### Circuit Implementation
- [ ] [CIR-001](domains/circuit/NORMATIVE.md#CIR-001) — Circuit statement
- [ ] [CIR-002](domains/circuit/NORMATIVE.md#CIR-002) — Merkle membership constraint
- [ ] [CIR-003](domains/circuit/NORMATIVE.md#CIR-003) — Aggregate key constraint
- [ ] [CIR-004](domains/circuit/NORMATIVE.md#CIR-004) — Majority threshold constraint
- [ ] [CIR-005](domains/circuit/NORMATIVE.md#CIR-005) — Public inputs
- [ ] [CIR-006](domains/circuit/NORMATIVE.md#CIR-006) — Circuit parameters

---

## Phase 3 — On-Chain Puzzles

### Network Coin
- [ ] [NET-001](domains/network_coin/NORMATIVE.md#NET-001) — Singleton identity
- [ ] [NET-002](domains/network_coin/NORMATIVE.md#NET-002) — AggSigMe registration
- [ ] [NET-003](domains/network_coin/NORMATIVE.md#NET-003) — Registration coin creation
- [ ] [NET-004](domains/network_coin/NORMATIVE.md#NET-004) — Self-recreation
- [ ] [NET-005](domains/network_coin/NORMATIVE.md#NET-005) — Pubkey memo

### Registration Coin
- [ ] [REG-001](domains/registration_coin/NORMATIVE.md#REG-001) — Puzzle structure
- [ ] [REG-002](domains/registration_coin/NORMATIVE.md#REG-002) — Lineage verification
- [ ] [REG-003](domains/registration_coin/NORMATIVE.md#REG-003) — Collateral lock
- [ ] [REG-004](domains/registration_coin/NORMATIVE.md#REG-004) — Announcement assertion
- [ ] [REG-005](domains/registration_coin/NORMATIVE.md#REG-005) — Collateral return
- [ ] [REG-006](domains/registration_coin/NORMATIVE.md#REG-006) — Epoch replay protection

### Checkpoint Singleton
- [ ] [CHK-001](domains/checkpoint/NORMATIVE.md#CHK-001) — Singleton identity
- [ ] [CHK-002](domains/checkpoint/NORMATIVE.md#CHK-002) — State tracking
- [ ] [CHK-003](domains/checkpoint/NORMATIVE.md#CHK-003) — Groth16 + BLS verification
- [ ] [CHK-004](domains/checkpoint/NORMATIVE.md#CHK-004) — State update and announcement
- [ ] [CHK-005](domains/checkpoint/NORMATIVE.md#CHK-005) — Membership query
- [ ] [CHK-006](domains/checkpoint/NORMATIVE.md#CHK-006) — Permissionless query
- [ ] [CHK-007](domains/checkpoint/NORMATIVE.md#CHK-007) — VK immutability

---

## Phase 4 — Off-Chain Infrastructure

### Indexer
- [ ] [IDX-001](domains/indexer/NORMATIVE.md#IDX-001) — State tracking
- [ ] [IDX-002](domains/indexer/NORMATIVE.md#IDX-002) — Lineage verification
- [ ] [IDX-003](domains/indexer/NORMATIVE.md#IDX-003) — Merkle consistency
- [ ] [IDX-004](domains/indexer/NORMATIVE.md#IDX-004) — Reorg handling
- [ ] [IDX-005](domains/indexer/NORMATIVE.md#IDX-005) — Persistent cache

---

## Phase 5 — Deployment & Operations

### Deployment
- [ ] [DEP-001](domains/deployment/NORMATIVE.md#DEP-001) — Trusted setup
- [ ] [DEP-002](domains/deployment/NORMATIVE.md#DEP-002) — Genesis coin
- [ ] [DEP-003](domains/deployment/NORMATIVE.md#DEP-003) — Initial state
- [ ] [DEP-004](domains/deployment/NORMATIVE.md#DEP-004) — VK verification
- [ ] [DEP-005](domains/deployment/NORMATIVE.md#DEP-005) — Artifact publication

### Validator Operations
- [ ] [VAL-001](domains/validator/NORMATIVE.md#VAL-001) — Key generation
- [ ] [VAL-002](domains/validator/NORMATIVE.md#VAL-002) — Registration
- [ ] [VAL-003](domains/validator/NORMATIVE.md#VAL-003) — Signing protocol
- [ ] [VAL-004](domains/validator/NORMATIVE.md#VAL-004) — Voluntary exit
- [ ] [VAL-005](domains/validator/NORMATIVE.md#VAL-005) — Forced exit

---

## Phase 6 — Security Verification

### Security Properties
- [ ] [SEC-001](domains/security/NORMATIVE.md#SEC-001) — Majority assumption documented
- [ ] [SEC-002](domains/security/NORMATIVE.md#SEC-002) — Two-check completeness
- [ ] [SEC-003](domains/security/NORMATIVE.md#SEC-003) — Collateral security
- [ ] [SEC-004](domains/security/NORMATIVE.md#SEC-004) — Trusted setup ceremony
- [ ] [SEC-005](domains/security/NORMATIVE.md#SEC-005) — Lineage enforcement
- [ ] [SEC-006](domains/security/NORMATIVE.md#SEC-006) — Epoch replay protection

---

## Summary

| Phase | Domain | Requirements | Status |
|-------|--------|--------------|--------|
| 0 | Setup | 6 | 6/6 |
| 1 | SMT | 6 | 6/6 |
| 1 | Wire | 6 | 0/6 |
| 2 | Circuit | 6 | 0/6 |
| 3 | Network Coin | 5 | 0/5 |
| 3 | Registration Coin | 6 | 0/6 |
| 3 | Checkpoint | 7 | 0/7 |
| 4 | Indexer | 5 | 0/5 |
| 5 | Deployment | 5 | 0/5 |
| 5 | Validator | 5 | 0/5 |
| 6 | Security | 6 | 0/6 |
| **Total** | | **63** | **12/63** |
