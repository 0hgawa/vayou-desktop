; Override the per-extension DefaultIcon so each video/audio extension shows
; its own colored squircle + chip instead of the generic app icon. Tauri's
; APP_ASSOCIATE registers the ProgID as the bare extension (e.g. "mp4"),
; so the registry path is Software\Classes\<ext>\DefaultIcon.

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "Software\Classes\mp4\DefaultIcon"  "" "$INSTDIR\resources\icons\file\mp4.ico,0"
  WriteRegStr SHCTX "Software\Classes\m4v\DefaultIcon"  "" "$INSTDIR\resources\icons\file\m4v.ico,0"
  WriteRegStr SHCTX "Software\Classes\mkv\DefaultIcon"  "" "$INSTDIR\resources\icons\file\mkv.ico,0"
  WriteRegStr SHCTX "Software\Classes\avi\DefaultIcon"  "" "$INSTDIR\resources\icons\file\avi.ico,0"
  WriteRegStr SHCTX "Software\Classes\mov\DefaultIcon"  "" "$INSTDIR\resources\icons\file\mov.ico,0"
  WriteRegStr SHCTX "Software\Classes\webm\DefaultIcon" "" "$INSTDIR\resources\icons\file\webm.ico,0"
  WriteRegStr SHCTX "Software\Classes\wmv\DefaultIcon"  "" "$INSTDIR\resources\icons\file\wmv.ico,0"
  WriteRegStr SHCTX "Software\Classes\flv\DefaultIcon"  "" "$INSTDIR\resources\icons\file\flv.ico,0"
  WriteRegStr SHCTX "Software\Classes\ts\DefaultIcon"   "" "$INSTDIR\resources\icons\file\ts.ico,0"
  WriteRegStr SHCTX "Software\Classes\mpg\DefaultIcon"  "" "$INSTDIR\resources\icons\file\mpg.ico,0"
  WriteRegStr SHCTX "Software\Classes\mpeg\DefaultIcon" "" "$INSTDIR\resources\icons\file\mpeg.ico,0"
  WriteRegStr SHCTX "Software\Classes\mp3\DefaultIcon"  "" "$INSTDIR\resources\icons\file\mp3.ico,0"
  WriteRegStr SHCTX "Software\Classes\flac\DefaultIcon" "" "$INSTDIR\resources\icons\file\flac.ico,0"
  WriteRegStr SHCTX "Software\Classes\wav\DefaultIcon"  "" "$INSTDIR\resources\icons\file\wav.ico,0"
  WriteRegStr SHCTX "Software\Classes\ogg\DefaultIcon"  "" "$INSTDIR\resources\icons\file\ogg.ico,0"
  WriteRegStr SHCTX "Software\Classes\aac\DefaultIcon"  "" "$INSTDIR\resources\icons\file\aac.ico,0"
  WriteRegStr SHCTX "Software\Classes\wma\DefaultIcon"  "" "$INSTDIR\resources\icons\file\wma.ico,0"
  WriteRegStr SHCTX "Software\Classes\m4a\DefaultIcon"  "" "$INSTDIR\resources\icons\file\m4a.ico,0"
  WriteRegStr SHCTX "Software\Classes\opus\DefaultIcon" "" "$INSTDIR\resources\icons\file\opus.ico,0"

  ; Tell Explorer the icon associations changed (SHCNE_ASSOCCHANGED).
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
!macroend
