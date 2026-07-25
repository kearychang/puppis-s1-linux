# Validate the proprietary protocol strictly

The production client rejects frames with invalid magic, length, reserved byte, response type, CRC, UTF-8/JSON, or command correlation even where the official Windows receiver appears more permissive. Idempotent getters may reconnect and retry once, while mutations must reconcile observed state before any recovery action; malformed responses indicate protocol incompatibility rather than partial success, favoring integrity and diagnosability over tolerance of undocumented firmware behavior.
