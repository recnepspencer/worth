# Second lanes pass: free the regenerable Query incremental cache (own worktree only), then run the
# lanes that did not finish in the first pass: Query Tier 2, Store and Bank selected failures.
Set-Location "C:\programming\worth worktees\worktrees\m3-host-home"
$e = "target\lane-evidence"
$cache = "workspaces\worth-query\target\debug\incremental"
$before = [math]::Round((Get-PSDrive C).Free / 1GB, 1)
if (Test-Path $cache) { Remove-Item -Recurse -Force -Confirm:$false $cache }
$after = [math]::Round((Get-PSDrive C).Free / 1GB, 1)
@("$((Get-Date).ToUniversalTime().ToString('o')) deleted $cache (regenerable): C free $before GB -> $after GB") |
    Out-File -Encoding utf8 -Append "$e\fable-repair-disk.log"
& "$e\fable_repair_lanes.ps1" -Stages @("query-tier2-b", "store-focused", "bank-focused")
