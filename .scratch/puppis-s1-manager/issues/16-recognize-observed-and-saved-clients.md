# 16 — Recognize observed and saved clients

**What to build:** Make passive client evidence useful for personal PC-VR troubleshooting by showing honest network details, allowing up to ten explicit saved labels, and replacing the internal revision badge with observed-status freshness.

**Blocked by:** 13 — Report client-device evidence honestly; 15 — Package the accessible Ubuntu v1 experience.

**Type:** implementation

**Status:** claimed

- [ ] Overview refreshes passive client evidence every 15 seconds only while visible, displays last-successful observation freshness, and reports delayed updates after 45 seconds; numeric snapshot revisions remain diagnostic metadata only.
- [ ] Current production evidence remains recently observed until a validated Puppis station getter qualifies connected-client claims; no ping, probe, wake, access control, or reported-hostname lookup is introduced.
- [ ] Client rows are grouped by MAC, show the full MAC and all currently observed IPv4 addresses, use “Unlabeled client” without a saved label, and exclude host, network, broadcast, fixed Puppis, and other Puppis-owned addresses.
- [ ] A label can be saved only from an observed client, is trimmed, 1–40 printable Unicode characters, and unique case-insensitively; saved labels take display precedence and may be explicitly reassociated to a new observed MAC.
- [ ] At most ten saved clients retain the label, MAC, last-observed time, and latest observed address set; saved-but-unobserved clients remain manageable on Overview without a connected Puppis.
- [ ] Rename, Forget, and confirmed Clear all operations have the agreed complete-deletion behavior; unsaved clients disappear when current evidence disappears.
- [ ] Saved clients use an atomic, versioned, user-only XDG JSON file; corrupt, unreadable, or unsupported state disables saved-client mutations without blocking networking or read-only evidence and requires confirmed reset.
- [ ] Raw labels and MACs never enter logs or diagnostics; diagnostics contain only bundle-local saved-client aliases, latest observation metadata, addresses, and current-observation state.
- [ ] Rust application-contract, XDG adapter, diagnostics, and Svelte contract tests cover the behavior without physical hardware mutation.

## Comments

The design was resolved through a `grill-with-docs` session and recorded in root `CONTEXT.md` and ADR 0024.
