param(
    [string]$OutputPath = "plans/platform-migrations/store-platform-rename-inventory.csv"
)

$ErrorActionPreference = "Stop"

$storeRoots = @(
    "workspaces/forge-store/",
    "plans/forge-store/"
)
$storeReference = [regex]::new(
    "(?i)(forge[-_ ]store|forgestore)",
    [System.Text.RegularExpressions.RegexOptions]::CultureInvariant
)
$forgeOccurrence = [regex]::new(
    "forge",
    [System.Text.RegularExpressions.RegexOptions]::IgnoreCase -bor
        [System.Text.RegularExpressions.RegexOptions]::CultureInvariant
)
$tokenPattern = [regex]::new(
    "[A-Za-z0-9_-]*forge[A-Za-z0-9_-]*",
    [System.Text.RegularExpressions.RegexOptions]::IgnoreCase -bor
        [System.Text.RegularExpressions.RegexOptions]::CultureInvariant
)
$ordinaryEnglish = [regex]::new(
    "(?i)^(?:forged|forger|forgers|forges|forgeable|forgeability|forging|unforgeable|unforgeability|forgery|forgeries|forget|forgets|forgetting|forgetful|forgetfulness|forgot|forgotten)$",
    [System.Text.RegularExpressions.RegexOptions]::CultureInvariant
)
$proseExtensions = @(".md", ".txt", ".rst", ".adoc")
$excludedPaths = @(
    $OutputPath.Replace("\", "/"),
    "scripts/migrations/inventory_store_platform_rename.ps1"
)

function Get-CaseForm([string]$value) {
    if ($value -ceq $value.ToUpperInvariant()) { return "upper" }
    if ($value -ceq $value.ToLowerInvariant()) { return "lower" }
    if ($value.Length -gt 1 -and
        $value.Substring(0, 1) -ceq $value.Substring(0, 1).ToUpperInvariant() -and
        $value.Substring(1) -ceq $value.Substring(1).ToLowerInvariant()) {
        return "title"
    }
    return "mixed"
}

function Get-TokenAt([string]$text, [int]$index) {
    foreach ($match in $tokenPattern.Matches($text)) {
        if ($match.Index -le $index -and $index -lt ($match.Index + $match.Length)) {
            return $match.Value
        }
    }
    return $text.Substring($index, 5)
}

function Get-Classification(
    [string]$path,
    [string]$token,
    [string]$context,
    [string]$kind
) {
    $normalized = $token.ToLowerInvariant()
    $extension = [System.IO.Path]::GetExtension($path).ToLowerInvariant()
    $isProse = $proseExtensions -contains $extension

    $tokenSegments = @($normalized -split "[_-]" | Where-Object { $_ })
    if ($ordinaryEnglish.IsMatch($normalized) -or
        @($tokenSegments | Where-Object { $ordinaryEnglish.IsMatch($_) }).Count -gt 0 -or
        $normalized -match "forged|forgery|forgeries|forgeable|unforgeable") {
        return @("ordinary_english", "preserve", "Known English or security vocabulary")
    }
    if ($normalized -eq "direct_execution_forge") {
        return @("ordinary_english", "preserve", "Reviewed hostile scenario describing an attempted forgery")
    }
    if ($normalized -match "forge[-_]?store|forgestore") {
        return @("store_brand", "rename", "Store product or crate identifier")
    }
    if ($normalized -match "^forge[-_]" -or $normalized -match "^forge[a-z0-9]") {
        return @("platform_identifier", "rename", "Platform-prefixed identifier")
    }
    if ($context -match "(?i)forge[-_ ]store|forge (?:foundational|proof|platform|workspace|query|relational|ui|certification)") {
        return @("platform_brand", "rename", "Platform or product name in context")
    }
    if ($normalized -eq "forge" -and $token -ceq "Forge") {
        return @("platform_brand", "rename", "Reviewed capitalized platform proper noun")
    }
    if ($normalized -eq "non-forge") {
        return @("platform_brand", "rename", "Reviewed platform adjective")
    }
    if ($normalized -eq "forge" -and $context -match "(?i)(?:forge_workspace|[/\\]forge(?:[/\\_.-]|$)|\bforge\.(?:query|store|proof|platform))") {
        return @("platform_path", "rename", "Reviewed platform path or namespace")
    }
    if ($normalized -eq "forge" -and $context -match "(?i)(?:\b(?:cannot|can|could|may|must|should|to|never|not|without|unable to|able to)\s+forge\b|\bforge\s+(?:a|an|the|stronger|authority|proof|receipt|witness|output|access|state|record|value|object|capability)\b)") {
        return @("ordinary_english", "preserve", "Reviewed use of forge as a security or construction verb")
    }
    if ($kind -eq "path") {
        return @("path_identifier", "rename", "Tracked path identifier")
    }
    if ($isProse -and $normalized -eq "forge") {
        return @("ambiguous_prose", "manual_review", "Exact prose word requires semantic adjudication")
    }
    if (-not $isProse) {
        return @("code_or_config_identifier", "rename", "Non-prose identifier or configuration value")
    }
    return @("ambiguous_prose", "manual_review", "Unclassified prose occurrence")
}

function Get-ProposedValue([string]$value, [string]$action) {
    if ($action -ne "rename") { return $value }
    return $value.Replace("FORGE", "WORTH").Replace("Forge", "Worth").Replace("forge", "worth")
}

function New-InventoryRow(
    [string]$kind,
    [string]$path,
    [Nullable[int]]$line,
    [Nullable[int]]$column,
    [string]$matchedText,
    [string]$token,
    [string]$context,
    [int]$ordinal
) {
    $classification = Get-Classification $path $token $context $kind
    $action = $classification[1]
    $lineIdentity = if ($null -eq $line) { 0 } else { $line.Value }
    [PSCustomObject]@{
        id = "{0}:{1}:{2}:{3}" -f $kind, $path, $lineIdentity, $ordinal
        occurrence_kind = $kind
        path = $path
        line = $line
        column = $column
        matched_text = $matchedText
        containing_token = $token
        case_form = Get-CaseForm $matchedText
        context = $context
        semantic_class = $classification[0]
        action = $action
        proposed_value = Get-ProposedValue $matchedText $action
        confidence = if ($action -eq "manual_review") { "unreviewed" } else { "high" }
        reason = $classification[2]
        reviewer_note = ""
    }
}

$trackedPaths = @(git ls-files)
if ($LASTEXITCODE -ne 0) { throw "git ls-files failed" }

$candidatePaths = [System.Collections.Generic.HashSet[string]]::new(
    [System.StringComparer]::OrdinalIgnoreCase
)
foreach ($path in $trackedPaths) {
    $normalizedPath = $path.Replace("\", "/")
    if ($excludedPaths -contains $normalizedPath) { continue }

    $owned = $false
    foreach ($root in $storeRoots) {
        if ($normalizedPath.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)) {
            $owned = $true
            break
        }
    }
    if ($owned -or $storeReference.IsMatch($normalizedPath)) {
        [void]$candidatePaths.Add($normalizedPath)
    }
}

$externalReferences = @(
    git grep -I -l -i -E "forge[-_ ]store|forgestore" -- . ":(exclude)$OutputPath" ":(exclude)scripts/migrations/inventory_store_platform_rename.ps1"
)
if ($LASTEXITCODE -notin @(0, 1)) { throw "git grep failed" }
foreach ($path in $externalReferences) {
    [void]$candidatePaths.Add($path.Replace("\", "/"))
}

$rows = [System.Collections.Generic.List[object]]::new()
foreach ($path in ($candidatePaths | Sort-Object)) {
    $pathOrdinal = 0
    foreach ($match in $forgeOccurrence.Matches($path)) {
        $pathOrdinal++
        $token = Get-TokenAt $path $match.Index
        $rows.Add((New-InventoryRow "path" $path $null ($match.Index + 1) $match.Value $token $path $pathOrdinal))
    }

    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
    try {
        $bytes = [System.IO.File]::ReadAllBytes($path)
        if ($bytes -contains 0) { continue }
        $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    } catch {
        continue
    }

    $lineNumber = 0
    foreach ($contentLine in ($text -split "`r?`n")) {
        $lineNumber++
        $lineOrdinal = 0
        foreach ($match in $forgeOccurrence.Matches($contentLine)) {
            $lineOrdinal++
            $token = Get-TokenAt $contentLine $match.Index
            $contextStart = [Math]::Max(0, $match.Index - 80)
            $contextLength = [Math]::Min(200, $contentLine.Length - $contextStart)
            $context = $contentLine.Substring($contextStart, $contextLength).Trim()
            $rows.Add((New-InventoryRow "content" $path $lineNumber ($match.Index + 1) $match.Value $token $context $lineOrdinal))
        }
    }
}

$directory = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Force -Path $directory | Out-Null
$rows | Export-Csv -LiteralPath $OutputPath -NoTypeInformation -Encoding utf8

"INVENTORY_ROWS=$($rows.Count)"
$rows | Group-Object semantic_class | Sort-Object Name | ForEach-Object {
    "CLASS_$($_.Name)=$($_.Count)"
}
$rows | Group-Object action | Sort-Object Name | ForEach-Object {
    "ACTION_$($_.Name)=$($_.Count)"
}
