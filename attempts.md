# CHK-008 Debugging Attempts Log

## Problem
Tests `vv_req_chk_008_checkpoint_path_with_real_proof` and `vv_req_chk_008_checkpoint_in_simulator`
fail with "path into atom" when executing the checkpoint spend path with real Groth16 proofs and
BLS signatures.

## Root Cause (Found)
**Rue compiles structs as nil-terminated proper CLVM lists, but test code builds them as improper pairs.**

Example: `State { state_root, epoch, validator_merkle_root, validator_count }` compiles to:
```
(state_root . (epoch . (vmr . (vc . nil))))   <- Rue expects this
```
But `build_chk_path_env` constructs:
```
(state_root . (epoch . (vmr . vc)))            <- test builds this (WRONG)
```

When the checkpoint path accesses `STATE.validator_count` (the LAST field), the compiled CLVM
does `car(cdr^3(STATE))`. With the improper pair, `cdr^3` gives the raw atom `vc`, and
`car(vc)` produces "path into atom".

The membership path (CHK-005) never accesses the last field of STATE, so it passes despite the
same bug in `build_query_env`.

## Key Technical Detail: CLVM Path Bit Ordering
CLVM paths use **LSB-first** bit ordering after stripping the leading 1 bit. Each bit:
- 0 = car (first)
- 1 = cdr (rest)

Example: path 383 = 0b101111111 -> strip leading 1 -> 01111111 -> LSB first: 7 cdrs then car
= `car(cdr^7(body_env))` = parameter 7 = `is_checkpoint`. Confirmed correct.

## Attempts That Did NOT Work

### 1. MSB-first path analysis (incorrect)
Spent significant time computing paths using MSB-first bit ordering. This led to paths
navigating into helper function structures rather than parameters, producing contradictions
(e.g., "path 383 navigates into helper H3 body, not is_checkpoint").

### 2. Manual CLVM hex decoding
Attempted to decode the compiled hex byte-by-byte. Got confused by CLVM atom encoding
(single-byte atoms 0x01-0x7f vs multi-byte length headers 0x80+). Eventually confirmed
structure via `rue build` S-expression output instead.

### 3. Investigating env restructuring levels
Hypothesized the checkpoint branch creates 2-3 levels of `(c X 1)` env prepending, causing
helper paths to become invalid. While the 3-level structure IS correct (body_env -> level1_env
-> level2_env), the paths are adjusted correctly by the Rue compiler. The actual issue is
struct format, not env nesting.

### 4. Investigating "contradictions" in path 5887
Path 5887 was identified as STATE.validator_merkle_root in level2_env. With dummy 48-byte
scalars, the first assertion (`sha256(STATE.vmr) == scalars.s1`) FAILS because
sha256(32-byte vmr) != 48-byte zeros. This causes "clvm raise" BEFORE reaching the second
assertion that accesses STATE.validator_count (which would cause "path into atom"). With real
32-byte scalars, the first assertion PASSES, and the code proceeds to the second assertion
where it hits "path into atom" on STATE.validator_count.

### 5. Investigating helper function paths
Hypothesized that `(a 43 ...)` in the THEN branch was trying to call a helper function at an
invalid path. After correcting to LSB-first traversal, path 43 in level2_env correctly
resolves to H2 (curry_tree_hash helper). The helper paths are valid.

### 6. Investigating scalar format (32-byte vs 48-byte)
Scalars are typed as `PublicKey` (48 bytes) in Rue but contain 32-byte SHA-256 hashes. The
32-byte format is correct for the assertions (`sha256(input) == scalar`) and for `g1_multiply`
(which treats the scalar as a big-endian integer). The scalar SIZE is not the issue.

## Fix #1 (Applied): Nil-terminated structs
Add nil terminators to all Rue struct constructions in test env builders:
- `STATE`: `(sr . (ep . (vmr . (vc . nil))))`
- `VK`: `(alpha . (beta . (gamma . (delta . nil))))`
- `IC`: `(ic0 . (ic1 . ... (ic6 . nil)...))`
- `Proof`: `(a . (b . (c . nil)))`
- `Scalars`: `(s1 . (s2 . ... (s6 . nil)...))`

Reference: The `l2_driver_state_channel/driver` codebase uses `#[derive(ToClvm)]` with
`#[clvm(list)]` which automatically produces nil-terminated proper lists for all structs.

**Result**: Fixed "path into atom". Now CHK-005/006/007 pass. CHK-008 progresses past scalar
assertions but fails with "bls_pairing_identity failed".

## Current Issue: bls_pairing_identity failed (after Fix #1)

With nil-terminated structs, the checkpoint path reaches the Groth16 pairing check but fails.

### Verified:
- Proof valid: arkworks' own `verify_proof` returns Ok(true)
- Points valid: all G1/G2 points decompress with blst (no format conversion needed)
- ark format = ZCash/Chia format: confirmed by comparing G1 generator bytes
- Individual g1_multiply: CLVM matches arkworks (even for scalars with MSB set)
- Full vk_input: CLVM (with quoted values) matches arkworks
- Direct pairing: PASSES when using ark-computed vk_input in a CLVM program
- CLVM dialect: flag=0 enables BLS ops (hardforked in universally per clvmr source)

### Not yet identified:
- Why the Rue puzzle's bls_pairing_identity fails when all the building blocks are correct
- Possible issues: path misresolution in the puzzle's deeply nested env structure
  that only manifests when ALL operations are composed together

### Attempted:
7. Ark→ZCash G1/G2 format conversion (byte reversal + flag remapping) — WRONG:
   arkworks BLS12-381 already uses ZCash-compatible format. Conversion broke valid points.
8. Signed scalar interpretation in bytes_to_scalar — applied but didn't fix pairing.
   CLVM's `int_atom` reads scalars as signed. The signed fix makes arkworks match CLVM
   for individual g1_multiply operations. However, the pairing still fails in the Rue puzzle.
9. Full vk_input computation in CLVM (with quoted values) — matches arkworks exactly.
   This proves CLVM's arithmetic is compatible with arkworks for these specific values.

### Fix #2 (Applied): Recompile stale puzzle hex
The compiled `puzzles/compiled/checkpoint_inner.hex` was from an OLDER version of the Rue
source. Recompiling with `rue build puzzles/checkpoint_inner.rue --hex` produced different
bytes (same length 3328 chars but different content). The old hex had CLVM paths that
didn't match the current struct layout expectations.

After recompiling: ALL 9 CHK-008 tests pass, including real Groth16 verification and
simulator checkpoint spend.

### How it was found:
By running the puzzle's INNER2 body directly against the real level2_env (extracted from
the puzzle's own env_builders), the pairing FAILED. But running the SAME pairing check
as a hand-built CLVM program with the SAME paths against the SAME env PASSED. The only
difference was the INNER2 body code came from the compiled hex. Comparing `rue build --hex`
output with the stored hex revealed they differed.
