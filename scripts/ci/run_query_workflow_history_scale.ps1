param(
    [ValidateSet("Scale", "Cold", "Diagnostic")]
    [string]$Lane = "Scale"
)

$ErrorActionPreference = "Stop"
$taskRoot = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
$queryManifest = Join-Path $taskRoot "workspaces/worth-query/Cargo.toml"
$testName = switch ($Lane) {
    "Scale" { "workflow_history_and_instance_scale" }
    "Cold" { "workflow_history_locality_smoke" }
    "Diagnostic" { "workflow_history_native_cost_diagnostic" }
}
$qualifiedTest = "workflow_history_scale::$testName"

# Fixed development qualification envelope, excluding compilation. Sampled
# process-private bytes are a watchdog, not production allocation admission.
# A budget stop is a failed lane; never interpret it as successful exhaustion.
# These exact courts run in process; this runner does not supervise descendants.
$maximumPrivateBytes = 4GB
$maximumSeconds = 180
$pollMilliseconds = 100

$artifacts = @(& cargo test --manifest-path $queryManifest -p worth-query-certification `
    --test application_graph --no-run --message-format=json)
if ($LASTEXITCODE -ne 0) { throw "workflow history test compilation failed" }
$executables = @($artifacts | ForEach-Object {
    $message = $_ | ConvertFrom-Json
    if ($message.reason -eq "compiler-artifact" -and
        $message.target.name -eq "application_graph" -and $message.executable) {
        $message.executable
    }
} | Select-Object -Unique)
if ($executables.Count -ne 1) { throw "expected one application_graph test executable" }
$testExecutable = (Resolve-Path -LiteralPath $executables[0]).Path
$listing = @(& $testExecutable $qualifiedTest --ignored --exact --list)
if ($LASTEXITCODE -ne 0 -or $listing -notcontains "${qualifiedTest}: test") {
    throw "the exact scheduled workflow test was not discovered"
}

Write-Output "workflow history lane=$Lane profile=test max_private_bytes=$maximumPrivateBytes max_seconds=$maximumSeconds poll_ms=$pollMilliseconds"
Write-Output "os=$([Environment]::OSVersion) logical_processors=$([Environment]::ProcessorCount) process_64bit=$([Environment]::Is64BitProcess)"
& rustc --version

$startInfo = New-Object System.Diagnostics.ProcessStartInfo
$startInfo.FileName = $testExecutable
$startInfo.WorkingDirectory = $taskRoot
$startInfo.Arguments = "$qualifiedTest --ignored --exact --nocapture --test-threads=1"
$startInfo.UseShellExecute = $false
$startInfo.CreateNoWindow = $true
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
$testProcess = New-Object System.Diagnostics.Process
$testProcess.StartInfo = $startInfo
$peakPrivateBytes = 0L
$budgetFailure = $null
$started = $false
$timer = [Diagnostics.Stopwatch]::StartNew()
try {
    if (-not $testProcess.Start()) { throw "workflow test process did not start" }
    $started = $true
    $stdout = $testProcess.StandardOutput.ReadToEndAsync()
    $stderr = $testProcess.StandardError.ReadToEndAsync()
    while (-not $testProcess.WaitForExit($pollMilliseconds)) {
        $testProcess.Refresh()
        if ($testProcess.HasExited) { break }
        $peakPrivateBytes = [Math]::Max($peakPrivateBytes, $testProcess.PrivateMemorySize64)
        if ($peakPrivateBytes -gt $maximumPrivateBytes) {
            $budgetFailure = "process-private memory budget exceeded"
        } elseif ($timer.Elapsed.TotalSeconds -gt $maximumSeconds) {
            $budgetFailure = "wall-time budget exceeded"
        }
        if ($budgetFailure) {
            # This Process object owns only the exact test executable launched
            # above. It never targets Cargo, another test, or a name-matched PID.
            if (-not $testProcess.HasExited) { $testProcess.Kill() }
            $testProcess.WaitForExit()
            break
        }
    }
    Write-Output $stdout.GetAwaiter().GetResult()
    Write-Output $stderr.GetAwaiter().GetResult()
    Write-Output "elapsed_seconds=$($timer.Elapsed.TotalSeconds) sampled_peak_private_bytes=$peakPrivateBytes exit_code=$($testProcess.ExitCode)"
    if ($budgetFailure) { throw "workflow history lane failed: $budgetFailure" }
    if ($testProcess.ExitCode -ne 0) { throw "workflow history test failed" }
} finally {
    # Also release the owned process if inspection or output handling fails.
    if ($started -and -not $testProcess.HasExited) {
        $testProcess.Kill()
        $testProcess.WaitForExit()
    }
    $testProcess.Dispose()
}
