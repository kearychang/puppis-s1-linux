# Architecture

The application is divided into three layers:

1. `crates/puppis-core` owns domain state, protocol validation, USB and NetworkManager adapters, privacy controls, and task-oriented operations.
2. `src-tauri` exposes a narrow Tauri command boundary and application lifecycle.
3. `src` is a client-only Svelte interface that renders revisioned state and submits explicit intents.

The Rust core is authoritative. The frontend does not independently infer device safety, profile ownership, authorization, or mutation success. NetworkManager is authoritative for host profiles; the Puppis is authoritative for device configuration.

Read [CONTEXT.md](../../CONTEXT.md) for canonical terminology and the [ADR index](../adr/README.md) for the decisions that constrain changes.
