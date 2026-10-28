param(
    [switch]$Workspace
)

$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "../..")
$store = Join-Path $root "workspaces/worth-store"

function Invoke-Checked {
    param([string]$Label, [string[]]$Arguments)

    Write-Host "==> $Label"
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

Push-Location $store
try {
    & python (Join-Path $root "scripts/ci/check_worth_store_phase34_dependencies.py")
    if ($LASTEXITCODE -ne 0) {
        throw "Phase 34 dependency direction failed with exit code $LASTEXITCODE"
    }

    Invoke-Checked "layout indexes check" @("check", "-p", "worth-store-layout-indexes")
    Invoke-Checked "physical isolation check" @(
        "check", "-p", "worth-store-physical-isolation", "--features", "certification-authority"
    )
    Invoke-Checked "layout indexes unit and residue tests" @(
        "test", "-p", "worth-store-layout-indexes", "--lib"
    )
    Invoke-Checked "layout authority compile-fail suites" @(
        "test", "-p", "worth-store-layout-indexes", "--test", "layout_compile_fail"
    )
    Invoke-Checked "physical isolation placement scenarios" @(
        "test", "-p", "worth-store-physical-isolation", "--test", "layout_scenarios"
    )
    Invoke-Checked "durable LSM membership authority" @(
        "test", "-p", "worth-store-lsm-authority"
    )
    Invoke-Checked "layout access planning scenarios" @(
        "test", "-p", "worth-store-layout-indexes", "--test", "layout_scenarios"
    )
    Invoke-Checked "layout corruption, rebuild and bootstrap courtroom" @(
        "test", "-p", "worth-store-certification", "--test", "layout_access"
    )
    Invoke-Checked "test support compilation" @(
        "test", "-p", "worth-store-test-support", "--no-run"
    )
    Invoke-Checked "physical format compilation" @(
        "test", "-p", "worth-store-physical-format", "--no-run"
    )

    if ($Workspace) {
        Invoke-Checked "Worth Store workspace check" @("check", "--workspace")
    }
}
finally {
    Pop-Location
}
