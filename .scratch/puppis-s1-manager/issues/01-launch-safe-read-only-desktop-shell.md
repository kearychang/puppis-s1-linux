# 01 — Launch a safe, read-only desktop shell

**What to build:** A launchable Puppis S1 Manager for Linux desktop application that initially observes without changing the host or device. The Overview presents the five independent status dimensions through the same deep application interface production and tests will use, including a useful no-device state.

**Blocked by:** None — can start immediately.

**Type:** implementation

**Status:** resolved

- [x] The project builds as a UI-independent Rust core and a thin Tauri 2 shell with a client-only Svelte application using strict TypeScript and npm lockfiles.
- [x] Tauri receives intent-oriented operations and monotonic, revisioned, redacted snapshots only through the deep application interface; stable typed failures include safe guidance without leaking raw implementation errors.
- [x] Overview independently represents physical USB link, host internet sharing, Puppis identity/protocol reachability, device configuration health, and client evidence, including unknown and unavailable states.
- [x] The production webview loads only bundled content, has no navigation or release developer tools, and exposes no generic shell, filesystem, or network capability.
- [x] No startup or observation path mutates NetworkManager, the Puppis, or persistent device configuration.
- [x] Application-interface and Svelte contract tests demonstrate the launch and no-device experience.

## Answer

Implemented the two-crate Rust/Tauri workspace, strict client-only Svelte shell, revisioned application snapshot and safe failure contract, five-dimensional no-device Overview, local-only CSP/capability configuration, and focused Rust/Svelte tests.
