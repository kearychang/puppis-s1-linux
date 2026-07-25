# 09 — Complete qualified 5 GHz channel and password controls

**What to build:** Complete the production-qualified 5 GHz experience by adding safe channel selection and write-only password replacement through the existing transaction and recovery behavior.

**Blocked by:** 08 — Apply a recoverable 5 GHz SSID transaction.

**Type:** implementation

**Status:** resolved

- [x] Wi-Fi offers only firmware- and policy-qualified 5 GHz channel choices and validates all values before mutation.
- [x] A replacement password travels only through the narrow mutation intent, is held only as long as required in memory, and is cleared from the frontend immediately after submission or cancellation.
- [x] Channel and password changes use read-before/write/read-after/full-object verification and verified rollback rather than trusting a response alone.
- [x] Each operation participates in the global mutation lease, interruption marker, and recovery-required lockout.
- [x] Logs, snapshots, typed failures, diagnostics, and tests contain no existing or replacement password.
- [x] Fixture-backed application tests and safe Svelte tests cover qualified choices, invalid values, verified changes, rollback paths, password clearing, and unsupported firmware.

## Answer

Implemented qualified 5 GHz automatic/channel-36 selection and write-only password replacement through the common verified transaction, official-format validation, complete-object preservation, and secret-exclusion tests.
