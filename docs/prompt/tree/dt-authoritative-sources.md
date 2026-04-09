# Authoritative sources

| Layer | Location |
|-------|----------|
| CHIP specification | [`docs/resources/chip-groth16-l2-consensus.md`](../../resources/chip-groth16-l2-consensus.md) |
| Quick reference | [`docs/resources/quick-reference.md`](../../resources/quick-reference.md) |
| Component specs | [`docs/resources/spec-*.md`](../../resources/) |
| Normative requirements | `docs/requirements/domains/<domain>/NORMATIVE.md` |
| Dedicated spec per ID | `docs/requirements/domains/<domain>/specs/<ID>.md` |
| Verification table | `docs/requirements/domains/<domain>/VERIFICATION.md` |
| Machine status | `docs/requirements/domains/<domain>/TRACKING.yaml` |
| Implementation order | [`docs/requirements/IMPLEMENTATION_ORDER.md`](../../requirements/IMPLEMENTATION_ORDER.md) |
| Registry | [`docs/requirements/REQUIREMENTS_REGISTRY.yaml`](../../requirements/REQUIREMENTS_REGISTRY.yaml) |

## Resource files (spec-*.md)

| File | Content |
|------|---------|
| `spec-sparse-merkle-tree.md` | SMT structure, slot assignment, proofs |
| `spec-wire-format.md` | Point encoding, proof format, messages |
| `spec-groth16-circuit.md` | Circuit constraints, public inputs |
| `spec-checkpoint-singleton.md` | Checkpoint puzzle, spend paths |
| `spec-registration-coin.md` | Registration puzzle, collateral |
| `spec-indexer.md` | Off-chain state tracking |
| `spec-security.md` | Trust model, assumptions |
| `spec-trusted-setup.md` | Ceremony, VK verification |
| `spec-validator-onboarding.md` | Validator lifecycle |

**Trace:** `IMPLEMENTATION_ORDER` link → **NORMATIVE** anchor → **Spec:** file → implement → update **VERIFICATION** / **TRACKING** / checkbox.

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-hard-rules.md`](dt-hard-rules.md) |
| **Next** | [`dt-git.md`](dt-git.md) |

*Back to [`tree/README.md`](README.md).*
