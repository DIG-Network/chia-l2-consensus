# Requirements Domains

Quick navigation to all domain requirement documents.

| Domain | Prefix | NORMATIVE | VERIFICATION | TRACKING | SPECS |
|--------|--------|-----------|--------------|----------|-------|
| Project Setup | SETUP | [NORMATIVE](setup/NORMATIVE.md) | [VERIFICATION](setup/VERIFICATION.md) | [TRACKING](setup/TRACKING.yaml) | [specs/](setup/specs/) |
| Network Coin | NET | [NORMATIVE](network_coin/NORMATIVE.md) | [VERIFICATION](network_coin/VERIFICATION.md) | [TRACKING](network_coin/TRACKING.yaml) | [specs/](network_coin/specs/) |
| Registration Coin | REG | [NORMATIVE](registration_coin/NORMATIVE.md) | [VERIFICATION](registration_coin/VERIFICATION.md) | [TRACKING](registration_coin/TRACKING.yaml) | [specs/](registration_coin/specs/) |
| Checkpoint Singleton | CHK | [NORMATIVE](checkpoint/NORMATIVE.md) | [VERIFICATION](checkpoint/VERIFICATION.md) | [TRACKING](checkpoint/TRACKING.yaml) | [specs/](checkpoint/specs/) |
| Groth16 Circuit | CIR | [NORMATIVE](circuit/NORMATIVE.md) | [VERIFICATION](circuit/VERIFICATION.md) | [TRACKING](circuit/TRACKING.yaml) | [specs/](circuit/specs/) |
| Sparse Merkle Tree | SMT | [NORMATIVE](smt/NORMATIVE.md) | [VERIFICATION](smt/VERIFICATION.md) | [TRACKING](smt/TRACKING.yaml) | [specs/](smt/specs/) |
| Wire Format | WIRE | [NORMATIVE](wire/NORMATIVE.md) | [VERIFICATION](wire/VERIFICATION.md) | [TRACKING](wire/TRACKING.yaml) | [specs/](wire/specs/) |
| Indexer | IDX | [NORMATIVE](indexer/NORMATIVE.md) | [VERIFICATION](indexer/VERIFICATION.md) | [TRACKING](indexer/TRACKING.yaml) | [specs/](indexer/specs/) |
| Security | SEC | [NORMATIVE](security/NORMATIVE.md) | [VERIFICATION](security/VERIFICATION.md) | [TRACKING](security/TRACKING.yaml) | [specs/](security/specs/) |
| Deployment | DEP | [NORMATIVE](deployment/NORMATIVE.md) | [VERIFICATION](deployment/VERIFICATION.md) | [TRACKING](deployment/TRACKING.yaml) | [specs/](deployment/specs/) |
| Validator Operations | VAL | [NORMATIVE](validator/NORMATIVE.md) | [VERIFICATION](validator/VERIFICATION.md) | [TRACKING](validator/TRACKING.yaml) | [specs/](validator/specs/) |
| Crate API | API | [NORMATIVE](crate_api/NORMATIVE.md) | [VERIFICATION](crate_api/VERIFICATION.md) | [TRACKING](crate_api/TRACKING.yaml) | [specs/](crate_api/specs/) |

## Domain Structure

Each domain directory contains:

```
<domain>/
├── NORMATIVE.md      # Authoritative requirement statements (MUST/SHOULD/MAY)
├── VERIFICATION.md   # QA approach and status per requirement
├── TRACKING.yaml     # Machine-readable status, tests, and notes
└── specs/            # Individual requirement specification files
    ├── PREFIX-001.md
    ├── PREFIX-002.md
    └── ...
```

## Domain Descriptions

- **Project Setup**: Rust toolchain, dependencies, and project structure
- **Network Coin**: Singleton gatekeeper for validator registration
- **Registration Coin**: Validator collateral and identity management
- **Checkpoint Singleton**: On-chain L2 state authority
- **Groth16 Circuit**: ZK proof of membership and majority
- **Sparse Merkle Tree**: Validator set data structure
- **Wire Format**: Serialization and message formats
- **Indexer**: Off-chain state tracking and lineage verification
- **Security**: Security assumptions and properties
- **Deployment**: Trusted setup and network deployment
- **Validator Operations**: Validator lifecycle management
