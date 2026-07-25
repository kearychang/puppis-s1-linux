# 07 — Show qualified radio settings without exposing credentials

**What to build:** Show the current non-secret 5 GHz and 2.4 GHz configuration and explain which operations are available for the verified firmware, while keeping existing credentials entirely inside Rust.

**Blocked by:** 03 — Verify a P1411 with the strict device protocol.

**Type:** implementation

**Status:** resolved

- [x] Wi-Fi reads and presents current non-secret settings for both radios only when requested or needed by the visible view.
- [x] Existing passwords never cross Tauri, enter frontend state, appear as masked placeholders derived from a secret, or enter logs, diagnostics, or persisted state.
- [x] Firmware and per-operation qualification produce explicit supported, unsupported, and unavailable capabilities rather than optimistic controls.
- [x] The 5 GHz SSID/password/channel evidence, 2.4 GHz SSID/automatic/channel-6 evidence, and unresolved 2.4 GHz password evidence are represented independently.
- [x] Password-bearing getters are on-demand only; background telemetry polling never invokes configuration getters, setters, or other mutations.
- [x] Application-interface and Svelte tests cover both bands, unknown firmware, partial capability, credential absence, stale revisions, and visible explanations.

## Answer

Implemented on-demand dual-band protocol reads, Rust-only full radio state, redacted radio snapshots, exact firmware/per-operation/country capability gating, independent 5 GHz and 2.4 GHz evidence, explicit 2.4 GHz password unavailability, and safe core coverage.
