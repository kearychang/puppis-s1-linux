# 05 — Observe host sharing and profile conflicts

**What to build:** Show how the selected USB interface is configured by NetworkManager, whether host internet sharing is usable, and whether managed, external, or conflicting profiles exist—all without modifying them.

**Blocked by:** 02 — Identify Puppis candidates and USB link health.

**Type:** implementation

**Status:** resolved

- [x] The unprivileged application reads NetworkManager directly over system D-Bus; it does not use `nmcli`, `sudo`, a privileged helper, a daemon, or a custom polkit policy.
- [x] Network shows upstream/default-route health, shared IPv4 state, downstream addressing, IPv6 policy, profile activation, and authorization limitations using safe domain language.
- [x] App-owned managed sharing profiles, external sharing profiles, unrelated profiles, and subnet conflicts are distinguished accurately.
- [x] Existing working external profiles are reported as external and are never claimed, rewritten, disabled, or deleted by observation.
- [x] Host sharing remains independent of Puppis protocol reachability in the application snapshot.
- [x] Safe adapter and application-interface tests cover no upstream, VPN/upstream variants, conflicts, external profiles, managed ownership markers, and authorization failures.

## Answer

Implemented the host-sharing domain snapshot and a read-only NetworkManager system-D-Bus observer for effective upstream, selected interface, active/settings profile, shared IPv4, IPv6, address, autoconnect, ownership, authorization expectation, and subnet conflicts. External-profile immutability and independent status behavior are covered at the application seam.
