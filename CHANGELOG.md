# Changelog

All notable changes to this project are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and
[Conventional Commits](https://www.conventionalcommits.org).

## [0.2.0] - 2026-07-18

### Features
- **wallet:** Migrate off dig-l1-wallet onto dig-wallet-backend engine seam (#3)

## [0.1.2] - 2026-07-12

### CI
- Add flaky-test management (#489) (#2)

## [0.1.1] - 2026-07-04

### CI
- Release automation + auto-publish on version tag (#230 auto-publish-everything)- Add PR quality gates (fmt/clippy/test/build) [#230] (#1)

## [0.1.0] - 2026-07-04

### Features
- **setup:** Add setup domain and implement SETUP-001 toolchain config- **setup:** Implement SETUP-002 Cargo.toml configuration- **setup:** Implement SETUP-003 project structure with Rue- **setup:** Implement SETUP-004 core dependencies- **setup:** Implement SETUP-005 Rue tooling- **setup:** Implement SETUP-006 build configuration- **smt:** Implement SMT-001 fixed depth tree structure- **smt:** Implement SMT-002 deterministic slot assignment- **smt:** Implement SMT-003 leaf values (active/empty)- **smt:** Implement SMT-004 proof format verification- **smt:** Implement SMT-005 cross-implementation test vectors (partial)- **smt:** Implement SMT-006 empty tree optimization- **wire:** Implement WIRE-001 checkpoint message format- **wire:** Implement WIRE-002 point encoding constants- **wire:** Implement WIRE-003 Groth16 proof format- **wire:** Implement WIRE-004 membership announcement format- **wire:** Implement WIRE-005 registration message format- **wire:** Implement WIRE-006 scalar() function- **circuit:** Implement CIR-001 circuit statement structure- **circuit:** Implement CIR-002 Merkle membership constraint- **circuit:** Implement CIR-003 aggregate key constraint- **circuit:** Implement CIR-004 majority threshold constraint- **circuit:** Implement CIR-005 public inputs specification- **circuit:** Implement CIR-006 circuit parameters- **puzzle:** Implement NET-001 network coin singleton identity- **puzzle:** Implement NET-002 AggSigMe registration verification- **puzzle:** Implement NET-003 registration coin creation- **puzzle:** Implement NET-004 network coin self-recreation- **puzzle:** Implement NET-005 pubkey memo convention- Implement phases 5-7 (deployment, validator, security, crate API)- Add epoch binding + state hash + network ID requirements (CHK-009 through CHK-012)- **security:** Add CLVM vulnerability audit requirements (SEC-007 through SEC-009)- **security:** Implement CHK-012 network ID binding- **security:** Implement SEC-007 CLVM vulnerability audit (13 tests)- **security:** Implement SEC-008 condition injection protection- **security:** Implement SEC-009 destination binding (risk documented)- **api:** Implement API-002 NetworkConfig serde + methods- Complete all remaining requirements (API-002 through API-005, SEC-008/009)- Add Phase 8 RPC domain for 16 todo!() stubs (RPC-001 through RPC-004)- Implement CHK-013 validator attestation binding (11 tests)- Implement CHK-014 permissionless submission / forgery resistance- **security:** Implement SEC-010 comprehensive attack surface (20 vectors)- Complete all 108 requirements — withdraw delay coin, RPC integration, API conformance

### Bug Fixes
- Repair 4 failing WIRE-006 tests and 2 doc test failures

### Documentation
- **prompt:** Add decision tree with VV test-driven workflow- **prompt:** Add VV test file pattern and chia-wallet-sdk guidance- **prompt:** Add Rue compilation requirements- **prompt:** Add commit and push requirements after each requirement- **prompt:** Add crate architecture rule for spec-consensus-crate.md- **prompt:** Integrate GitNexus and Repomix into workflow- Comprehensive LLM-friendly comments with spec line references across all src/- Enhance puzzle driver comments with spec line references- Complete comprehensive comment pass across all src/ leaf files- Comprehensive test documentation across all 69 VV test files- Rewrite CHIP to reflect current system architecture- Rewrite README as comprehensive LLM-friendly public API reference

### CI
- Enforce version increment in PRs (package.json / Cargo.toml)- Enforce Conventional Commits with commitlint on PRs- Enforce Conventional Commits with commitlint on PRs- Release automation (git-cliff changelog + tag on merge); publish is manual workflow_dispatch (#230)

### Chores
- **circuit:** Increase MAX_SIGNERS from 64 to 20,000- Sync TRACKING.yaml statuses with implementation state- Update tracking for CHK-011, CHK-012 completion- Sync all TRACKING.yaml with actual test coverage- **changelog:** Add git-cliff config for Conventional-Commit changelog


