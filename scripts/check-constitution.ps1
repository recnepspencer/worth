param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)

$ErrorActionPreference = "Stop"

function Read-ConstitutionInvocation([string[]]$RawArguments) {
    if ($RawArguments.Count -eq 0) { return "human" }
    if ($RawArguments.Count -ne 2 -or $RawArguments[0] -ne "--format" -or $RawArguments[1] -notin @("human", "json")) {
        [Console]::Error.WriteLine("usage: check-constitution.ps1 [--format human|json]")
        exit 2
    }
    return $RawArguments[1]
}

function Resolve-ConstitutionToolCommands {
    $toolRoot = Split-Path -Parent $PSScriptRoot
    $checkoutRoot = if ($env:WORTH_CONSTITUTION_ROOT) { [System.IO.Path]::GetFullPath($env:WORTH_CONSTITUTION_ROOT) } else { $toolRoot }
    $prebuilt = $env:WORTH_CONSTITUTION_PREBUILT -eq "1"
    $targetRoot = if ($env:WORTH_CONSTITUTION_TOOL_TARGET) { [System.IO.Path]::GetFullPath($env:WORTH_CONSTITUTION_TOOL_TARGET) } else { Join-Path $toolRoot "target/constitution-tools" }
    $suffix = if ($IsWindows -or $env:OS -eq "Windows_NT") { ".exe" } else { "" }
    $binaryRoot = Join-Path $targetRoot "debug"
    return [pscustomobject]@{
        ToolRoot = $toolRoot; CheckoutRoot = $checkoutRoot; Prebuilt = $prebuilt
        Boundary = if ($prebuilt) { Join-Path $binaryRoot "boundary-check$suffix" } else { "cargo" }
        Context = if ($prebuilt) { Join-Path $binaryRoot "agent-context$suffix" } else { "cargo" }
    }
}

function New-ConstitutionDiagnostic([string]$Code, [string]$Subject, [string]$Message, [string]$LegalHome) {
    return [ordered]@{ code = $Code; subject = $Subject; message = $Message; legal_home = $LegalHome }
}

function Write-ConstitutionFailure([string]$Format, [object]$Diagnostic, [string]$HumanMessage) {
    if ($Format -eq "json") {
        $json = [ordered]@{ schema = "worth.constitution.v1"; ok = $false; diagnostics = @($Diagnostic) } | ConvertTo-Json -Depth 8
        [Console]::Out.WriteLine($json)
    } else { [Console]::Error.WriteLine($HumanMessage) }
}

function Test-PreparedToolReadiness([object]$Commands, [string]$Format) {
    if (-not $Commands.Prebuilt) { return $true }
    if (-not (Test-Path -LiteralPath $Commands.Boundary) -or -not (Test-Path -LiteralPath $Commands.Context)) {
        $diagnostic = New-ConstitutionDiagnostic "CONSTITUTION_PREBUILT_TOOLS_MISSING" "target/constitution-tools" "edit-time constitution executables are not prepared" "scripts/prepare-constitution-hook.ps1; run the session bootstrap before editing governed surfaces"
        Write-ConstitutionFailure $Format $diagnostic "constitution tools are not prepared; run scripts/prepare-constitution-hook.ps1"
        return $false
    }
    $boundaryInputs = @(Get-Item (Join-Path $Commands.ToolRoot "tools/boundary-check/Cargo.toml"); Get-ChildItem (Join-Path $Commands.ToolRoot "tools/boundary-check/src") -Recurse -File)
    $contextInputs = @(Get-Item (Join-Path $Commands.ToolRoot "tools/agent-context/Cargo.toml"); Get-ChildItem (Join-Path $Commands.ToolRoot "tools/agent-context/src") -Recurse -File)
    $stale = ($boundaryInputs | Where-Object LastWriteTimeUtc -gt (Get-Item $Commands.Boundary).LastWriteTimeUtc) -or ($contextInputs | Where-Object LastWriteTimeUtc -gt (Get-Item $Commands.Context).LastWriteTimeUtc)
    if ($stale) {
        $diagnostic = New-ConstitutionDiagnostic "CONSTITUTION_PREBUILT_TOOLS_STALE" "target/constitution-tools" "edit-time constitution executables are older than their source inputs" "scripts/prepare-constitution-hook.ps1; rebuild the edit-time executables after changing constitutional tool sources"
        Write-ConstitutionFailure $Format $diagnostic "constitution tools are stale; run scripts/prepare-constitution-hook.ps1"
        return $false
    }
    return $true
}

function Invoke-ConstitutionChild([scriptblock]$Invocation) {
    $ErrorActionPreference = "Continue"
    $output = & $Invocation 2>&1
    $exitCode = $LASTEXITCODE
    $ErrorActionPreference = "Stop"
    return [pscustomobject]@{ Output = @($output); ExitCode = $exitCode }
}

function Invoke-BoundaryCheck([object]$Commands, [string]$Format) {
    if ($Commands.Prebuilt) {
        return Invoke-ConstitutionChild { & $Commands.Boundary --root . --config tools/boundary-check/config/road1.toml --format $Format }
    }
    return Invoke-ConstitutionChild { & $Commands.Boundary run --quiet --manifest-path (Join-Path $Commands.ToolRoot "tools/boundary-check/Cargo.toml") -- --root . --config tools/boundary-check/config/road1.toml --format $Format }
}

function Invoke-AgentContextCheck([object]$Commands) {
    if ($Commands.Prebuilt) {
        return Invoke-ConstitutionChild { & $Commands.Context check --root . --config tools/boundary-check/config/road1.toml }
    }
    return Invoke-ConstitutionChild { & $Commands.Context run --quiet --manifest-path (Join-Path $Commands.ToolRoot "tools/agent-context/Cargo.toml") -- check --root . --config tools/boundary-check/config/road1.toml }
}

function Project-BoundaryDiagnostics([object]$Execution, [string]$Format) {
    if ($Execution.ExitCode -eq 0) { return @() }
    if ($Format -ne "json") { $Execution.Output | ForEach-Object { [Console]::Error.WriteLine($_) }; return @() }
    try { return @($Execution.Output -join "`n" | ConvertFrom-Json) }
    catch { return @(New-ConstitutionDiagnostic "CONSTITUTION_BOUNDARY_CHECK" "tools/boundary-check" ($Execution.Output -join "`n") "tools/boundary-check/config/road1.toml") }
}

function Project-AgentContextDiagnostics([object]$Execution, [string]$Format) {
    if ($Execution.ExitCode -eq 0) { return @() }
    if ($Format -ne "json") { $Execution.Output | ForEach-Object { [Console]::Error.WriteLine($_) }; return @() }
    return @(New-ConstitutionDiagnostic "CONSTITUTION_AGENT_CONTEXT" "AGENT_CONTEXT.md" ($Execution.Output -join "`n") "tools/boundary-check/config/road1.toml; regenerate the governed AGENT_CONTEXT.md files with tools/agent-context")
}

function Write-ConstitutionResult([string]$Format, [object]$Boundary, [object]$Context, [object[]]$Diagnostics) {
    $ok = $Boundary.ExitCode -eq 0 -and $Context.ExitCode -eq 0
    if ($Format -eq "json") {
        [ordered]@{ schema = "worth.constitution.v1"; ok = $ok; diagnostics = @($Diagnostics) } | ConvertTo-Json -Depth 8
    } elseif ($ok) { Write-Output "constitution: boundary-check and agent-context are valid" }
    if (-not $ok) { exit 1 }
}

$format = Read-ConstitutionInvocation $Arguments
$commands = Resolve-ConstitutionToolCommands
if (-not (Test-PreparedToolReadiness $commands $format)) { exit 1 }

Push-Location $commands.CheckoutRoot
try {
    $boundaryExecution = Invoke-BoundaryCheck $commands $format
    $contextExecution = Invoke-AgentContextCheck $commands
} finally { Pop-Location }

$diagnostics = @()
$diagnostics += @(Project-BoundaryDiagnostics $boundaryExecution $format)
$diagnostics += @(Project-AgentContextDiagnostics $contextExecution $format)
Write-ConstitutionResult $format $boundaryExecution $contextExecution $diagnostics
