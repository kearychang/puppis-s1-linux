"""PROTOTYPE: pure P1411 frame encoder/decoder; not production code."""

from __future__ import annotations

import binascii
import json


MAGIC = 0xAA
REQUEST_TYPE = 0x01
RESPONSE_TYPE = 0x02
RESERVED = 0x00
ENDPOINT = ("192.168.137.254", 10081)


def encode_frame(message: dict, frame_type: int = REQUEST_TYPE) -> bytes:
    payload = json.dumps(
        message,
        ensure_ascii=False,
        separators=(",", ":"),
    ).encode("utf-8")
    body = bytes((frame_type, RESERVED)) + payload
    total_length = 6 + len(body)
    if total_length > 0xFF:
        raise ValueError("observed P1411 frame has a one-byte length field")
    checksum = binascii.crc32(body)
    return bytes((MAGIC, total_length)) + checksum.to_bytes(4, "big") + body


def encode_getter(function_name: str) -> bytes:
    return encode_frame({"fun": function_name, "args": {}})


def decode_frame(frame: bytes) -> dict:
    if len(frame) < 8:
        raise ValueError("frame is shorter than the eight-byte header")
    if frame[0] != MAGIC:
        raise ValueError(f"unexpected magic 0x{frame[0]:02x}")
    if frame[1] != len(frame):
        raise ValueError(f"length field {frame[1]} does not match {len(frame)} bytes")
    if frame[7] != RESERVED:
        raise ValueError(f"unexpected reserved byte 0x{frame[7]:02x}")

    wire_checksum = int.from_bytes(frame[2:6], "big")
    calculated_checksum = binascii.crc32(frame[6:])
    if wire_checksum != calculated_checksum:
        raise ValueError(
            f"CRC mismatch: wire={wire_checksum:08x} calculated={calculated_checksum:08x}"
        )

    return {
        "endpoint": f"{ENDPOINT[0]}:{ENDPOINT[1]}",
        "total_length": frame[1],
        "crc32": f"{wire_checksum:08x}",
        "type": frame[6],
        "reserved": frame[7],
        "message": json.loads(frame[8:].decode("utf-8")),
    }
