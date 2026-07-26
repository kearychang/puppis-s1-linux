#!/usr/bin/env python3
"""Read-only P1411 getter client. This module never sends setter commands."""

from __future__ import annotations

import argparse
import json
import socket

from research.p1411_protocol.protocol_logic import ENDPOINT, decode_frame, encode_getter


def recv_exact(sock: socket.socket, size: int) -> bytes:
    chunks: list[bytes] = []
    remaining = size
    while remaining:
        chunk = sock.recv(remaining)
        if not chunk:
            raise ConnectionError("device closed the connection")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)


def recv_frame(sock: socket.socket) -> bytes:
    header = recv_exact(sock, 8)
    total_length = header[1]
    if total_length < 8:
        raise ValueError(f"invalid total length: {total_length}")
    return header + recv_exact(sock, total_length - 8)


def get(function: str, *, timeout: float = 3.0) -> dict:
    host, port = ENDPOINT
    with socket.create_connection((host, port), timeout=timeout) as sock:
        sock.settimeout(timeout)
        sock.sendall(encode_getter(function))
        decoded = decode_frame(recv_frame(sock))
    message = decoded["message"]
    if message.get("fun") != function:
        raise ValueError(
            f"response correlation failed: expected {function!r}, "
            f"received {message.get('fun')!r}"
        )
    return message


def redact(value):
    if isinstance(value, dict):
        return {
            key: "[REDACTED]"
            if key.lower()
            in {
                "pwd",
                "password",
                "sn",
                "serial",
                "ssid",
                "bssid",
                "mac",
                "hostname",
                "host_name",
            }
            else redact(item)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [redact(item) for item in value]
    return value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("function", nargs="+", help="getter name, for example get5GHotspot")
    args = parser.parse_args()
    for function in args.function:
        if not function.startswith("get"):
            parser.error(f"refusing non-getter command: {function}")
        print(json.dumps(redact(get(function)), indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
