# Ubuntu 26.04 amd64 release matrix

This matrix is advisory evidence for the user's release decision. It is never run by normal tests or packaging because several rows require a real supported environment. Record the package version, Ubuntu image/build, P1411 firmware, tester, date, and outcome for every exercised row. Do not perform a device mutation unless the operation is qualified and separately confirmed in the application.

| Area | Manual scenario | Evidence to record |
| --- | --- | --- |
| Package | Clean install, upgrade, app exit, and removal | `.deb` metadata; desktop launch; confirmation that live networking and connected clients remain untouched |
| Package | Inspect installed payload | No daemon, service, autostart, root helper, custom policy, network profile, firewall state, or vendor artifact |
| Environment | Ubuntu 26.04 `amd64` clean setup and existing setup | OS/architecture and observed first-launch checklist |
| USB | No device, one candidate, multiple candidates, unplug/replug, USB 2, and SuperSpeed | Independent USB status and explicit candidate selection |
| Network | No upstream, Ethernet, Wi-Fi, VPN, external sharing profile, managed profile, and Puppis subnet conflict | Existing route/profile preservation and authorization explanation |
| Lifecycle | Reboot, suspend/resume, device replug, and ordinary application exit | NetworkManager-owned state remains authoritative; no background app process |
| Protocol | Unknown firmware and qualified firmware | Read-only gating and exact firmware/capability display |
| Wi-Fi | Each qualified reversible SSID/channel/password operation | Preflight snapshot, explicit confirmation, verified result, and redacted output |
| Safety gate | 2.4 GHz password control | Control remains unavailable with the qualification explanation |
| Accessibility | Keyboard-only flow, fixed shortcuts, screen reader, 100/125/150% text | Focus visibility, labels, announcements, reflow, and non-color status cues |
| Diagnostics | Preview, export, inspect, and clear local logs | No password, SSID, serial, MAC, raw frame, or form value leakage |

Factory reset is outside this matrix until ticket 14 independently qualifies it through fresh authorization and a complete recovery rehearsal.
