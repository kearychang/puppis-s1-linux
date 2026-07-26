# 16 — Recognize observed and saved clients

**What to build:** Make passive client evidence useful for personal PC-VR troubleshooting by showing honest network details, allowing up to ten explicit saved labels, and replacing the internal revision badge with observed-status freshness.

**Blocked by:** 13 — Report client-device evidence honestly; 15 — Package the accessible Ubuntu v1 experience.

**Type:** implementation

**Status:** resolved

- [x] Overview refreshes passive client evidence every 15 seconds only while visible, displays last-successful observation freshness, and reports delayed updates after 45 seconds; numeric snapshot revisions remain diagnostic metadata only.
- [x] Current production evidence remains recently observed until a validated Puppis station getter qualifies connected-client claims; no ping, probe, wake, access control, or reported-hostname lookup is introduced.
- [x] Client rows are grouped by MAC, show the full MAC and all currently observed IPv4 addresses, use “Unlabeled client” without a saved label, and exclude host, network, broadcast, fixed Puppis, and other Puppis-owned addresses.
- [x] A label can be saved only from an observed client, is trimmed, 1–40 printable Unicode characters, and unique case-insensitively; saved labels take display precedence and may be explicitly reassociated to a new observed MAC.
- [x] At most ten saved clients retain the label, MAC, last-observed time, and latest observed address set; saved-but-unobserved clients remain manageable on Overview without a connected Puppis.
- [x] Rename, Forget, and confirmed Clear all operations have the agreed complete-deletion behavior; unsaved clients disappear when current evidence disappears.
- [x] Saved clients use an atomic, versioned, user-only XDG JSON file; corrupt, unreadable, or unsupported state disables saved-client mutations without blocking networking or read-only evidence and requires confirmed reset.
- [x] Raw labels and MACs never enter logs or diagnostics; diagnostics contain only bundle-local saved-client aliases, latest observation metadata, addresses, and current-observation state.
- [x] Rust application-contract, XDG adapter, diagnostics, and Svelte contract tests cover the behavior without physical hardware mutation.

## Comments

The design was resolved through a `grill-with-docs` session and recorded in root `CONTEXT.md` and ADR 0024.

## Answer

Implemented passive Overview client evidence grouped by full MAC with all observed IPv4 addresses, Puppis/host endpoint filtering, honest recently-observed wording, 15-second Overview-only refresh, and last-successful freshness with delayed-state reporting. Added explicit save, rename, forget, confirmed clear, and confirmed private-MAC reassociation for up to ten labeled clients, including saved-but-unobserved management without a connected Puppis.

Saved recognition now uses an atomic versioned `0600` XDG state file with safe corruption/version failure and confirmed reset. Diagnostics use bundle-local aliases and expose only allowed observation metadata; raw labels and MACs remain local to the presentation contract and are excluded from diagnostics and logs. The lasting model and privacy boundary are recorded in root `CONTEXT.md` and `docs/adr/0024-persist-only-explicitly-saved-client-recognition.md`.
