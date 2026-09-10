#!/usr/bin/env bash
# check_float_literals.sh — CI lint for hardcoded tolerance literals.
#
# Scans production Rust code for `1e-NN` patterns that should use
# ToleranceProvider instead of hardcoded constants.
#
# ALLOWLIST:
#   - Test code: *_tests.rs, *_test.rs, */tests/*, */tests.rs, #[cfg(test)]
#   - Deprecated code: */_deprecated/*
#   - Tolerance definitions: WORTH-core/src/tolerance.rs
#   - Doc comments: lines starting with ///  or //!
#
# Exit code 0 = clean, 1 = violations found.

set -euo pipefail

CRATES_DIR="$(cd "$(dirname "$0")/.." && pwd)/crates"

# Files to scan: production .rs files (not tests, not deprecated)
violations=$(
  find "$CRATES_DIR" -name '*.rs' \
    -not -path '*/_deprecated/*' \
    -not -path '*/tests/*' \
    -not -path '*/tests.rs' \
    -not -name '*_tests.rs' \
    -not -name '*_test.rs' \
    -not -path '*/tolerance.rs' \
    -not -path '*/testing/*' \
    -not -path '*/target/*' \
  | xargs grep -n '1e-[0-9]\+' 2>/dev/null \
  | grep -v '^\s*//' \
  | grep -v '///\|//!' \
  | grep -v '#\[cfg(test)\]' \
  | grep -v '#\[test\]' \
  || true
)

if [ -z "$violations" ]; then
  echo "✅ No hardcoded tolerance literals found in production code."
  exit 0
else
  echo "❌ Hardcoded tolerance literals found in production code!"
  echo ""
  echo "These should use WORTH_core::ToleranceProvider methods instead:"
  echo "  - geometry_epsilon()  for geometric identity checks"
  echo "  - vertex_tolerance()  for per-vertex thresholds"
  echo "  - global_default()    for conservative fallback"
  echo ""
  echo "Or use WORTH_core comparison predicates:"
  echo "  - approximately_equal(a, b, &tol)"
  echo "  - positions_coincident(&a, &b, &tol)"
  echo "  - is_effectively_zero(val, &tol)"
  echo "  - is_degenerate_magnitude_sq(mag_sq, &tol)"
  echo ""
  echo "Violations:"
  echo "$violations"
  exit 1
fi
