#!/usr/bin/env python3
"""Transactional P1411 PrismPulse/Wi-Fi-hotspot mode switch and restoration."""

from __future__ import annotations

import argparse
import json
import socket
import time

from research.p1411_protocol.hardware_lab.live_read import get, recv_frame
from research.p1411_protocol.protocol_logic import ENDPOINT, decode_frame, encode_frame


MODE_NAMES = {"1": "PrismPulse", "2": "Wi-Fi hotspot", "3": "Wi-Fi adapter"}
POLL_SECONDS = 45
CONFIRMATION = "TEST-MODE-AND-RESTORE"


def send_mode(mode: str, *, timeout: float = 5.0) -> dict | None:
    if mode not in MODE_NAMES:
        raise ValueError(f"unsupported mode: {mode}")
    frame = encode_frame({"fun": "setMode", "args": {"mode": mode}})
    with socket.create_connection(ENDPOINT, timeout=timeout) as sock:
        sock.settimeout(timeout)
        sock.sendall(frame)
        try:
            message = decode_frame(recv_frame(sock))["message"]
        except (TimeoutError, socket.timeout, ConnectionError):
            return None
    if message.get("fun") != "setMode":
        raise ValueError(f"unexpected setter response: {message.get('fun')!r}")
    return message


def current_mode() -> str:
    message = get("getDevice", timeout=4.0)
    if message.get("status") != "ok":
        raise RuntimeError("getDevice did not return status=ok")
    mode = message.get("data", {}).get("mode")
    if mode not in MODE_NAMES:
        raise RuntimeError(f"unknown current mode: {mode!r}")
    return mode


def wait_for_mode(expected: str) -> None:
    deadline = time.monotonic() + POLL_SECONDS
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            if current_mode() == expected:
                return
        except (OSError, ValueError, ConnectionError) as error:
            last_error = error
        time.sleep(1)
    suffix = f"; last error: {type(last_error).__name__}" if last_error else ""
    raise TimeoutError(f"mode did not become {expected} within {POLL_SECONDS}s{suffix}")


def safe_response(response: dict | None) -> dict:
    if response is None:
        return {"received": False, "note": "connection closed during mode transition"}
    return {"received": True, "fun": response.get("fun"), "status": response.get("status")}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--confirm")
    args = parser.parse_args()
    if not args.execute or args.confirm != CONFIRMATION:
        parser.error(
            f"live proof requires --execute --confirm {CONFIRMATION}; no write was sent"
        )

    original = current_mode()
    if original == "3":
        raise RuntimeError(
            "refusing automatic test from Wi-Fi adapter mode because its host addressing differs"
        )
    temporary = "2" if original == "1" else "1"
    result = {
        "command": "setMode",
        "original": {"value": original, "name": MODE_NAMES[original]},
        "temporary": {"value": temporary, "name": MODE_NAMES[temporary]},
    }
    temporary_verified = False
    try:
        result["temporary_response"] = safe_response(send_mode(temporary))
        wait_for_mode(temporary)
        temporary_verified = True
        result["temporary_readback_verified"] = True
    finally:
        result["restore_attempted"] = True
        result["restore_response"] = safe_response(send_mode(original))
        wait_for_mode(original)
        result["restoration_readback_verified"] = True
    result["transaction_succeeded"] = temporary_verified
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
