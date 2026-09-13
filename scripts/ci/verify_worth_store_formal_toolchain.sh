#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
crate_root="$repo_root/workspaces/worth-store/crates/worth-store-formal-models"
toolchain="$crate_root/formal-toolchain.toml"

read_value() {
  local name="$1"
  sed -n "s/^${name}[[:space:]]*=[[:space:]]*\"\([^\"]*\)\"$/\1/p" "$toolchain" | head -n 1
}

version="$(read_value version)"
download_url="$(read_value download_url)"
expected_sha256="$(read_value sha256)"

tool_cache="${WORTH_STORE_FORMAL_TOOL_CACHE:-$repo_root/workspaces/worth-store/target/formal-tools}"
mkdir -p "$tool_cache"
jar_path="$tool_cache/tla2tools-$version.jar"
state_root="$(mktemp -d "$tool_cache/states.XXXXXX")"
trap 'rm -rf -- "$state_root"' EXIT

if [[ ! -f "$jar_path" ]]; then
  curl --fail --location --silent --show-error --output "$jar_path" "$download_url"
fi

actual_sha256="$(sha256sum "$jar_path" | awk '{print $1}')"
if [[ "$actual_sha256" != "$expected_sha256" ]]; then
  echo "TLC digest mismatch: expected $expected_sha256, got $actual_sha256" >&2
  exit 1
fi

cargo run --quiet \
  --manifest-path "$crate_root/Cargo.toml" \
  --bin worth_store_protocol_check -- \
  "$(command -v java)" "$jar_path" "$state_root"

echo "verified direct Worth Store protocol checks with TLC $version ($actual_sha256)"
