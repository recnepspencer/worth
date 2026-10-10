#!/usr/bin/env bash

set -euo pipefail

echo "[signal-resource-bounds] stale-state / boundedness lane"

cargo test -p worth-signal --lib adversarial_edges -- --nocapture
cargo test -p worth-signal --lib adversarial_diagnostics -- --nocapture
cargo test -p worth-signal --lib lifecycle -- --nocapture
cargo test -p worth-signal --lib transaction_stress -- --nocapture
cargo test -p worth-signal --lib phase5_state -- --nocapture

echo "[signal-resource-bounds] stale-state / boundedness lane is green"
