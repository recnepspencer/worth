#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[relational-perf-baseline] Running relational perf lane and baseline check..."

PERF_SUMMARY_PATH="${RELATIONAL_PERF_SUMMARY_PATH:-${RUNNER_TEMP:-/tmp}/relational-perf-summary.jsonl}"
PERF_BASELINE_PATH="${RELATIONAL_PERF_BASELINE_PATH:-$ROOT_DIR/plans/engineering/worth_relational_performance_baseline.jsonl}"

RELATIONAL_PERF_SUMMARY_PATH="$PERF_SUMMARY_PATH" \
RELATIONAL_PERF_BASELINE_PATH="$PERF_BASELINE_PATH" \
  ./scripts/ci/run_relational_perf_lane.sh

python scripts/ci/check_relational_perf_baseline.py \
  --baseline "$PERF_BASELINE_PATH" \
  --current "$PERF_SUMMARY_PATH"

echo "[relational-perf-baseline] PASS"
