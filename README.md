# Puppis S1 Manager for Linux

An unofficial, community-built Linux desktop manager for the PrismXR Puppis S1 (P1411).

The application provides a read-first view of the adapter, prepares host networking through NetworkManager, exposes only hardware operations qualified for the detected firmware, and recognizes previously saved client labels without storing Wi-Fi credentials.

> [!IMPORTANT]
> This project is not affiliated with or endorsed by PrismXR. The v1 binary is supported only on Ubuntu 26.04 LTS `amd64`. Other modern Linux distributions may work from source but have not been validated.

## Capabilities

- Discover and verify a connected P1411 before enabling device changes.
- Explain USB-link, host-sharing, device-mode, radio, and client-evidence state independently.
- Create and reconcile an application-owned NetworkManager sharing profile without running the GUI as root.
- Read qualified radio settings and perform guarded, recoverable operations.
- Preview privacy-redacted diagnostics before explicitly exporting them.
- Persist only explicit client labels and non-secret application state.

The application makes no outbound internet requests, installs no daemon or privileged helper, and never runs physical-hardware mutation in normal tests or CI.

## Install

Download the `.deb` and matching `.sha256` file from [GitHub Releases](https://github.com/kearychang/puppis-s1-linux/releases). Verify and install it as described in [Installation](docs/user/installation.md).

The first public release supports Ubuntu 26.04 LTS on `amd64`. See [Linux distribution compatibility](docs/research/linux-distribution-compatibility.md) for the distinction between supported and likely compatible systems.

## Use

Connect the Puppis S1 over USB, open **Puppis S1 Manager**, and follow the read-first checklist. The app explains proposed NetworkManager changes before the desktop requests authorization.

See the [user guide](docs/user/README.md) for network requirements, status interpretation, privacy behavior, and troubleshooting.

Keyboard shortcuts are `Alt+1` Overview, `Alt+2` Network, `Alt+3` Wi-Fi, `Alt+4` Diagnostics, and `Alt+A` About. Text scaling is available at 100%, 125%, and 150%.

## Develop

The production application is a Rust core behind a narrow Tauri 2 shell with a client-only Svelte interface.

```bash
npm ci
cargo test --workspace
npm test
npm run check
npm run build
```

Start with [Building and testing](docs/development/building.md) and [Architecture](docs/development/architecture.md). Independently written protocol research is retained under [`research/`](research/README.md); live hardware-lab tools are explicitly separated from production code and safe automation.

## Releases and support

Safe CI checks source changes without hardware. Tagged releases build on Ubuntu 24.04 to keep the native ABI floor lower, then require manual Ubuntu 26.04 hardware validation and human approval before publication. Compiled artifacts are attached to GitHub Releases and are never committed.

- [Documentation index](docs/README.md)
- [Release process](docs/release/README.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [License](LICENSE)
- [Attribution and trademark notice](NOTICE.md)
