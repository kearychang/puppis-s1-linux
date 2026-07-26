# Research tools

This directory contains independently written, unsupported tools that preserve reproducible protocol evidence. It is not part of the production application or release package.

- [`p1411_protocol/offline/`](p1411_protocol/offline/) contains a frame inspector that performs no network I/O.
- [`p1411_protocol/captures/`](p1411_protocol/captures/) contains synthetic or privacy-redacted evidence fixtures.
- [`p1411_protocol/hardware_lab/`](p1411_protocol/hardware_lab/) contains live-device tools with explicit safety boundaries.

Read the [P1411 protocol research note](p1411_protocol/README.md) before using any tool. Vendor binaries, decompiled source, raw traces, credentials, and private captures are deliberately absent.
