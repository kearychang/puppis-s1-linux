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
