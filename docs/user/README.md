# User guide

Puppis S1 Manager is a local desktop application for observing and configuring a supported PrismXR Puppis S1. It communicates with the adapter and Linux system services only; it does not contact PrismXR or any other internet service.

1. Connect the Puppis S1 directly over USB.
2. Open the application and review **Overview** before enabling sharing or changing device settings.
3. Use **Network** to inspect the USB link, upstream route, profile ownership, and subnet conflicts.
4. Use **Wi-Fi** only after the app verifies the supported model and qualified firmware.
5. Use **Diagnostics** to preview redacted information before choosing whether to export it.

Status dimensions are deliberately independent. A healthy USB link does not prove host sharing, device identity, radio state, or an active client. Saved client labels mean only that the user chose to remember a MAC-to-label association; they do not prove that a device is currently connected.

See [Network model and troubleshooting](network-and-troubleshooting.md) when setup is incomplete or a state appears contradictory.
