# Serial post-repair lanes for the master-merge repair batch (Fable integration takeover).
# One Cargo at a time. Every stage keeps its own header log; a red stage does not stop later stages.
# Never selects ignored bodies, query-scale, the exhaustive oracle or the scenario suite: selection is
# the reviewed permitted policy (master2-tier2-permitted.py) and prior-failure names only.
param([string[]]$Stages = @("root-focused", "query-tier2", "bank-focused", "store-focused"))
Set-Location "C:\programming\worth worktees\worktrees\m3-host-home"
$e = "target\lane-evidence"
$commands = @{
    "root-focused"  = @("python", "$e\fable_focused_runtime.py", "root", "--out", "$e\fable-repair-root-focused")
    "query-tier2"   = @("python", "$e\master2-tier2-permitted.py", "--execute", "--only", "query", "--out", "$e\fable-repair-tier2")
    # Second attempt: the first died in its build with "no space on device" (stale 143 GB incremental cache).
    "query-tier2-b" = @("python", "$e\master2-tier2-permitted.py", "--execute", "--only", "query", "--out", "$e\fable-repair-tier2-b")
    "bank-focused"  = @("python", "$e\fable_focused_runtime.py", "bank", "--out", "$e\fable-repair-bank-focused")
    "store-focused" = @("python", "$e\fable_focused_runtime.py", "store", "--out", "$e\fable-repair-store-focused")
}
foreach ($stage in $Stages) {
    $command = $commands[$stage]
    $log = "$e\fable-repair-$stage.log"
    $free = [math]::Round((Get-PSDrive C).Free / 1GB, 1)
    @("COMMAND: $($command -join ' ')", "HEAD: $(git rev-parse HEAD)",
      "CLEAN: false (repair batch applied, uncommitted)", "C_FREE_GB: $free",
      "START: $((Get-Date).ToUniversalTime().ToString('o'))") | Out-File -Encoding utf8 $log
    if ($free -lt 25) {
        "REFUSED: disk floor" | Out-File -Encoding utf8 -Append $log
        continue
    }
    & $command[0] $command[1..($command.Length - 1)] *>> $log
    $code = $LASTEXITCODE
    @("END: $((Get-Date).ToUniversalTime().ToString('o'))", "EXIT: $code") | Out-File -Encoding utf8 -Append $log
    "STAGE $stage EXIT $code"
}
