#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

CAP=400
ALLOWLIST="scripts/ci/relational_line_cap_allowlist.txt"

echo "[relational-line-caps] enforcing ${CAP}-line source file cap"

violations=0
while IFS= read -r file; do
  line_count="$(wc -l < "$file" | tr -d ' ')"
  if (( line_count > CAP )); then
    if [[ -f "$ALLOWLIST" ]] && grep -Fxq "$file" "$ALLOWLIST"; then
      echo "[relational-line-caps] allowlisted: ${file} (${line_count} lines)"
    else
      echo "FAIL: ${file} is ${line_count} lines (cap ${CAP})"
      violations=1
    fi
  fi
done < <(find \
  crates/worth-relational/src \
  crates/worth-relational-contracts/src \
  crates/worth-relational-storage/src \
  crates/worth-relational-branch/src \
  crates/worth-relational-indexes/src \
  crates/worth-relational-lineage/src \
  crates/worth-relational-merge/src \
  crates/worth-relational-mutation/src \
  crates/worth-relational-durability/src \
  -type f -name '*.rs' | sort)

if (( violations != 0 )); then
  exit 1
fi

echo "[relational-line-caps] PASS"
