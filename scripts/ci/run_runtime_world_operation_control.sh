#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

# Check executed cases, not a --list count that includes ignored tests.
output=$(mktemp)
trap 'rm -f "$output"' EXIT
status=0
cargo test -p worth-runtime-world --features test-operation-control \
  --test runtime_world_certification operation_control:: -- --color never 2>&1 \
  | tee "$output" || status=$?
if (( status != 0 )); then
  exit "$status"
fi
if ! grep -Eq '^test court::operation_control::[^ ]+ \.\.\. ok$' "$output"; then
  echo "Runtime World operation-control lane executed zero passing cases" >&2
  exit 1
fi
