# Checkpoint Forgery Attack Matrix

Comprehensive analysis of all attack vectors against the checkpoint spend path.
Each vector maps to requirements, tests, and current mitigation status.

**Last audited:** 2026-04-11

---

## Summary

| Status | Count | Description |
|--------|-------|-------------|
| MITIGATED | 12 | Attack blocked by implemented defenses |
| CRITICAL GAP | 1 | SEC-011: Phantom majority — CIR-003 not enforced |
| TRADE-OFF | 3 | Known limitations with documented risk acceptance |
| ARCHITECTURAL | 1 | Honest majority assumption (inherent to BFT) |

**Total vectors analyzed: 17**

---

## CRITICAL GAP

### U. Phantom Majority Forgery (SEC-011) — CRITICAL

**Attack:** Attacker with proving key + one BLS key generates valid Groth16
proof claiming arbitrary signer count. Circuit accepts because `agg_signers`
is unconstrained (CIR-003 not implemented).

**Impact:** Complete checkpoint forgery — arbitrary L2 state committed to L1.

**Exploitability:** Trivial. Proving key is public. No validator registration needed.

| Requirement | Status | Tests |
|-------------|--------|-------|
| SEC-011 | ❌ GAP | `vv_req_sec_011.rs` (7 tests PROVE vulnerability) |
| CIR-003 | ❌ NOT IMPLEMENTED | Phase 3 future work |

**Fix:** Implement CIR-003 — enforce `sum(pk₁..pkₖ) == agg_signers` in circuit.

---

## MITIGATED ATTACKS

### A. Proof Replay (Epoch N → M)

**Attack:** Reuse a valid proof from epoch N at epoch M.

**Defense:** Epoch is in checkpoint_message (112-byte preimage). Different epoch
produces different message hash, different scalar s6, pairing check fails.

| Requirement | Status | Tests |
|-------------|--------|-------|
| CHK-009 | ✅ | `vv_req_chk_009.rs` (9 tests) |
| CHK-010 | ✅ | `vv_req_chk_010.rs` (7 tests) |

---

### B. Cross-Network Replay

**Attack:** Use proof from network A on network B.

**Defense:** `network_coin_launcher_id` is curried into checkpoint puzzle and
included in checkpoint_message hash. Different network → different hash.

| Requirement | Status | Tests |
|-------------|--------|-------|
| CHK-012 | ✅ | `vv_req_chk_012.rs` (7 tests) |

---

### C. State Root Forgery

**Attack:** Change state_root after majority signs.

**Defense:** state_root is first field in checkpoint_message hash. Changing it
changes the message, invalidating both the Groth16 proof and BLS signature.

| Requirement | Status | Tests |
|-------------|--------|-------|
| CHK-011 | ✅ | `vv_req_chk_011.rs` (8 tests) |

---

### D. Minority Checkpoint

**Attack:** Submit checkpoint with fewer than majority signers (2k ≤ n).

**Defense:** CIR-004 enforces `2k > validator_count` via 64-bit decomposition.
Proof generation fails if constraint is unsatisfied.

| Requirement | Status | Tests |
|-------------|--------|-------|
| CIR-004 | ✅ | `vv_req_cir_004.rs` (16 tests) |
| SEC-001 | ✅ | `vv_req_sec_001.rs` (9 tests) |

---

### E. Fake Registration

**Attack:** Create registration coin without going through network coin.

**Defense:** Indexer verifies lineage (parent must be network coin spend).
Coins without valid lineage excluded from validator set.

| Requirement | Status | Tests |
|-------------|--------|-------|
| REG-002 | ✅ | `vv_req_reg_002.rs` (12 tests) |
| SEC-005 | ✅ | `vv_req_sec_005.rs` (10 tests) |

---

### G. Epoch Manipulation

**Attack:** Provide arbitrary epoch in solution to skip/repeat epochs.

**Defense:** Puzzle computes `new_epoch = old_epoch + 1` internally. Epoch is
NOT taken from the solution.

| Requirement | Status | Tests |
|-------------|--------|-------|
| CHK-009 | ✅ | `vv_req_chk_009.rs` (9 tests) |
| SEC-010 (vector G) | ✅ | `vv_req_sec_010.rs` |

---

### H. Double Checkpoint (Race Condition)

**Attack:** Submit two valid checkpoints for same epoch simultaneously.

**Defense:** Singleton UTXO model — spending destroys the coin. First checkpoint
wins; second fails because coin no longer exists.

| Requirement | Status | Tests |
|-------------|--------|-------|
| CHK-010 | ✅ | `vv_req_chk_010.rs` (7 tests) |

---

### I. Signature Subtraction Attack

**Attack:** Extract individual signatures from aggregate to forge new aggregates.

**Defense:** BLS aggregate signature is opaque G2 point. Cannot decompose without
knowing individual signatures.

| Requirement | Status | Tests |
|-------------|--------|-------|
| SEC-002 | ✅ | `vv_req_sec_002.rs` (9 tests) |

---

### J. Rogue Key Attack

**Attack:** Register pubkey that cancels others in aggregation.

**Defense:** Chia uses BLS augmented scheme (DST includes pubkey). Augmented
scheme prevents rogue key attacks without proof-of-possession.

| Requirement | Status | Tests |
|-------------|--------|-------|
| SEC-010 (vector J) | ✅ | `vv_req_sec_010.rs` |

---

### K. Merkle Tree Forgery

**Attack:** Construct Merkle proof for non-member pubkey.

**Defense:** SHA-256 Merkle proof verification in checkpoint puzzle. Cannot forge
SHA-256 proof without finding collision.

| Requirement | Status | Tests |
|-------------|--------|-------|
| SMT-004 | ✅ | `vv_req_smt_004.rs` |
| SMT-005 | ⚠️ | `vv_req_smt_005.rs` (cross-impl partial) |

---

### N. Singleton Destruction

**Attack:** Spend singleton without recreating it, permanently disabling L2.

**Defense:** Singleton wrapper enforces recreation. Inner puzzle output is always
`[recreate, announce]` with no condition passthrough.

| Requirement | Status | Tests |
|-------------|--------|-------|
| CHK-001 | ✅ | `vv_req_chk_001_to_007.rs` |
| SEC-008 | ✅ | `vv_req_sec_008.rs` (9 tests) |

---

### O. VK Substitution

**Attack:** Replace verification key to accept forged proofs.

**Defense:** VK is curried into puzzle at deployment. Changing VK changes puzzle
hash, breaking singleton lineage.

| Requirement | Status | Tests |
|-------------|--------|-------|
| CHK-007 | ✅ | `vv_req_chk_001_to_007.rs` |
| DEP-004 | ✅ | `vv_req_dep_004.rs` (10 tests) |

---

### S. Announcement Spoofing

**Attack:** Forge membership announcement to steal collateral.

**Defense:** Announcement is computed by checkpoint puzzle, not provided in
solution. Checkpoint singleton coin ID in outer hash prevents cross-coin spoofing.

| Requirement | Status | Tests |
|-------------|--------|-------|
| SEC-006 | ✅ | `vv_req_sec_006.rs` (11 tests) |
| CHK-005 | ✅ | `vv_req_chk_001_to_007.rs` |

---

### T. Bundle Splitting

**Attack:** Split spend bundle to separate registration coin spend from
membership query, using announcement from different transaction.

**Defense:** `AssertCoinAnnouncement` requires announcement in same block.
Chia's UTXO model enforces atomic bundles.

| Requirement | Status | Tests |
|-------------|--------|-------|
| REG-004 | ✅ | `vv_req_reg_004.rs` (18 tests) |

---

## TRADE-OFFS (Accepted Risks)

### F/M. Collateral Destination RBF

**Attack:** Farmer front-runs registration coin spend to redirect collateral.

**Risk level:** Low. Announcement requirement means attacker must also spend
checkpoint singleton in same bundle. Practical exploitation unlikely.

| Requirement | Status | Tests |
|-------------|--------|-------|
| SEC-009 | ✅ (documented) | `vv_req_sec_009.rs` (8 tests) |

---

### L. Front-Run Registration

**Attack:** Intercept registration transaction to register attacker's key instead.

**Risk level:** Low. AggSigMe covers pubkey — attacker cannot substitute their
key without the victim's signature.

| Requirement | Status | Tests |
|-------------|--------|-------|
| NET-002 | ✅ | `vv_req_net_002.rs` (12 tests) |

---

### P. Registration Spam

**Attack:** Register many cheap validators to control majority.

**Risk level:** Medium. Collateral requirement makes this expensive. Sequential
singleton registration limits throughput.

| Requirement | Status | Tests |
|-------------|--------|-------|
| SEC-010 (vector P) | ✅ (documented) | `vv_req_sec_010.rs` |

---

## ARCHITECTURAL ASSUMPTION

### Honest Majority (SEC-001)

**Assumption:** >50% of validators are honest.

If violated: Majority can submit arbitrary checkpoints regardless of all other
defenses. This is inherent to BFT consensus and cannot be mitigated by the
circuit or puzzle design.

| Requirement | Status | Tests |
|-------------|--------|-------|
| SEC-001 | ✅ (documented) | `vv_req_sec_001.rs` (9 tests) |

---

## ADDITIONAL VECTORS NOT IN SEC-010

### V1. All-Zero Proof Submission

**Attack:** Submit proof = (identity, identity, identity) to exploit pairing edge cases.

**Risk level:** Negligible. Pairing with identity points produces identity, which
fails the verification equation unless the statement is trivially true.

**Tests:** Not explicitly tested. Covered implicitly by Groth16 soundness.

---

### V2. Scalar Length Manipulation

**Attack:** Provide malformed scalar bytes (short/long) in solution.

**Risk level:** Low. CLVM sha256 and g1_multiply operate on atom bytes directly.
Wrong-length scalars produce wrong VK input → pairing fails.

**Tests:** Not explicitly tested at CLVM level. Covered by scalar assertion tests.

---

### V3. Proof Size Manipulation

**Attack:** Submit proof with wrong size (not 192 bytes).

**Risk level:** Low. CLVM puzzle reads proof.a, proof.b, proof.c as atoms.
Wrong sizes produce invalid G1/G2 points → pairing fails or CLVM error.

**Tests:** Not explicitly tested.

---

## Requirement Coverage Matrix

| Attack | Requirement | Test File | Result |
|--------|-------------|-----------|--------|
| A. Proof replay | CHK-009, CHK-010 | chk_009, chk_010 | ✅ Blocked |
| B. Cross-network | CHK-012 | chk_012 | ✅ Blocked |
| C. State forgery | CHK-011 | chk_011 | ✅ Blocked |
| D. Minority | CIR-004, SEC-001 | cir_004, sec_001 | ✅ Blocked |
| E. Fake registration | REG-002, SEC-005 | reg_002, sec_005 | ✅ Blocked |
| F/M. Collateral RBF | SEC-009 | sec_009 | ⚠️ Accepted |
| G. Epoch manipulation | CHK-009 | chk_009 | ✅ Blocked |
| H. Double checkpoint | CHK-010 | chk_010 | ✅ Blocked |
| I. Sig subtraction | SEC-002 | sec_002 | ✅ Blocked |
| J. Rogue key | SEC-010 | sec_010 | ✅ Blocked |
| K. Merkle forgery | SMT-004 | smt_004 | ✅ Blocked |
| L. Front-run reg | NET-002 | net_002 | ⚠️ Accepted |
| N. Singleton destroy | CHK-001, SEC-008 | chk_001, sec_008 | ✅ Blocked |
| O. VK substitution | CHK-007, DEP-004 | chk_007, dep_004 | ✅ Blocked |
| P. Reg spam | SEC-010 | sec_010 | ⚠️ Accepted |
| R. Stale proof | SEC-010 | sec_010 | ✅ Blocked |
| S. Announcement spoof | SEC-006 | sec_006 | ✅ Blocked |
| T. Bundle splitting | REG-004 | reg_004 | ✅ Blocked |
| **U. Phantom majority** | **SEC-011** | **sec_011** | **❌ VULNERABLE** |
