#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[relational-perf] Running worth-relational performance lane..."

PERF_LOG_PATH="${RELATIONAL_PERF_LOG_PATH:-${RUNNER_TEMP:-/tmp}/relational-perf.log}"
PERF_REPORT_PATH="${RELATIONAL_PERF_REPORT_PATH:-${RUNNER_TEMP:-/tmp}/relational-perf-report.jsonl}"
PERF_SUMMARY_PATH="${RELATIONAL_PERF_SUMMARY_PATH:-${RUNNER_TEMP:-/tmp}/relational-perf-summary.jsonl}"
PERF_MARKDOWN_PATH="${RELATIONAL_PERF_MARKDOWN_PATH:-${RUNNER_TEMP:-/tmp}/relational-perf-summary.md}"
PERF_BASELINE_PATH="${RELATIONAL_PERF_BASELINE_PATH:-$ROOT_DIR/plans/engineering/worth_relational_performance_baseline.jsonl}"
PERF_COMPARE_PATH="${RELATIONAL_PERF_COMPARE_PATH:-}"
PERF_ARCHIVE_DIR="${RELATIONAL_PERF_ARCHIVE_DIR:-}"

if [[ -z "$PERF_COMPARE_PATH" && -f "$PERF_BASELINE_PATH" ]]; then
  PERF_COMPARE_PATH="$PERF_BASELINE_PATH"
fi

if [[ -z "$PERF_COMPARE_PATH" && -n "$PERF_ARCHIVE_DIR" && -f "$PERF_ARCHIVE_DIR/latest-summary.jsonl" ]]; then
  PERF_COMPARE_PATH="$PERF_ARCHIVE_DIR/latest-summary.jsonl"
fi

WORTH_RELATIONAL_PERF_SAMPLES="${WORTH_RELATIONAL_PERF_SAMPLES:-3}" \
  cargo test -p worth-relational performance_profiles -- --ignored --nocapture --test-threads=1 \
  | tee "$PERF_LOG_PATH"

grep -E '^\{' "$PERF_LOG_PATH" > "$PERF_REPORT_PATH"
grep -E '^\{.*"samples":' "$PERF_REPORT_PATH" > "$PERF_SUMMARY_PATH"

SUMMARY_ARGS=(--input "$PERF_SUMMARY_PATH" --output "$PERF_MARKDOWN_PATH")
if [[ -n "$PERF_COMPARE_PATH" ]]; then
  SUMMARY_ARGS+=(--compare "$PERF_COMPARE_PATH")
fi
python scripts/ci/summarize_relational_perf.py "${SUMMARY_ARGS[@]}"

if [[ -n "$PERF_ARCHIVE_DIR" ]]; then
  RUN_STAMP="$(date -u +"%Y%m%dT%H%M%SZ")"
  RUN_DIR="$PERF_ARCHIVE_DIR/$RUN_STAMP"
  mkdir -p "$RUN_DIR"
  cp "$PERF_LOG_PATH" "$RUN_DIR/perf.log"
  cp "$PERF_REPORT_PATH" "$RUN_DIR/perf-report.jsonl"
  cp "$PERF_SUMMARY_PATH" "$RUN_DIR/perf-summary.jsonl"
  cp "$PERF_MARKDOWN_PATH" "$RUN_DIR/perf-summary.md"
  cp "$PERF_LOG_PATH" "$PERF_ARCHIVE_DIR/latest.log"
  cp "$PERF_REPORT_PATH" "$PERF_ARCHIVE_DIR/latest-report.jsonl"
  cp "$PERF_SUMMARY_PATH" "$PERF_ARCHIVE_DIR/latest-summary.jsonl"
  cp "$PERF_MARKDOWN_PATH" "$PERF_ARCHIVE_DIR/latest-summary.md"
  echo "[relational-perf] Archived artifacts to $RUN_DIR"
fi

echo "[relational-perf] Wrote full log to $PERF_LOG_PATH"
echo "[relational-perf] Wrote JSON samples to $PERF_REPORT_PATH"
echo "[relational-perf] Wrote JSON summaries to $PERF_SUMMARY_PATH"
echo "[relational-perf] Wrote markdown summary to $PERF_MARKDOWN_PATH"

echo "[relational-perf] PASS"
