#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[relational-complexity] enforcing declared complexity contracts and budget proofs"

module="crates/worth-relational/src/tests/complexity/contracts.rs"
doc="plans/engineering/worth_relational_complexity_budgets.md"

if [[ ! -f "$module" ]]; then
  echo "FAIL: missing complexity contract test module: $module"
  exit 1
fi

if [[ ! -f "$doc" ]]; then
  echo "FAIL: missing complexity budget doc: $doc"
  exit 1
fi

if ! rg -q "^// CONTRACT:" "$module"; then
  echo "FAIL: complexity contract tag missing in $module"
  exit 1
fi

if ! rg -q "COMPLEXITY_CONTRACTS" crates/worth-relational/src/performance/data/mod.rs; then
  echo "FAIL: missing runtime complexity registry"
  exit 1
fi

echo "[relational-complexity] declaration and proof lanes"
cargo test -p worth-relational tests::complexity::contracts::complexity_contract_registry_covers_runtime_hot_paths -- --nocapture
cargo test -p worth-relational tests::complexity::contracts::complexity_contract_partition_edition_acquisition_is_declared_and_measured -- --nocapture
cargo test -p worth-relational tests::complexity::contracts::complexity_budget_snapshot_pin_maintenance_is_incremental -- --nocapture
cargo test -p worth-relational tests::complexity::contracts::complexity_contract_visibility_scans_are_explicitly_measured -- --nocapture
cargo test -p worth-relational tests::complexity::contracts::complexity_contract_invariant_materialization_is_declared_and_measured -- --nocapture
cargo test -p worth-relational tests::complexity::contracts::complexity_budget_live_history_trimming_is_touched_record_bounded -- --nocapture
cargo test -p worth-relational tests::complexity::contracts::complexity_budget_bidirectional_adjacency_avoids_relation_scans -- --nocapture

echo "[relational-complexity] PASS"
