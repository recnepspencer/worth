#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

cargo clippy -p worth-relational -p worth-relational-contracts -p worth-relational-storage -p worth-relational-branch -p worth-relational-indexes -p worth-relational-lineage -p worth-relational-mutation -p worth-relational-durability -p worth-relational-publication -p worth-runtime-bridge -p worth-signal --lib -- -D warnings
cargo clippy -p worth-court-relational-certification --test worth_court_relational_certification -- -D warnings
cargo clippy -p worth-signal --lib --tests -- -D warnings
