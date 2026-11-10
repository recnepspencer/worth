# Serial lanes for merge batch 1 (P-C 2dce9c178c, H8 1ce3832b78, P-A e4d3b01c8d) on top of 54b4fa230d.
# One Cargo at a time. Every stage keeps its own header log; a red stage does not stop later stages.
# Selection is the reviewed permitted policy (master2-tier2-permitted.py): never ignored bodies,
# query-scale, the exhaustive Home oracle or the Store scenario suite.
# The user's machine sets CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER (rust-lld 1.94.0), which overrides the
# committed linker. These lanes clear it for this process only, so the committed wrapper
# (scripts/windows_linker.cmd, /pdbpagesize:16384) links every binary.
param([string[]]$Stages = @("check"), [string]$Tag = "batch1")
Set-Location "C:\programming\worth worktees\worktrees\m3-host-home"
Remove-Item Env:\CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER -ErrorAction SilentlyContinue
$e = "target\lane-evidence"
$commands = @{
    "check"      = @("python", "$e\master2_checks.py", "fable-$Tag", "root", "query", "store", "bank")
    "root-t2"    = @("python", "$e\master2-tier2-permitted.py", "--execute", "--only", "root", "--out", "$e\fable-$Tag-tier2-root")
    "query-t2"   = @("python", "$e\master2-tier2-permitted.py", "--execute", "--only", "query", "--out", "$e\fable-$Tag-tier2-query")
    "fixture-t2" = @("python", "$e\master2-tier2-permitted.py", "--execute", "--only", "fixture", "--out", "$e\fable-$Tag-tier2-fixture")
    "store-t2"   = @("python", "$e\master2-tier2-permitted.py", "--execute", "--only", "store", "--out", "$e\fable-$Tag-tier2-store")
    "bank-t2"    = @("python", "$e\master2-tier2-permitted.py", "--execute", "--only", "bank", "--out", "$e\fable-$Tag-tier2-bank")
}
foreach ($stage in $Stages) {
    $command = $commands[$stage]
    $log = "$e\fable-$Tag-$stage.log"
    $free = [math]::Round((Get-PSDrive C).Free / 1GB, 1)
    $clean = if ((git status --porcelain -- crates workspaces tools scripts .cargo | Measure-Object).Count -eq 0) { "true" } else { "false" }
    @("COMMAND: $($command -join ' ')", "HEAD: $(git rev-parse HEAD)", "SOURCE_CLEAN: $clean",
      "LINKER_OVERRIDE_CLEARED: true", "C_FREE_GB: $free",
      "START: $((Get-Date).ToUniversalTime().ToString('o'))") | Out-File -Encoding utf8 $log
    if ($free -lt 25) {
        "REFUSED: disk floor" | Out-File -Encoding utf8 -Append $log
        "STAGE $stage REFUSED disk floor"
        continue
    }
    & $command[0] $command[1..($command.Length - 1)] *>> $log
    $code = $LASTEXITCODE
    @("END: $((Get-Date).ToUniversalTime().ToString('o'))", "EXIT: $code") | Out-File -Encoding utf8 -Append $log
    "STAGE $stage EXIT $code"
}
