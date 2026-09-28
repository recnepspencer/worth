$ErrorActionPreference = "Stop"

$RootDir = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $RootDir

Write-Host "[relational-perf-baseline] Running relational perf lane and baseline check..."

$RunnerTemp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$PerfSummaryPath = if ($env:RELATIONAL_PERF_SUMMARY_PATH) { $env:RELATIONAL_PERF_SUMMARY_PATH } else { Join-Path $RunnerTemp "relational-perf-summary.jsonl" }
$PerfBaselinePath = if ($env:RELATIONAL_PERF_BASELINE_PATH) { $env:RELATIONAL_PERF_BASELINE_PATH } else { Join-Path $RootDir "plans\engineering\worth_relational_performance_baseline.jsonl" }

$env:RELATIONAL_PERF_SUMMARY_PATH = $PerfSummaryPath
$env:RELATIONAL_PERF_BASELINE_PATH = $PerfBaselinePath

powershell -ExecutionPolicy Bypass -File .\scripts\ci\run_relational_perf_lane.ps1
python .\scripts\ci\check_relational_perf_baseline.py --baseline $PerfBaselinePath --current $PerfSummaryPath

Write-Host "[relational-perf-baseline] PASS"
