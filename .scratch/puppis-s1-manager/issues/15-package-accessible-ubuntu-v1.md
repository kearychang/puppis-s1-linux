# 15 — Package the accessible Ubuntu v1 experience

**What to build:** Deliver the completed safe feature set as an accessible, reproducible Ubuntu 26.04 `amd64` Debian package with coherent first-launch guidance, attribution, and release gates.

**Blocked by:** 04 — Preview and export a private diagnostics bundle; 06 — Create and reconcile system-wide managed sharing; 09 — Complete qualified 5 GHz channel and password controls; 10 — Apply qualified 2.4 GHz SSID and channel transactions; 11 — Resolve and gate 2.4 GHz password mutation; 12 — Switch safely between PrismPulse and Wi-Fi hotspot modes; 13 — Report client-device evidence honestly.

**Type:** implementation

**Status:** ready-for-agent

- [ ] Overview, Network, Wi-Fi, and Diagnostics form a coherent keyboard-operable experience with visible focus, semantic labels, appropriate announcements, and fixed documented hotkeys.
- [ ] First launch is an adaptive read-first checklist: a working verified Puppis opens Overview, candidates require selection, network authorization follows explanation, and diagnostics remain reachable.
- [ ] About identifies the application as unofficial and displays exactly “Created collaboratively by Keary Chang + Codex AI (OpenAI).” Licensing identifies the original project as MIT licensed to Keary Chang and excludes vendor artifacts.
- [ ] Rust, Tauri, frontend, and Debian metadata share one version, and committed Rust and npm lockfiles make dependency changes deliberate.
- [ ] The `amd64` Debian package targets Ubuntu 26.04, installs desktop metadata and declared runtime dependencies only, and performs no installation-time networking or mutation.
- [ ] Install, upgrade, removal, and application exit leave live networking and connected clients untouched; no daemon, service, autostart process, root helper, custom policy, profile, or firewall state is installed.
- [ ] `cargo test --workspace` and the safe Svelte suite are release-blocking and contain no physical hardware mutations; any failure blocks release until the user decides otherwise.
- [ ] Formatting, linting, type checking, package inspection, and the documented manual hardware/environment matrix are reported as advisory evidence rather than automatic release blockers.
- [ ] The v1 package may ship with 2.4 GHz password replacement deliberately unavailable if ticket 11 preserves and explains the safety gate; ticket 14 remains independently non-blocking.
