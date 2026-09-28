$ErrorActionPreference = "Stop"

$excludedPaths = @(
    "plans/platform-migrations/store-platform-rename-inventory.csv",
    "scripts/migrations/inventory_store_platform_rename.ps1",
    "scripts/migrations/apply_store_platform_rename.ps1",
    "scripts/migrations/normalize_store_brand_references.ps1"
)
$replacementPairs = @(
    @("FORGE-STORE", "WORTH-STORE"),
    @("Forge-Store", "Worth-Store"),
    @("forge-store", "worth-store"),
    @("FORGE_STORE", "WORTH_STORE"),
    @("Forge_Store", "Worth_Store"),
    @("forge_store", "worth_store"),
    @("FORGESTORE", "WORTHSTORE"),
    @("ForgeStore", "WorthStore"),
    @("forgestore", "worthstore"),
    @("FORGE STORE", "WORTH STORE"),
    @("Forge Store", "Worth Store"),
    @("forge store", "worth store"),
    @("FORGE-FOUNDATIONAL", "WORTH-FOUNDATIONAL"),
    @("Forge-Foundational", "Worth-Foundational"),
    @("forge-foundational", "worth-foundational"),
    @("FORGE_FOUNDATIONAL", "WORTH_FOUNDATIONAL"),
    @("Forge_Foundational", "Worth_Foundational"),
    @("forge_foundational", "worth_foundational"),
    @("FORGE FOUNDATIONAL", "WORTH FOUNDATIONAL"),
    @("Forge Foundational", "Worth Foundational"),
    @("forge foundational", "worth foundational"),
    @("FORGE-PROOF", "WORTH-PROOF"),
    @("Forge-Proof", "Worth-Proof"),
    @("forge-proof", "worth-proof"),
    @("FORGE_PROOF", "WORTH_PROOF"),
    @("Forge_Proof", "Worth_Proof"),
    @("forge_proof", "worth_proof"),
    @("FORGE PROOF", "WORTH PROOF"),
    @("Forge Proof", "Worth Proof"),
    @("forge proof", "worth proof"),
    @("FORGE-QUERY", "WORTH-QUERY"),
    @("Forge-Query", "Worth-Query"),
    @("forge-query", "worth-query"),
    @("FORGE_QUERY", "WORTH_QUERY"),
    @("Forge_Query", "Worth_Query"),
    @("forge_query", "worth_query"),
    @("FORGEQUERY", "WORTHQUERY"),
    @("ForgeQuery", "WorthQuery"),
    @("forgequery", "worthquery"),
    @("FORGE QUERY", "WORTH QUERY"),
    @("Forge Query", "Worth Query"),
    @("forge query", "worth query"),
    @("FORGE-RELATIONAL", "WORTH-RELATIONAL"),
    @("Forge-Relational", "Worth-Relational"),
    @("forge-relational", "worth-relational"),
    @("FORGE_RELATIONAL", "WORTH_RELATIONAL"),
    @("Forge_Relational", "Worth_Relational"),
    @("forge_relational", "worth_relational"),
    @("FORGE RELATIONAL", "WORTH RELATIONAL"),
    @("Forge Relational", "Worth Relational"),
    @("forge relational", "worth relational"),
    @("FORGE-UI", "WORTH-UI"),
    @("Forge-UI", "Worth-UI"),
    @("forge-ui", "worth-ui"),
    @("FORGE_UI", "WORTH_UI"),
    @("Forge_UI", "Worth_UI"),
    @("forge_ui", "worth_ui"),
    @("FORGE UI", "WORTH UI"),
    @("Forge UI", "Worth UI"),
    @("forge ui", "worth ui"),
    @("FORGE-AUTOMATION-RUNNER", "WORTH-AUTOMATION-RUNNER"),
    @("Forge-Automation-Runner", "Worth-Automation-Runner"),
    @("forge-automation-runner", "worth-automation-runner"),
    @("Forge-quality", "Worth-quality"),
    @("Forge standard", "Worth standard"),
    @("Forge composition", "Worth composition")
)
$runnerCasePairs = @(
    @("WORTH-store", "worth-store"),
    @("WORTH_store", "worth_store"),
    @("WORTHStore", "WorthStore"),
    @("WORTH Store", "Worth Store")
)

function Replace-AsciiBytes(
    [byte[]]$bytes,
    [byte[]]$source,
    [byte[]]$replacement
) {
    if ($source.Length -ne $replacement.Length) {
        throw "Replacement must preserve byte length"
    }

    $count = 0
    for ($offset = 0; $offset -le $bytes.Length - $source.Length; $offset++) {
        $matches = $true
        for ($index = 0; $index -lt $source.Length; $index++) {
            if ($bytes[$offset + $index] -ne $source[$index]) {
                $matches = $false
                break
            }
        }
        if (-not $matches) { continue }

        [Array]::Copy($replacement, 0, $bytes, $offset, $replacement.Length)
        $count++
        $offset += $source.Length - 1
    }
    return $count
}

$candidatePaths = @(
    git grep -I -l -i -E "forge[-_ ](store|foundational|proof|query|relational|ui|automation-runner|quality)|forge(store|query)|Forge (standard|composition)" -- . 2>$null |
        Where-Object { $excludedPaths -notcontains $_ }
)
$changedFiles = 0
$changedOccurrences = 0
foreach ($path in $candidatePaths) {
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $fileChanges = 0
    foreach ($pair in $replacementPairs) {
        $source = [System.Text.Encoding]::ASCII.GetBytes($pair[0])
        $replacement = [System.Text.Encoding]::ASCII.GetBytes($pair[1])
        $fileChanges += Replace-AsciiBytes $bytes $source $replacement
    }
    if ($fileChanges -eq 0) { continue }

    [System.IO.File]::WriteAllBytes($path, $bytes)
    $changedFiles++
    $changedOccurrences += $fileChanges
}

$trackedPaths = @(
    git ls-files |
        Where-Object { $_ -match "(?i)forge[-_](store|foundational|proof|query|relational|ui|automation-runner)|forge(store|query)" }
)
$renamedPaths = 0
foreach ($oldPath in $trackedPaths) {
    $newPath = $oldPath
    foreach ($pair in $replacementPairs) {
        $newPath = $newPath.Replace($pair[0], $pair[1])
    }
    if ($newPath -ceq $oldPath) { continue }

    $parent = Split-Path -Parent $newPath
    if ($parent -and -not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    if (Test-Path -LiteralPath $newPath) {
        throw "Path rename collision: $newPath"
    }
    git mv -- $oldPath $newPath
    if ($LASTEXITCODE -ne 0) { throw "git mv failed: $oldPath" }
    $renamedPaths++
}

$runnerPaths = @(git ls-files -- automation/phase_runner automation/legacy_phase_runner)
foreach ($path in $runnerPaths) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $fileChanges = 0
    foreach ($pair in $runnerCasePairs) {
        $source = [System.Text.Encoding]::ASCII.GetBytes($pair[0])
        $replacement = [System.Text.Encoding]::ASCII.GetBytes($pair[1])
        $fileChanges += Replace-AsciiBytes $bytes $source $replacement
    }
    if ($fileChanges -eq 0) { continue }
    [System.IO.File]::WriteAllBytes($path, $bytes)
    $changedFiles++
    $changedOccurrences += $fileChanges
}

$runnerPathsToRename = @(
    git ls-files -- automation/phase_runner automation/legacy_phase_runner |
        Where-Object { $_ -cmatch "WORTH-store|WORTH_store|WORTHStore" }
)
foreach ($oldPath in $runnerPathsToRename) {
    $newPath = $oldPath
    foreach ($pair in $runnerCasePairs) {
        $newPath = $newPath.Replace($pair[0], $pair[1])
    }
    if ($newPath -ceq $oldPath) { continue }
    git mv -- $oldPath $newPath
    if ($LASTEXITCODE -ne 0) { throw "git mv failed: $oldPath" }
    $renamedPaths++
}

"CHANGED_REFERENCE_FILES=$changedFiles"
"CHANGED_REFERENCE_OCCURRENCES=$changedOccurrences"
"RENAMED_REFERENCE_PATHS=$renamedPaths"
