# Publish revisioned redacted state snapshots

Rust is the sole source of domain truth and publishes small, fully redacted state snapshots carrying a monotonic revision. Svelte owns only presentation and transient form drafts, never infers successful operations locally, and requests a fresh snapshot after an event gap; full snapshots are preferred over a broad patch vocabulary because recoverable synchronization matters more than minimizing local IPC payloads.
