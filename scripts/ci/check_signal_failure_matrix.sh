#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[signal-failure-matrix] planner apply-group rollback / no-leak"
cargo test -p worth-signal --lib \
  tests::adversarial_parallel::full_parallel_apply_failure_does_not_leak_partial_semantic_state \
  -- --nocapture

echo "[signal-failure-matrix] transaction precompute failure"
cargo test -p worth-signal --lib \
  tests::diagnostics::execution_failures_and_rollbacks_automatically_record_diagnostics \
  -- --nocapture

echo "[signal-failure-matrix] event begin failure"
cargo test -p worth-signal --lib \
  tests::diagnostics::event_bus_begin_failures_record_failure_and_rollback_diagnostics \
  -- --nocapture

echo "[signal-failure-matrix] event flush / commit-promotion failure"
cargo test -p worth-signal --lib \
  tests::diagnostics::commit_promotion_failures_record_failure_and_rollback_diagnostics \
  -- --nocapture

echo "[signal-failure-matrix] poisoned transaction commit + rollback"
cargo test -p worth-signal --lib \
  logic::transaction::tests::poisoned_transaction_returns_poisoned_outcome \
  -- --nocapture
cargo test -p worth-signal --lib \
  logic::transaction::tests::poisoned_rollback_rewinds_graph \
  -- --nocapture

echo "[signal-failure-matrix] rollback/commit churn no semantic leakage"
cargo test -p worth-signal --lib \
  logic::transaction::tests::hostile_rollback_and_commit_cycles_do_not_leak_semantic_events \
  -- --nocapture

echo "[signal-failure-matrix] commit failure does not leak committed outcome"
cargo test -p worth-signal --lib \
  logic::transaction::tests::hostile_commit_failure_does_not_leak_committed_semantic_outcome \
  -- --nocapture

echo "[signal-failure-matrix] repeated rollback retention remains current"
cargo test -p worth-signal --lib \
  tests::diagnostics::repeated_rollbacks_keep_latest_rollback_current_and_bounded \
  -- --nocapture

echo "[signal-failure-matrix] PASS"
