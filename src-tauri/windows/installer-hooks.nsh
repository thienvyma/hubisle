; Ensure upgrades cannot leave running or stale old/new executables.
; Never recursively remove $INSTDIR in this hook. Tauri's normal update-mode
; uninstaller removes the files it installed; these hooks remove known
; historical names and the separate legacy program directory. User data lives
; under %APPDATA%\islemap-thienvyma and
; %LOCALAPPDATA%\islemap-thienvyma-data.

!macro ISLEMAP_THIENVYMA_STOP_PROCESSES
  DetailPrint "Stopping islemap-thienvyma and its telemetry sidecar..."
  nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /F /T /IM "islemap-thienvyma.exe"'
  Pop $0
  nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /F /T /IM "theisle-overlay.exe"'
  Pop $0
  nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /F /T /IM "islemap-thienvyma-telemetry.exe"'
  Pop $0
  nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /F /T /IM "isle-pulse-local-telemetry.exe"'
  Pop $0
  Sleep 750
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro ISLEMAP_THIENVYMA_STOP_PROCESSES

  ; Delete before copying because Windows can otherwise preserve an unversioned
  ; external sidecar during an upgrade or same-version repair install.
  Delete /REBOOTOK "$INSTDIR\islemap-thienvyma.exe"
  Delete /REBOOTOK "$INSTDIR\theisle-overlay.exe"
  Delete /REBOOTOK "$INSTDIR\islemap-thienvyma-telemetry.exe"
  Delete /REBOOTOK "$INSTDIR\isle-pulse-local-telemetry.exe"
  Delete /REBOOTOK "$INSTDIR\THIRD_PARTY_NOTICES.local-telemetry.md"

  ; The previous product name used a separate per-user install directory.
  ; Remove only that program directory; roaming/local user data is stored in
  ; TheIsleOverlay and is migrated by the application on first launch.
  RMDir /r /REBOOTOK "$LOCALAPPDATA\Isle Pulse Overlay"
  Delete "$DESKTOP\Isle Pulse Overlay.lnk"
  Delete "$SMPROGRAMS\Isle Pulse Overlay.lnk"
  Delete "$SMPROGRAMS\Isle Pulse Overlay\Isle Pulse Overlay.lnk"
  RMDir "$SMPROGRAMS\Isle Pulse Overlay"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Isle Pulse Overlay"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Isle Pulse Overlay"
  DeleteRegKey HKCU "Software\Huỳnh Vỹ\Isle Pulse Overlay"

  ; Never report a successful upgrade while an executable is still stale.
  ${If} ${FileExists} "$INSTDIR\islemap-thienvyma.exe"
  ${OrIf} ${FileExists} "$INSTDIR\theisle-overlay.exe"
  ${OrIf} ${FileExists} "$INSTDIR\islemap-thienvyma-telemetry.exe"
  ${OrIf} ${FileExists} "$INSTDIR\isle-pulse-local-telemetry.exe"
  ${OrIf} ${FileExists} "$LOCALAPPDATA\Isle Pulse Overlay\*.*"
    MessageBox MB_ICONSTOP "islemap-thienvyma could not replace the old installation. Close the app and run the installer again."
    Abort
  ${EndIf}
  SetOverwrite on
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro ISLEMAP_THIENVYMA_STOP_PROCESSES
!macroend
