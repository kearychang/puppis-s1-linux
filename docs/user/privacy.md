# Privacy

Puppis S1 Manager communicates only with the local Puppis endpoint, Linux system services, and local files needed for its documented state. It includes no analytics, telemetry, crash reporting, update checks, vendor API calls, or remote web content.

The application does not persist Wi-Fi passwords, SSIDs, device serials, raw protocol frames, or full device snapshots. It may persist explicitly saved client labels and MAC addresses, small non-secret preferences, interrupted-operation markers, and a capped structured log. Saved client entries can be inspected and removed by the user.

Diagnostics are created only after explicit export. The preview and export pipeline redact credentials and identifying network/device fields, but users should still review a bundle before sharing it.

Repository fixtures and documentation use synthetic or redacted values. Never attach raw traces or personal diagnostics to a public GitHub issue.
