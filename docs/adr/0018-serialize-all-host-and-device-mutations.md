# Serialize all host and device mutations

Read-only observation may proceed concurrently, but one application-wide mutation lease covers every operation that changes NetworkManager or the Puppis so a host-network transition cannot invalidate a device transaction. Normal cancellation is accepted only before the first mutation; afterward the operation must reach verified success, verified restoration, or recovery required, and the UI exposes the active operation while disabling conflicts.
