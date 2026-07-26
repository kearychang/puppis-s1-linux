# Puppis S1 Manager for Linux v1

Status: ready-for-agent

## Problem Statement

Linux users of the original PrismXR Puppis S1 (P1411) do not have a vendor-supported desktop manager. Ubuntu can already use the device as a standard USB Ethernet interface and can share the host's internet connection through NetworkManager, but setup, verification, diagnostics, radio configuration, and recovery currently require manual commands or the unsupported Windows application.

The user needs an unofficial Ubuntu desktop application that can safely identify the correct hardware, establish host internet sharing without disrupting unrelated networking, expose only device behavior proven against qualified firmware, and explain failures precisely. The application handles network configuration and Wi-Fi credentials, so optimistic device identification, loose protocol parsing, blind retries, secret leakage, or broad privilege are unacceptable.

The reverse-engineering prototype established the P1411 TCP transport and fully validated 5 GHz SSID, password, and channel changes plus modes 1 and 2. A later 2.4 GHz prototype fully validated SSID and channel 6 transactions, but an original password could not be restored after a successful password change. The device was explicitly reconciled by making its 2.4 GHz password match its live 5 GHz password, with no prototype credentials left active. Production must preserve that distinction: 2.4 GHz password mutation is a v1 goal but is not yet safe to expose.

## Solution

Build **Puppis S1 Manager for Linux**, an unofficial Ubuntu 26.04 `amd64` desktop application distributed as a `.deb`. A Rust core will own hardware discovery, NetworkManager integration, the P1411 protocol, qualification policy, settings transactions, recovery, diagnostics, and authoritative state. A local-only Tauri 2 shell with a client-only Svelte/TypeScript frontend will present four areas: Overview, Network, Wi-Fi, and Diagnostics.

The application will initially observe without changing anything. It will identify generic USB-network candidates, use a user-selected and checkpoint-protected temporary host configuration when protocol identity must be bootstrapped, and persist a system-wide managed sharing profile only after the device identifies itself as P1411. Existing external profiles remain untouched. NetworkManager and its existing polkit policy provide all required authorization; the app itself remains unprivileged.

Device mutations will be available only for qualified firmware and qualified operation/value combinations. Every reversible mutation will read the original state, send one complete requested state, read and verify the complete result, and restore on unacceptable outcomes. Ambiguous results are reconciled before recovery and never blindly retried. The application enters recovery required when state cannot be verified or restored and blocks ordinary mutations until the user completes evidence-based reconciliation.

## User Stories

1. As an Ubuntu user, I want to install one native Debian package, so that I can launch the manager from my desktop environment.
2. As an Ubuntu user, I want the application to identify itself as unofficial, so that I do not mistake it for PrismXR-supported software.
3. As a privacy-conscious user, I want the application to make no outbound internet requests, so that managing local hardware does not create an external telemetry surface.
4. As a user without a connected Puppis, I want the application to remain useful in a read-only state, so that I can understand what hardware is missing.
5. As a user connecting a generic ASIX adapter, I want it labeled only as a Puppis candidate, so that the app does not falsely identify unrelated hardware.
6. As a user with one Puppis candidate, I want to explicitly select it before bootstrap, so that no network configuration is applied to the wrong interface.
7. As a user with multiple candidates, I want to see and select exactly one device, so that v1 never coordinates ambiguous or multiple Puppis networks.
8. As a user bootstrapping a candidate, I want the temporary host configuration protected by timed rollback, so that a failed identity check does not strand my interface.
9. As a user bootstrapping a non-Puppis device, I want the temporary configuration removed after failed identity, so that the app leaves no persistent configuration behind.
10. As a user with a verified Puppis, I want the app to report the P1411 identity and exact firmware build, so that I know which device is being managed.
11. As a user with unknown firmware, I want diagnostics but no device mutations, so that a new or old build cannot be changed optimistically.
12. As a user with qualified firmware, I want to see which individual capabilities are qualified, so that read-only and mutable features are explicit.
13. As a user, I want the negotiated USB speed shown prominently, so that I can distinguish a USB bottleneck from Wi-Fi performance.
14. As a user on a USB 2 path, I want an actionable warning without losing device controls, so that I can fix the port, cable, or hub when convenient.
15. As a user on SuperSpeed, I want that healthy state reported separately from Wi-Fi link rate, so that unlike measurements are not conflated.
16. As a user, I want USB and host-network state to update from system events, so that the dashboard reacts promptly without unnecessary polling.
17. As a user, I want the application to recognize an already-correct external sharing profile, so that existing working configuration remains useful.
18. As a user with an external profile, I want the application never to edit or delete it, so that it cannot appropriate configuration it does not own.
19. As a user choosing repair, I want to be told that an app-owned profile will replace the active downstream configuration, so that activation is not surprising.
20. As a user enabling managed sharing, I want a system-wide profile, so that NetworkManager can restore it after reboot or device replug without the app running.
21. As a user, I want managed sharing to use `192.168.137.1/24`, so that the fixed P1411 management endpoint remains reachable.
22. As a user, I want the Puppis-facing profile to provide IPv4 sharing without a downstream default route, so that it does not divert the host's internet traffic.
23. As a user, I want v1 to disable IPv6 on its managed profile, so that untested router advertisements or routes cannot alter the supported topology.
24. As a user whose host already routes `192.168.137.0/24` elsewhere, I want setup to stop and identify the conflict, so that the app does not create ambiguous routing.
25. As a user with Ethernet, Wi-Fi, or a VPN upstream, I want the app to respect the host's effective default route, so that it does not become a general route manager.
26. As a VPN user, I want the app to describe but not override effective VPN routing, so that host policy remains authoritative.
27. As a user without an upstream route, I want a clear warning rather than invented upstream configuration, so that unrelated connections are not modified.
28. As a user authorizing a sharing change, I want the normal desktop polkit prompt, so that the entire application never runs as root.
29. As a user, I want NetworkManager to own forwarding, DHCP/DNS, NAT, and shared-mode firewall rules, so that the app does not install competing firewall configuration.
30. As a user disabling sharing, I want the managed profile deactivated and autoconnection disabled but the profile retained, so that re-enabling is predictable.
31. As a user removing managed setup, I want a distinct confirmed action that deletes only the managed profile, so that cleanup is intentional.
32. As a user uninstalling or upgrading the package, I want live networking left untouched, so that package maintenance cannot disconnect clients.
33. As a user, I want Overview to show USB, sharing, protocol, configuration-health, and client-evidence statuses independently, so that one failure does not hide healthy dimensions.
34. As a user, I want a summary of the independent status dimensions, so that I can see the most actionable issue without losing detail.
35. As a user, I want exact typed failures with recovery guidance, so that raw socket or D-Bus errors do not become confusing UI messages.
36. As a user, I want read-only device status to refresh slowly while the app is open, so that information remains current without excessive protocol traffic.
37. As a user viewing client telemetry, I want it polled only while relevant UI is visible, so that background reads yield to explicit operations.
38. As a user, I want any phone, headset, or other wireless station treated as a generic client device, so that the app does not restrict the network to a Quest.
39. As a user, I want “connected client” used only for Puppis-reported active stations, so that cached host neighbors are not presented as current connections.
40. As a user, I want host neighbor evidence labeled “recently observed,” so that its uncertainty is honest.
41. As a user, I want no automatic client whitelisting, blacklisting, disconnection, or device-type access rules, so that inferred identity never controls access.
42. As a user, I want no persistent client labels in v1, so that randomized hardware addresses do not create stale private state.
43. As a user opening Wi-Fi settings, I want the current non-secret 2.4 GHz and 5 GHz configuration, so that I can prepare deliberate changes.
44. As a user, I want the existing Wi-Fi password withheld from the webview, so that it cannot leak through frontend state or IPC events.
45. As a user changing a password, I want the new value cleared from the form after submission, so that it does not linger in frontend memory longer than needed.
46. As a user, I want SSID, password, and channel to be the only mutable radio fields in v1, so that unqualified encryption, bandwidth, region, and enable controls remain protected.
47. As a user, I want hidden setter fields preserved from the live snapshot, so that the app never invents protocol defaults.
48. As a user, I want each radio to have its own Apply action, so that a failure on one radio does not roll back a successful transaction on the other.
49. As a user, I want to change multiple exposed fields within one radio in one transaction, so that one complete desired radio state is verified atomically from my perspective.
50. As a user, I want automatic channel available when qualified, so that the device can select its channel normally.
51. As a user choosing a manual channel, I want only values qualified for the radio, firmware, and reported country, so that arbitrary numeric input cannot create an unsafe combination.
52. As a user on an unknown country/channel combination, I want radio settings readable but manual channel changes disabled, so that uncertainty fails safely.
53. As a 5 GHz user, I want qualified SSID, password, and channel transactions, so that the proven controls are available in production.
54. As a 2.4 GHz user, I want qualified SSID transactions, so that the live-validated control is available.
55. As a Canadian 2.4 GHz user on qualified firmware, I want automatic and channel 6 behavior represented by qualification evidence, so that the proven channel path is available.
56. As a 2.4 GHz user, I want password mutation hidden until restoration behavior is understood and qualified, so that the prototype failure cannot be repeated in production.
57. As a user applying a reversible mutation, I want the full original state read first, so that restoration has an exact in-memory target.
58. As a user applying a reversible mutation, I want the complete requested object read back and compared, so that `status=ok` alone is not treated as success.
59. As a user whose mutation receives no response, I want the app to reconnect and reconcile state before doing anything else, so that it does not duplicate an applied command.
60. As a user whose device matches the requested state after reconnect, I want the operation reported successful, so that a lost response does not create false failure.
61. As a user whose device matches the original state after reconnect, I want the operation reported failed with no change, so that the outcome is precise.
62. As a user whose state is mixed or unknown, I want one restoration attempt followed by verification, so that recovery is bounded.
63. As a user whose restoration cannot be verified, I want the app to enter recovery required and block ordinary mutations, so that it cannot compound unknown state.
64. As a user in recovery required, I want read-only diagnostics to remain available, so that I can understand and reconcile the device.
65. As a user in recovery required, I want the state cleared only after coherent verification and explicit acceptance or a dedicated recovery transaction, so that restart or dismissal cannot hide risk.
66. As a user, I want host-network and Puppis mutations serialized application-wide, so that a network transition cannot interrupt a device transaction.
67. As a user, I want conflicting controls disabled while a mutation is active, so that the operation ordering is visible and enforceable.
68. As a user closing the window after a mutation point, I want the app to finish verification or recovery first, so that ordinary close cannot abandon the transaction.
69. As a user returning after a crash or power loss, I want a secret-free interrupted-operation marker to trigger recovery required, so that the app does not claim an unknown result.
70. As a user, I want secrets kept only in memory during a transaction, so that crash recovery never depends on persisting credentials.
71. As a user switching between PrismPulse mode and Wi-Fi hotspot mode, I want a confirmation naming both roles and warning that clients will disconnect, so that the disruption is expected.
72. As a user, I want mode 1 and mode 2 transitions verified and recoverable, so that a response alone does not establish the active role.
73. As a user encountering Wi-Fi adapter mode, I want it recognized for diagnostics but unavailable as a transition, so that the unproven host-address change cannot strand management.
74. As a user, I want factory reset absent until a separately authorized live reset and full recovery rehearsal qualify it, so that discovery of a command is not mistaken for safe support.
75. As a user with qualified factory reset, I want it isolated in a destructive area, so that it cannot be confused with ordinary settings.
76. As a user initiating factory reset, I want identity reverified and erased settings summarized, so that the destructive target and consequence are current.
77. As a user initiating factory reset, I want to type `RESET PUPPIS`, so that approval is fresh and deliberate.
78. As a user completing factory reset, I want no automatic-restoration promise and no success claim until outcome verification, so that recovery limits are honest.
79. As a support-seeking user, I want a diagnostics preview before export, so that I can inspect what will be shared.
80. As a privacy-conscious user, I want passwords and password-bearing frames always excluded, so that a support artifact cannot disclose credentials.
81. As a privacy-conscious user, I want SSIDs excluded and serials/MACs replaced with bundle-local aliases, so that exported diagnostics do not expose persistent identifiers.
82. As a support-seeking user, I want firmware, USB speed, routes, NetworkManager state, validation failures, and safe error details retained, so that redaction does not make the bundle useless.
83. As a user, I want local logs inspectable and clearable, so that troubleshooting state remains under my control.
84. As a user, I want local logs capped at 5 MiB and free of credentials, SSIDs, serials, MACs, raw frames, and form values, so that retention is bounded and private.
85. As a user, I want no database or cached device configuration, so that NetworkManager and the Puppis remain the sources of truth.
86. As a user, I want non-secret preferences and interrupted markers stored in versioned XDG state, so that upgrades can preserve minimal application state safely.
87. As a keyboard user, I want every control operable by keyboard with sensible fixed shortcuts, so that the app does not require a pointer.
88. As a screen-reader user, I want labeled controls and status semantics, so that device health and destructive actions are understandable.
89. As a low-vision user, I want increased text scale and non-color-only status cues, so that the UI remains legible.
90. As an English-language v1 user, I want strings centralized even before localization ships, so that later translations do not require UI redesign.
91. As a package maintainer, I want one version shared across Rust, Tauri, frontend, and Debian metadata, so that artifacts cannot disagree about the release.
92. As a package maintainer, I want committed Rust and npm lockfiles, so that dependency changes are deliberate and reproducible.
93. As a maintainer, I want dependency and firmware-qualification updates to occur only in reviewed releases, so that runtime behavior cannot drift automatically.
94. As a maintainer, I want vendor binaries, logos, decompiled source, and raw traces excluded from source and releases, so that the MIT project contains only redistributable original material and safe evidence.
95. As a maintainer, I want the application credited to Keary Chang + Codex AI (OpenAI), so that human-AI collaboration is explicit without misidentifying the copyright holder.
96. As a release authority, I want safe Rust and Svelte test failures to block release, so that core observable behavior cannot knowingly regress.
97. As a release authority, I want hardware-mutating tests excluded from normal tests and CI, so that ordinary automation cannot change physical configuration.
98. As a release authority, I want hardware checks, formatting, linting, type-checking, packaging checks, and environment matrices reported as advisory evidence, so that I retain the final ship/hold decision.

## Implementation Decisions

- The supported environment is Ubuntu 26.04 `amd64` with an original P1411. Other Linux distributions, CPU architectures, and Puppis revisions receive no v1 compatibility promise.
- The display name is **Puppis S1 Manager for Linux**, package name is `puppis-s1-manager`, and application ID is `io.github.kearychang.PuppisS1Manager`.
- The app is explicitly unofficial, uses original visual assets, and displays the attribution “Created collaboratively by Keary Chang + Codex AI (OpenAI).” Original project material is MIT licensed to Keary Chang.
- Begin with two Cargo crates: a UI-independent core library and a thin Tauri shell. Modules do not require separate crates to remain real seams.
- The deep application module is the single external interface used by Tauri and application-level tests. It exposes intent-oriented operations and full redacted state snapshots rather than low-level module calls.
- Rust owns authoritative domain state. Snapshots carry monotonic revisions; Svelte owns only presentation and transient form drafts and requests a fresh snapshot after an event gap.
- Failures crossing Tauri contain a stable machine-readable code, safe message, recovery/retry guidance, and an optional redacted diagnostic reference. Raw implementation errors remain internal.
- The Tauri production shell contains one local-only webview. It has no navigation, remote content, generic shell/filesystem/network plugins, or release-mode developer tools. Commands are explicitly allowlisted for the main window. User-selected documentation links open in the system browser.
- Use Svelte with strict TypeScript as a static client application and npm with a committed lockfile. There is no SSR or application server.
- The USB network discovery module enumerates Puppis candidates, correlates USB topology with Linux interfaces, and reports link evidence and negotiated speed.
- The host sharing module observes and reconciles NetworkManager state for an opaque selected interface identifier. It owns subnet-conflict checks, temporary bootstrap configuration, timed checkpoints, stable profile ownership, activation, disablement, deletion, and rollback.
- USB network discovery and host sharing remain separate internal modules for inspectability and learning. The application module owns their ordering and correlation; neither interface crosses Tauri.
- The application process is unprivileged and calls NetworkManager directly over system D-Bus. Existing NetworkManager/polkit policy provides interactive authorization. There is no root GUI, `sudo`, `nmcli`, setuid binary, custom privileged helper, custom polkit rule, or app daemon in v1.
- Managed sharing uses a system-wide, stably marked NetworkManager profile with IPv4 shared mode, `192.168.137.1/24`, no downstream gateway/default route, autoconnection, and IPv6 disabled.
- Correct external sharing profiles are observable but immutable. Repair may activate a managed profile only after explaining the active downstream change. Package install, upgrade, and removal never alter networking.
- A Puppis subnet conflict blocks changes. V1 never rewrites upstream connections, route metrics, VPN policy, nftables, UFW, or firewalld.
- The deep Puppis module exposes task-oriented device state and operations. It hides endpoint details, proprietary command names, raw frames, setter schemas, full secret-bearing snapshots, TCP lifecycle, qualification logic, and recovery machinery.
- The protocol codec enforces magic, total length, reserved byte, response type, CRC32, UTF-8/JSON, and response-command correlation even when the official client is more permissive.
- The P1411 transport uses one long-lived connection with exactly one in-flight command because responses lack request IDs. Background reads yield to explicit operations, transactions hold exclusive protocol access, and stale queued reads are discarded after disconnect.
- Idempotent getters may reconnect and retry once. Mutations never retry until observed state has been reconciled.
- USB and NetworkManager observations are event-driven. Slow device-status polling occurs only while the app is open. Client/streaming telemetry polls only while relevant UI is visible. Password-bearing configuration getters are on-demand only; mutations are never polling actions.
- Operational state has five orthogonal dimensions: physical USB link, host internet sharing, Puppis identity/protocol reachability, device configuration health, and client evidence.
- The application manages exactly one selected Puppis at a time. Generic USB identity never proves P1411 identity. Bootstrap uses a temporary NetworkManager configuration under a timed checkpoint, promotes only a verified P1411 to managed persistence, and sends no device mutation during identity bootstrap.
- Firmware qualification is an explicit allowlist of exact builds and capabilities, not a minimum-version comparison. Unknown, newer, or older builds remain read-only.
- A qualified channel is specific to radio, exact firmware, and reported country. Arbitrary channel entry is prohibited.
- Radio setters preserve the complete live eight-field object (`ssid`, `pwd`, `pt`, `ch`, `code`, `en`, `encrypt`, `bw`) while changing only user-requested qualified fields.
- Each radio has an independent Apply action. Within one radio, all requested exposed changes form one settings transaction; there is no cross-radio Apply All transaction.
- Existing passwords remain inside Rust and never cross Tauri or enter persistent storage. A new password necessarily enters the frontend draft, is sent only through the narrow command, and is cleared immediately afterward.
- A settings transaction snapshots the original complete object, writes the complete requested object, reads and verifies the complete requested object, and restores and verifies the original object on an unacceptable outcome.
- After an ambiguous response, matching the requested state establishes success; matching the original establishes failure with no change; a mixed, unknown, or unreadable state permits one restoration attempt. Unverified restoration enters recovery required.
- One application-wide mutation lease serializes every NetworkManager or Puppis mutation. Read-only observation may continue where safe. Cancellation ends at the first mutation; after that, verification or recovery must reach a terminal state.
- Ordinary window close is prevented after the cancellation point. Unexpected termination persists only a secret-free interrupted-operation marker and causes recovery required on restart.
- Recovery required cannot be dismissed. It clears only after the app verifies coherent state and the user accepts it as the new baseline or completes a dedicated recovery transaction.
- The validated 5 GHz production capability covers SSID, password, and qualified channel mutations on qualified firmware.
- The validated 2.4 GHz production capability currently covers SSID and Canadian channel 6/automatic behavior on the tested firmware. Password mutation remains unavailable until the failed original-password restoration is explained and a complete reversible proof succeeds.
- Mode transitions expose PrismPulse mode and Wi-Fi hotspot mode only. Every change receives ordinary confirmation and a client-disconnect warning, then uses verification and recovery semantics. Wi-Fi adapter mode is recognized but cannot be selected.
- Factory reset is excluded from the enabled capability set until a fresh, separately authorized live reset and complete onboarding/recovery rehearsal succeed. Once qualified, it remains a distinct destructive operation requiring current identity, an erased-settings summary, no restoration promise, the exact phrase `RESET PUPPIS`, and verified reset outcome.
- Client devices remain generic. Only validated Puppis station data supports “connected client”; host neighbor/DHCP evidence supports “recently observed” only. V1 provides no automatic identity, access control, blacklist, disconnection, or persistent label behavior.
- The diagnostics module accepts structured redacted snapshots and produces preview/export artifacts. It cannot inspect raw protocol frames, credentials, or NetworkManager directly. Identifier aliasing and mandatory exclusion are enforced at this seam.
- NetworkManager and the Puppis remain authoritative. V1 uses no database or persisted device snapshot. Versioned XDG state contains only non-secret preferences and interrupted-operation markers.
- Local structured logs are user-inspectable and clearable, capped at 5 MiB, and exclude credentials, SSIDs, serials, MACs, raw frames, and frontend form values. Diagnostics reprocess logs through mandatory redaction.
- V1 makes no outbound requests, analytics submissions, crash reports, update checks, vendor API calls, or remote-content loads. Package updates are external to the app.
- Commit `Cargo.lock` and `package-lock.json`. Dependency and firmware-qualification changes occur only through reviewed releases.
- Use one SemVer version across application metadata. Development begins at `0.1.0`; the user's first approved public v1 is `1.0.0`.
- The package installs no daemon, service, custom policy, network profile, or live network state. It declares runtime dependencies and desktop metadata only.
- The top-level UI areas are Overview, Network, Wi-Fi, and Diagnostics. Factory reset, if qualified, is nested in a destructive section.
- First launch is an adaptive read-first checklist. An already-working verified Puppis opens Overview; candidates require selection; authorization follows an explanation; read-only diagnostics remain reachable throughout.
- V1 is English-only. Strings are centralized, controls are keyboard-operable and screen-reader labeled, increased text scale is supported, and status never relies on color alone. Shortcuts are fixed implementation choices and are not user-configurable.
- No public CLI ships. Developer reconnaissance harnesses remain throwaway/read-only evidence tools rather than a second product interface.
- Vendor binaries, artwork, decompiled source, raw traces, raw password-bearing frames, actual passwords, and serials are excluded from source and release artifacts. Independently written logic, factual notes, checksums, and privacy-redacted fixtures may be retained as evidence.
- The project remains a self-contained subproject of the parent repository and must not depend on or modify sibling projects.

## Testing Decisions

- The highest and preferred test seam is the deep application interface used by Tauri. Tests issue user intents, observe operation results and revisioned redacted snapshots, and assert externally visible behavior without reaching through the interface.
- A good test describes a user-observable invariant, safety rule, state transition, or artifact. It survives internal refactors and does not assert private call order, private fields, D-Bus object paths, raw transport implementation, or Svelte internals.
- The application module is tested with replaceable adapters for true external dependencies. Its tests cover candidate bootstrap, identity success/failure, managed/external profile policy, mutation serialization, operation gating, ambiguous outcomes, recovery required, crash markers, and snapshot revisions.
- The Puppis module is tested through its task-oriented interface with a scripted transport adapter. Tests cover strict frame rejection, response correlation, single-flight behavior, read retries, firmware/capability gating, hidden-field preservation, complete-object verification, reconciliation, restoration, and recovery lockout.
- The protocol codec also receives focused fixture tests because binary validation is a pure internal seam with independently captured evidence. Tests include known request/response fixtures plus corrupt magic, length, reserved byte, type, CRC, UTF-8, JSON, and mismatched command cases.
- The USB network discovery module is tested through its own internal interface because its split was deliberate. Synthetic sysfs/udev observations cover generic ASIX false positives, candidate evidence, interface correlation, multiple candidates, unplug/replug, and USB speed classification.
- The host sharing module is tested through its internal interface with a fake or disposable NetworkManager adapter. Tests cover observation, ownership markers, external-profile immutability, temporary bootstrap, checkpoint timeout/rollback, system-wide profile reconciliation, disable versus remove, conflicts, no upstream, IPv4-only policy, and preservation of upstream routes.
- The diagnostics module is tested only through preview/export results. Golden structured inputs verify mandatory password/raw-frame exclusion, SSID exclusion, bundle-local identifier aliasing, useful retained evidence, deterministic safe error representation, and the absence of implementation-specific scraping.
- Svelte tests operate against the application/Tauri contract rather than mocking internal Rust modules. They cover the five status dimensions, first-launch checklist, candidate selection, confirmation flows, mutation lockout, recovery-required UX, client-evidence language, diagnostics preview, password-form clearing, accessibility labels, keyboard operation, and destructive-area isolation.
- Existing validated protocol fixtures and the throwaway live prototypes are prior art and primary evidence. Production tests reuse privacy-redacted facts, not prototype architecture or secret-bearing frames.
- The 2.4 GHz partial fixture is a required regression case: it proves SSID and channel behavior while demonstrating that password mutation must remain unavailable until a later complete reversible fixture replaces the failed evidence.
- `cargo test --workspace` is release-blocking and contains only safe non-hardware-mutating tests.
- The safe Svelte test suite is release-blocking and never reaches physical hardware.
- Hardware-mutating tests are excluded from normal test commands, CI, and packaging. They require a distinct explicit command, a detected qualified device, a clean preflight snapshot, operation-specific confirmation, redacted output, and verified final state.
- Factory reset testing remains in a separately authorized path and cannot be enabled by a generic hardware-test flag.
- Formatting, linting, type-checking, production packaging checks, and manual environment/hardware matrices are advisory evidence rather than mechanical release blockers. Their outcomes remain visible to the user, who is the final release authority.
- Advisory manual coverage includes clean and existing setup, conflicts, upstream variants and VPNs, reboot/replug/suspend, USB 2 and SuperSpeed, unknown firmware, package install/remove, diagnostics inspection, and qualified reversible device operations.

## Out of Scope

- Linux distributions other than Ubuntu 26.04.
- CPU architectures other than `amd64`.
- Puppis revisions/models other than the original P1411.
- Flatpak, AppImage, Snap, RPM, or an APT repository.
- A public CLI, resident tray process, autostart application, background daemon, root helper, setuid binary, or custom polkit policy.
- Bluetooth/EaseLink discovery, pairing, onboarding, or recovery.
- Wi-Fi adapter mode (mode 3), including experimental toggles.
- Region/country mutation.
- Radio encryption, bandwidth, or enable/disable controls.
- Arbitrary channel entry or unqualified country/channel combinations.
- Enabled 2.4 GHz password mutation until complete reversible qualification succeeds.
- Firmware download, installation, upgrade, downgrade, authenticity verification, latest-version claims, or update recommendations.
- Automatic client identity, Quest-only behavior, automatic labels, access control, blacklist management, or client disconnection.
- Upstream selection, route reprioritization, VPN policy management, bridges, or direct firewall rule management.
- IPv6 host sharing.
- Multiple simultaneously managed Puppis devices.
- Automatic restoration after factory reset.
- Factory reset itself unless and until separately qualified; inability to qualify it does not block the rest of v1.
- Persistent credentials, cached device configuration, a local database, cloud accounts, analytics, crash reporting, or runtime update checks.
- Remote web content or generic Tauri shell/filesystem/network capabilities.
- Localization beyond English in v1, although strings remain localizable.
- User-configurable keyboard shortcuts.
- Redistribution of vendor binaries, vendor artwork, decompiled source, private traces, or password-bearing captures.

## Further Notes

- The canonical vocabulary is defined in the project glossary. In particular, distinguish Puppis candidate from verified Puppis, connected client from recently observed client, supported mutation from discovered command, settings transaction from factory reset, and managed sharing profile from external sharing profile.
- Existing ADRs are authoritative for firmware allowlisting, Tauri/Rust architecture, unprivileged NetworkManager integration, mutation reconciliation, network-profile ownership, candidate bootstrap, factory-reset qualification, orthogonal status, upstream routing, strict protocol validation, single-flight transport, module seams, diagnostics, typed failures, revisioned snapshots, mutation serialization, no outbound requests, minimal persistence, hardware-test isolation, repository placement, and vendor-artifact exclusion.
- The current physical device ended the 2.4 GHz prototype reconciled in PrismPulse mode with both radios on automatic channel, no prototype SSID/password active, and the 2.4 GHz password intentionally matching the live 5 GHz password. The original 2.4 GHz password was not recoverable and was not persisted.
- The 2.4 GHz password failure is not permission to repeat the live experiment automatically. A future investigation must begin from the redacted partial fixture and obtain explicit authorization before any new hardware mutation.
- Factory reset remains prohibited without fresh, explicit authorization acknowledging configuration erasure, regardless of whether a dry-run frame or UI exists.
- Prototype scripts and fixtures are primary evidence, not production architecture. Production is implemented independently and test-first.
- After this spec, use the ticketing flow to create blocker-ordered tracer-bullet tickets. The 2.4 GHz password investigation must block only the ticket that exposes that mutation, not foundational codec, host sharing, diagnostics, or already-qualified radio work.
