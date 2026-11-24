#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

# The seven-configuration test matrix owns conditional reachability. Dead-code
# diagnostics are therefore excluded while every Clippy finding remains fatal.
cargo clippy -p worth-relational -p worth-relational-contracts -p worth-relational-storage -p worth-relational-branch -p worth-relational-indexes -p worth-relational-lineage -p worth-relational-merge -p worth-relational-mutation -p worth-runtime-bridge -p worth-signal --lib -- -D warnings -A dead-code
cargo clippy -p worth-court-relational-certification --test worth_court_relational_certification -- -D warnings -A dead-code
cargo clippy -p worth-signal --lib --tests -- -D warnings -A dead-code
