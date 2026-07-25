#!/usr/bin/env python3
"""PROTOTYPE: prove reversible P1411 2.4 GHz settings on qualified hardware.

Question: Does set2GHotspot support the same independently verified and restored
SSID, channel, and password transactions already demonstrated for 5 GHz?

The live path changes one field at a time, compares the complete getter object,
and restores the original object in a finally block. Secrets are never printed.
"""

from __future__ import annotations

import argparse
import json
import re
import socket
import time

from live_read import get, recv_frame
from protocol_logic import ENDPOINT, decode_frame, encode_frame
from safe_ssid_roundtrip import POLL_SECONDS, safe_response


GETTER = "get2GHotspot"
SETTER = "set2GHotspot"
CONFIRMATION = "TEST-2G-AND-RESTORE"
QUALIFIED_FIRMWARE = "B-MD2FP1411V1.22-250108-r0e17"
REQUIRED_FIELDS = {"ssid", "pwd", "pt", "ch", "code", "en", "encrypt", "bw"}
SSID_VALUES = ("Codex24Test", "Codex24Test2")
PASSWORD_VALUES = ("Codex24Test123!", "Codex24Test456!")
CHANNEL_VALUES = ("6", "11")


def send_settings(settings: dict[str, str], *, timeout: float = 4.0) -> dict | None:
    frame = encode_frame({"fun": SETTER, "args": settings})
    with socket.create_connection(ENDPOINT, timeout=timeout) as sock:
        sock.settimeout(timeout)
        sock.sendall(frame)
        try:
            message = decode_frame(recv_frame(sock))["message"]
        except (TimeoutError, socket.timeout, ConnectionError):
            return None
    if message.get("fun") != SETTER:
        raise ValueError(f"unexpected setter response function: {message.get('fun')!r}")
    if message.get("status") != "ok":
        raise RuntimeError("set2GHotspot did not return status=ok")
    return message


def read_settings() -> dict[str, str]:
    message = get(GETTER)
    if message.get("status") != "ok" or not isinstance(message.get("data"), dict):
        raise RuntimeError("get2GHotspot did not return a settings object")
    settings = dict(message["data"])
    if set(settings) != REQUIRED_FIELDS:
        raise RuntimeError(
            f"unexpected 2.4 GHz schema: {sorted(settings)}; refusing mutation"
        )
    if not all(isinstance(value, str) for value in settings.values()):
        raise RuntimeError("2.4 GHz settings contain non-string values; refusing mutation")
    return settings


def wait_for_settings(expected: dict[str, str]) -> None:
    deadline = time.monotonic() + POLL_SECONDS
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            if read_settings() == expected:
                return
        except (OSError, ValueError, ConnectionError) as error:
            last_error = error
        time.sleep(1)
    suffix = f"; last error: {type(last_error).__name__}" if last_error else ""
    raise TimeoutError(f"complete 2.4 GHz read-back did not match{suffix}")


def alternate(original: str, choices: tuple[str, ...]) -> str:
    return next(value for value in choices if value != original)


def display(field: str, value: str) -> str:
    return "[REDACTED]" if field in {"ssid", "pwd"} else value


def prove_field(
    original: dict[str, str], field: str, temporary_value: str
) -> dict[str, object]:
    temporary = dict(original)
    temporary[field] = temporary_value
    result: dict[str, object] = {
        "field": field,
        "original_value": display(field, original[field]),
        "temporary_value": display(field, temporary_value),
        "complete_object_compared": True,
        "all_other_fields_preserved": True,
    }
    changed_verified = False
    try:
        result["temporary_set_response"] = safe_response(send_settings(temporary))
        wait_for_settings(temporary)
        result["temporary_readback_verified"] = True
        changed_verified = True
    finally:
        result["restore_attempted"] = True
        result["restore_response"] = safe_response(send_settings(original))
        wait_for_settings(original)
        result["restoration_readback_verified"] = True
    result["transaction_succeeded"] = changed_verified
    return result


def preflight() -> tuple[dict[str, str], dict[str, object]]:
    device = get("getDevice")
    data = device.get("data", {})
    if device.get("status") != "ok" or data.get("model") != "P1411":
        raise RuntimeError("device did not verify as P1411; refusing mutation")
    if data.get("fw") != QUALIFIED_FIRMWARE:
        raise RuntimeError("firmware is not qualified for this prototype")
    if data.get("mode") not in {"1", "2"}:
        raise RuntimeError("mode is not qualified for this prototype")

    original = read_settings()
    if not re.fullmatch(r"[a-zA-Z0-9]{1,18}", SSID_VALUES[0]):
        raise RuntimeError("temporary SSID violates official P1411 constraints")
    if not re.fullmatch(r"[a-zA-Z0-9?!@&$%*_~^#\-/.+:;=]{8,63}", PASSWORD_VALUES[0]):
        raise RuntimeError("temporary password violates official P1411 constraints")
    if original["code"] != "CA":
        raise RuntimeError("live country is not the qualified CA test case")
    return original, {
        "model": "P1411",
        "firmware": QUALIFIED_FIRMWARE,
        "mode": data.get("mode"),
        "country": original["code"],
        "schema": sorted(original),
        "original_settings_held_in_memory": True,
        "secrets_logged": False,
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

    original, preflight_result = preflight()
    result: dict[str, object] = {
        "question": "Does 2.4 GHz match the validated 5 GHz transaction standard?",
        "command": SETTER,
        "preflight": preflight_result,
        "transactions": [],
    }
    temporary_values = {
        "ssid": alternate(original["ssid"], SSID_VALUES),
        "ch": alternate(original["ch"], CHANNEL_VALUES),
        "pwd": alternate(original["pwd"], PASSWORD_VALUES),
    }
    for field in ("ssid", "ch", "pwd"):
        result["transactions"].append(
            prove_field(original, field, temporary_values[field])
        )

    wait_for_settings(original)
    result["final_original_object_verified"] = True
    result["verdict"] = "2.4 GHz matched the 5 GHz transaction standard"
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
