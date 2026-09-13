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
  ; Let NSIS replace files in place. Removing the working executable before
  ; extraction made an antivirus block or interrupted update leave no usable
  ; installation behind.
  SetOverwrite on
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Do not remove any legacy installation until the complete replacement is
  ; present. This keeps an interrupted or blocked upgrade recoverable.
  ${If} ${FileExists} "$INSTDIR\islemap-thienvyma.exe"
  ${AndIf} ${FileExists} "$INSTDIR\islemap-thienvyma-telemetry.exe"
    ; The previous product name used a separate per-user install directory.
    ; Roaming/local user data is intentionally kept and migrated by the app.
    Delete /REBOOTOK "$INSTDIR\theisle-overlay.exe"
    Delete /REBOOTOK "$INSTDIR\isle-pulse-local-telemetry.exe"
    RMDir /r /REBOOTOK "$LOCALAPPDATA\Isle Pulse Overlay"
    Delete "$DESKTOP\Isle Pulse Overlay.lnk"
    Delete "$SMPROGRAMS\Isle Pulse Overlay.lnk"
    Delete "$SMPROGRAMS\Isle Pulse Overlay\Isle Pulse Overlay.lnk"
    RMDir "$SMPROGRAMS\Isle Pulse Overlay"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Isle Pulse Overlay"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Isle Pulse Overlay"
    DeleteRegKey HKCU "Software\Huỳnh Vỹ\Isle Pulse Overlay"
  ${Else}
    MessageBox MB_ICONSTOP "Windows or security software prevented islemap-thienvyma from being installed. The previous installation was kept; review Windows Security Protection history, then try again."
    Abort
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro ISLEMAP_THIENVYMA_STOP_PROCESSES
!macroend
