#!/usr/bin/env python3
"""Guarded P1411 factory-reset command; dry-run unless explicitly armed."""

from __future__ import annotations

import argparse
import json
import socket
import time

from live_read import get, recv_frame, redact
from protocol_logic import ENDPOINT, decode_frame, encode_frame


CONFIRMATION = "ERASE-PUPPIS-CONFIG"


def reset_frame() -> bytes:
    return encode_frame({"fun": "setFactory", "args": {}})


def snapshot() -> dict:
    return {
        function: redact(get(function, timeout=4.0))
        for function in ("getDevice", "get5GHotspot", "get2GHotspot", "getCountryCode")
    }


def send_reset(*, timeout: float = 5.0) -> dict | None:
    with socket.create_connection(ENDPOINT, timeout=timeout) as sock:
        sock.settimeout(timeout)
        sock.sendall(reset_frame())
        try:
            message = decode_frame(recv_frame(sock))["message"]
        except (TimeoutError, socket.timeout, ConnectionError):
            return None
    if message.get("fun") != "setFactory":
        raise ValueError(f"unexpected reset response: {message.get('fun')!r}")
    return message


def wait_for_reconnect(seconds: int = 120) -> dict:
    deadline = time.monotonic() + seconds
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            message = get("getDevice", timeout=4.0)
            if message.get("status") == "ok":
                return redact(message)
        except (OSError, ValueError, ConnectionError) as error:
            last_error = error
        time.sleep(2)
    suffix = f"; last error: {type(last_error).__name__}" if last_error else ""
    raise TimeoutError(f"Puppis did not reconnect within {seconds}s{suffix}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true", help="transmit the destructive command")
    parser.add_argument("--confirm", help=f"must exactly equal {CONFIRMATION!r}")
    args = parser.parse_args()

    frame = reset_frame()
    if not args.execute:
        print(json.dumps({
            "dry_run": True,
            "endpoint": f"{ENDPOINT[0]}:{ENDPOINT[1]}",
            "command": {"fun": "setFactory", "args": {}},
            "frame_hex": frame.hex(),
            "warning": "No packet was sent. Factory reset erases the current router configuration.",
        }, indent=2))
        return

    if args.confirm != CONFIRMATION:
        parser.error(f"destructive execution requires --confirm {CONFIRMATION}")

    before = snapshot()
    response = send_reset()
    after_device = wait_for_reconnect()
    print(json.dumps({
        "executed": True,
        "before": before,
        "response": None if response is None else {
            "fun": response.get("fun"),
            "status": response.get("status"),
        },
        "reconnected_device": after_device,
        "warning": "Configuration was reset and was not automatically restored.",
    }, indent=2))


if __name__ == "__main__":
    main()
