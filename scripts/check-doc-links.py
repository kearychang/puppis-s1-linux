#!/usr/bin/env python3
"""Check relative Markdown links without making network requests."""

from __future__ import annotations

import pathlib
import re
import sys
import urllib.parse


ROOT = pathlib.Path(__file__).resolve().parents[1]
LINK = re.compile(r"(?<!!)\[[^]]*\]\(([^)]+)\)")
failures: list[str] = []

for document in sorted(ROOT.rglob("*.md")):
    if any(part in {"node_modules", "target", "dist", ".git"} for part in document.parts):
        continue
    text = document.read_text(encoding="utf-8")
    for raw_target in LINK.findall(text):
        target = raw_target.strip().split(maxsplit=1)[0].strip("<>")
        if not target or target.startswith(("#", "http://", "https://", "mailto:")):
            continue
        path_text = urllib.parse.unquote(target.split("#", 1)[0])
        resolved = (document.parent / path_text).resolve()
        try:
            resolved.relative_to(ROOT)
        except ValueError:
            failures.append(f"{document.relative_to(ROOT)}: link escapes repository: {target}")
            continue
        if not resolved.exists():
            failures.append(f"{document.relative_to(ROOT)}: missing target: {target}")

if failures:
    print("\n".join(failures), file=sys.stderr)
    raise SystemExit(1)

print("Relative Markdown links are valid.")
