# 14 — Qualify and expose guarded factory reset

**What to build:** Only after a separately authorized live reset and complete recovery rehearsal, expose factory reset as an isolated destructive operation with fresh target verification and honest outcome reporting.

**Blocked by:** 08 — Apply a recoverable 5 GHz SSID transaction; 12 — Switch safely between PrismPulse and Wi-Fi hotspot modes; fresh explicit human authorization for the live destructive qualification.

**Type:** hardware-qualification

**Status:** ready-for-human

- [ ] No production reset control exists until the exact operation has passed a separately authorized live reset, rediscovery, identity verification, and recovery rehearsal on supported firmware.
- [ ] Before execution, the application reverifies the selected Puppis identity and summarizes the settings and connectivity consequences that may be erased.
- [ ] Factory reset is isolated from ordinary settings and requires the user to type `RESET PUPPIS` as fresh confirmation.
- [ ] The operation participates in mutation serialization but remains a distinct destructive workflow rather than being mislabeled as an ordinary reversible settings transaction.
- [ ] Success is not claimed until the post-reset outcome is verified, and the UI makes no automatic-restoration promise.
- [ ] Safe fixture and UI tests cover capability gating, destructive-area isolation, stale identity, confirmation mismatch, uncertain outcome, and recovery guidance; physical execution is never part of release-blocking suites.
- [ ] This ticket does not block the Ubuntu v1 package while factory reset remains unqualified and absent.
