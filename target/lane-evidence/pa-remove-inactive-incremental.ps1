$laneWorkspace = (Resolve-Path -LiteralPath '.').Path
$laneIncrementalRoot = (Resolve-Path -LiteralPath 'target/debug/incremental').Path
$laneExpectedRoot = Join-Path $laneWorkspace 'target\debug\incremental'
if ($laneIncrementalRoot -ne $laneExpectedRoot) { throw 'Unexpected incremental cache root' }
$laneActiveCompilers = Get-CimInstance Win32_Process -Filter "name = 'rustc.exe' OR name = 'cargo.exe'" | Where-Object { $_.CommandLine -like '*m3-p3-reopen*' }
if ($laneActiveCompilers) { throw 'Lane compiler still active; refuse cache cleanup' }
$laneCacheNames = @('worth_relational-1ggk801mgdyyz','worth_relational-1ib91gwk51ly6')
Write-Output "free_before_bytes=$((Get-PSDrive C).Free)"
foreach ($laneCacheName in $laneCacheNames) {
    $laneCachePath = (Resolve-Path -LiteralPath (Join-Path $laneIncrementalRoot $laneCacheName)).Path
    if (-not $laneCachePath.StartsWith($laneExpectedRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Cache escaped intended worktree root' }
    $laneCacheBytes = (Get-ChildItem -LiteralPath $laneCachePath -Recurse -File | Measure-Object Length -Sum).Sum
    Write-Output "removing_inactive_cache=$laneCachePath bytes=$laneCacheBytes"
    Remove-Item -LiteralPath $laneCachePath -Recurse -Force
}
Write-Output "free_after_bytes=$((Get-PSDrive C).Free)"
