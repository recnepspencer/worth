#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

CONFIGURATION="${1:-}"
case "$CONFIGURATION" in
  default)
    PACKAGE_FEATURE_ARGS=()
    CONTROL_FEATURE_ARGS=(--features test-operation-control)
    ;;
  profile-compact|profile-compact,parallel|profile-standard|profile-standard,parallel|profile-extended|profile-extended,parallel)
    PACKAGE_FEATURE_ARGS=(--no-default-features --features "$CONFIGURATION")
    CONTROL_FEATURE_ARGS=(--no-default-features --features "$CONFIGURATION,test-operation-control")
    ;;
  *)
    echo "FAIL: expected default or one reviewed Signal profile configuration" >&2
    exit 2
    ;;
esac

echo "[signal-owner-matrix] package regression: $CONFIGURATION"
cargo test -p worth-signal "${PACKAGE_FEATURE_ARGS[@]}" --all-targets

ROSTER_ARGS=()
while IFS= read -r name; do
  [[ -n "$name" ]] || continue
  ROSTER_ARGS+=(--expect-name "$name")
done < scripts/ci/signal_owner_service_operation_control_roster.txt

bash scripts/ci/run_signal_owner_service_selection.sh \
  --test signal_owner_services \
  "${CONTROL_FEATURE_ARGS[@]}" \
  --selection adversarial::operation_control:: \
  "${ROSTER_ARGS[@]}"

if [[ "$CONFIGURATION" == "default" || "$CONFIGURATION" == "profile-extended,parallel" ]]; then
  bash scripts/ci/run_signal_owner_service_selection.sh \
    --test signal_owner_services \
    "${CONTROL_FEATURE_ARGS[@]}" \
    --selection adversarial::capacity_cleanup:: \
    --ignored \
    --expect-name adversarial::capacity_cleanup::live_branch_capacity_denies_then_retirement_restores_one_slot \
    --expect-name adversarial::capacity_cleanup::operation_capacity_denies_the_65th_and_restores_after_release \
    --expect-name adversarial::capacity_cleanup::retention_capacity_denies_then_all_releases_restore_one_lease
fi
