# Hard rules

1. **Resource files are authoritative** — Specs in [`docs/resources/`](../../resources/) define the protocol. Requirement specs cite these with line numbers.

2. **Cross-implementation consistency** — Rust and Chialisp MUST produce identical outputs:
   - SMT slot assignment: `first_8_bytes_be(sha256(pubkey)) mod 2^32`
   - Leaf hashes: `sha256(pubkey)` for active, `sha256(zeros)` for empty
   - Merkle path: left child first in hash concatenation
   - Wire formats: exact byte layouts per spec

3. **Integer encoding** — All integers in wire formats MUST be fixed-width big-endian. Variable-length encoding is forbidden.

4. **Trusted setup immutability** — Circuit parameters (MAX_SIGNERS, TREE_DEPTH) are fixed at trusted setup. Changing them requires a new ceremony.

5. **Test vectors required** — Every hash computation and wire format must have test vectors verified in both Rust and Chialisp.

6. **BLS12-381 point format** — Use ZCash compressed format (48 bytes G1, 96 bytes G2) with proper infinity and sign encoding.

7. **After `git pull`** — Treat `- [x]` in [`IMPLEMENTATION_ORDER.md`](../../requirements/IMPLEMENTATION_ORDER.md) as **done**; only `- [ ]` is selectable.

---

## Continue the tree

| | |
|--|--|
| **Previous** | [`dt-role.md`](dt-role.md) |
| **Next** | [`dt-authoritative-sources.md`](dt-authoritative-sources.md) |

*Back to [`tree/README.md`](README.md).*
