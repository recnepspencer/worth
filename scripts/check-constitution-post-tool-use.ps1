$ErrorActionPreference = "Stop"
$HookBudgetSeconds = 15

function Read-PostToolUsePayload {
    $payloadText = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($payloadText)) { return $null }
    try { return $payloadText | ConvertFrom-Json }
    catch { return $null }
}

function Test-EditCapableTool($ToolName) {
    return [string]$ToolName -match '^(Write|Edit|MultiEdit|apply_patch|Bash)$'
}

function Get-NestedEditedPaths($Value) {
    if ($null -eq $Value) { return }
    if ($Value -is [System.Management.Automation.PSCustomObject]) {
        foreach ($property in $Value.PSObject.Properties) {
            if ($property.Name -in @("file_path", "path")) {
                [string]$property.Value
            } else {
                Get-NestedEditedPaths $property.Value
            }
        }
    } elseif ($Value -is [System.Collections.IEnumerable] -and $Value -isnot [string]) {
        foreach ($item in $Value) { Get-NestedEditedPaths $item }
    }
}

function Get-ApplyPatchTargets($PatchText) {
    $matches = [regex]::Matches(
        [string]$PatchText,
        '(?m)^\*\*\* (?:Add|Update|Delete) File: (.+)$|^\*\*\* Move to: (.+)$'
    )
    foreach ($target in $matches) {
        if ($target.Groups[1].Success) { $target.Groups[1].Value.Trim() }
        else { $target.Groups[2].Value.Trim() }
    }
}

function Get-EditedTargets($Payload) {
    $targets = @(Get-NestedEditedPaths $Payload.tool_input)
    if ([string]$Payload.tool_name -eq "apply_patch") {
        $targets += @(Get-ApplyPatchTargets $Payload.tool_input.patch)
    }
    return $targets
}

function Test-GovernedSurfacePath($CandidatePath) {
    $path = [string]$CandidatePath
    $path = $path.Replace('\', '/') -replace '^\./', ''
    return (
        $path -match '(^|/)workspaces/worth-contracts/' -or
        $path -match '(^|/)workspaces/worth-query/' -or
        $path -eq 'Cargo.toml' -or
        $path -match '(^|/)crates/worth-proof/' -or
        $path -match '(^|/)tools/' -or
        $path -match '(^|/)scripts/(?:check|prepare)-constitution[^/]*\.ps1$' -or
        $path -match '(^|/)\.github/workflows/ci\.yml$' -or
        $path -match '(^|/)\.claude/settings\.json$'
    )
}

function Test-GovernedEdit($ToolName, $EditedTargets) {
    if ($ToolName -eq "Bash") { return $true }
    if ($ToolName -eq "apply_patch" -and $EditedTargets.Count -eq 0) { return $true }
    foreach ($target in $EditedTargets) {
        if (Test-GovernedSurfacePath $target) { return $true }
    }
    return $false
}

function Invoke-BudgetedConstitutionCheck {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $env:WORTH_CONSTITUTION_PREBUILT = "1"
    & (Join-Path $PSScriptRoot "check-constitution.ps1") --format json
    $constitutionExit = $LASTEXITCODE
    $stopwatch.Stop()
    if ($stopwatch.Elapsed.TotalSeconds -gt $HookBudgetSeconds) {
        [Console]::Error.WriteLine("constitution PostToolUse hook exceeded ${HookBudgetSeconds}s edit-loop budget: $([Math]::Round($stopwatch.Elapsed.TotalSeconds, 2))s")
        exit 1
    }
    exit $constitutionExit
}

$payload = Read-PostToolUsePayload
if ($null -eq $payload) { exit 0 }
$toolName = [string]$payload.tool_name
if (-not (Test-EditCapableTool $toolName)) { exit 0 }
$editedTargets = @(Get-EditedTargets $payload)
if (-not (Test-GovernedEdit $toolName $editedTargets)) { exit 0 }
Invoke-BudgetedConstitutionCheck
