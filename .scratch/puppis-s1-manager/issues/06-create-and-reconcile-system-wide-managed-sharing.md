# 06 — Create and reconcile system-wide managed sharing

**What to build:** Let the user deliberately create or repair the app-owned, system-wide NetworkManager sharing profile and leave NetworkManager able to restore it after reboot or replug without the app running.

**Blocked by:** 03 — Verify a P1411 with the strict device protocol; 05 — Observe host sharing and profile conflicts.

**Type:** implementation

**Status:** resolved

- [x] The first-launch flow explains the proposed network change before normal desktop polkit authorization is requested.
- [x] Bootstrap uses a user-selected candidate and a timed NetworkManager checkpoint, sends no device mutation, and promotes configuration to persistent managed sharing only after P1411 verification.
- [x] The managed profile is system-wide and stably owned, uses IPv4 shared mode at `192.168.137.1/24`, installs no downstream default route, autoconnects, and disables IPv6.
- [x] Reconciliation can create, activate, repair, disable, and remove only the app-owned profile while preserving upstream routes and every external profile.
- [x] Conflicts, missing upstream connectivity, denied authorization, verification failure, checkpoint timeout, disconnect, and rollback produce truthful terminal states and recovery guidance.
- [x] NetworkManager remains responsible for forwarding, DHCP/DNS, NAT, and shared-mode firewall behavior; package and application code install no competing rules.
- [x] Safe tests cover ownership, bootstrap, checkpoint rollback, profile reconciliation, disable versus remove, conflicts, no upstream, IPv4-only policy, and external-profile immutability.

## Answer

Implemented checkpoint-protected candidate bootstrap and verification, stable system-wide managed-profile creation/update/activation over NetworkManager D-Bus, exact IPv4 shared addressing and IPv6 policy, safe authorization failures and rollback, owned-profile-only disable/remove, global mutation serialization, Tauri intents, Network UI, and fake-adapter/application/Svelte coverage.
