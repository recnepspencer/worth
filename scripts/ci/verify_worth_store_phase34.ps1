$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "../..")

# Rust courts are selected once by the complete prebuilt Store landing lane.
& python (Join-Path $root "scripts/ci/check_worth_store_phase34_dependencies.py")
if ($LASTEXITCODE -ne 0) {
    throw "Phase 34 dependency direction failed with exit code $LASTEXITCODE"
}
