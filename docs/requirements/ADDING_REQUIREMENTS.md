# Adding Requirements

This document describes the process for adding new requirements to the
chia-l2-consensus project.

---

## Adding a New Requirement to an Existing Domain

### Step 1: Add Normative Statement

Edit `docs/requirements/domains/{domain}/NORMATIVE.md`:

```markdown
<a id="PREFIX-NNN"></a>**PREFIX-NNN** Requirement statement using MUST/SHOULD/MAY.
> **Spec:** [`PREFIX-NNN.md`](../../../design/requirements/{domain}/PREFIX-NNN.md)
```

### Step 2: Add Tracking Entry

Edit `docs/requirements/domains/{domain}/TRACKING.yaml`:

```yaml
- id: PREFIX-NNN
  section: "Section Name"
  summary: "Brief description"
  status: gap
  spec_ref: "docs/design/requirements/{domain}/PREFIX-NNN.md"
  tests: []
  notes: ""
```

### Step 3: Add Verification Row

Edit `docs/requirements/domains/{domain}/VERIFICATION.md`:

```markdown
| [PREFIX-NNN](NORMATIVE.md#PREFIX-NNN) | ❌ | Summary | Verification approach |
```

### Step 4: Create Specification File

Create `docs/design/requirements/{domain}/PREFIX-NNN.md` with full specification
details. Use the template below.

### Step 5: Update Implementation Order

Add checkbox to `docs/requirements/IMPLEMENTATION_ORDER.md` in the appropriate
phase.

---

## Specification File Template

```markdown
# PREFIX-NNN — Requirement Title

> **Authoritative requirement:** [PREFIX-NNN](../../../requirements/domains/{domain}/NORMATIVE.md#PREFIX-NNN)
> **Verification:** [VERIFICATION.md](../../../requirements/domains/{domain}/VERIFICATION.md)
> **Tracking:** [TRACKING.yaml](../../../requirements/domains/{domain}/TRACKING.yaml)
> **CHIP reference:** [chip-groth16-l2-consensus.md](../../../resources/chip-groth16-l2-consensus.md) §N

## Summary

[Detailed description of the requirement, including context and rationale]

## Specification

[Technical details, algorithms, data structures, interfaces]

## Acceptance Criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] ...

## Implementation Notes

- **Primary codebase:** [location]
- **Dependencies:** [related requirements]
- **Constraints:** [technical constraints]

## Verification

[How to verify this requirement is correctly implemented]

## References

- [Related spec documents]
- [External references]
```

---

## Adding a New Domain

### Step 1: Create Domain Directories

```bash
mkdir -p docs/requirements/domains/{domain_id}
mkdir -p docs/design/requirements/{domain_id}
```

### Step 2: Create Three-Document Set

Create in `docs/requirements/domains/{domain_id}/`:
- `NORMATIVE.md` — Requirement statements
- `VERIFICATION.md` — QA approach table
- `TRACKING.yaml` — Machine-readable status

### Step 3: Register Domain

Add entry to `docs/requirements/REQUIREMENTS_REGISTRY.yaml`

### Step 4: Register Prefix

Add prefix mapping to `docs/requirements/SCHEMA.md`

### Step 5: Update Navigation

Add domain to `docs/requirements/domains/README.md`

---

## Checklist

- [ ] Normative statement added with HTML anchor and MUST/SHOULD/MAY keyword
- [ ] TRACKING.yaml entry added with correct spec_ref path
- [ ] VERIFICATION.md row added with verification approach
- [ ] Specification file created with full details
- [ ] IMPLEMENTATION_ORDER.md checkbox added
- [ ] Cross-references to related requirements included
- [ ] CHIP section reference included where applicable
