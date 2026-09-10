param(
    [Parameter(Mandatory = $true)]
    [string]$Executable
)

$ErrorActionPreference = "Stop"

$resolved = (Resolve-Path -LiteralPath $Executable).Path
$bytes = [System.IO.File]::ReadAllBytes($resolved)

if ($bytes.Length -lt 256 -or $bytes[0] -ne 0x4d -or $bytes[1] -ne 0x5a) {
    throw "Not a valid PE executable: $resolved"
}

$peOffset = [BitConverter]::ToInt32($bytes, 0x3c)
if ($peOffset -lt 0 -or ($peOffset + 96) -ge $bytes.Length) {
    throw "Invalid PE header offset in: $resolved"
}

$optionalHeader = $peOffset + 24
$subsystemOffset = $optionalHeader + 68
$subsystem = [BitConverter]::ToUInt16($bytes, $subsystemOffset)

if ($subsystem -ne 2) {
    throw "Expected Windows GUI subsystem (2), found $subsystem in: $resolved"
}

Write-Output "PASS: Windows GUI subsystem (2): $resolved"
