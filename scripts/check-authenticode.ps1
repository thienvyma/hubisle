param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string[]] $Path,
    [string] $ExpectedPublisher = "SignPath Foundation"
)

$ErrorActionPreference = "Stop"

foreach ($item in $Path) {
    $resolved = Resolve-Path -LiteralPath $item -ErrorAction Stop
    $signature = Get-AuthenticodeSignature -LiteralPath $resolved.Path
    if ($signature.Status -ne 'Valid') {
        throw "Authenticode is not valid for $($resolved.Path): $($signature.Status) $($signature.StatusMessage)"
    }
    if ($null -eq $signature.SignerCertificate) {
        throw "Authenticode signer certificate is missing for $($resolved.Path)."
    }
    if ($null -eq $signature.TimeStamperCertificate) {
        throw "Authenticode timestamp is missing for $($resolved.Path)."
    }
    if ($ExpectedPublisher -and $signature.SignerCertificate.Subject -notlike "*$ExpectedPublisher*") {
        throw "Unexpected Authenticode publisher for $($resolved.Path): $($signature.SignerCertificate.Subject)"
    }
    Write-Host "Valid Authenticode: $($resolved.Path)"
    Write-Host "Publisher: $($signature.SignerCertificate.Subject)"
    Write-Host "Timestamp: $($signature.TimeStamperCertificate.Subject)"
}
