# Building and testing

## Ubuntu prerequisites

Install a current stable Rust toolchain, Node.js/npm, and the native dependencies required by Tauri, WebKitGTK, and the Rust D-Bus binding. Tauri's current distribution-specific package list is the source of truth: [Linux prerequisites](https://v2.tauri.app/start/prerequisites/).

Install JavaScript dependencies exactly from the lockfile:

```bash
npm ci
```

## Safe checks

```bash
cargo test --workspace
npm test
npm run check
npm run build
```

`npm run release:check` runs the release gate and advisory checks. These commands use fixtures and fake adapters; they do not require a Puppis and must never mutate hardware or host networking.

## Desktop shell

```bash
npx tauri dev
```

The application itself remains unprivileged. NetworkManager and the desktop's existing polkit policy authorize system-wide sharing changes.

## Debian package

```bash
npm run package:deb
scripts/inspect-deb.sh "target/release/bundle/deb/Puppis S1 Manager_0.1.0_amd64.deb"
```

Official release binaries are built on Ubuntu 24.04 for a lower glibc baseline, then manually exercised on the supported Ubuntu 26.04 environment.
