# Safety-boundary tripwire: fail the build if any forbidden Win32 call site
# appears in the Rust sources. The sole reviewed exception is SendInput in the
# foreground-gated, fixed `/unstuck` macro; arbitrary input remains forbidden.
# (Matches call sites `Name(` — the names may legitimately appear in comments.)
$forbidden = "OpenProcess|ReadProcessMemory|WriteProcessMemory|SetWindowsHookEx\w*|SendInput|keybd_event|mouse_event|SetParent|CreateRemoteThread"
$pattern = "\b($forbidden)\s*\("

$unstuckMacro = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\src-tauri\src\win\unstuck.rs"))

$hits = Get-ChildItem -Recurse -Include *.rs "$PSScriptRoot\..\src-tauri\src", "$PSScriptRoot\..\src-tauri\crates" |
    Select-String -Pattern $pattern |
    Where-Object {
        $_.Line.Trim() -notmatch '^(//|//!|///)' -and
        -not ($_.Path -eq $unstuckMacro -and $_.Matches.Value -match '^SendInput\s*\($')
    }

if ($hits) {
    Write-Output "FORBIDDEN API CALL SITES FOUND:"
    $hits | ForEach-Object { Write-Output "  $($_.Path):$($_.LineNumber): $($_.Line.Trim())" }
    exit 1
}
Write-Output "forbidden-API check passed"
exit 0
