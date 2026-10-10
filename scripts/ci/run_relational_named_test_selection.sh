#!/usr/bin/env bash
set -euo pipefail

# Enforces one policy for worth-relational CI test lanes: a named test selection
# must reach at least one test that the lane will actually execute, and is then
# executed under that exact same selection. The preflight and the execution
# share a single filter vector, so a lane can never assert one selection and
# run another.
#
# libtest's --list reports every test matching the filter regardless of its
# #[ignore] posture, but an ordinary run executes only the non-ignored ones.
# Listing alone therefore cannot prove executability: a family that became
# entirely #[ignore]d would list as present and then run nothing, reporting
# success. In ordinary posture this script subtracts the --ignored listing from
# the full listing to count what will really run. Under --ignored the listing is
# already exactly the set that executes.
#
# "At least one executable test" is the right floor for a family selection, but
# it is too weak for a lane whose claim depends on a known set of tests running
# together. Such a lane declares that set with repeated --expect-name. Every
# declared name must then answer to an executable test, and no undeclared test
# may join the selection, so a rename, a deletion or a new #[ignore] turns the
# lane red instead of quietly shrinking what it proves.
#
# Usage:
#   run_relational_named_test_selection.sh (--lib | --test <target>) \
#     --selection <filter> [--exact] [--ignored] \
#     [--expect-name <exact test path>]...

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

PACKAGE="worth-relational"
CARGO_TARGET_ARGS=()
SELECTION_FILTER_ARGS=()
HARNESS_FILTER_ARGS=()
IGNORED_PROBE_ARGS=()
EXPECTED_NAMES=()
SELECTION=""
TARGET_LABEL=""
POSTURE_LABEL="<default>"
IGNORED_POSTURE=""

parse_selection_request() {
  local exact="" ignored=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --lib)
        [[ -n "$TARGET_LABEL" ]] && fail_usage "--lib and --test are mutually exclusive"
        CARGO_TARGET_ARGS=(--lib)
        TARGET_LABEL="--lib"
        shift
        ;;
      --test)
        [[ -n "$TARGET_LABEL" ]] && fail_usage "--lib and --test are mutually exclusive"
        [[ $# -ge 2 ]] || fail_usage "--test requires a target name"
        CARGO_TARGET_ARGS=(--test "$2")
        TARGET_LABEL="--test $2"
        shift 2
        ;;
      --selection)
        [[ $# -ge 2 ]] || fail_usage "--selection requires a test filter"
        SELECTION="$2"
        shift 2
        ;;
      --expect-name)
        [[ $# -ge 2 ]] || fail_usage "--expect-name requires an exact test path"
        EXPECTED_NAMES+=("$2")
        shift 2
        ;;
      --exact)
        exact="--exact"
        shift
        ;;
      --ignored)
        ignored="--ignored"
        shift
        ;;
      *)
        fail_usage "unknown argument '$1'"
        ;;
    esac
  done

  [[ -n "$TARGET_LABEL" ]] || fail_usage "one of --lib or --test <target> is required"
  [[ -n "$SELECTION" ]] || fail_usage "--selection <filter> is required"

  # One authoritative selection, built once. Every vector below derives from it,
  # so the preflight, the executability probe and the execution cannot drift.
  SELECTION_FILTER_ARGS=("$SELECTION")
  [[ -n "$exact" ]] && SELECTION_FILTER_ARGS+=("$exact")

  HARNESS_FILTER_ARGS=("${SELECTION_FILTER_ARGS[@]}")
  [[ -n "$ignored" ]] && HARNESS_FILTER_ARGS+=("$ignored")

  # Same target, filter and --exact posture; only --ignored is added.
  IGNORED_PROBE_ARGS=("${SELECTION_FILTER_ARGS[@]}" --ignored)

  IGNORED_POSTURE="$ignored"

  POSTURE_LABEL="${exact:+$exact }${ignored}"
  POSTURE_LABEL="${POSTURE_LABEL:-<default>}"
}

fail_usage() {
  echo "FAIL: $1" >&2
  echo "Usage: $(basename "${BASH_SOURCE[0]}") (--lib | --test <target>) --selection <filter> [--exact] [--ignored] [--expect-name <exact test path>]..." >&2
  exit 2
}

report_empty_selection() {
  cat >&2 <<EOF
FAIL: selection '$SELECTION' matched 0 compiled tests.
  package : $PACKAGE
  target  : $TARGET_LABEL
  posture : $POSTURE_LABEL
A named CI selection must reach at least one test. The test name, its #[ignore]
posture, or its target membership changed. Enumerate the real names with:
  cargo test -p $PACKAGE $TARGET_LABEL -- --list
EOF
  exit 1
}

report_non_executable_selection() {
  local selected="$1"
  cat >&2 <<EOF
FAIL: selection '$SELECTION' matched $selected test(s), all of them #[ignore]d.
  package : $PACKAGE
  target  : $TARGET_LABEL
  posture : $POSTURE_LABEL
This lane runs in ordinary posture, which executes only non-ignored tests, so
the selection would report success while executing nothing. Listing cannot
prove executability on its own: --list reports ignored tests too.
Either remove #[ignore] from at least one matching test, or move this selection
to a scheduled lane and pass --ignored so the ignored tests are the ones that
run. Enumerate the real postures with:
  cargo test -p $PACKAGE $TARGET_LABEL -- --list
  cargo test -p $PACKAGE $TARGET_LABEL -- --list --ignored
EOF
  exit 1
}

report_roster_drift() {
  local executable="$1" missing="$2" undeclared="$3"
  {
    echo "FAIL: selection '$SELECTION' no longer matches its declared roster."
    echo "  package   : $PACKAGE"
    echo "  target    : $TARGET_LABEL"
    echo "  posture   : $POSTURE_LABEL"
    echo "  declared  : ${#EXPECTED_NAMES[@]} test(s)"
    echo "  executable: $executable test(s)"
    if [[ -n "$missing" ]]; then
      echo "Declared tests no executable test answers to:"
      sed 's/^/  - /' <<<"$missing"
    fi
    if [[ -n "$undeclared" ]]; then
      echo "Executable tests the declaration does not name ($(count_names "$undeclared")):"
      sed 's/^/  + /' <<<"$undeclared"
    fi
    cat <<EOF
This lane's claim depends on the declared tests running together, so a renamed,
deleted, newly #[ignore]d or newly added test has to be reflected in the
declaration rather than silently changing what the lane proves. Enumerate the
real names and postures with:
  cargo test -p $PACKAGE $TARGET_LABEL -- --list
  cargo test -p $PACKAGE $TARGET_LABEL -- --list --ignored
EOF
  } >&2
  exit 1
}

# Emits one bare test path per line for the given harness filter vector.
list_selected_tests() {
  cargo test -p "$PACKAGE" "${CARGO_TARGET_ARGS[@]}" \
    -- --list "$@" | sed -n 's/^\([A-Za-z0-9_:]\+\): test$/\1/p' || true
}

count_names() {
  grep -c . <<<"$1" || true
}

assert_roster_is_exact() {
  local executable_names="$1" executable="$2"
  local missing="" undeclared="" name

  for name in "${EXPECTED_NAMES[@]}"; do
    grep -qxF -- "$name" <<<"$executable_names" || missing+="$name"$'\n'
  done

  while IFS= read -r name; do
    [[ -n "$name" ]] || continue
    printf '%s\n' "${EXPECTED_NAMES[@]}" | grep -qxF -- "$name" \
      || undeclared+="$name"$'\n'
  done <<<"$executable_names"

  if [[ -n "$missing" || -n "$undeclared" ]]; then
    report_roster_drift "$executable" "${missing%$'\n'}" "${undeclared%$'\n'}"
  fi

  echo "[relational-selection] the ${#EXPECTED_NAMES[@]} declared test(s) are exactly the executable set"
}

assert_selection_is_executable() {
  local selected_names ignored_names executable_names="" name
  local selected ignored_selected executable

  # Captured once per posture. The roster check reads these same listings rather
  # than asking cargo again, so it cannot observe a different selection.
  selected_names="$(list_selected_tests "${HARNESS_FILTER_ARGS[@]}")"
  selected="$(count_names "$selected_names")"

  if [[ "$selected" -eq 0 ]]; then
    report_empty_selection
  fi

  if [[ -n "$IGNORED_POSTURE" ]]; then
    # Under --ignored the listing is already exactly the set that executes.
    executable="$selected"
    executable_names="$selected_names"
  else
    ignored_names="$(list_selected_tests "${IGNORED_PROBE_ARGS[@]}")"
    ignored_selected="$(count_names "$ignored_names")"
    executable=$(( selected - ignored_selected ))

    if [[ "$executable" -le 0 ]]; then
      report_non_executable_selection "$selected"
    fi

    while IFS= read -r name; do
      [[ -n "$name" ]] || continue
      grep -qxF -- "$name" <<<"$ignored_names" || executable_names+="$name"$'\n'
    done <<<"$selected_names"
    executable_names="${executable_names%$'\n'}"
  fi

  echo "[relational-selection] '$SELECTION' selects $executable executable test(s) under $TARGET_LABEL"

  if [[ ${#EXPECTED_NAMES[@]} -gt 0 ]]; then
    assert_roster_is_exact "$executable_names" "$executable"
  fi
}

execute_named_selection() {
  echo "[relational-selection] executing '$SELECTION'"
  cargo test -p "$PACKAGE" "${CARGO_TARGET_ARGS[@]}" \
    -- "${HARNESS_FILTER_ARGS[@]}" --nocapture --test-threads=1
}

parse_selection_request "$@"
assert_selection_is_executable
execute_named_selection
