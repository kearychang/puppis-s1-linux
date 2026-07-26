# 10 — Apply qualified 2.4 GHz SSID and channel transactions

**What to build:** Give 2.4 GHz users the same production-standard transaction experience for the live-qualified SSID and Canadian automatic/channel-6 paths as the qualified 5 GHz controls.

**Blocked by:** 08 — Apply a recoverable 5 GHz SSID transaction.

**Type:** implementation

**Status:** resolved

- [x] A user can change and restore the 2.4 GHz SSID through the common settings transaction, full-object verification, rollback, and recovery-required behavior.
- [x] Qualified Canadian automatic and channel 6 behavior is represented and can be applied; unqualified channel/region combinations are unavailable rather than guessed.
- [x] The UI gives 2.4 GHz operations the same validation, progress, confirmation where needed, terminal-state, accessibility, and credential-safety standard as 5 GHz.
- [x] The redacted partial 2.4 GHz fixture is a required regression case proving SSID and channel behavior without treating its password phase as successful qualification.
- [x] No test in the release-blocking Rust or Svelte suites mutates physical hardware.
- [x] Failure and interruption tests prove the common mutation lease, rollback, crash marker, and recovery lockout apply equally to 2.4 GHz.

## Answer

Implemented qualified 2.4 GHz SSID and Canadian automatic/channel-6 transactions through the same validation, full-object verification, rollback, marker, lease, and recovery machinery as 5 GHz. Unqualified channels are rejected and the partial fixture policy remains enforced.
