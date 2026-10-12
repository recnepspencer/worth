#!/usr/bin/env bash

set -euo pipefail

echo "[signal-contracts] adversarial contract matrix lane"

cargo test -p worth-signal --lib phase3_semantics -- --nocapture
cargo test -p worth-signal --lib phase4_planner -- --nocapture
cargo test -p worth-signal --lib adversarial_parallel -- --nocapture
cargo test -p worth-signal --lib observability -- --nocapture
cargo test -p worth-signal --lib diagnostics -- --nocapture
cargo test -p worth-signal --lib logic::events::tests -- --nocapture
cargo test -p worth-signal --lib logic::transaction::tests -- --nocapture

echo "[signal-contracts] adversarial contract matrix lane is green"
