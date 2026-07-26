#!/usr/bin/env bash
set -u

blocker_failures=0

run_blocker() {
  local label="$1"
  shift
  echo "RELEASE BLOCKER: $label"
  if "$@"; then echo "PASS: $label"; else echo "FAIL: $label"; blocker_failures=$((blocker_failures + 1)); fi
}

run_advisory() {
  local label="$1"
  shift
  echo "ADVISORY: $label"
  if "$@"; then echo "PASS: $label"; else echo "REVIEW NEEDED: $label"; fi
}

run_blocker "safe Rust workspace tests" cargo test --workspace
run_blocker "safe Svelte suite" npm test
run_advisory "shared version and committed lockfiles" node scripts/check-version.mjs
run_advisory "Rust formatting" cargo fmt --all -- --check
run_advisory "Rust linting" cargo clippy --workspace --all-targets -- -D warnings
run_advisory "Svelte and TypeScript checking" npm run check
run_advisory "frontend production build" npm run build
echo "ADVISORY: complete docs/release/manual-matrix.md on Ubuntu 26.04 amd64 before approving a public package."

if (( blocker_failures > 0 )); then
  echo "$blocker_failures release-blocking test suite(s) failed. Release is blocked pending the user's decision."
  exit 1
fi
echo "Release-blocking safe tests passed. Advisory evidence remains subject to user review."
