$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$target = if ($env:WORTH_CONSTITUTION_TOOL_TARGET) {
    [System.IO.Path]::GetFullPath($env:WORTH_CONSTITUTION_TOOL_TARGET)
} else {
    Join-Path $root "target/constitution-tools"
}

Push-Location $root
try {
    & cargo build --quiet --locked --offline --manifest-path tools/boundary-check/Cargo.toml --target-dir $target
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    & cargo build --quiet --locked --offline --manifest-path tools/agent-context/Cargo.toml --target-dir $target
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
