#!/usr/bin/env bash

set -euo pipefail

echo "[signal-adversarial] workflow and property certification lane"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/worth-signal-adversarial-shared}"
export RUSTFLAGS="${RUSTFLAGS:-} -C debuginfo=0"

cargo test -p worth-signal --lib adversarial_properties -- --nocapture
cargo test -p worth-signal --lib tests::adversarial_workflows::geometry_kernel_adversarial_seed_matrix_keeps_invariants -- --exact --nocapture
cargo test -p worth-signal --lib tests::adversarial_workflows::fintech_adversarial_seed_matrix_keeps_invariants -- --exact --nocapture
cargo test -p worth-signal --lib tests::adversarial_workflows::policy_overlap_for_generated_workflows_matches_guaranteed_truth -- --exact --nocapture
cargo test -p worth-signal --lib tests::adversarial_workflows::focused_parallel_branch_restore_and_evaluate_dirty_regression -- --exact --nocapture --test-threads=1
cargo test -p worth-signal --lib tests::adversarial_workflows::parallel_geometry_hostile_session_matches_serial_truth -- --exact --nocapture --test-threads=1
cargo test -p worth-signal --lib tests::adversarial_workflows::parallel_fintech_hostile_session_matches_serial_truth -- --exact --nocapture --test-threads=1

if [[ "${1:-}" == "--long" ]]; then
  cargo test -p worth-signal --lib adversarial_workflows -- --ignored --nocapture
fi

echo "[signal-adversarial] lane is green"
