# 03 — Verify a P1411 with the strict device protocol

**What to build:** Verify that the selected candidate is an original P1411 by communicating through the strict device protocol, then show identity, reachability, firmware qualification, and safe failure states without changing device settings.

**Blocked by:** 02 — Identify Puppis candidates and USB link health.

**Type:** implementation

**Status:** resolved

- [x] The protocol rejects invalid magic, total length, reserved byte, response type, CRC32, UTF-8/JSON, and response-command correlation.
- [x] The transport permits one in-flight protocol exchange and produces stable safe failures for timeout, disconnect, malformed response, and command mismatch.
- [x] A candidate becomes a verified Puppis only after successful P1411 identity verification; generic USB evidence alone is never sufficient.
- [x] Exact firmware allowlisting governs mutation capability, while unknown firmware remains available for read-only diagnostics.
- [x] Overview independently shows protocol reachability and device identity/qualification without collapsing other healthy status dimensions.
- [x] Fixture-backed tests exercise valid identity, every strict-validation boundary, unknown firmware, disconnect, and response mismatch without physical hardware.

## Answer

Implemented strict byte-compatible P1411 framing and validation, safe TCP getter transport, response correlation, exact firmware qualification, candidate promotion only after live model verification, typed failure states, verified-device UI, and fixture/application/Svelte tests.
