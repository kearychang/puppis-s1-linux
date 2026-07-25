# Puppis S1 on Ubuntu 26.04: desktop-app feasibility and next steps

Research date: 2026-07-25
Target: original PrismXR Puppis S1 (P1411), Ubuntu 26.04 LTS, Meta Quest 3
Scope: a Linux desktop manager for the router and host-side internet sharing; not a VR streaming runtime.

## Executive conclusion

A useful Ubuntu desktop app is feasible now for the **host-side half** of the problem. The connected Puppis S1 already appears as a normal USB Ethernet interface using Linux's in-box `cdc_ncm` driver, and NetworkManager's standard `ipv4.method=shared` mode is already providing the expected `192.168.137.1/24` gateway, forwarding, DHCP/DNS, and NAT behavior. No Puppis-specific kernel driver is indicated.

The remaining reverse-engineering task is bounded. PrismXR does not publish a device API or Linux application, but inspection of the official Windows desktop application installed locally shows that its P1411 control path is a proprietary **TCP protocol at `192.168.137.254:10081`**, not a conventional web UI. Decompilation recovered the complete outer frame and CRC32 calculation, and a successful Wine session validated both against real `getFirstlogin` and `getDevice` traffic. The `set5GHotspot` schema was subsequently recovered; temporary SSID, channel, and password changes were each accepted, read back, restored, and read back again. The matching `set2GHotspot` schema was also recovered: SSID and channel `6` passed the same standard, while password restoration failed and remains unqualified after a successful authorized reconciliation. Remaining protocol unknowns include authentication or negotiation on other configurations, untested setters and value combinations, error behavior, and compatibility across firmware versions. Bluetooth/EaseLink appears relevant to mobile onboarding but is no longer the leading transport for this desktop app.

A follow-up port change resolved the initial USB performance issue. The unit first negotiated USB High-Speed at `480` Mb/s, but now negotiates **SuperSpeed at `5000` Mb/s** on `/sys/bus/usb/devices/2-5` with USB version `3.20`. The original port or connection path was therefore the bottleneck candidate, not the Linux driver or Puppis USB function. The app should still treat negotiated USB speed as a top-level diagnostic because the failure mode is real and actionable.

The recommended delivery strategy is:

1. Build a read-only CLI probe and a host-sharing controller first.
2. Turn the recovered and capture-validated TCP framing into a read-only protocol client.
3. Add device controls only through a separately tested protocol adapter.
4. Put a GTK 4/libadwaita UI over those proven modules.

## What the primary sources establish

### Product and supported software

- PrismXR describes the S1 as a Wi-Fi 6/AX3000 router with 2x2 MIMO, 160 MHz bandwidth, 1024-QAM, advertised rates of 2,402 Mb/s at 5 GHz and 575 Mb/s at 2.4 GHz, and a 1.3 GHz dual-core chipset. It describes EaseLink as a Bluetooth connection between the PrismXR app and the S1. It lists macOS and Windows 10/11 as compatible operating systems; Linux is not listed. [Official Puppis S1 product page](https://www.prismxr.com/products/prismxr-puppis-s1-ax3000-wifi6-router-for-pc-vr-streaming-quest-3-compatible)
- PrismXR's product collection calls the S1 an 802.11ax router and says practical streaming bitrate with Quest 3/3S is up to roughly 400–600 Mb/s. This is a vendor performance claim, not an independently measured guarantee. [Official Puppis collection](https://www.prismxr.com/collections/puppis)
- The S1 supports three distinct roles: dedicated PrismPulse PC-VR networking, a general Wi-Fi hotspot, and a Wi-Fi adapter for the PC. These are router modes, not Linux Wi-Fi interface modes. [Official Puppis S1 product page](https://www.prismxr.com/products/prismxr-puppis-s1-ax3000-wifi6-router-for-pc-vr-streaming-quest-3-compatible)
- PrismXR currently publishes mobile applications for iOS and Android and a desktop download explicitly labelled Windows. It does not publish a Linux desktop package on its downloads page. The locally installed official desktop package's bundled `PrismXR_ReadMeFirst` narrows support to Windows 10/11 x86/x64 and P1411 firmware 1.21 or newer. [Official PrismXR downloads](https://www.prismxr.com/pages/downloads)
- PrismXR says its Windows desktop app provides one-click setup, network sharing, a diagnostic covering up to 21 network conditions, Windows-setting optimization, and real-time monitoring. This describes a useful parity target but not an API. [Official desktop-app announcement](https://www.prismxr.com/blogs/news/how-the-puppis-s1-desktop-app-enhances-the-vr-experience)

### Internet-sharing behavior

- PrismXR documents two upstream choices for the S1: its own 2.4 GHz Wi-Fi connection or PC Internet Sharing, with PC sharing taking precedence. Its Windows recovery configuration is `192.168.137.1` with mask `255.255.255.0`. [Official S1 FAQ](https://www.prismxr.com/pages/puppis-s1-faqs)
- NetworkManager defines IPv4 `shared` mode on the **downstream** interface as the mode that shares internet access to a subnet. A manual address selects the subnet; otherwise NetworkManager selects a `10.42.x.0/24` subnet. Shared mode always enables forwarding and provides an automatically selected DHCP range unless one is supplied. [NetworkManager IPv4 settings reference](https://www.networkmanager.dev/docs/api/latest/settings-ipv4.html)
- Consequently, PrismXR's “PC Internet Sharing” should be modeled internally as a routed/NAT downstream network, not as an Ethernet layer-2 bridge. “Internet sharing” is accurate user-facing language; “bridge” is technically misleading here.

### APIs suitable for a Linux app

- NetworkManager exposes connection profiles and active connections over its system D-Bus API, including state-change signals and IP configuration. It also exposes permissions for network control and system-profile modification, with results indicating allowed, denied, or authentication required. This is the correct integration boundary; shelling out to `nmcli` is appropriate for a prototype but not the final module interface. [NetworkManager D-Bus API](https://www.networkmanager.dev/docs/api/latest/spec.html), [active connection API](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.Connection.Active.html), [permission types](https://www.networkmanager.dev/docs/api/latest/nm-dbus-types.html)
- BlueZ exposes discovery, connection, and remote GATT services/characteristics over D-Bus. `Adapter1.StartDiscovery()` creates device objects; `Device1` represents a peer; the GATT API exposes characteristic reads, writes, and notifications. These are sufficient platform primitives if EaseLink is a conventional BLE GATT protocol. [BlueZ adapter API](https://bluez.readthedocs.io/en/latest/adapter-api/), [device API](https://bluez.readthedocs.io/en/latest/device-api/), [GATT API](https://bluez.readthedocs.io/en/latest/gatt-api/)
- Network changes may require authorization. NetworkManager already integrates system policy and exposes whether authentication is needed; polkit provides an interactive authorization path. Prefer that path to running the whole GUI as root. [NetworkManager permission types](https://www.networkmanager.dev/docs/api/latest/nm-dbus-types.html), [polkit authorization API](https://polkit.pages.freedesktop.org/polkit/eggdbus-interface-org.freedesktop.PolicyKit1.Authority.html)
- For an Ubuntu-first application, GTK 4/libadwaita is a small native UI layer with first-party GNOME documentation. It is not essential to the protocol work, so the backend should remain UI-independent. [libadwaita documentation](https://gnome.pages.gitlab.gnome.org/libadwaita/)

## Read-only observations from the connected machine

These observations were collected locally on 2026-07-25. They are evidence for this physical unit and connection, not universal Puppis specifications.

| Observation | Result | Meaning |
|---|---|---|
| Host | Ubuntu 26.04 LTS; Linux 7.0.0-28; NetworkManager 1.54.3; BlueZ 5.85 | Required platform services are present. |
| USB identity | `0b95:1790`, manufacturer `ASIX`, product `AX88179A`, serial `[REDACTED]` | The PC-facing function is a commodity USB Ethernet controller. The ID alone must not identify a Puppis because other AX88179A products use it. |
| Active USB interfaces | CDC NCM control class `02:0d:00` and data class `0a:00:01`; driver `cdc_ncm` | The kernel is using a standards-based network function. |
| Negotiated USB speed | Initially `480` Mb/s at `1-4`; after changing ports, `5000` Mb/s at `2-5` with USB version `3.20` | SuperSpeed is now confirmed. Preserve a below-SuperSpeed warning in the app because the original port/path produced a real 480 Mb/s limitation. |
| Linux network device | `enx020000000101`, up, carrier present, MTU 1500 | Stable detection can combine udev properties, interface path, and behavioral checks. |
| Host upstream | `enp5s0`, `192.168.2.147/24`, default route via `192.168.2.1` | Wired Ethernet is currently the internet-facing interface. |
| Puppis-side host address | `192.168.137.1/24`; no default route on the Puppis interface | Matches PrismXR's documented PC-sharing subnet and does not divert host internet traffic. |
| NetworkManager profile | `ipv4.method=shared`, autoconnect enabled; global IPv4 forwarding observed enabled | Host-side sharing is already configured in the supported NetworkManager way. |
| DHCP/neighbors | Puppis hostname `P1411`, MAC `02:00:00:00:01:01`, appeared at `.77` and `.254`; a phone client was `.166` | `.77` and `.254` are addresses owned/proxied by the S1. The client list can be correlated with DHCP data when permissions allow. |
| Local services | TCP/53 was open on `.77` and `.254`; ports 22, 80, 443, 554, 1900, 5000, 8000, 8080, 8443, and 8888 were closed | The S1 provides or proxies DNS, but no conventional web-management service was found on the bounded scan. This does not rule out UDP or an untested proprietary port. |
| Traffic counters | About 131 MB transmitted and 5.5 MB received, with zero TX/RX errors at inspection time | The USB network path was active and clean during this short observation. |
| Bluetooth scan | No device could be confidently identified as Puppis during a 12-second scan while the SET/pairing mode was not deliberately activated | This is inconclusive. PrismXR says pairing mode requires pressing SET and lasts three minutes. |

### Direct inspection of the official Windows application

The official PrismXR Desktop v1.2.5 installation under `~/.wine` was inspected as a first-party binary artifact. This was static inspection plus a normal Wine run; no binary was modified.

- It is a .NET Framework 4.7.2 WPF application. The package includes `PrismXR.exe`, a roughly 2.9 MB PDB, `YncTcpSorket.dll`, logs, and a release-note date of 2025-11-18.
- The bundled readme identifies the target as Puppis S1 P1411, requires Windows 10/11 x86/x64 and P1411 firmware 1.21+, and lists quick setup for region/password/network plus diagnostics.
- UTF-16 strings in `YncTcpSorket.dll` identify `192.168.137.254`, `InitTCP`, `remotePort`, read/send logic, checksum/framing logic, and a JSON-like command wrapper of the form `{ fun = ..., args = ... }`. Hex string fragments correspond to JSON beginning with `{"fun":"` and continuing with `","args":{}}`.
- The command vocabulary includes `getDevice`, `getNetRf`, `getNetIPInfo`, `getConnAp`, `getCountryCode`, `get5GHotspot`, `get2GHotspot`, and `getStreamingInfo`, with corresponding setters plus mode, upgrade, DHCP, AP/station list, blacklist, and factory-reset operations.
- Firmware/config strings reference `ftp://192.168.137.254:21/router.cfg`. That is evidence of an FTP-related workflow in the client, not proof that anonymous FTP is currently available or safe to use.
- A Wine run received valid replies for the device and network getters, including a `1411 Model` response. Windows-specific WMI, service, ICS, and interface-management steps failed, which cleanly separates the working router protocol from the host-integration problem.

These findings supersede the earlier transport uncertainty: the desktop protocol exists and works through Wine. The endpoint, complete outer frame, length, checksum, 5 GHz setter schema, and reversible 5 GHz SSID/password/channel mutations are now proven. The 2.4 GHz setter schema plus reversible SSID and channel `6` mutations are also proven; 2.4 GHz password mutation is not qualified because its original value did not restore. Authentication on other configurations, error behavior, other setter semantics, and the firmware procedure remain unknown.

### Recovered and capture-validated TCP framing

Decompilation and a successful read-only Wine session on 2026-07-25 resolved the transport details needed for a decoder:

- `PrismXR.exe` calls `InitTCP(InitIPAddress, 10081)`, and the DLL defaults `InitIPAddress` to `192.168.137.254`. The captured connection was `192.168.137.1:<ephemeral>` to `192.168.137.254:10081`.
- A frame is `AA | LEN | CRC32 | TYPE | 00 | JSON`: one-byte magic `0xAA`, one-byte total length, four-byte big-endian CRC32, one-byte message type, one reserved zero byte, and compact JSON.
- CRC32 is the standard reflected algorithm with polynomial `0xEDB88320`, initial value `0xffffffff`, and final complement. It covers the bytes beginning at `TYPE`, equivalent to Python `binascii.crc32(frame[6:])`.
- The captured client requests used type `0x01`; the device response used type `0x02`. The request payloads were `{"fun":"getFirstlogin","args":{}}` and `{"fun":"getDevice","args":{}}`.
- Both request frames and the response passed independent total-length and CRC checks. The decoded response identified model/alias `P1411`, product `PrismXR Puppis S1`, firmware `B-MD2FP1411V1.22-250108-r0e17`, and status `ok`; its serial was omitted from saved fixtures.
- The official receiver reads an eight-byte header followed by `LEN - 8` payload bytes and parses the JSON as UTF-8. It appears not to reject an invalid response CRC, so the Linux implementation should validate magic, length, reserved byte, type, CRC, and JSON itself.

The throwaway protocol prototype, safe fixtures, and exact evidence notes are in [`prototype/p1411_protocol/README.md`](../../prototype/p1411_protocol/README.md). `set5GHotspot`, `set2GHotspot`, and modes 1/2 were transmitted under guarded workflows. The redacted fixtures distinguish complete 5 GHz success from the partial 2.4 GHz result and authorized reconciliation. A raw full-system trace was deliberately excluded from the repository because it contains unrelated Wine activity and private identifiers; raw write frames were also omitted because they contain Wi-Fi passwords.

ASIX itself describes the AX88179A as a USB 3.2 Gen 1 to Gigabit Ethernet controller supporting Linux's native CDC-NCM driver. This corroborates the local enumeration and explains why no vendor driver is necessary. [ASIX AX88179A product documentation](https://www.asix.com.tw/en/product/USBEthernet/Super-Speed_USB_Ethernet/AX88179A)

## Known, inferred, and unknown capability matrix

| Capability | Status | Evidence / implementation consequence |
|---|---|---|
| Detect USB network link | Proven locally | udev + NetworkManager; do not key only on generic ASIX VID/PID. |
| Configure host internet sharing | Proven locally and documented | NetworkManager D-Bus profile with `ipv4.method=shared` and `192.168.137.1/24`. |
| Preserve the host's upstream default route | Proven locally | S1 profile has no gateway; app should verify rather than rewrite upstream metrics indiscriminately. |
| Detect client presence | Partly proven | Neighbor entries show activity, but DHCP leases were not readable as an unprivileged user and MAC identity is not enough to label a Quest. |
| Identify Quest 3 reliably | Likely available, not yet decoded | The desktop client contains station-list, blacklist, and streaming-info commands. Validate returned identity and require user confirmation where MAC randomization makes it ambiguous. |
| Read HMD RSSI/link rate/channel | Likely available, not yet decoded | The desktop client exposes RF and streaming-info getters over the proprietary TCP path. |
| Change PrismPulse 5 GHz SSID/password/channel | Proven locally and restored | `set5GHotspot` accepted each isolated mutation in the full eight-field object, returned `ok`, exposed it through its getter, and accepted restoration. Production code must use read-before/write/read-after and rollback. |
| Change 2.4 GHz SSID/channel | Proven locally and restored | `set2GHotspot` accepted isolated SSID and channel `6` mutations in the full eight-field object; complete requested and original objects were verified after each step. |
| Change 2.4 GHz password | Applied but not safely reversible | The temporary password applied and verified, but the original password did not restore. A separately authorized recovery set it to the live 5 GHz password and verified the full object. Keep production password mutation disabled pending root-cause evidence. |
| Change region | Command found; mutation unproven | Region remains safety/regulatory-sensitive and should stay disabled until country/channel validation and reconnect behavior are separately proven. |
| Switch PrismPulse/hotspot modes | Proven locally and restored | `setMode` with values `1` and `2` returned `ok`; both transitions were verified through `getDevice`. Treat switching as an atomic operation with rollback. |
| Switch to Wi-Fi adapter mode | Schema recovered; live transition unproven | Value `3` is confirmed, but the official Windows workflow also changes the host Puppis-interface address to `192.168.137.253`. Implement the Linux network transition before enabling it. |
| Factory reset | Command and guard implemented; destructive live test pending | `setFactory` has no arguments and is sent by PrismXR only after confirmation. The CLI requires an explicit erase token. PrismXR documents physical recovery but not post-reset default credentials, so no packet was sent without a separate destructive-test confirmation. [Official reset procedure](https://www.prismxr.com/pages/how-to-reset-puppis-s1-to-factory-defaults) |
| Firmware update | Vendor app can; protocol and image validation unknown | Exclude from MVP to avoid bricking the router. PrismXR's FAQ also contains inconsistent “latest” firmware references (V1.16 and V1.19), so versions must be queried dynamically. |
| Official Meta Quest Link streaming on Ubuntu | Outside this app's scope | This project manages the network. The user still needs a Linux-capable streaming stack such as the one they already choose; it should not be bundled or impersonated by the manager. |

## Recommended architecture

Keep four explicit seams so protocol uncertainty cannot contaminate the UI:

1. **Host network service** — talks directly to NetworkManager over D-Bus. It discovers the S1-facing interface, creates/repairs a named connection profile, activates it, listens for state changes, and reports authorization requirements. Use a NetworkManager checkpoint before changing an existing profile so a failed activation can be rolled back.
2. **Device transport** — a TCP adapter for `192.168.137.254` behind a narrow interface: connect, read capability/version, invoke typed getter/setter, and subscribe/poll telemetry. Keep framing/checksum parsing isolated and fixture-tested. Bluetooth should remain a separate optional onboarding transport, not a dependency of host sharing.
3. **Domain/service layer** — exposes typed state such as `Disconnected`, `UsbLimited`, `SharingReady`, `RouterReady`, `QuestConnected`, and `Degraded`, plus explicit operations. Do not leak D-Bus object paths, GATT handles, or raw JSON/bytes into the UI.
4. **GTK/libadwaita UI** — status/dashboard, guided setup, diagnostics, and settings. It consumes state and invokes operations but owns no networking or BLE logic.

For the production implementation, Rust is a strong fit (`zbus`/NetworkManager bindings, an async TCP codec, GTK 4 bindings) because this is a long-running hardware manager with asynchronous state and binary protocol parsing. For protocol reconnaissance, a small Python CLI is faster to iterate. Do not select Electron merely to avoid the D-Bus work; it would still need a privileged/native backend.

Package as a native Debian package first. A Flatpak can follow after the system-bus and Bluetooth permissions are intentionally designed and tested. Do not add a custom root daemon unless NetworkManager/polkit demonstrably cannot perform a required operation.

## Evidence-driven next steps

### Phase 0 — baseline the physical link (SuperSpeed resolved)

1. **Completed 2026-07-25:** after changing USB ports, `/sys/bus/usb/devices/2-5/speed` reports `5000` and the device reports USB version `3.20`.
2. Record a repeatable baseline with the Quest connected to PrismPulse: interface identity, USB speed, IPs/routes, NetworkManager profile, ping latency/loss to the Quest candidate, and an `iperf3` result if an endpoint is available on the headset.
3. Verify internet access from the Quest while the host profile remains `shared`; verify the PC's default route stays on `enp5s0`.

Exit criterion: SuperSpeed is confirmed, the Quest can reach the host and internet, and the measurements can be repeated without the future GUI.

### Phase 1 — build a read-only probe

Create a CLI that outputs structured JSON and human-readable diagnostics:

- Enumerate USB network devices through udev and correlate them to NetworkManager devices.
- Score a candidate as Puppis using multiple signals: ASIX identity, active CDC-NCM interface, expected subnet/peer behavior, and later protocol identity. Never claim certainty from `0b95:1790` alone.
- Report USB negotiated speed with a warning below SuperSpeed.
- Report carrier, profile method, downstream address, default-route isolation, forwarding/connectivity, counters, and active neighbor candidates.
- Report Bluetooth candidates only while the user deliberately puts the S1 in pairing mode.

Exit criterion: unplug/replug and reboot tests identify the correct interface without hard-coded names or USB topology paths, and never modify networking.

### Phase 2 — prove host sharing as an idempotent operation

Add a command that creates or repairs an app-owned NetworkManager profile bound to the detected interface:

- IPv4 method `shared`.
- Address `192.168.137.1/24` for PrismXR compatibility.
- No downstream gateway/default route.
- Autoconnect enabled, with a deliberately low priority.
- Preserve unrelated user profiles; display and request authorization through NetworkManager/polkit.
- Observe activation via D-Bus signals and roll back on failure.

Test with Ethernet, host Wi-Fi, VPN present/absent, S1 unplug/replug, reboot, suspend/resume, and an occupied `192.168.137.0/24` subnet. The last case needs a product decision: warn and stop, or use another subnet knowing vendor behavior may assume `.137`.

Exit criterion: one command safely reaches the same desired state from clean, already-correct, partially configured, and failed states.

### Phase 3 — validate and extend the TCP management protocol

Pursue the shortest evidence path in this order:

1. **Completed 2026-07-25:** decompiled `YncTcpSorket.dll` and the relevant `PrismXR.exe` call sites, recovering the endpoint, connection lifecycle, byte framing, length, CRC32, encoding, and response dispatch.
2. **Completed 2026-07-25:** captured a successful read-only Wine session, correlated `getFirstlogin` and `getDevice` bytes with the decompiled logic, independently validated all captured lengths and CRCs, and saved privacy-redacted fixtures.
3. **Prototype completed 2026-07-25:** implemented a live read-only Python getter client using the recovered codec, response correlation, validation, and secret redaction.
4. **5 GHz settings and modes 1/2 completed 2026-07-25:** recovered the official setter call sites, changed SSID, password, channel, and mode separately through the reconstructed client, received `status=ok`, verified every temporary getter value, restored the original state after every test, and verified the final state.
5. **2.4 GHz partial proof completed 2026-07-25:** recovered the matching eight-field `set2GHotspot` schema. SSID and channel `6` each passed complete-object mutation and restoration. A temporary password applied and verified, but the original password did not restore; password mutation remains unqualified. A separately authorized recovery set 2.4 GHz to the live 5 GHz password in memory and verified the complete object, leaving mode 1, automatic channels, and no prototype credentials active.
5. **Factory-reset guard completed 2026-07-25:** recovered the no-argument `setFactory` command, validated its exact frame offline, and implemented an explicit `--execute --confirm ERASE-PUPPIS-CONFIG` gate plus pre-reset snapshot and reconnect monitoring. A live reset remains pending because it intentionally erases configuration and vendor documentation does not disclose the post-reset credentials.
6. Ask `customerservice@prismxr.io` for the protocol/SDK and redistribution terms in parallel. The implementation should prefer vendor-confirmed schemas if supplied.
7. Investigate BLE only if onboarding or recovery cannot be completed through TCP; use SET pairing mode and read-only GATT enumeration first.

Exit criterion: the already recovered endpoint, lifecycle, framing/checksum, and identity are joined by documented authentication behavior (if any), success/error responses, version compatibility, and at least one reversible operation, all regression-tested.

### Phase 4 — desktop MVP

Ship only proven behavior:

- Dashboard: S1 presence, USB speed, host upstream, internet-sharing state, probable Quest presence, counters, and actionable warnings.
- Guided setup/repair for NetworkManager sharing.
- Diagnostics bundle with explicit consent and redaction of SSIDs, credentials, serials, and MAC addresses.
- Firmware/version display, PrismPulse SSID/password and channel controls, HMD telemetry, and modes 1/2 switching behind capability detection.
- Factory reset only in an advanced/destructive section with typed confirmation, a configuration summary, and explicit notice that automatic restoration is not performed.

Do not include firmware updating in the first release. Add it only after image authenticity/integrity, power-loss behavior, downgrade protection, recovery, and firmware-version compatibility are understood.

## Suggested first three tickets

1. **Read-only hardware/network probe** — JSON model, udev/NetworkManager correlation, USB-speed warning, fixtures, and unplug/replug tests.
2. **NetworkManager sharing controller** — app-owned profile, idempotent reconciliation, checkpoint/rollback, polkit UX, and integration tests in disposable network namespaces where possible.
3. **P1411 TCP protocol client** — turn the working prototype into a fixture-tested client with getters, transactional 5 GHz SSID/password/channel writes, modes 1/2, and an explicitly confirmed factory-reset operation. Require read-before/write/read-after and rollback for reversible commands; keep mode 3 and every other setter disabled until separately proven.

The desktop UI should begin only after tickets 1 and 2 expose stable interfaces. Ticket 3 may proceed concurrently, but its uncertainty must not block a useful host-sharing and diagnostics application.

## Key risks and guardrails

- **False identification:** ASIX's VID/PID is generic. Use layered evidence and let the user confirm an ambiguous device.
- **Performance misdiagnosis:** Wi-Fi link rate is not throughput. This unit initially negotiated at 480 Mb/s and reached 5000 Mb/s after changing ports, so display USB negotiation and Wi-Fi link rate separately.
- **Network disruption:** Never rewrite the upstream connection or default route as part of ordinary setup. Use an app-owned downstream profile and rollback.
- **Firewall advice:** Do not reproduce advice to disable the firewall globally. Diagnose required traffic and add narrowly scoped rules only if testing proves they are needed.
- **Regulatory settings:** Validate country/region and allowed channels; never blindly replay captured values.
- **Protocol/firmware drift:** Capability-negotiate and maintain captures/fixtures by app and firmware version. Start from the readme's P1411 firmware 1.21 floor, but query rather than assume the live version.
- **Credentials/privacy:** Keep Wi-Fi keys in the desktop secret service, never logs; redact MACs/serials in exported diagnostics by default.
- **Firmware safety:** Treat firmware as out of scope until signing, recovery, and vendor authorization are known.

## Bottom line

The host networking foundation is already working on Ubuntu 26.04, the USB connection now negotiates at SuperSpeed, and the official desktop application's router-control path already works under Wine. Its endpoint, framing, checksum, 5 GHz SSID/password/channel writes, and 2.4 GHz SSID/channel `6` writes are recovered and live-validated. The 2.4 GHz password write applies but is not safely reversible and must remain disabled pending investigation. Mode 3, region, reset, and firmware remain guarded.
