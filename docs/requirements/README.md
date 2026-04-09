# chia-l2-consensus Requirements

This directory contains the formal requirements for the chia-l2-consensus
system, following a two-tier requirements structure with full traceability.

## Quick Links

- [SCHEMA.md](SCHEMA.md) — Data model and conventions
- [ADDING_REQUIREMENTS.md](ADDING_REQUIREMENTS.md) — Process for adding requirements
- [IMPLEMENTATION_ORDER.md](IMPLEMENTATION_ORDER.md) — Phased implementation checklist
- [REQUIREMENTS_REGISTRY.yaml](REQUIREMENTS_REGISTRY.yaml) — Central domain registry
- [domains/README.md](domains/README.md) — Domain navigation table

## Structure

```
requirements/
├── README.md                    # This file
├── SCHEMA.md                    # Data model and conventions
├── ADDING_REQUIREMENTS.md       # How to add requirements
├── IMPLEMENTATION_ORDER.md      # Phased checklist
├── REQUIREMENTS_REGISTRY.yaml   # Central registry
└── domains/
    ├── README.md               # Domain navigation
    ├── network_coin/           # NET-* requirements
    ├── registration_coin/      # REG-* requirements
    ├── checkpoint/             # CHK-* requirements
    ├── circuit/                # CIR-* requirements
    ├── smt/                    # SMT-* requirements
    ├── wire/                   # WIRE-* requirements
    ├── indexer/                # IDX-* requirements
    ├── security/               # SEC-* requirements
    ├── deployment/             # DEP-* requirements
    └── validator/              # VAL-* requirements
```

## Three-Document Pattern

Each domain contains:

| File | Purpose |
|------|---------|
| `NORMATIVE.md` | Authoritative requirement statements (MUST/SHOULD/MAY) |
| `VERIFICATION.md` | QA approach and status per requirement |
| `TRACKING.yaml` | Machine-readable status, tests, and notes |

## Specification Files

Individual requirement specifications are in `docs/design/requirements/{domain}/`:

```
design/requirements/
├── network_coin/
│   ├── NET-001.md
│   ├── NET-002.md
│   └── ...
├── checkpoint/
│   ├── CHK-001.md
│   └── ...
└── ...
```

## Reference Documents

Requirements are distilled from:

- [chip-groth16-l2-consensus.md](../resources/chip-groth16-l2-consensus.md) — CHIP specification
- [quick-reference.md](../resources/quick-reference.md) — Quick reference guide

## Requirement Count

| Domain | Count |
|--------|-------|
| Network Coin | 5 |
| Registration Coin | 6 |
| Checkpoint Singleton | 7 |
| Groth16 Circuit | 6 |
| Sparse Merkle Tree | 6 |
| Wire Format | 6 |
| Indexer | 5 |
| Security | 6 |
| Deployment | 5 |
| Validator Operations | 5 |
| **Total** | **57** |
