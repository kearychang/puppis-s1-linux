# Installation

## Supported system

The v1 binary supports Ubuntu 26.04 LTS on `amd64` with NetworkManager and a normal graphical desktop session. Other systems may be compatible but are not supported until they pass the same release matrix.

## Install a release

Download the `.deb` and `.sha256` files for the same version from [GitHub Releases](https://github.com/kearychang/puppis-s1-linux/releases), then run:

```bash
sha256sum --check puppis-s1-manager_VERSION_amd64.deb.sha256
sudo apt install ./puppis-s1-manager_VERSION_amd64.deb
```

Replace `VERSION` with the downloaded release version. The application installs a desktop entry and executable only. It does not install a service, daemon, autostart entry, root helper, custom policy, network profile, or firewall rule.

Removing the package does not alter live networking or delete NetworkManager profiles previously created by the application.
