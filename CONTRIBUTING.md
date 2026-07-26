# Contributing

Thanks for helping improve Puppis S1 Manager for Linux.

## Start with an issue

Use [GitHub Issues](https://github.com/kearychang/puppis-s1-linux/issues) for bugs, enhancements, and design proposals. Do not include passwords, SSIDs, MAC addresses, serial numbers, diagnostics, raw packet captures, or other private device and network data. Report security-sensitive problems through the private process in [SECURITY.md](SECURITY.md).

## Development

Follow [Building and testing](docs/development/building.md), the vocabulary in [CONTEXT.md](CONTEXT.md), and applicable [architecture decisions](docs/adr/README.md). Keep the Rust core independent of the desktop shell and preserve the read-first, least-privilege design.

Before opening a pull request, run:

```bash
cargo test --workspace
npm test
npm run check
npm run build
```

Tests and examples must use synthetic or redacted fixtures. Pull requests must never run live hardware changes, require a connected Puppis, or modify host networking.

## Hardware research

Do not repeat a live protocol mutation merely to reproduce historical evidence. Hardware-changing research requires the device and firmware preconditions, exact confirmation, restoration behavior, and redaction rules documented in [`research/p1411_protocol/hardware_lab/`](research/p1411_protocol/hardware_lab/README.md). Such work is never part of routine CI and should be proposed in an issue before execution.

By contributing, you agree that your contribution is licensed under the repository's [MIT License](LICENSE).
