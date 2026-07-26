# P1411 TCP protocol research

**Research evidence—not production code or a supported user interface.** The offline inspector cannot access hardware. All scripts that connect to a device are quarantined in [`hardware_lab/`](hardware_lab/README.md), excluded from application packages and hosted CI, and every mutation requires an exact confirmation phrase.

## Question

What is the read-only P1411 desktop protocol used by PrismXR: endpoint, connection lifecycle, frame layout, checksum, request encoding, and response correlation? The answer was derived from the official `YncTcpSorket.dll`/`PrismXR.exe` managed IL and checked against one successful PrismXR 1.2.5 Wine session.

## Run

```bash
python3 -m research.p1411_protocol.offline.tui
```

The terminal inspector decodes the two exact captured request frames and a privacy-redacted response fixture. It can also encode a getter locally or validate pasted frame hex. It performs no network I/O.

The read-only live getter can be run with:

```bash
python3 -m research.p1411_protocol.hardware_lab.live_read getDevice get5GHotspot
```

It rejects non-getter command names and redacts passwords and serials.

The confirmation-gated 2.4 GHz SSID/channel/password proof can be run with one command:

```bash
python3 -m research.p1411_protocol.hardware_lab.safe_2g_roundtrip \
  --execute --confirm TEST-2G-AND-RESTORE
```

It is a throwaway live prototype for qualified P1411 firmware 1.22. It changes one
field at a time, compares the complete getter object, restores the original object
in a `finally` block, and never prints the original SSID or either password.

If the password proof leaves the prototype password active, the separately
authorized recovery command replaces it with the current 5 GHz password entirely
in memory:

```bash
python3 -m research.p1411_protocol.hardware_lab.recover_2g_password_from_5g \
  --execute --confirm RECOVER-2G-FROM-5G
```

## Verdict

- Endpoint: TCP `192.168.137.254:10081` from source address `192.168.137.1`.
- Connection: one long-lived IPv4 TCP stream. PrismXR starts a receive thread and periodically sends getters.
- Frame: `AA | LEN | CRC32 | TYPE | 00 | compact JSON`.
- `LEN` is the total frame length observed as one byte.
- `CRC32` is the standard reflected CRC-32 using polynomial `0xEDB88320`, initial/final XOR as implemented by `binascii.crc32`. It covers `TYPE | 00 | JSON` and is serialized as the four-byte big-endian representation of the numeric checksum.
- Observed type `0x01` means request and `0x02` means response. The DLL can generate `0x03`, but its meaning was not established by this capture.
- Getter JSON is `{"fun":"<name>","args":{}}` without whitespace.
- Response JSON contains the same `fun`, a command-specific `data` object, and `"status":"ok"` on success.
- The official receive loop reads eight header bytes, then `LEN - 8` JSON bytes. It does not visibly validate the response CRC in `ReadMsg`; a Linux client should validate it anyway.
- The recovered 5 GHz write schema is `ssid`, `pwd`, `pt`, `ch`, `code`, `en`, `encrypt`, and `bw`.
- Transactional `set5GHotspot` tests independently changed SSID, channel, and password. Every temporary write received `status=ok` and passed getter read-back; every original value was then restored and independently read back.
- A live `set2GHotspot` proof independently changed SSID, channel, and password. SSID and channel `6` met the complete-object change-and-restore standard. The temporary password applied and verified, but the original password did not restore within 30 seconds, so password mutation is not qualified. Further tests stopped; a separately authorized recovery replaced the prototype password with the live 5 GHz password entirely in memory and verified the complete 2.4 GHz object. The device is reconciled in mode 1 with automatic channels and no prototype credentials active.
- `setMode` uses `{ "mode": "1|2|3" }`, mapping to PrismPulse, Wi-Fi hotspot, and Wi-Fi adapter. A `1 → 2 → 1` round trip returned `ok` and passed `getDevice` verification at both transitions.
- `setFactory` takes no arguments. Its exact safe frame is recovered, and the official client accepts `status=ok`; the live destructive command has not been transmitted.
- The official P1411 UI restricts SSIDs to 1–18 ASCII alphanumeric characters (`^[a-zA-Z0-9]{1,18}$`) and passwords to 8–63 characters matching `^[a-zA-Z0-9?!@&$%*_~^#\-/.+:;=]{8,63}$`. The router accepted the longer hyphenated temporary SSID used by the direct protocol proof, but production should retain the official P1411 limits unless broader compatibility is deliberately tested.

Captured examples:

```text
getFirstlogin
aa 29 79 ed c0 93 01 00 7b 22 66 75 6e 22 3a 22 ...

getDevice
aa 25 bd 52 00 5d 01 00 7b 22 66 75 6e 22 3a 22 ...

getDevice response header
aa cb cd b9 04 55 02 00
```

All three captured frames passed total-length and CRC validation against the reconstructed algorithm. The response reported model P1411, firmware `B-MD2FP1411V1.22-250108-r0e17`, and status `ok`; its serial number is deliberately omitted.

## Static call-site evidence

- `YncTcpSorket.YncTcpSorket::.cctor` sets `InitIPAddress` to `192.168.137.254`.
- `SessionViewModel::LoginMethod` calls `InitTCP(InitIPAddress, 10081)`.
- `ResetConnect` also calls `InitTCP(InitIPAddress, 10081)`.
- `CRC::.cctor` builds a table using signed constant `-306674912`, or unsigned `0xEDB88320`.
- `ComputeChecksum` starts at `0xffffffff` and applies a final bitwise complement.
- `GetCommonData`/`GetParamsCommon` assemble the magic, length, CRC, type, reserved byte, and compact JSON.
- `SendMsg` hex-decodes that frame and sends it through the connected socket.
- `ReadMsg` reads eight bytes, takes byte 1 as total length, reads `length - 8`, decodes the remainder as UTF-8 JSON, and raises the response event.
- `ReadSend::MessageStr` parses that JSON, checks `status`, correlates on `fun`, and dispatches a typed event.

## Write-path evidence

- [`captures/validated-write-roundtrip.json`](captures/validated-write-roundtrip.json) records the redacted successful setter response, getter read-back, restoration response, and final verification.
- [`captures/partial-2g-roundtrip.json`](captures/partial-2g-roundtrip.json) records the redacted 2.4 GHz SSID/channel success, password-restoration failure, authorized recovery, and final reconciled state.
- [`hardware_lab/live_read.py`](hardware_lab/live_read.py) is a live read-only getter client with identifying-field redaction.
- [`hardware_lab/safe_ssid_roundtrip.py`](hardware_lab/safe_ssid_roundtrip.py) is the narrowly allowlisted write proof. It preserves the current password only in memory and restores the original full settings object in a `finally` block.
- [`hardware_lab/safe_setting_roundtrip.py`](hardware_lab/safe_setting_roundtrip.py) applies the same guarded proof to channel or password, with password values always redacted.
- [`hardware_lab/safe_mode_roundtrip.py`](hardware_lab/safe_mode_roundtrip.py) tests only modes 1 and 2 and restores the original mode. Mode 3 is excluded because the official Windows workflow also readdresses the host interface.
- [`hardware_lab/factory_reset.py`](hardware_lab/factory_reset.py) defaults to an offline dry run. Transmission requires both `--execute` and the exact confirmation token `ERASE-PUPPIS-CONFIG`.
- [`captures/validated-mode-roundtrip.json`](captures/validated-mode-roundtrip.json) records the successful reversible mode test.
- [`captures/factory-reset-dry-run.json`](captures/factory-reset-dry-run.json) records the recovered reset frame and why it has not yet been transmitted.
- Raw write frames and CRCs are omitted because they include the existing Wi-Fi password.

## Remaining unknowns

- Type `0x03` semantics.
- Authentication or pairing expectations on other firmware/configurations; the tested unit accepted local getters without an authentication exchange.
- Encoding behavior for non-ASCII request fields. The official sender uses `.NET Encoding.Default`; only ASCII getters were captured.
- Valid channel/radio/encryption combinations beyond the tested channel `36`, error frames, retransmission, and protocol compatibility outside P1411 firmware 1.22.
- Why a 2.4 GHz password change applied but the original password did not restore while the complete original object restored for SSID and channel tests.
- Mode 3's complete Linux host-addressing transition and rollback behavior.
- Post-factory-reset defaults and onboarding behavior; PrismXR documents the physical reset procedure but not default credentials.
- Whether any legitimate frame can exceed the observed one-byte length limit.

Only `set5GHotspot`, `set2GHotspot`, and `setMode` were sent. The 5 GHz transactions and mode `1 → 2 → 1` round trip restored their starting state. The 2.4 GHz proof fully restored SSID and channel, failed to restore the original password, and then completed the separately authorized reconciliatory password change documented above. No region, reset, blacklist, mode 3, or firmware command was sent.
