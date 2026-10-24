; Registers Crest as a browser so it can be picked in Settings > Default apps.
!define CREST_KEY "Software\Clients\StartMenuInternet\Crest"

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "${CREST_KEY}" "" "Crest"
  WriteRegStr HKCU "${CREST_KEY}\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "${CREST_KEY}\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe"'
  WriteRegStr HKCU "${CREST_KEY}\Capabilities" "ApplicationName" "Crest"
  WriteRegStr HKCU "${CREST_KEY}\Capabilities" "ApplicationDescription" "Navegador leve com Spaces"
  WriteRegStr HKCU "${CREST_KEY}\Capabilities" "ApplicationIcon" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "${CREST_KEY}\Capabilities\URLAssociations" "http" "CrestURL"
  WriteRegStr HKCU "${CREST_KEY}\Capabilities\URLAssociations" "https" "CrestURL"
  WriteRegStr HKCU "${CREST_KEY}\Capabilities\FileAssociations" ".htm" "CrestHTML"
  WriteRegStr HKCU "${CREST_KEY}\Capabilities\FileAssociations" ".html" "CrestHTML"
  WriteRegStr HKCU "${CREST_KEY}\Capabilities\FileAssociations" ".pdf" "CrestHTML"
  WriteRegStr HKCU "${CREST_KEY}\Capabilities\StartMenu" "StartMenuInternet" "Crest"
  WriteRegStr HKCU "Software\RegisteredApplications" "Crest" "${CREST_KEY}\Capabilities"

  WriteRegStr HKCU "Software\Classes\CrestURL" "" "Crest URL"
  WriteRegStr HKCU "Software\Classes\CrestURL" "URL Protocol" ""
  WriteRegStr HKCU "Software\Classes\CrestURL\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "Software\Classes\CrestURL\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  WriteRegStr HKCU "Software\Classes\CrestHTML" "" "Documento Crest"
  WriteRegStr HKCU "Software\Classes\CrestHTML\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "Software\Classes\CrestHTML\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  ; Tell Explorer the associations changed.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKCU "${CREST_KEY}"
  DeleteRegValue HKCU "Software\RegisteredApplications" "Crest"
  DeleteRegKey HKCU "Software\Classes\CrestURL"
  DeleteRegKey HKCU "Software\Classes\CrestHTML"
!macroend
