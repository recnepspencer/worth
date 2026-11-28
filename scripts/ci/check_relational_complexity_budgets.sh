#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

registry="crates/worth-relational/src/tests/complexity/complexity_contract_inventory.rs"
runner="$ROOT_DIR/scripts/ci/run_relational_named_test_selection.sh"

echo "[relational-complexity] checking declared complexity contracts and proof registrations"

runtime_proofs=(
  tests::complexity::contracts::commit_budgets::contract_registry::every_complexity_contract_proof_names_a_registered_test
  tests::complexity::contracts::commit_budgets::contract_registry::complexity_contract_registry_covers_runtime_hot_paths
  tests::complexity::contracts::commit_budgets::partition_local_commit::complexity_contract_partition_edition_acquisition_is_declared_and_measured
  tests::complexity::contracts::visibility_budgets::snapshot_visibility::complexity_budget_snapshot_release_uses_only_the_carried_root_obligation
  tests::complexity::contracts::visibility_budgets::snapshot_visibility::complexity_contract_visibility_scans_are_explicitly_measured
  tests::complexity::contracts::visibility_budgets::invariant_materialization::complexity_contract_invariant_materialization_is_declared_and_measured
  tests::complexity::contracts::visibility_budgets::retention_and_adjacency::live_history_trimming_runs_only_in_the_declared_full_retention_pass
  tests::complexity::contracts::visibility_budgets::retention_and_adjacency::complexity_budget_bidirectional_adjacency_avoids_relation_scans
)
for proof in "${runtime_proofs[@]}"; do
  bash "$runner" --lib --selection "$proof" --exact --expect-name "$proof"
done

# Runtime enumerates its own binary. Check each extracted proof in its owner.
owner_proofs="$(rg -o --no-filename '"internal_port::[a-z0-9_:]+"' "$registry" "${registry%.rs}" | tr -d '"' | sort -u)"
while IFS= read -r proof; do
  case "$proof" in
    internal_port::intent_admission::*) package=worth-relational-mutation ;;
    internal_port::storage::residency::*) package=worth-relational-durability ;;
    internal_port::runtime::state::subsystems::indexing::*|internal_port::indexes::*) package=worth-relational-indexes ;;
    internal_port::storage::overlay::*|internal_port::runtime::state::subsystems::storage::*) package=worth-relational-storage ;;
    *) printf 'No owner for complexity proof %s\n' "$proof" >&2; exit 1 ;;
  esac
  bash "$runner" --package "$package" --lib --selection "$proof" --exact --expect-name "$proof"
done <<< "$owner_proofs"

echo "[relational-complexity] PASS"
