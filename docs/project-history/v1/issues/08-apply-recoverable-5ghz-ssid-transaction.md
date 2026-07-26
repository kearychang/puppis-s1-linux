# 08 — Apply a recoverable 5 GHz SSID transaction

**What to build:** Let a user change the 5 GHz SSID through a serialized settings transaction that verifies the actual device state and either reaches a known result or locks further mutation behind recovery.

**Blocked by:** 07 — Show qualified radio settings without exposing credentials.

**Type:** implementation

**Status:** resolved

- [x] A global mutation lease serializes every NetworkManager or Puppis mutation while allowing safe read-only observation.
- [x] The transaction reads current state, validates the requested change, writes once, reads back, verifies the complete relevant settings object, and reports success only after verification.
- [x] A failed verification attempts rollback to the in-memory pre-change state and verifies that rollback before claiming recovery.
- [x] Cancellation is allowed before the first mutation; afterward the transaction must reach a verified success, verified rollback, or recovery-required terminal state.
- [x] An interrupted-operation marker contains no credentials and causes startup to enter recovery-required mode, where mutations are locked but diagnostics and reconciliation remain available.
- [x] Wi-Fi clearly presents progress, success, rollback, disconnect, and recovery-required outcomes and refreshes from a new authoritative snapshot.
- [x] Safe tests at the application interface cover success, validation failure, concurrent intent rejection, write failure, readback mismatch, rollback success/failure, interruption, and recovery lockout.

## Answer

Implemented the shared application-wide mutation lease, validated 5 GHz SSID intent, secret-free interruption marker, read-before/write/read-after complete-object verification, observed-state reconciliation, verified rollback, recovery-required lockout, redacted status, real protocol setter path, and scripted external-adapter tests for success and failure terminals.
