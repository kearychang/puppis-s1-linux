#!/usr/bin/env python3
"""PROTOTYPE TUI: inspect captured P1411 frames without network access."""

from __future__ import annotations

import json
import os

from protocol_logic import decode_frame, encode_getter


FRAMES = {
    "1": (
        "captured getFirstlogin request",
        "aa2979edc09301007b2266756e223a2267657446697273746c6f67696e222c2261726773223a7b7d7d",
    ),
    "2": (
        "captured getDevice request",
        "aa25bd52005d01007b2266756e223a22676574446576696365222c2261726773223a7b7d7d",
    ),
    "3": (
        "privacy-redacted getDevice response fixture",
        "aac887937b9402007b2266756e223a22676574446576696365222c2264617461223a7b226d6f6465223a2231222c22616c696173223a225031343131222c226d6f64656c223a225031343131222c22706e223a22507269736d585220507570706973205331222c22736e223a225b52454441435445445d222c226677223a22422d4d443246503134313156312e32322d3235303130382d7230653137222c2273747265616d223a2231222c227374696d65223a22373636227d2c22737461747573223a226f6b227d",
    ),
}

BOLD = "\x1b[1m"
DIM = "\x1b[2m"
RESET = "\x1b[0m"


def render(state: dict) -> None:
    if os.isatty(1):
        print("\x1b[2J\x1b[H", end="")
    print(f"{BOLD}P1411 protocol prototype — no network writes{RESET}\n")
    print(f"{BOLD}source:{RESET} {state['source']}")
    if state.get("error"):
        print(f"{BOLD}error:{RESET} {state['error']}")
    else:
        decoded = state["decoded"]
        for field in ("endpoint", "total_length", "crc32", "type", "reserved"):
            print(f"{BOLD}{field}:{RESET} {decoded[field]}")
        print(f"{BOLD}message:{RESET}")
        print(json.dumps(decoded["message"], indent=2, ensure_ascii=False))
        print(f"{DIM}hex: {state['hex']}{RESET}")
    print(
        "\n"
        f"{BOLD}[1]{RESET} first-login  "
        f"{BOLD}[2]{RESET} device request  "
        f"{BOLD}[3]{RESET} redacted response  "
        f"{BOLD}[g]{RESET} encode getter locally  "
        f"{BOLD}[p]{RESET} paste frame  "
        f"{BOLD}[q]{RESET} quit"
    )


def load(source: str, frame_hex: str) -> dict:
    try:
        frame = bytes.fromhex(frame_hex)
        return {
            "source": source,
            "hex": frame.hex(),
            "decoded": decode_frame(frame),
        }
    except (ValueError, UnicodeError, json.JSONDecodeError) as error:
        return {"source": source, "hex": frame_hex, "error": str(error)}


def main() -> None:
    state = load(*FRAMES["2"])
    while True:
        render(state)
        action = input("\nchoice> ").strip().lower()
        if action == "q":
            return
        if action in FRAMES:
            state = load(*FRAMES[action])
        elif action == "g":
            name = input("getter function name> ").strip()
            state = load(f"locally encoded getter: {name}", encode_getter(name).hex())
        elif action == "p":
            state = load("pasted frame", input("frame hex> ").strip())


if __name__ == "__main__":
    main()
