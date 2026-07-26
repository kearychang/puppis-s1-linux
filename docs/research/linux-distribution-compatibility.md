# Linux distribution compatibility

**Research date:** 2026-07-26

## Conclusion

There is no application-level reason Puppis S1 Manager must run only on Ubuntu 26.04. Its native interfaces—Linux sysfs/procfs, TCP, the system D-Bus, and NetworkManager—are available on many desktop Linux distributions. Tauri also documents development prerequisites for Debian/Ubuntu, Arch, Fedora, Gentoo, openSUSE, Alpine, and NixOS. [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/)

The important distinction is packaging and validation:

- **Supported and tested:** Ubuntu 26.04 LTS on `amd64`, after the existing hardware and release matrix is completed.
- **Likely compatible:** other modern `amd64` desktop distributions with WebKitGTK 4.1, NetworkManager managing the Puppis interface, a working system D-Bus, and an interactive polkit agent. This remains unverified until tested on real installations.
- **Not portable as currently built:** the existing Ubuntu 26.04 `.deb` is not a general Linux binary. It uses Debian-family package names and its executable requires glibc symbols through `GLIBC_2.39`.

Keeping Ubuntu 26.04 `amd64` as the first release's sole supported target is therefore honest. It should not be described as the only distribution on which the source can work.

## What the application actually requires

| Layer | Project evidence | Compatibility consequence |
| --- | --- | --- |
| Desktop shell | Tauri 2 renders through WebKitGTK 4.1 and GTK 3 on Linux. The project requests `libwebkit2gtk-4.1-0` and `libgtk-3-0` in [`src-tauri/tauri.conf.json`](../../src-tauri/tauri.conf.json). | A compatible WebKitGTK 4.1/GTK 3 runtime is required. Tauri publishes build packages for several distribution families. [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/), [WebKitGTK API 4.1 reference](https://webkitgtk.org/reference/webkit2gtk/stable/index.html) |
| Host networking | [`crates/puppis-core/src/network.rs`](../../crates/puppis-core/src/network.rs) connects to `org.freedesktop.NetworkManager` on the system D-Bus and uses shared IPv4 profiles, checkpoints, and volatile activation. | NetworkManager must be installed, running, and managing the USB Ethernet interface. A distribution using only another network daemon is not functionally compatible without installing/configuring NetworkManager. NetworkManager documents these operations as its public system-D-Bus API. [NetworkManager developer API overview](https://networkmanager.dev/docs/developers/), [D-Bus manager API](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.html) |
| Privilege boundary | The process remains unprivileged and relies on NetworkManager authorization. | NetworkManager defaults to authorizing non-root requests with polkit. An interactive desktop session needs an authentication agent when permission is reported as `auth`; desktop environments normally supply one, while headless/minimal sessions may not. [NetworkManager authorization configuration](https://www.networkmanager.dev/docs/api/latest/NetworkManager.conf.html), [polkit architecture](https://polkit.pages.freedesktop.org/polkit/polkit.8.html), [polkit authentication agents](https://polkit.pages.freedesktop.org/polkit/polkit-agents.html) |
| USB discovery | [`crates/puppis-core/src/usb.rs`](../../crates/puppis-core/src/usb.rs) reads Linux `/sys/class/net`; client and route observations also read `/proc`. | This is Linux-specific but not Ubuntu-specific. Containers and sandboxes may hide the host USB/network namespaces even when the application builds successfully. |
| Native ABI | The Rust executable dynamically links glibc, GTK 3, WebKitGTK 4.1, libsoup 3, and `libdbus-1.so.3`. | A binary built on a new distribution can require symbols absent on older distributions. Tauri explicitly recommends building on the oldest intended base system. [Tauri Debian limitations](https://v2.tauri.app/distribute/debian/) |

The checkpoint flag used by the project (`DELETE_NEW_CONNECTIONS`, value `0x02`) has existed since NetworkManager 1.6, so it is not itself a modern-Ubuntu lock-in. [NetworkManager checkpoint flags](https://networkmanager.dev/docs/api/latest/nm-dbus-types.html)

## Current `.deb` boundary

Local inspection of the 0.1.0 release build found:

- architecture: `amd64`;
- declared dependencies: `libdbus-1-3`, `network-manager`, `libwebkit2gtk-4.1-0`, and `libgtk-3-0`;
- highest required glibc symbol version in the executable: `GLIBC_2.39`.

This gives the following expectation, not a support promise:

| Distribution class | Current `.deb` expectation | Source-build expectation |
| --- | --- | --- |
| Ubuntu 26.04 `amd64` | Supported after its manual matrix passes. | Expected. |
| Ubuntu 24.04 `amd64` | **Plausible but must be tested.** Ubuntu 24.04 provides glibc 2.39, matching the executable's current highest symbol requirement. [Ubuntu 24.04 `libc6`](https://packages.ubuntu.com/noble/amd64/libc6) | Expected; Tauri supports the needed development stack. |
| Ubuntu 22.04 and Debian 12 | The current binary is expected to fail because those bases predate glibc 2.39, even though both provide WebKitGTK 4.1 packages. Tauri names them as suitable *build* baselines precisely to produce older-compatible binaries. [Tauri Debian limitations](https://v2.tauri.app/distribute/debian/), [Ubuntu 22.04 WebKitGTK 4.1](https://packages.ubuntu.com/jammy/libwebkit2gtk-4.1-0), [Debian 12 WebKitGTK 4.1](https://packages.debian.org/bookworm/libwebkit2gtk-4.1-0) | Likely after rebuilding on that distribution and testing NetworkManager/polkit behavior. |
| Debian 13 | The native ABI is new enough, but the current package metadata requests `libgtk-3-0` while Debian 13 uses `libgtk-3-0t64`; installation therefore needs explicit testing and probably a Debian-built package. [Debian 13 GTK 3 runtime](https://packages.debian.org/trixie/amd64/libgtk-3-0t64), [Debian 13 WebKitGTK 4.1](https://packages.debian.org/trixie/amd64/libwebkit2gtk-4.1-0) | Likely with a native Debian build. |
| Fedora, Arch, openSUSE, Gentoo | The `.deb` is the wrong package format and dependency vocabulary. | Likely: Tauri provides official prerequisite instructions for each, but the NetworkManager/polkit workflow still needs runtime testing. [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) |
| Alpine | The glibc `.deb` cannot run natively on musl. | Tauri documents Alpine builds, but its different C library and often-minimal desktop/session setup make this a higher-effort target. |

An AppImage would avoid distribution package names, but it would not remove the glibc baseline constraint or the need for host NetworkManager, system D-Bus, and polkit. [Tauri AppImage limitations](https://v2.tauri.app/distribute/appimage/)

## Verification strategy

1. Build release binaries on the oldest intended binary baseline, not the newest. For a low-cost expansion, build on Ubuntu 24.04 and manually validate that same artifact on Ubuntu 26.04. Tauri recommends Docker or GitHub Actions for controlled baseline builds. [Tauri Debian limitations](https://v2.tauri.app/distribute/debian/)
2. Keep Ubuntu 26.04 hardware validation authoritative for the first release. GitHub currently offers an `ubuntu-26.04` hosted runner in public preview, while `ubuntu-24.04` is generally available. [GitHub-hosted runner labels](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job)
3. Add container jobs for compile/package checks on selected distributions. GitHub Actions supports a chosen job-container image, making source compatibility checks feasible. [GitHub Actions container jobs](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/run-jobs-in-a-container)
4. Do not treat container success as functional support. Ordinary hosted CI has neither the Puppis USB hardware nor a normal logged-in desktop's system D-Bus/polkit-agent environment. Real-distribution tests must cover launch, WebKit rendering, NetworkManager discovery, authorization prompts, shared-mode activation/rollback, and package removal.
5. Before claiming a second supported distribution, run the full manual matrix in [`docs/release/manual-matrix.md`](../release/manual-matrix.md) on a clean installation and publish a package native to that distribution family.

## Recommendation

For the first public release, state:

> The supported binary is Ubuntu 26.04 LTS on `amd64`. Other modern Linux distributions may work from source when WebKitGTK 4.1, NetworkManager, system D-Bus, and polkit are available, but they have not yet been validated.

Separately, build the release artifact on Ubuntu 24.04 and validate it on Ubuntu 26.04. That preserves the honest Ubuntu 26.04 support promise while avoiding an unnecessary newer-glibc floor and creating a realistic path to later Ubuntu 24.04 support.
