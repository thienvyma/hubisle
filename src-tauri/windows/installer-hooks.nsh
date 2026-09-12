; Ensure upgrades cannot leave a running or version-stale Isle Pulse binary.
; User data lives under %APPDATA%/%LOCALAPPDATA%, outside $INSTDIR.

!macro ISLE_PULSE_STOP_PROCESSES
  DetailPrint "Stopping Isle Pulse Overlay and its telemetry sidecar..."
  nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /F /T /IM "theisle-overlay.exe"'
  Pop $0
  nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /F /T /IM "isle-pulse-local-telemetry.exe"'
  Pop $0
  Sleep 750
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro ISLE_PULSE_STOP_PROCESSES

  ; Delete before copying because Windows can otherwise preserve an unversioned
  ; external sidecar during an upgrade or same-version repair install.
  Delete /REBOOTOK "$INSTDIR\theisle-overlay.exe"
  Delete /REBOOTOK "$INSTDIR\isle-pulse-local-telemetry.exe"
  Delete /REBOOTOK "$INSTDIR\THIRD_PARTY_NOTICES.local-telemetry.md"

  ; Never report a successful upgrade while either executable is still stale.
  ${If} ${FileExists} "$INSTDIR\theisle-overlay.exe"
  ${OrIf} ${FileExists} "$INSTDIR\isle-pulse-local-telemetry.exe"
    MessageBox MB_ICONSTOP "Isle Pulse could not replace the old installation. Close the app and run the installer again."
    Abort
  ${EndIf}
  SetOverwrite on
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro ISLE_PULSE_STOP_PROCESSES
!macroend
