#!/usr/bin/env bash
set -euo pipefail

# Run one reviewed Signal owner-service roster.  Listing, the ignored probe,
# and execution are built from the same target/features/filter vectors so a
# renamed, deleted, or newly ignored case cannot make a lane silently empty.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

PACKAGE="worth-signal"
TARGET_ARGS=()
FEATURE_ARGS=()
DEFAULT_ARGS=()
SELECTION=""
EXACT=""
IGNORED=""
EXPECTED_NAMES=()

usage_error() {
  echo "FAIL: $1" >&2
  echo "Usage: $(basename "${BASH_SOURCE[0]}") (--lib | --test <target>) [--no-default-features] [--features <list>] --selection <filter> [--exact] [--ignored] [--expect-name <exact test path>]..." >&2
  exit 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --lib)
      [[ ${#TARGET_ARGS[@]} -eq 0 ]] || usage_error "--lib and --test are mutually exclusive"
      TARGET_ARGS=(--lib)
      shift
      ;;
    --test)
      [[ ${#TARGET_ARGS[@]} -eq 0 ]] || usage_error "--lib and --test are mutually exclusive"
      [[ $# -ge 2 ]] || usage_error "--test requires a target name"
      TARGET_ARGS=(--test "$2")
      shift 2
      ;;
    --no-default-features)
      DEFAULT_ARGS+=(--no-default-features)
      shift
      ;;
    --features)
      [[ $# -ge 2 ]] || usage_error "--features requires a feature list"
      FEATURE_ARGS=(--features "$2")
      shift 2
      ;;
    --selection)
      [[ $# -ge 2 ]] || usage_error "--selection requires a filter"
      SELECTION="$2"
      shift 2
      ;;
    --exact)
      EXACT="--exact"
      shift
      ;;
    --ignored)
      IGNORED="--ignored"
      shift
      ;;
    --expect-name)
      [[ $# -ge 2 ]] || usage_error "--expect-name requires an exact test path"
      EXPECTED_NAMES+=("$2")
      shift 2
      ;;
    *)
      usage_error "unknown argument '$1'"
      ;;
  esac
done

[[ ${#TARGET_ARGS[@]} -gt 0 ]] || usage_error "one of --lib or --test is required"
[[ -n "$SELECTION" ]] || usage_error "--selection is required"

SELECTION_ARGS=("$SELECTION")
[[ -n "$EXACT" ]] && SELECTION_ARGS+=("$EXACT")
HARNESS_ARGS=("${SELECTION_ARGS[@]}")
[[ -n "$IGNORED" ]] && HARNESS_ARGS+=("$IGNORED")
IGNORED_PROBE_ARGS=("${SELECTION_ARGS[@]}" --ignored)

count_lines() {
  grep -c . <<<"$1" || true
}

list_selected_tests() {
  cargo test -p "$PACKAGE" "${DEFAULT_ARGS[@]}" "${FEATURE_ARGS[@]}" \
    "${TARGET_ARGS[@]}" -- --list "$@" \
    | sed -n 's/^\([[:alnum:]_:][[:alnum:]_:]*\): test$/\1/p'
}

fail_empty() {
  echo "FAIL: '$SELECTION' matched no compiled tests for $PACKAGE ${TARGET_ARGS[*]}" >&2
  exit 1
}

fail_all_ignored() {
  echo "FAIL: '$SELECTION' matched only #[ignore] tests in ordinary posture" >&2
  echo "Use --ignored for the scheduled roster or remove #[ignore] from a focused case." >&2
  exit 1
}

assert_exact_roster() {
  local executable="$1"
  local missing=""
  local unexpected=""
  local name
  for name in "${EXPECTED_NAMES[@]}"; do
    grep -qxF -- "$name" <<<"$executable" || missing+="$name"$'\n'
  done
  while IFS= read -r name; do
    [[ -n "$name" ]] || continue
    printf '%s\n' "${EXPECTED_NAMES[@]}" | grep -qxF -- "$name" \
      || unexpected+="$name"$'\n'
  done <<<"$executable"
  if [[ -n "$missing" || -n "$unexpected" ]]; then
    echo "FAIL: '$SELECTION' roster drifted" >&2
    [[ -n "$missing" ]] && { echo "Missing executable cases:" >&2; sed 's/^/  - /' <<<"${missing%$'\n'}" >&2; }
    [[ -n "$unexpected" ]] && { echo "Unexpected executable cases:" >&2; sed 's/^/  + /' <<<"${unexpected%$'\n'}" >&2; }
    exit 1
  fi
}

selected_names="$(list_selected_tests "${HARNESS_ARGS[@]}")"
selected_count="$(count_lines "$selected_names")"
[[ "$selected_count" -gt 0 ]] || fail_empty

if [[ -n "$IGNORED" ]]; then
  executable_names="$selected_names"
else
  ignored_names="$(list_selected_tests "${IGNORED_PROBE_ARGS[@]}")"
  executable_names=""
  while IFS= read -r name; do
    [[ -n "$name" ]] || continue
    grep -qxF -- "$name" <<<"$ignored_names" || executable_names+="$name"$'\n'
  done <<<"$selected_names"
  executable_names="${executable_names%$'\n'}"
  [[ -n "$executable_names" ]] || fail_all_ignored
fi

if [[ ${#EXPECTED_NAMES[@]} -gt 0 ]]; then
  assert_exact_roster "$executable_names"
fi

echo "[signal-owner-selection] '$SELECTION' selects $(count_lines "$executable_names") executable test(s)"
echo "[signal-owner-selection] executing the reviewed roster"
cargo test -p "$PACKAGE" "${DEFAULT_ARGS[@]}" "${FEATURE_ARGS[@]}" \
  "${TARGET_ARGS[@]}" -- "${HARNESS_ARGS[@]}" --nocapture --test-threads=1
