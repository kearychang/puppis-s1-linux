# 13 — Report client-device evidence honestly

**What to build:** Show generic client-device activity without implying that cached host observations prove a device is currently connected or using inferred identity to control access.

**Blocked by:** 03 — Verify a P1411 with the strict device protocol; 05 — Observe host sharing and profile conflicts.

**Type:** implementation

**Status:** resolved

- [x] Only validated Puppis station data may label a device as a connected client; host neighbor or DHCP evidence is labeled recently observed.
- [x] Phones, headsets, and all other stations are treated generically, with no Quest-only assumptions.
- [x] The application provides no automatic device identity, whitelist, blacklist, disconnection, access rule, or persistent client label behavior.
- [x] Client identifiers are not persisted and follow the same safe display, logging, and diagnostics-redaction rules as other identifiers.
- [x] Slow device-status polling occurs only while the application is open, and client/streaming telemetry polling occurs only while its relevant dashboard section is visible.
- [x] Telemetry scheduling never suppresses explicit reads or invokes configuration getters, setters, or mutations; explicit operations take priority.
- [x] Application and Svelte tests cover connected versus recently observed wording, stale neighbors, empty evidence, generic devices, visibility-based polling, and operation priority.

## Answer

Implemented generic, non-persistent client evidence with strict provenance: only validated protocol station evidence may say connected, while host neighbor evidence remains recently observed. Added bundle-local diagnostic aliases, visibility-scoped telemetry polling that never invokes configuration operations, and Rust/Svelte contract coverage for wording, stale and empty evidence, generic devices, and operation priority.
