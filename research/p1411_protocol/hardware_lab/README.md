# Live hardware laboratory

> [!CAUTION]
> These scripts are unsupported research tools. They can change radio settings, disconnect clients, or erase device configuration. They are not installed with the application and are never run by hosted CI.

`live_read` accepts getters only and redacts identifying fields. Every mutation script requires `--execute` plus its exact operation-specific confirmation phrase; most attempt transactional restoration, but restoration is not guaranteed after a disconnect, process failure, power loss, firmware difference, or unqualified starting state.

Before any live operation:

1. Read the script and the [research verdict](../README.md).
2. Verify the exact P1411 model and qualified firmware.
3. Ensure the device begins in a coherent, recorded state.
4. Understand the operation's restoration and recovery limitations.
5. Never run a mutation merely to reproduce historical evidence.

Invoke tools as modules from the repository root, for example:

```bash
python3 -m research.p1411_protocol.hardware_lab.live_read getDevice
```

Do not attach output to public issues without manually confirming that it contains no credentials or identifying device/network data.
