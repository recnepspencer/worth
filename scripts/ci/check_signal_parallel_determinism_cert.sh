#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ITERATIONS="${1:-4}"
OUT_DIR="${2:-${RUNNER_TEMP:-/tmp}/signal-determinism-cert}"
mkdir -p "${OUT_DIR}"

echo "[signal-determinism-cert] iterations=${ITERATIONS} output=${OUT_DIR}"

first_run=""
for iteration in $(seq 1 "${ITERATIONS}"); do
  run_dir="${OUT_DIR}/run-${iteration}"
  bash scripts/ci/check_signal_semantic_snapshots.sh "${run_dir}"
  if [[ -z "${first_run}" ]]; then
    first_run="${run_dir}"
    continue
  fi

  while IFS= read -r snapshot; do
    echo "[signal-determinism-cert] Diffing run-1/${snapshot} against run-${iteration}/${snapshot}"
    diff -u "${first_run}/${snapshot}" "${run_dir}/${snapshot}"
  done < <(cd "${first_run}" && find . -maxdepth 1 -name '*.json' -print | sed 's|^\./||' | sort)
done

echo "[signal-determinism-cert] Running hostile ignored parity loop"
cargo test -p worth-signal --lib adversarial_parallel -- --ignored --nocapture

echo "[signal-determinism-cert] PASS"
