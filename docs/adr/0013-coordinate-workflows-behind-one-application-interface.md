# Coordinate workflows behind one application interface

Tauri exposes one intent-oriented application interface and state snapshots rather than separate host-network and Puppis interfaces that the Svelte frontend must sequence. A deep Rust application module coordinates multi-step workflows such as checkpoint-protected candidate preparation, protocol verification, and managed-profile persistence, keeping ordering constraints, compensation, and partial-failure behavior out of the webview.
