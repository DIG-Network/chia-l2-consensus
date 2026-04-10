# Network Coin — Verification

| ID | Status | Summary | Verification Approach |
|----|--------|---------|----------------------|
| [NET-001](NORMATIVE.md#NET-001) | ✅ | Singleton identity | Deploy network coin; verify singleton wrapper; attempt duplicate creation fails |
| [NET-002](NORMATIVE.md#NET-002) | ❌ | AggSigMe registration | Spend with valid signature succeeds; spend with wrong key rejected; message format matches spec |
| [NET-003](NORMATIVE.md#NET-003) | ❌ | Registration coin creation | Created coin has correct puzzle hash; currying params match; amount equals COLLATERAL_AMOUNT |
| [NET-004](NORMATIVE.md#NET-004) | ❌ | Self-recreation | After spend, new network coin exists at same puzzle hash with 1 mojo |
| [NET-005](NORMATIVE.md#NET-005) | ❌ | Pubkey memo | Inspect CreateCoin condition; first memo is 48-byte pubkey; indexer detects registration |

**Status legend:** ✅ verified · ⚠️ partial · ❌ gap
