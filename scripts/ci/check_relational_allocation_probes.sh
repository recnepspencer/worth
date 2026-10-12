#!/usr/bin/env bash
#
# The allocation-slope court always compiles in the unit-test binary, which
# owns its counting allocator. Named selection proves the drivers execute.
#
# This lane is a declaration, not a second selection engine: the shared named
# selection authority does the listing, the executability preflight and the run.
# What is local to this lane is which tests must be there, and that has to be
# spelled out rather than counted. The two driver tests re-execute the test
# binary with the probe gate set; the two isolated probes return immediately
# and pass when that gate is absent. So deleting a driver leaves a selection
# that still lists tests, still executes them, and still goes green while
# measuring no slope at all. Naming all four is what makes a rename or a
# deletion a red lane instead of a smaller green one.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SELECTOR="tests::complexity::contracts::substrate_edition_budgets::allocation_slope"

echo "[relational-allocation-probes] enforcing executed allocation-slope evidence"

exec bash "${ROOT_DIR}/scripts/ci/run_relational_named_test_selection.sh" \
  --lib \
  --selection "$SELECTOR" \
  --expect-name "${SELECTOR}::bounded_adjacency_allocation_is_flat_in_fanout_beyond_the_budget" \
  --expect-name "${SELECTOR}::bounded_adjacency_allocation_is_flat_in_partitions_the_anchor_is_not_in" \
  --expect-name "${SELECTOR}::isolated_bounded_adjacency_fanout_allocation_slope_probe" \
  --expect-name "${SELECTOR}::isolated_bounded_adjacency_partition_allocation_slope_probe"
