# Requirements Schema

This document defines the data model and conventions for all requirements in the
chia-l2-consensus project.

---

## Three-Document Pattern

Each domain has exactly three files in `docs/requirements/domains/{domain}/`:

| File | Purpose |
|------|---------|
| `NORMATIVE.md` | Authoritative requirement statements with MUST/SHOULD/MAY keywords |
| `VERIFICATION.md` | QA approach and verification status per requirement |
| `TRACKING.yaml` | Machine-readable status, test references, and implementation notes |

Each requirement also has a dedicated specification file in
`docs/design/requirements/{domain}/{PREFIX-NNN}.md`.

---

## Requirement ID Format

**Pattern:** `{PREFIX}-{NNN}`

- **PREFIX**: 3-4 letter domain identifier (uppercase)
- **NNN**: Zero-padded numeric ID starting at 001

| Domain | Directory | Prefix | Description |
|--------|-----------|--------|-------------|
| Network Coin | `network_coin/` | `NET` | Network coin singleton puzzle |
| Registration Coin | `registration_coin/` | `REG` | Validator registration and collateral |
| Checkpoint Singleton | `checkpoint/` | `CHK` | Checkpoint state and verification |
| Groth16 Circuit | `circuit/` | `CIR` | ZK circuit constraints and proof generation |
| Sparse Merkle Tree | `smt/` | `SMT` | Merkle tree structure and proofs |
| Wire Format | `wire/` | `WIRE` | Message formats and serialization |
| Indexer | `indexer/` | `IDX` | Off-chain state tracking |
| Security | `security/` | `SEC` | Security assumptions and properties |
| Deployment | `deployment/` | `DEP` | Deployment and setup procedures |
| Validator | `validator/` | `VAL` | Validator onboarding and operations |

**Immutability:** Requirement IDs are permanent. Deprecate requirements rather
than renumbering.

---

## Requirement Keywords

Per RFC 2119:

| Keyword | Meaning | Impact |
|---------|---------|--------|
| **MUST** | Absolute requirement | Blocks "done" status if not met |
| **MUST NOT** | Absolute prohibition | Blocks "done" status if violated |
| **SHOULD** | Expected behavior; may be deferred with rationale | Phase 2+ polish items |
| **SHOULD NOT** | Discouraged behavior | Phase 2+ polish items |
| **MAY** | Optional, nice-to-have | Stretch goals |

---

## Status Values

| Status | Description |
|--------|-------------|
| `gap` | Not implemented |
| `partial` | Implementation in progress or incomplete |
| `implemented` | Code complete, awaiting verification |
| `verified` | Implemented and verified per VERIFICATION.md |
| `deferred` | Explicitly postponed with rationale |

---

## TRACKING.yaml Item Schema

```yaml
- id: PREFIX-NNN           # Requirement ID (required)
  section: "Section Name"  # Logical grouping within domain (required)
  summary: "Brief title"   # Human-readable description (required)
  status: gap              # One of: gap, partial, implemented, verified, deferred
  spec_ref: "docs/design/requirements/{domain}/{PREFIX-NNN}.md"
  tests: []                # Array of test names or ["manual"]
  notes: ""                # Implementation notes, blockers, or evidence
```

---

## Master Spec Reference

All requirements trace back to the CHIP specification:
[chip-groth16-l2-consensus.md](../resources/chip-groth16-l2-consensus.md)

Individual requirement specs reference specific CHIP sections using `§N`
notation where applicable.
