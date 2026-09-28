$ErrorActionPreference = "Stop"

$RootDir = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $RootDir

Write-Host "[relational-perf] Running worth-relational performance lane..."

$RunnerTemp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$PerfLogPath = if ($env:RELATIONAL_PERF_LOG_PATH) { $env:RELATIONAL_PERF_LOG_PATH } else { Join-Path $RunnerTemp "relational-perf.log" }
$PerfReportPath = if ($env:RELATIONAL_PERF_REPORT_PATH) { $env:RELATIONAL_PERF_REPORT_PATH } else { Join-Path $RunnerTemp "relational-perf-report.jsonl" }
$PerfSummaryPath = if ($env:RELATIONAL_PERF_SUMMARY_PATH) { $env:RELATIONAL_PERF_SUMMARY_PATH } else { Join-Path $RunnerTemp "relational-perf-summary.jsonl" }
$PerfMarkdownPath = if ($env:RELATIONAL_PERF_MARKDOWN_PATH) { $env:RELATIONAL_PERF_MARKDOWN_PATH } else { Join-Path $RunnerTemp "relational-perf-summary.md" }
$PerfBaselinePath = if ($env:RELATIONAL_PERF_BASELINE_PATH) { $env:RELATIONAL_PERF_BASELINE_PATH } else { Join-Path $RootDir "plans\engineering\worth_relational_performance_baseline.jsonl" }
$PerfComparePath = $env:RELATIONAL_PERF_COMPARE_PATH
$PerfArchiveDir = $env:RELATIONAL_PERF_ARCHIVE_DIR
$PerfSamples = if ($env:WORTH_RELATIONAL_PERF_SAMPLES) { $env:WORTH_RELATIONAL_PERF_SAMPLES } else { "3" }

if (-not $PerfComparePath -and (Test-Path $PerfBaselinePath)) {
    $PerfComparePath = $PerfBaselinePath
}

if (-not $PerfComparePath -and $PerfArchiveDir) {
    $latestSummaryPath = Join-Path $PerfArchiveDir "latest-summary.jsonl"
    if (Test-Path $latestSummaryPath) {
        $PerfComparePath = $latestSummaryPath
    }
}

$env:WORTH_RELATIONAL_PERF_SAMPLES = $PerfSamples

$cargoCommand = "cargo test -p worth-relational performance_profiles -- --ignored --nocapture --test-threads=1 2>&1"
$testOutput = cmd /c $cargoCommand
$testOutput | Tee-Object -FilePath $PerfLogPath

if ($LASTEXITCODE -ne 0) {
    throw "worth-relational performance lane failed with exit code $LASTEXITCODE"
}

$jsonLines = $testOutput | Where-Object { $_ -match '^\{' }
$jsonLines | Set-Content -Path $PerfReportPath
$jsonLines | Where-Object { $_ -match '"samples":' } | Set-Content -Path $PerfSummaryPath

$summaryArgs = @("scripts/ci/summarize_relational_perf.py", "--input", $PerfSummaryPath, "--output", $PerfMarkdownPath)
if ($PerfComparePath) {
    $summaryArgs += @("--compare", $PerfComparePath)
}
python @summaryArgs

if ($PerfArchiveDir) {
    $runStamp = [DateTime]::UtcNow.ToString("yyyyMMddTHHmmssZ")
    $runDir = Join-Path $PerfArchiveDir $runStamp
    New-Item -ItemType Directory -Force -Path $runDir | Out-Null
    Copy-Item -Path $PerfLogPath -Destination (Join-Path $runDir "perf.log") -Force
    Copy-Item -Path $PerfReportPath -Destination (Join-Path $runDir "perf-report.jsonl") -Force
    Copy-Item -Path $PerfSummaryPath -Destination (Join-Path $runDir "perf-summary.jsonl") -Force
    Copy-Item -Path $PerfMarkdownPath -Destination (Join-Path $runDir "perf-summary.md") -Force
    Copy-Item -Path $PerfLogPath -Destination (Join-Path $PerfArchiveDir "latest.log") -Force
    Copy-Item -Path $PerfReportPath -Destination (Join-Path $PerfArchiveDir "latest-report.jsonl") -Force
    Copy-Item -Path $PerfSummaryPath -Destination (Join-Path $PerfArchiveDir "latest-summary.jsonl") -Force
    Copy-Item -Path $PerfMarkdownPath -Destination (Join-Path $PerfArchiveDir "latest-summary.md") -Force
    Write-Host "[relational-perf] Archived artifacts to $runDir"
}

Write-Host "[relational-perf] Wrote full log to $PerfLogPath"
Write-Host "[relational-perf] Wrote JSON samples to $PerfReportPath"
Write-Host "[relational-perf] Wrote JSON summaries to $PerfSummaryPath"
Write-Host "[relational-perf] Wrote markdown summary to $PerfMarkdownPath"
Write-Host "[relational-perf] PASS"
