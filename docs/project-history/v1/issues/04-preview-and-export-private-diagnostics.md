# 04 — Preview and export a private diagnostics bundle

**What to build:** Give the user a useful, inspectable diagnostics preview and an explicitly saved support bundle while enforcing privacy at the diagnostics boundary.

**Blocked by:** 02 — Identify Puppis candidates and USB link health.

**Type:** implementation

**Status:** resolved

- [x] Diagnostics consumes structured redacted snapshots and cannot inspect credentials, raw protocol frames, or NetworkManager directly.
- [x] The preview and exported artifact exclude SSIDs and passwords and replace serials, MAC addresses, and other persistent identifiers with bundle-local aliases.
- [x] Available firmware, USB speed, routes, NetworkManager state, qualification/validation failures, operation state, and safe error details remain useful; later feature tickets can extend the structured snapshot without bypassing redaction.
- [x] Export occurs only after preview and an explicit user-selected destination; no diagnostic data is sent over the network.
- [x] Versioned XDG logs are bounded to 5 MiB and never persist credentials, raw identifiers, raw protocol frames, or cached device configuration.
- [x] Golden-result tests verify mandatory exclusion, deterministic aliasing, retained evidence, safe errors, preview behavior, and export behavior.

## Answer

Implemented structured diagnostics generation with bundle-local aliases and mandatory exclusions, preview-before-export freshness enforcement, explicit-path export, Tauri intents, accessible preview UI, and golden-style core/Svelte coverage.
