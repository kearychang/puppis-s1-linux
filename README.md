# Puppis S1 Manager for Linux

An unofficial Ubuntu desktop manager for the PrismXR Puppis S1 (P1411).

Created collaboratively by Keary Chang + Codex AI (OpenAI).

The production application is a Rust core behind a narrow Tauri 2 shell with a client-only Svelte interface. It observes first, talks directly to NetworkManager over system D-Bus, and enables Puppis mutations only for exact qualified firmware and operations.

The files under `prototype/` are research evidence and are not the production architecture. Normal tests never mutate physical hardware.

## Development

Requirements are a current Rust toolchain, Node.js/npm, NetworkManager development headers, and Ubuntu's WebKitGTK 4.1 development package.

```bash
cargo test --workspace
npm test
npm run check
npm run build
```

Run the desktop shell with the Tauri CLI after installing its development tooling. The application itself remains unprivileged; NetworkManager and the desktop's existing polkit policy authorize system-wide sharing changes.

## Release and Debian package

`npm run release:check` runs the only two release-blocking suites first: `cargo test --workspace` and the safe Svelte suite. It then reports formatting, linting, type checking, version/lockfile consistency, and the production frontend build as advisory evidence. Normal automation never runs a physical hardware mutation.

On Ubuntu 26.04 `amd64`, `npm run package:deb` runs that gate and builds only the Debian target. Inspect the resulting artifact with `scripts/inspect-deb.sh path/to/package.deb`, then record advisory environment and hardware results in [the manual release matrix](docs/release/manual-matrix.md). The package has no maintainer scripts: install, upgrade, and removal cannot modify live networking or connected clients, and it installs no daemon, service, autostart entry, root helper, custom policy, network profile, or firewall configuration.

Fixed keyboard shortcuts are `Alt+1` for Overview, `Alt+2` for Network, `Alt+3` for Wi-Fi, `Alt+4` for Diagnostics, and `Alt+A` for About. The header text-size control offers 100%, 125%, and 150% scaling.
