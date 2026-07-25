# Expose typed safe operation failures

Failures cross the Tauri seam as stable machine-readable codes with safe user messages, retry or recovery guidance, and optional redacted diagnostic references. Raw D-Bus, socket, protocol, and stack-trace text remains internal to structured logging, preventing implementation details and secrets from becoming a frontend contract while allowing the UI to present consistent actionable outcomes.
