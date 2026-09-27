param(
    [string]$InventoryPath = "plans/platform-migrations/store-platform-rename-inventory.csv"
)

$ErrorActionPreference = "Stop"
$protectedRoots = @("workspaces/forge-store/", "plans/forge-store/")
$rows = @(Import-Csv -LiteralPath $InventoryPath | Where-Object {
    if ($_.occurrence_kind -ne "content") { return $false }
    foreach ($root in $protectedRoots) {
        if ($_.path.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)) {
            return $true
        }
    }
    return $false
})

function Test-ForgeAt([byte[]]$bytes, [int]$offset) {
    if ($offset + 5 -gt $bytes.Length) { return $false }
    $expected = @(102, 111, 114, 103, 101)
    for ($i = 0; $i -lt 5; $i++) {
        $value = $bytes[$offset + $i]
        if ($value -ge 65 -and $value -le 90) { $value += 32 }
        if ($value -ne $expected[$i]) { return $false }
    }
    return $true
}

$changedFiles = 0
$renamedOccurrences = 0
$preservedOccurrences = 0

foreach ($group in ($rows | Group-Object path | Sort-Object Name)) {
    $path = $group.Name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Inventory source path is missing: $path"
    }

    $bytes = [System.IO.File]::ReadAllBytes($path)
    $positions = [System.Collections.Generic.List[int]]::new()
    for ($offset = 0; $offset -le $bytes.Length - 5; $offset++) {
        if (Test-ForgeAt $bytes $offset) {
            $positions.Add($offset)
            $offset += 4
        }
    }

    $orderedRows = @($group.Group | Sort-Object { [int]$_.line }, { [int]$_.column })
    if ($positions.Count -ne $orderedRows.Count) {
        throw "Occurrence drift in ${path}: bytes=$($positions.Count), inventory=$($orderedRows.Count)"
    }

    $changed = $false
    for ($index = 0; $index -lt $orderedRows.Count; $index++) {
        $row = $orderedRows[$index]
        $offset = $positions[$index]
        $actual = [System.Text.Encoding]::ASCII.GetString($bytes, $offset, 5)
        if ($actual -cne $row.matched_text) {
            throw "Case drift in ${path}: expected '$($row.matched_text)', found '$actual'"
        }

        if ($row.action -eq "preserve") {
            $preservedOccurrences++
            continue
        }
        if ($row.action -ne "rename") {
            throw "Unadjudicated action '$($row.action)' in $path"
        }

        $replacement = [System.Text.Encoding]::ASCII.GetBytes($row.proposed_value)
        if ($replacement.Length -ne 5) {
            throw "Replacement length changed for $($row.id)"
        }
        [Array]::Copy($replacement, 0, $bytes, $offset, 5)
        $changed = $true
        $renamedOccurrences++
    }

    if ($changed) {
        [System.IO.File]::WriteAllBytes($path, $bytes)
        $changedFiles++
    }
}

"CHANGED_FILES=$changedFiles"
"RENAMED_CONTENT_OCCURRENCES=$renamedOccurrences"
"PRESERVED_CONTENT_OCCURRENCES=$preservedOccurrences"
