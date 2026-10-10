#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[signal-perf] Running worth-signal performance lane..."

PERF_REPORT_PATH="${SIGNAL_PERF_REPORT_PATH:-${RUNNER_TEMP:-/tmp}/signal-parallel-perf-report.json}"

cargo test -p worth-signal tests::performance::push_perf_10k_nodes --quiet
cargo test -p worth-signal tests::performance::ondemand_defer_perf_10k_nodes --quiet
cargo test -p worth-signal tests::performance::chain_1000_minimal_recomputation --quiet
cargo test -p worth-signal tests::transaction_stress::rollback_heavy_workload_leaves_runtime_consistent --quiet
cargo test -p worth-signal tests::transaction_stress::stress_100k_nodes_transaction_commit -- --ignored --quiet
cargo run -q -p worth-signal --bin signal_parallel_perf_report > "$PERF_REPORT_PATH"

echo "[signal-perf] Wrote phase report to $PERF_REPORT_PATH"

echo "[signal-perf] PASS"
