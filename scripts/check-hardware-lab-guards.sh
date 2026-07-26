#!/usr/bin/env bash
set -euo pipefail

expect_refusal() {
  local module="$1"
  shift
  local output
  if output="$(python3 -m "$module" "$@" 2>&1)"; then
    echo "FAIL: $module ran without explicit confirmation" >&2
    exit 1
  fi
  if [[ "$output" != *"no write was sent"* ]]; then
    echo "FAIL: $module did not fail at its confirmation guard" >&2
    echo "$output" >&2
    exit 1
  fi
  echo "PASS: $module refuses an unconfirmed mutation"
}

expect_refusal research.p1411_protocol.hardware_lab.safe_ssid_roundtrip
expect_refusal research.p1411_protocol.hardware_lab.safe_setting_roundtrip ch
expect_refusal research.p1411_protocol.hardware_lab.safe_2g_roundtrip
expect_refusal research.p1411_protocol.hardware_lab.safe_mode_roundtrip
expect_refusal research.p1411_protocol.hardware_lab.recover_2g_password_from_5g

factory_output="$(python3 -m research.p1411_protocol.hardware_lab.factory_reset)"
if [[ "$factory_output" != *'"dry_run": true'* ]]; then
  echo "FAIL: factory reset did not default to a dry run" >&2
  exit 1
fi
echo "PASS: factory reset defaults to a dry run"
