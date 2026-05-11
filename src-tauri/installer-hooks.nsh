; Vayou file association registration.
;
; Implements the Capabilities + per-extension ProgID model documented
; by Microsoft and used by mpv (io.mpv.<ext>), VLC (VLC.<Type>) and
; OnlyOffice. Tauri's APP_ASSOCIATE macro is bypassed (fileAssociations
; is empty in tauri.conf.json) because it generates bare-extension
; ProgIDs that collide with the global extension key and aren't
; recognized as valid ProgIDs by Windows for default-app picking — the
; cause of "Vayou not showing in Open With".

!define APPNAME "Vayou"
!define APPDESC "Native desktop video player"

!macro VayouRegExt EXT TYPENAME
  WriteRegStr SHCTX "Software\Classes\${APPNAME}.${EXT}" "" "${TYPENAME}"
  WriteRegStr SHCTX "Software\Classes\${APPNAME}.${EXT}" "FriendlyTypeName" "${TYPENAME}"
  WriteRegStr SHCTX "Software\Classes\${APPNAME}.${EXT}\DefaultIcon" "" "$INSTDIR\icons\file\${EXT}.ico,0"
  WriteRegStr SHCTX "Software\Classes\${APPNAME}.${EXT}\shell\open" "FriendlyAppName" "${APPNAME}"
  WriteRegStr SHCTX "Software\Classes\${APPNAME}.${EXT}\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  WriteRegStr SHCTX "Software\Classes\.${EXT}\OpenWithProgids" "${APPNAME}.${EXT}" ""
  WriteRegStr SHCTX "Software\${APPNAME}\Capabilities\FileAssociations" ".${EXT}" "${APPNAME}.${EXT}"
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\SupportedTypes" ".${EXT}" ""
!macroend

!macro VayouUnregExt EXT
  DeleteRegKey SHCTX "Software\Classes\${APPNAME}.${EXT}"
  DeleteRegValue SHCTX "Software\Classes\.${EXT}\OpenWithProgids" "${APPNAME}.${EXT}"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Capabilities bundle — what Settings → Default apps reads.
  WriteRegStr SHCTX "Software\${APPNAME}\Capabilities" "ApplicationName"        "${APPNAME}"
  WriteRegStr SHCTX "Software\${APPNAME}\Capabilities" "ApplicationDescription" "${APPDESC}"
  WriteRegStr SHCTX "Software\${APPNAME}\Capabilities" "ApplicationIcon"        "$INSTDIR\icons\icon.ico,0"

  ; "Open With" picker entry.
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe" "FriendlyAppName" "${APPNAME}"
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\DefaultIcon" "" "$INSTDIR\icons\icon.ico,0"
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'

  ; Per-extension ProgIDs, each carrying its own colored DefaultIcon.
  !insertmacro VayouRegExt "mp4"  "MP4 Video"
  !insertmacro VayouRegExt "m4v"  "M4V Video"
  !insertmacro VayouRegExt "mkv"  "Matroska Video"
  !insertmacro VayouRegExt "avi"  "AVI Video"
  !insertmacro VayouRegExt "mov"  "QuickTime Video"
  !insertmacro VayouRegExt "webm" "WebM Video"
  !insertmacro VayouRegExt "wmv"  "WMV Video"
  !insertmacro VayouRegExt "flv"  "Flash Video"
  !insertmacro VayouRegExt "ts"   "MPEG-TS Video"
  !insertmacro VayouRegExt "mpg"  "MPEG Video"
  !insertmacro VayouRegExt "mpeg" "MPEG Video"
  !insertmacro VayouRegExt "mp3"  "MP3 Audio"
  !insertmacro VayouRegExt "flac" "FLAC Audio"
  !insertmacro VayouRegExt "wav"  "WAV Audio"
  !insertmacro VayouRegExt "ogg"  "OGG Audio"
  !insertmacro VayouRegExt "aac"  "AAC Audio"
  !insertmacro VayouRegExt "wma"  "WMA Audio"
  !insertmacro VayouRegExt "m4a"  "M4A Audio"
  !insertmacro VayouRegExt "opus" "Opus Audio"

  ; Advertise the bundle to Settings → Default apps.
  WriteRegStr SHCTX "Software\RegisteredApplications" "${APPNAME}" "Software\${APPNAME}\Capabilities"

  ; Tell Explorer the icon associations changed.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro VayouUnregExt "mp4"
  !insertmacro VayouUnregExt "m4v"
  !insertmacro VayouUnregExt "mkv"
  !insertmacro VayouUnregExt "avi"
  !insertmacro VayouUnregExt "mov"
  !insertmacro VayouUnregExt "webm"
  !insertmacro VayouUnregExt "wmv"
  !insertmacro VayouUnregExt "flv"
  !insertmacro VayouUnregExt "ts"
  !insertmacro VayouUnregExt "mpg"
  !insertmacro VayouUnregExt "mpeg"
  !insertmacro VayouUnregExt "mp3"
  !insertmacro VayouUnregExt "flac"
  !insertmacro VayouUnregExt "wav"
  !insertmacro VayouUnregExt "ogg"
  !insertmacro VayouUnregExt "aac"
  !insertmacro VayouUnregExt "wma"
  !insertmacro VayouUnregExt "m4a"
  !insertmacro VayouUnregExt "opus"

  DeleteRegKey   SHCTX "Software\${APPNAME}"
  DeleteRegValue SHCTX "Software\RegisteredApplications" "${APPNAME}"
  DeleteRegKey   SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe"

  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
!macroend
