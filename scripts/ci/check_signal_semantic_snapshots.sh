#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${repo_root}"

output_dir="${1:-${RUNNER_TEMP:-/tmp}/signal-semantic-snapshots}"
mkdir -p "${output_dir}"
rm -f "${output_dir}"/*.json

profiles=(
  serial
  staged-2x1
  staged-4x2
  full-2x1
  full-4x2
)
runtime_policies=(
  operational
  development
  forensic
)

echo "[signal-snapshots] Writing canonical semantic snapshots to ${output_dir}"
for runtime_policy in "${runtime_policies[@]}"; do
  for profile in "${profiles[@]}"; do
    cargo run -q -p worth-signal --bin signal_semantic_snapshot -- "${profile}" "${runtime_policy}" \
      > "${output_dir}/${runtime_policy}-${profile}.json"
  done
done

status=0
for runtime_policy in "${runtime_policies[@]}"; do
  echo "[signal-snapshots] Re-running hottest profile for ${runtime_policy} to catch flaky drift"
  cargo run -q -p worth-signal --bin signal_semantic_snapshot -- full-4x2 "${runtime_policy}" \
    > "${output_dir}/${runtime_policy}-full-4x2-repeat.json"
  baseline="${output_dir}/${runtime_policy}-serial.json"
  for profile in "${profiles[@]:1}"; do
    echo "[signal-snapshots] Diffing ${runtime_policy}/serial against ${runtime_policy}/${profile}"
    if ! diff -u "${baseline}" "${output_dir}/${runtime_policy}-${profile}.json"; then
      status=1
    fi
  done
  echo "[signal-snapshots] Diffing repeated ${runtime_policy}/full-4x2 run"
  if ! diff -u "${output_dir}/${runtime_policy}-full-4x2.json" "${output_dir}/${runtime_policy}-full-4x2-repeat.json"; then
    status=1
  fi
done

if [[ "${status}" -ne 0 ]]; then
  echo "[signal-snapshots] semantic snapshot drift detected"
  exit "${status}"
fi

echo "[signal-snapshots] snapshots are stable"
