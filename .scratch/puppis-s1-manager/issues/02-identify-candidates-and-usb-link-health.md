# 02 — Identify Puppis candidates and USB link health

**What to build:** Let the user see plausible USB-network candidates, select one when necessary, and understand its USB link health without treating generic USB identity as proof that it is a Puppis.

**Blocked by:** 01 — Launch a safe, read-only desktop shell.

**Type:** implementation

**Status:** resolved

- [x] USB discovery reports stable opaque candidate identifiers and enough safe presentation data to distinguish candidates without exposing a raw device API to Tauri.
- [x] No candidate, one candidate, multiple candidates, disappearance, and reappearance are represented through revisioned application snapshots.
- [x] The user can select which candidate to inspect when selection is required, while the application manages at most one selected candidate at a time.
- [x] Overview distinguishes SuperSpeed from degraded USB 2 operation and gives actionable, non-alarming guidance.
- [x] Candidate selection never proves P1411 identity and never sends a device mutation.
- [x] Safe tests cover discovery events, ambiguity, stale selection, link-speed reporting, and frontend presentation.

## Answer

Implemented Linux sysfs USB-network discovery, opaque candidate identity, automatic/specified selection, disappearance handling, SuperSpeed/USB 2 health guidance, application/Tauri intents, and accessible candidate presentation with focused safe tests.
