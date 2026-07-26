#!/usr/bin/env bash
set -euo pipefail

failures=0

report_matches() {
  local label="$1"
  local pattern="$2"
  shift 2
  if rg -n --hidden --glob '!.git/**' --glob '!target/**' --glob '!node_modules/**' --glob '!dist/**' --glob '!scripts/audit-public-tree.sh' "$pattern" "$@"; then
    echo "FAIL: $label" >&2
    failures=$((failures + 1))
  else
    echo "PASS: $label"
  fi
}

tracked_forbidden="$(git ls-files | rg '(^|/)(__pycache__|node_modules|target|dist)(/|$)|\.(deb|rpm|AppImage|exe|dll|pcap|pcapng|raw\.strace|pyc)$' || true)"
if [[ -n "$tracked_forbidden" ]]; then
  echo "$tracked_forbidden" >&2
  echo "FAIL: generated, vendor, capture, or release artifact is tracked" >&2
  failures=$((failures + 1))
else
  echo "PASS: no forbidden artifact is tracked"
fi

report_matches "no personal absolute home paths" '/home/[[:alnum:]_.-]+/' .
report_matches "no personal author email" 'hotmail\.com' .
report_matches "no known private identifiers" 'k27chang|kingOgames|009C9230|enxc8a3629c9230|18:8b:15:55:73:91|72:30:0a:c2:37:3b|2a:52:1a:27:09:00|192\.168\.137\.(77|166|175|176)|"ssid": "prismpulse"' .
report_matches "no private key material" 'BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY' .
report_matches "no GitHub access token literals" 'github_pat_[[:alnum:]_]{20,}|gh[pousr]_[[:alnum:]]{20,}' .
report_matches "no AWS access key literals" 'AKIA[0-9A-Z]{16}' .

if (( failures > 0 )); then
  echo "$failures public-tree audit check(s) failed." >&2
  exit 1
fi

echo "Public-tree audit passed. This complements, but does not replace, GitHub secret scanning and manual review."
