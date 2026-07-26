#!/usr/bin/env python3
"""Transactional, allowlisted 5 GHz setting mutation and restoration proof."""

from __future__ import annotations

import argparse
import json
import time

from research.p1411_protocol.hardware_lab.live_read import get
from research.p1411_protocol.hardware_lab.safe_ssid_roundtrip import (
    POLL_SECONDS,
    safe_response,
    send_set5g,
)


TEMPORARY_VALUES = {
    "ch": "36",
    "pwd": "CodexTest123!",
}
CONFIRMATION = "TEST-5G-SETTING-AND-RESTORE"


def wait_for_field(field: str, expected: str) -> None:
    deadline = time.monotonic() + POLL_SECONDS
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            message = get("get5GHotspot", timeout=3.0)
            if message.get("status") == "ok" and message.get("data", {}).get(field) == expected:
                return
        except (OSError, ValueError, ConnectionError) as error:
            last_error = error
        time.sleep(1)
    suffix = f"; last error: {type(last_error).__name__}" if last_error else ""
    raise TimeoutError(f"{field} read-back did not match within {POLL_SECONDS}s{suffix}")


def display_value(field: str, value: str) -> str:
    return "[REDACTED]" if field == "pwd" else value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("field", choices=sorted(TEMPORARY_VALUES))
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
    original_value = original.get(args.field)
    if not isinstance(original_value, str):
        raise RuntimeError(f"current {args.field} is missing; refusing write")

    temporary_value = TEMPORARY_VALUES[args.field]
    if original_value == temporary_value:
        raise RuntimeError("temporary value already active; refusing ambiguous proof")
    temporary = dict(original)
    temporary[args.field] = temporary_value

    result = {
        "command": "set5GHotspot",
        "changed_field": args.field,
        "original_value": display_value(args.field, original_value),
        "temporary_value": display_value(args.field, temporary_value),
        "all_other_fields_preserved": True,
    }
    changed_verified = False
    try:
        result["temporary_set_response"] = safe_response(send_set5g(temporary))
        wait_for_field(args.field, temporary_value)
        changed_verified = True
        result["temporary_readback_verified"] = True
    finally:
        result["restore_attempted"] = True
        result["restore_response"] = safe_response(send_set5g(original))
        wait_for_field(args.field, original_value)
        result["restoration_readback_verified"] = True

    result["transaction_succeeded"] = changed_verified
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
