#!/usr/bin/env python3
"""Transactional P1411 5 GHz SSID write proof with automatic restoration.

This is deliberately limited to set5GHotspot. It preserves every current field,
changes only the SSID, verifies the change, restores the original object in a
finally block, and verifies restoration. Passwords are never printed.
"""

from __future__ import annotations

import argparse
import json
import socket
import time

from research.p1411_protocol.hardware_lab.live_read import get, recv_frame
from research.p1411_protocol.protocol_logic import ENDPOINT, encode_frame


COMMAND = "set5GHotspot"
TEMPORARY_SSID = "prismpulse-codex-test"
POLL_SECONDS = 30
CONFIRMATION = "TEST-5G-SSID-AND-RESTORE"


def send_set5g(args: dict, *, timeout: float = 4.0) -> dict | None:
    host, port = ENDPOINT
    frame = encode_frame({"fun": COMMAND, "args": args})
    with socket.create_connection((host, port), timeout=timeout) as sock:
        sock.settimeout(timeout)
        sock.sendall(frame)
        try:
            return decode_response(recv_frame(sock))
        except (TimeoutError, socket.timeout, ConnectionError):
            # Radio reconfiguration may close the official client's connection.
            return None


def decode_response(frame: bytes) -> dict:
    from research.p1411_protocol.protocol_logic import decode_frame

    message = decode_frame(frame)["message"]
    if message.get("fun") != COMMAND:
        raise ValueError(f"unexpected setter response function: {message.get('fun')!r}")
    return message


def wait_for_ssid(expected: str) -> dict:
    deadline = time.monotonic() + POLL_SECONDS
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            message = get("get5GHotspot", timeout=3.0)
            if message.get("status") == "ok" and message.get("data", {}).get("ssid") == expected:
                return message
        except (OSError, ValueError, ConnectionError) as error:
            last_error = error
        time.sleep(1)
    suffix = f"; last error: {type(last_error).__name__}" if last_error else ""
    raise TimeoutError(f"SSID did not become {expected!r} within {POLL_SECONDS}s{suffix}")


def safe_response(response: dict | None) -> dict:
    if response is None:
        return {"received": False, "note": "connection closed or response timed out"}
    return {
        "received": True,
        "fun": response.get("fun"),
        "status": response.get("status"),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--confirm")
    args = parser.parse_args()
    if not args.execute or args.confirm != CONFIRMATION:
        parser.error(
            f"live proof requires --execute --confirm {CONFIRMATION}; no write was sent"
        )

    original_message = get("get5GHotspot")
    if original_message.get("status") != "ok":
        raise RuntimeError("get5GHotspot did not return status=ok; refusing write")
    original = dict(original_message["data"])
    original_ssid = original.get("ssid")
    if not isinstance(original_ssid, str) or not original_ssid:
        raise RuntimeError("current SSID is missing; refusing write")

    temporary = dict(original)
    temporary["ssid"] = (
        TEMPORARY_SSID if original_ssid != TEMPORARY_SSID else "prismpulse-codex-test-2"
    )

    result = {
        "command": COMMAND,
        "changed_field": "ssid",
        "original_ssid": "[REDACTED]",
        "temporary_ssid": temporary["ssid"],
        "preserved_fields": sorted(key for key in original if key not in {"ssid", "pwd"}),
        "password_preserved_in_memory": "pwd" in original,
    }

    temporary_verified = False
    try:
        result["temporary_set_response"] = safe_response(send_set5g(temporary))
        wait_for_ssid(temporary["ssid"])
        temporary_verified = True
        result["temporary_readback_verified"] = True
    finally:
        result["restore_attempted"] = True
        result["restore_response"] = safe_response(send_set5g(original))
        wait_for_ssid(original_ssid)
        result["restoration_readback_verified"] = True

    result["transaction_succeeded"] = temporary_verified
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
