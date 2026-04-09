# Prompt architecture (chia-l2-consensus)

- **Entry:** [`prompt.md`](prompt.md) → [`start.md`](start.md).
- **Detail:** one topic per file under [`tree/`](tree/README.md); each page stays short and links **Next** where useful.
- **Authoritative specs:** [`docs/resources/`](../resources/) contains CHIP and detailed specifications.
- **Requirements:** [`docs/requirements/`](../requirements/README.md) with domains, NORMATIVE, VERIFICATION, TRACKING, and specs.

## Key documents

| Document | Purpose |
|----------|---------|
| [`chip-groth16-l2-consensus.md`](../resources/chip-groth16-l2-consensus.md) | CHIP specification (design rationale) |
| [`quick-reference.md`](../resources/quick-reference.md) | Quick lookup tables |
| [`spec-*.md`](../resources/) | Detailed component specifications |
| [`IMPLEMENTATION_ORDER.md`](../requirements/IMPLEMENTATION_ORDER.md) | Phased implementation checklist |

## System overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         L2 Application                          │
└─────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │  Checkpoint Message   │
                    │  + k BLS Signatures   │
                    └───────────┬───────────┘
                                │
┌───────────────────────────────┴───────────────────────────────┐
│                     Off-Chain (Rust)                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐│
│  │   Indexer   │  │  SMT (32)   │  │  Groth16 Proof Gen      ││
│  │  (state)    │  │  (pubkeys)  │  │  (arkworks)             ││
│  └─────────────┘  └─────────────┘  └─────────────────────────┘│
└───────────────────────────────┬───────────────────────────────┘
                                │
┌───────────────────────────────┴───────────────────────────────┐
│                     On-Chain (Chialisp)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐│
│  │Network Coin │  │Registration │  │  Checkpoint Singleton   ││
│  │ (singleton) │  │   Coins     │  │  (Groth16 + BLS verify) ││
│  └─────────────┘  └─────────────┘  └─────────────────────────┘│
└───────────────────────────────────────────────────────────────┘
```
