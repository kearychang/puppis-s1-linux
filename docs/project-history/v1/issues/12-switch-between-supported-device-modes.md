# 12 — Switch safely between PrismPulse and Wi-Fi hotspot modes

**What to build:** Let users switch deliberately between PrismPulse mode and Wi-Fi hotspot mode while expecting client disruption and receiving verified, recoverable results.

**Blocked by:** 08 — Apply a recoverable 5 GHz SSID transaction.

**Type:** implementation

**Status:** resolved

- [x] The current device role is shown using the canonical PrismPulse, Wi-Fi hotspot, and diagnostic-only Wi-Fi adapter terms.
- [x] Only mode 1 and mode 2 are selectable; mode 3 is recognized for diagnostics but cannot be selected.
- [x] Confirmation names both the current and requested roles and warns that connected clients will disconnect.
- [x] A transition uses the global mutation lease and reports success only after the resulting role is independently verified.
- [x] Failure, disconnect, interruption, and ambiguous readback follow the common rollback or recovery-required rules rather than trusting an acknowledgment.
- [x] Fixture-backed application and Svelte tests cover both supported directions, no-op requests, cancellation, warning text, verification failure, and mode-3 gating.

## Answer

Implemented canonical role state, separately confirmed PrismPulse/Wi-Fi-hotspot transitions, client-disconnection warning, diagnostic-only Wi-Fi-adapter recognition, protocol readback verification, rollback/recovery behavior, Tauri intent, accessible confirmation UI, and safe core/Svelte tests.
