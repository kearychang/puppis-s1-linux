#!/usr/bin/env bash
set -u

package_path="${1:-}"
if [[ -z "$package_path" || ! -f "$package_path" ]]; then
  echo "usage: scripts/inspect-deb.sh path/to/puppis-s1-manager.deb" >&2
  exit 2
fi

dpkg-deb --info "$package_path"
dpkg-deb --contents "$package_path"
architecture="$(dpkg-deb --field "$package_path" Architecture)"
depends="$(dpkg-deb --field "$package_path" Depends)"
contents="$(dpkg-deb --contents "$package_path")"

[[ "$architecture" == "amd64" ]] || { echo "expected amd64, found $architecture" >&2; exit 1; }
[[ "$depends" == *"libwebkit2gtk-4.1-0"* ]] || { echo "WebKitGTK runtime dependency is missing" >&2; exit 1; }
if grep -Eq '/(systemd|init\.d|polkit-1|NetworkManager/system-connections|xdg/autostart|ufw|nftables)/' <<<"$contents"; then
  echo "package contains prohibited service, policy, profile, autostart, or firewall paths" >&2
  exit 1
fi
echo "Package architecture, dependencies, desktop payload, and prohibited-path inspection passed."
