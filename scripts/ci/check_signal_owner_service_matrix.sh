#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

CONFIGURATION="${1:-}"
case "$CONFIGURATION" in
  default)
    PACKAGE_FEATURE_ARGS=()
    CONTROL_FEATURE_ARGS=()
    ;;
  profile-compact|profile-standard|profile-extended)
    PACKAGE_FEATURE_ARGS=(--no-default-features --features "$CONFIGURATION")
    CONTROL_FEATURE_ARGS=("${PACKAGE_FEATURE_ARGS[@]}")
    ;;
  *)
    echo "FAIL: expected default or one reviewed Signal profile configuration" >&2
    exit 2
    ;;
esac

echo "[signal-owner-matrix] package regression: $CONFIGURATION"
cargo test -p worth-signal "${PACKAGE_FEATURE_ARGS[@]}" --all-targets

bash scripts/ci/run_signal_owner_service_selection.sh \
  --test worth_signal \
  "${CONTROL_FEATURE_ARGS[@]}" \
  --selection signal_owner_services::adversarial::operation_control::

if [[ "$CONFIGURATION" == "default" || "$CONFIGURATION" == "profile-extended" ]]; then
  bash scripts/ci/run_signal_owner_service_selection.sh \
    --test worth_signal \
    "${CONTROL_FEATURE_ARGS[@]}" \
    --selection fast::signal_owner_services::adversarial::capacity_cleanup::
fi
