# Network model and troubleshooting

The Puppis USB Ethernet link normally uses `192.168.137.0/24`: the Linux host uses `192.168.137.1`, and the adapter control endpoint uses `192.168.137.254`. This is local adapter traffic; an internet speed test does not measure game-streaming throughput between the PC and headset.

## Subnet conflicts

Another active interface or VPN using `192.168.137.0/24` can make the adapter unreachable or route traffic to the wrong link. The app reports this as a subnet conflict and does not overwrite unrelated profiles. Change the conflicting network or disconnect it before enabling Puppis sharing.

## Authorization

The app is not run as root. NetworkManager uses the desktop's polkit agent to request permission for system-wide profile changes. If no prompt appears, confirm that NetworkManager is running and that the graphical session has a polkit authentication agent.

## Clients

An observed client has current network evidence. A saved client is a user-provided label associated with a MAC address and may be offline. The app refreshes observations every 15 seconds, retains at most ten saved labels, and never pings a MAC address to infer presence.

## Useful checks

- Confirm the Puppis appears as a USB Ethernet interface.
- Confirm no other route covers `192.168.137.0/24`.
- Confirm NetworkManager manages the interface.
- Reopen **Overview** after reconnecting the adapter.
- Preview Diagnostics for redacted state and recovery guidance.
