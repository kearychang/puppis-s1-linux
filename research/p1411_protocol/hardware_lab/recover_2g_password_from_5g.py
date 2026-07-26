#!/usr/bin/env python3
"""PROTOTYPE RECOVERY: replace the 2.4 GHz test password with the live 5 GHz password."""

from __future__ import annotations

import argparse
import json

from research.p1411_protocol.hardware_lab.live_read import get
from research.p1411_protocol.hardware_lab.safe_2g_roundtrip import (
    PASSWORD_VALUES,
    QUALIFIED_FIRMWARE,
    read_settings,
    send_settings,
    wait_for_settings,
)
from research.p1411_protocol.hardware_lab.safe_ssid_roundtrip import safe_response


CONFIRMATION = "RECOVER-2G-FROM-5G"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--confirm")
    args = parser.parse_args()
    if not args.execute or args.confirm != CONFIRMATION:
        parser.error(
            f"recovery requires --execute --confirm {CONFIRMATION}; no write was sent"
        )

    device = get("getDevice")
    device_data = device.get("data", {})
    if device.get("status") != "ok" or device_data.get("model") != "P1411":
        raise RuntimeError("device did not verify as P1411; refusing recovery")
    if device_data.get("fw") != QUALIFIED_FIRMWARE:
        raise RuntimeError("firmware is not qualified for this recovery")
    if device_data.get("mode") not in {"1", "2"}:
        raise RuntimeError("mode is not qualified for this recovery")

    current_2g = read_settings()
    if current_2g["pwd"] not in PASSWORD_VALUES:
        raise RuntimeError("2.4 GHz password is no longer a prototype value; refusing overwrite")

    message_5g = get("get5GHotspot")
    settings_5g = message_5g.get("data", {})
    password_5g = settings_5g.get("pwd")
    if message_5g.get("status") != "ok" or not isinstance(password_5g, str):
        raise RuntimeError("5 GHz password is unavailable; refusing recovery")
    if not password_5g or password_5g in PASSWORD_VALUES:
        raise RuntimeError("5 GHz password is not a safe recovery candidate")

    recovered = dict(current_2g)
    recovered["pwd"] = password_5g
    response = safe_response(send_settings(recovered))
    wait_for_settings(recovered)
    verified = read_settings()

    print(
        json.dumps(
            {
                "command": "set2GHotspot",
                "recovery": "replace prototype 2.4 GHz password with live 5 GHz password",
                "setter_response": response,
                "complete_2g_object_verified": verified == recovered,
                "password_matches_5g": verified["pwd"] == password_5g,
                "prototype_password_removed": verified["pwd"] not in PASSWORD_VALUES,
                "all_non_password_fields_preserved": all(
                    verified[key] == current_2g[key] for key in current_2g if key != "pwd"
                ),
                "secrets_logged": False,
                "device_state": "reconciled",
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
