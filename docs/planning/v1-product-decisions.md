# V1 product decisions

Confirmed: 2026-07-25

This brief records the product and delivery decisions settled during the production-planning interview. It is an input to the future specification, not an implementation specification itself. Architectural rationale lives in `docs/adr/`, and canonical domain language lives in `CONTEXT.md`.

## Product

- Display name: **Puppis S1 Manager for Linux**.
- Application ID: `io.github.kearychang.PuppisS1Manager`.
- Debian package: `puppis-s1-manager`.
- The application is unofficial and uses original visual assets.
- Attribution: **Created collaboratively by Keary Chang + Codex AI (OpenAI).**
- Original project material is licensed under MIT to Keary Chang; vendor artifacts are excluded.
- V1 supports Ubuntu 26.04 on `amd64` with the original PrismXR Puppis S1 (P1411).
- Client devices are generic and unrestricted; the application does not require or infer a Quest identity to grant network access.

## Delivery shape

- Production stack: Rust, Tauri 2, client-only Svelte with strict TypeScript, and npm.
- V1 ships only as an Ubuntu 26.04 `.deb`.
- Development versions begin at `0.1.0`; the first user-approved public v1 is `1.0.0`.
- No public CLI, app autostart, tray process, resident daemon, Bluetooth support, Flatpak, AppImage, ARM64 support, or runtime self-updater.
- The app makes no outbound internet requests and loads no remote web content.
- The project remains a self-contained subproject of the parent `kearychang/project` repository.

## V1 capabilities

- Discover Puppis candidates and verify P1411 identity through a checkpoint-protected temporary host configuration.
- Report the physical USB link, negotiated speed, and an actionable warning below SuperSpeed without blocking setup.
- Observe, create, activate, disable, repair, and explicitly remove an app-owned system-wide NetworkManager sharing profile.
- Preserve external NetworkManager profiles without editing or deleting them.
- Use IPv4-only downstream sharing at `192.168.137.1/24` while preserving the host's upstream routes and VPN policy.
- Stop without changing networking when another host route overlaps `192.168.137.0/24`.
- Display verified device identity, exact firmware, qualification state, supported capabilities, and useful read-only status.
- Configure 2.4 GHz and 5 GHz SSID, password, and qualified channel values through independent per-radio settings transactions.
- Change between PrismPulse mode and Wi-Fi hotspot mode after a disconnect warning and ordinary confirmation.
- Distinguish Puppis-reported connected clients from host-observed recent client activity; never claim a neighbor-cache entry is currently connected.
- Preview and export a privacy-redacted diagnostics bundle.
- Provide factory reset only after a separately authorized live reset and complete recovery rehearsal qualify it.

## Explicitly outside v1

- Wi-Fi adapter mode (mode 3), including any experimental toggle.
- Region/country mutation, encryption controls, bandwidth controls, and radio enable/disable controls.
- Firmware installation, downgrade, update recommendations, or “latest firmware” claims.
- Bluetooth/EaseLink discovery, pairing, onboarding, or recovery.
- Automatic client identification, access restriction, blacklisting, disconnection, or persistent client labels.
- Upstream selection, route reprioritization, VPN policy, or direct firewall management.
- Automatic restoration after factory reset.

## Mutation safety

- Device mutations are enabled only for specifically qualified firmware and operation combinations.
- Manual channels come from an allowlist qualified by radio, firmware, and reported country; arbitrary entry is prohibited.
- Existing Wi-Fi passwords never leave Rust, appear in logs or diagnostics, or enter persistent storage.
- Each settings transaction reads the full original state, applies one complete requested radio state, verifies it, and restores the original state when required.
- Ambiguous outcomes are reconciled by reading state before any recovery action; mutations are never blindly retried.
- One application-wide mutation lease prevents host-network changes and device mutations from overlapping.
- Ordinary close is prevented after a mutation passes its cancellation point.
- A crash or power loss leaves only a secret-free interrupted-operation marker and enters recovery required on restart.
- Recovery required clears only after coherent state is verified and the user accepts it as the new baseline or completes a dedicated recovery transaction.
- Factory reset, if qualified, requires freshly verified identity, an explicit erased-state warning, the exact phrase `RESET PUPPIS`, and verified outcome reporting.

## Host-network policy

- The unprivileged Rust process calls NetworkManager through system D-Bus and relies on NetworkManager/polkit authorization.
- V1 has no `sudo`, `nmcli`, setuid binary, custom polkit rule, or privileged helper.
- The managed sharing profile is system-wide and carries a stable ownership marker.
- Correct external profiles may satisfy sharing but remain externally managed.
- Repair may activate a managed profile after explaining that it replaces the active downstream configuration; the external profile remains intact.
- Disable deactivates the managed profile and disables autoconnection without deleting it.
- Removal is a separate confirmed in-app operation.
- Install, upgrade, and package removal never change profiles or live networking.
- NetworkManager owns shared-mode firewall rules; the app does not modify nftables, UFW, or firewalld.

## State and UI

- Rust is the sole source of domain truth and publishes full redacted snapshots with monotonic revisions.
- Operational status has independent dimensions for USB, host sharing, protocol identity/reachability, configuration health, and client evidence.
- The four top-level areas are Overview, Network, Wi-Fi, and Diagnostics; factory reset is nested in a destructive area.
- First launch is an adaptive, read-first checklist rather than a mandatory wizard.
- V1 is English-only but keyboard-operable, screen-reader labeled, text-scale friendly, and never communicates state by color alone.
- Fixed keyboard shortcuts are an implementation decision and are not user-configurable in v1.

## Privacy and persistence

- NetworkManager and the Puppis remain authoritative; v1 adds no database or cached device configuration.
- A versioned XDG state file contains only non-secret preferences and interrupted-operation markers.
- Local structured logs are user-inspectable and clearable, capped at 5 MiB, and exclude passwords, SSIDs, serials, MACs, raw frames, and frontend form values.
- Diagnostics replace persistent identifiers with bundle-local aliases and always exclude passwords and password-bearing frames.
- Diagnostics exist only after explicit preview/export.

## Modules

- A deep application module exposes intent-oriented Tauri operations and coordinates workflows.
- A deep Puppis module hides TCP lifecycle, codec details, proprietary command names, qualification, field preservation, transactions, and recovery.
- USB network discovery and NetworkManager host sharing are deliberately separate internal modules joined by an opaque interface identifier for inspectability and learning.
- A diagnostics module consumes structured redacted state rather than scraping protocol or system internals.
- The code begins with two Cargo crates: a UI-independent core library and a thin Tauri shell.
- Tauri errors are typed and safe; raw implementation errors remain internal.
- Production uses one local-only webview with explicitly allowlisted commands and no generic shell, filesystem, or network plugins.

## Protocol behavior

- The client strictly validates magic, length, reserved byte, response type, CRC, UTF-8/JSON, and command correlation.
- One long-lived connection permits exactly one in-flight command.
- Background telemetry reads yield to explicit operations; mutations hold exclusive protocol access through verification and recovery.
- Idempotent getters may reconnect and retry once; device mutations follow reconciliation rules instead.
- USB and NetworkManager state are event-driven.
- Status/client telemetry may use read-only protocol polling while the relevant UI is visible.
- Configuration getters run only for explicit settings and transaction workflows; configuration setters are never polled.

## Tests and release authority

- `cargo test --workspace` and the safe Svelte test suite are release-blocking.
- Normal tests, CI, and packaging checks never mutate physical hardware.
- Hardware mutation tests require a distinct explicit command, a qualified detected device, a clean snapshot, and operation-specific confirmation.
- Factory-reset testing remains separately authorized.
- Formatting, linting, type-checking, production packaging checks, and manual hardware/environment matrices are advisory and reported for the user's release decision.
- The user is the final release authority.
- 2.4 GHz mutation remains a v1 product requirement. The live prototype qualified SSID and channel `6` against the existing 5 GHz transaction standard. Password restoration failed and remains unqualified; the device was reconciled through a separately authorized change that made the 2.4 GHz password match the live 5 GHz password without exposing either secret.

## Deferred evidence work

- Determine why the original 2.4 GHz password could not be restored before considering 2.4 GHz password mutation qualified.
- Qualify additional firmware builds explicitly rather than by version ordering.
- Decode authoritative active-client and telemetry commands without conflating them with host neighbor evidence.
- Investigate mode 3 only as a post-v1 effort with a complete Linux addressing and rollback design.
- Qualify factory reset only through fresh explicit authorization and a complete reset/onboarding/recovery rehearsal.
