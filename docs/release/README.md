# Release process

Official binaries are published only through [GitHub Releases](https://github.com/kearychang/puppis-s1-linux/releases). Build output is not committed.

1. Confirm the version is consistent across Rust, npm, Tauri, and lockfiles.
2. Run safe CI and `npm run release:check` from a clean checkout.
3. Build the `amd64` Debian package on Ubuntu 24.04.
4. Inspect the package payload with `scripts/inspect-deb.sh`.
5. Generate a SHA-256 checksum for the exact `.deb`.
6. Exercise that artifact on a clean Ubuntu 26.04 system using the [manual release matrix](manual-matrix.md).
7. Review the draft GitHub Release, checksum, notes, and artifacts before publishing it manually.

Hosted CI never accesses physical hardware, mutates host networking, or publishes a release without human approval.
