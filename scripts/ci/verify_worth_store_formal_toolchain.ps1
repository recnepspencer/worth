param(
    [string]$ToolCache
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
$crateRoot = Join-Path $repoRoot "workspaces/worth-store/crates/worth-store-formal-models"
$toolchainPath = Join-Path $crateRoot "formal-toolchain.toml"
$toolchain = Get-Content -Raw -LiteralPath $toolchainPath

function Read-ToolchainValue([string]$Name) {
    $match = [regex]::Match($toolchain, "(?m)^$Name\s*=\s*`"([^`"]+)`"\r?$")
    if (-not $match.Success) {
        throw "missing $Name in $toolchainPath"
    }
    return $match.Groups[1].Value
}

$version = Read-ToolchainValue "version"
$downloadUrl = Read-ToolchainValue "download_url"
$expectedSha256 = Read-ToolchainValue "sha256"

if (-not $ToolCache) {
    $ToolCache = Join-Path $repoRoot "workspaces/worth-store/target/formal-tools"
}
New-Item -ItemType Directory -Force -Path $ToolCache | Out-Null
$jarPath = Join-Path $ToolCache "tla2tools-$version.jar"

if (-not (Test-Path -LiteralPath $jarPath)) {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $jarPath
}

$actualSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $jarPath).Hash.ToLowerInvariant()
if ($actualSha256 -ne $expectedSha256) {
    throw "TLC digest mismatch: expected $expectedSha256, got $actualSha256"
}

$java = Get-Command java -ErrorAction SilentlyContinue
if (-not $java) {
    throw "Java is required to run pinned TLC $version; install a JDK and put java on PATH"
}
$stateRoot = Join-Path $ToolCache ("states-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $stateRoot | Out-Null

try {
    & cargo run --quiet --manifest-path (Join-Path $crateRoot "Cargo.toml") `
        --bin worth_store_protocol_check -- `
        $java.Source $jarPath $stateRoot
    if ($LASTEXITCODE -ne 0) {
        throw "direct protocol checking failed"
    }

    Write-Output "verified direct Worth Store protocol checks with TLC $version ($actualSha256)"
}
finally {
    Remove-Item -LiteralPath $stateRoot -Recurse -Force -ErrorAction SilentlyContinue
}
