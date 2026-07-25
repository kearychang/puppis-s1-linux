# 11 — Resolve and gate 2.4 GHz password mutation

**What to build:** Resolve as much of the 2.4 GHz password restoration failure as the available evidence safely permits, and ensure the application exposes password replacement only after a complete reversible proof reaches the established 5 GHz standard.

**Blocked by:** 10 — Apply qualified 2.4 GHz SSID and channel transactions.

**Type:** investigation

**Status:** resolved

- [x] Investigation begins with the redacted partial fixture and recorded protocol evidence; it does not repeat a live device mutation without fresh explicit user authorization.
- [x] The production capability remains unavailable with a clear explanation unless temporary application, complete-object verification, original-value restoration, and post-restoration verification all succeed.
- [x] A failed, partial, interrupted, or merely write-acknowledged experiment cannot enable the capability.
- [x] If qualification succeeds, 2.4 GHz password replacement uses the same write-only frontend handling, transaction lease, read/write/readback verification, rollback, crash recovery, and secret exclusion as 5 GHz.
- [x] If the failure cannot be safely resolved, the ticket can conclude with the capability deliberately disabled, regression coverage preserving that gate, and actionable findings recorded without secrets.
- [x] Release-blocking safe tests consume fixtures only; any separately authorized hardware test remains isolated and advisory.

## Answer

The available redacted evidence does not explain or supersede the failed original-password restoration. Production therefore keeps 2.4 GHz password mutation unavailable, with an explicit reason and regression coverage proving that neither the successful temporary write nor the authorized reconciliation qualifies it. No live mutation was attempted.
