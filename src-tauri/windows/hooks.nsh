; Registers Blazon as a browser so it can be picked in Settings > Default apps.
!define BLAZON_KEY "Software\Clients\StartMenuInternet\Blazon"

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "${BLAZON_KEY}" "" "Blazon"
  WriteRegStr HKCU "${BLAZON_KEY}\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "${BLAZON_KEY}\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe"'
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities" "ApplicationName" "Blazon"
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities" "ApplicationDescription" "Navegador leve com Spaces"
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities" "ApplicationIcon" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities\URLAssociations" "http" "BlazonURL"
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities\URLAssociations" "https" "BlazonURL"
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities\FileAssociations" ".htm" "BlazonHTML"
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities\FileAssociations" ".html" "BlazonHTML"
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities\FileAssociations" ".pdf" "BlazonHTML"
  WriteRegStr HKCU "${BLAZON_KEY}\Capabilities\StartMenu" "StartMenuInternet" "Blazon"
  WriteRegStr HKCU "Software\RegisteredApplications" "Blazon" "${BLAZON_KEY}\Capabilities"

  WriteRegStr HKCU "Software\Classes\BlazonURL" "" "Blazon URL"
  WriteRegStr HKCU "Software\Classes\BlazonURL" "URL Protocol" ""
  WriteRegStr HKCU "Software\Classes\BlazonURL\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "Software\Classes\BlazonURL\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  WriteRegStr HKCU "Software\Classes\BlazonHTML" "" "Documento Blazon"
  WriteRegStr HKCU "Software\Classes\BlazonHTML\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "Software\Classes\BlazonHTML\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  ; Tell Explorer the associations changed.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKCU "${BLAZON_KEY}"
  DeleteRegValue HKCU "Software\RegisteredApplications" "Blazon"
  DeleteRegKey HKCU "Software\Classes\BlazonURL"
  DeleteRegKey HKCU "Software\Classes\BlazonHTML"
!macroend
