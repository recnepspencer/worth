#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[signal-core-profiles] checking standard profile"
cargo check -p worth-signal

echo "[signal-core-profiles] checking compact profile"
cargo check -p worth-signal --no-default-features --features profile-compact
cargo test -p worth-signal --lib --no-default-features --features profile-compact harness_adapter -- --nocapture

echo "[signal-core-profiles] checking extended profile"
cargo check -p worth-signal --no-default-features --features profile-extended
cargo test -p worth-signal --lib --no-default-features --features profile-extended harness_adapter -- --nocapture

echo "[signal-core-profiles] PASS"
