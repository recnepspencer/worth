#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${WORTH_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$ROOT_DIR"

CAP=400
ALLOWLIST="scripts/ci/workspace_rust_line_cap_allowlist.txt"
SCOPE="${1:-workspace}"

case "$SCOPE" in
  workspace)
    PATHS=(
      'crates/**/*.rs'
      'workspaces/worth-query-bank-world/crates/**/*.rs'
      'workspaces/worth-query/crates/**/*.rs'
      'workspaces/worth-ui/crates/**/*.rs'
      'workspaces/worth-ui/apps/**/*.rs'
      'workspaces/worth-store/crates/**/*.rs'
      'workspaces/worth-store/tools/**/*.rs'
    )
    ;;
  worth-ui)
    PATHS=('workspaces/worth-ui/crates/**/*.rs' 'workspaces/worth-ui/apps/**/*.rs')
    ;;
  worth-ui-apps)
    PATHS=('workspaces/worth-ui/apps/**/*.rs')
    ;;
  dirty)
    PATHS=()
    ;;
  *)
    echo "usage: $0 [workspace|worth-ui|worth-ui-apps|dirty]" >&2
    exit 2
    ;;
esac

echo "[workspace-rust-line-caps] enforcing ${CAP}-line cap for ${SCOPE} Rust files"

violations=0
while read -r line_count file; do
  [[ -n "$file" ]] || continue
  [[ "$file" != "total" ]] || continue

  if (( line_count > CAP )); then
    if [[ -f "$ALLOWLIST" ]] && grep -Fxq "$file" "$ALLOWLIST"; then
      echo "[workspace-rust-line-caps] allowlisted: ${file} (${line_count} lines)"
    else
      echo "FAIL: ${file} is ${line_count} lines (cap ${CAP})"
      violations=1
    fi
  fi
done < <(
  if [[ "$SCOPE" == dirty ]]; then
    {
      git diff --name-only --diff-filter=ACMR -z -- '*.rs'
      git diff --cached --name-only --diff-filter=ACMR -z -- '*.rs'
      git ls-files --others --exclude-standard -z -- '*.rs'
    }
  else
    git ls-files -z -- "${PATHS[@]}"
  fi \
    | while IFS= read -r -d '' file; do
        [[ -f "$file" ]] && printf '%s\0' "$file"
      done \
    | sort -zu \
    | xargs -0 -r -n 128 wc -l
)

if (( violations != 0 )); then
  exit 1
fi

echo "[workspace-rust-line-caps] PASS"
