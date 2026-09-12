$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$project = Join-Path $repoRoot "sidecars\local-telemetry\islemap-thienvyma-telemetry.csproj"
$publishDirectory = Join-Path $repoRoot "sidecars\local-telemetry\bin\publish\win-x64"
$binaryDirectory = Join-Path $repoRoot "src-tauri\binaries"
$source = Join-Path $publishDirectory "islemap-thienvyma-telemetry.exe"
$destination = Join-Path $binaryDirectory "islemap-thienvyma-telemetry-x86_64-pc-windows-msvc.exe"
$legacySource = Join-Path $publishDirectory "isle-pulse-local-telemetry.exe"
$legacyDestination = Join-Path $binaryDirectory "isle-pulse-local-telemetry-x86_64-pc-windows-msvc.exe"

New-Item -ItemType Directory -Force -Path $publishDirectory | Out-Null
New-Item -ItemType Directory -Force -Path $binaryDirectory | Out-Null
Remove-Item -LiteralPath $legacySource -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $legacyDestination -Force -ErrorAction SilentlyContinue

dotnet publish $project `
    --configuration Release `
    --runtime win-x64 `
    --self-contained true `
    -p:PublishSingleFile=true `
    -p:EnableCompressionInSingleFile=true `
    -p:DebugType=None `
    -p:DebugSymbols=false `
    --output $publishDirectory

if ($LASTEXITCODE -ne 0) {
    throw "Local telemetry sidecar publish failed."
}
if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
    throw "Published sidecar was not found at $source"
}

Copy-Item -LiteralPath $source -Destination $destination -Force
Write-Host "Local telemetry sidecar ready: $destination"
