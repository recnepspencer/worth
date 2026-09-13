#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

# The seven-configuration test matrix owns conditional reachability. Dead-code
# diagnostics are therefore excluded while every Clippy finding remains fatal.
cargo clippy -p worth-relational -p worth-runtime-bridge -p worth-signal --lib -- -D warnings -A dead-code
cargo clippy -p worth-relational --all-features --test relational_certification -- -D warnings -A dead-code
cargo clippy -p worth-signal --lib --tests --features test-operation-control -- -D warnings -A dead-code
