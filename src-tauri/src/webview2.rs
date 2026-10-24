//! WebView2 features Tauri doesn't expose: request blocking, audio state and
//! muting, per-Space download folders and saved passwords.

use std::path::PathBuf;

use tauri::{AppHandle, Emitter, Webview};
use webview2_com::{
    take_pwstr, DownloadStartingEventHandler, IsDocumentPlayingAudioChangedEventHandler,
    Microsoft::Web::WebView2::Win32::*, ProcessFailedEventHandler,
    WebResourceRequestedEventHandler,
};
use windows_core::{w, Interface, BOOL, HSTRING, PWSTR};

use crate::{blocker, update_tab};

fn request_kind(ctx: COREWEBVIEW2_WEB_RESOURCE_CONTEXT) -> &'static str {
    match ctx {
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET => "stylesheet",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE => "image",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA => "media",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT => "font",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT => "script",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST
        | COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH => "xmlhttprequest",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_WEBSOCKET => "websocket",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_PING => "ping",
        _ => "other",
    }
}

/// Hooks a tab (or Peek) webview. `tab` enables audio tracking.
pub fn install(
    app: &AppHandle,
    wv: &Webview,
    tab: Option<u64>,
    downloads: Option<PathBuf>,
    passwords: bool,
) {
    let app = app.clone();
    let _ = wv.with_webview(move |pv| unsafe {
        let Ok(core) = pv.controller().CoreWebView2() else {
            return;
        };
        let env = pv.environment();
        let mut token = 0i64;

        if let Ok(profile) = core.cast::<ICoreWebView2_13>().and_then(|c| c.Profile()) {
            if let Ok(p6) = profile.cast::<ICoreWebView2Profile6>() {
                let _ = p6.SetIsPasswordAutosaveEnabled(passwords);
                let _ = p6.SetIsGeneralAutofillEnabled(passwords);
            }
        }

        // Ad and tracker blocking, including iframes and workers where supported.
        let filtered = match core.cast::<ICoreWebView2_22>() {
            Ok(c22) => c22.AddWebResourceRequestedFilterWithRequestSourceKinds(
                w!("*"),
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
            ),
            Err(_) => {
                core.AddWebResourceRequestedFilter(w!("*"), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)
            }
        };
        if filtered.is_ok() {
            let _ = core.add_WebResourceRequested(
                &WebResourceRequestedEventHandler::create(Box::new(move |sender, args| {
                    let (Some(sender), Some(args)) = (sender, args) else {
                        return Ok(());
                    };
                    let mut ctx = COREWEBVIEW2_WEB_RESOURCE_CONTEXT::default();
                    args.ResourceContext(&mut ctx)?;
                    if ctx == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT {
                        return Ok(());
                    }
                    let mut uri = PWSTR::null();
                    args.Request()?.Uri(&mut uri)?;
                    let uri = take_pwstr(uri);
                    let mut source = PWSTR::null();
                    sender.Source(&mut source)?;
                    let source = take_pwstr(source);
                    if blocker::should_block(&uri, &source, request_kind(ctx)) {
                        let response =
                            env.CreateWebResourceResponse(None, 403, w!("Blocked"), w!(""))?;
                        args.SetResponse(&response)?;
                    }
                    Ok(())
                })),
                &mut token,
            );
        }

        if let (Some(id), Ok(c8)) = (tab, core.cast::<ICoreWebView2_8>()) {
            let app = app.clone();
            let _ = c8.add_IsDocumentPlayingAudioChanged(
                &IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |sender, _| {
                    let Some(sender) = sender else { return Ok(()) };
                    let mut playing = BOOL::default();
                    sender
                        .cast::<ICoreWebView2_8>()?
                        .IsDocumentPlayingAudio(&mut playing)?;
                    let playing = playing.as_bool();
                    update_tab(&app, id, |t| t.audio = playing);
                    Ok(())
                })),
                &mut token,
            );
        }

        // A crashed page process leaves a blank tab: log it and tell the sidebar.
        let app_pf = app.clone();
        let _ = core.add_ProcessFailed(
            &ProcessFailedEventHandler::create(Box::new(move |_, args| {
                let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
                if let Some(args) = args {
                    args.ProcessFailedKind(&mut kind)?;
                }
                crate::diag::error(&format!(
                    "processo do WebView2 falhou (tipo {}) na aba {:?}",
                    kind.0, tab
                ));
                if let Some(id) = tab {
                    let _ = app_pf.emit_to("ui", "tab-crashed", id);
                }
                Ok(())
            })),
            &mut token,
        );

        if let (Some(dir), Ok(c4)) = (downloads, core.cast::<ICoreWebView2_4>()) {
            let _ = c4.add_DownloadStarting(
                &DownloadStartingEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    let mut path = PWSTR::null();
                    args.ResultFilePath(&mut path)?;
                    let path = PathBuf::from(take_pwstr(path));
                    if let Some(name) = path.file_name() {
                        let _ = std::fs::create_dir_all(&dir);
                        args.SetResultFilePath(&HSTRING::from(dir.join(name).as_os_str()))?;
                    }
                    Ok(())
                })),
                &mut token,
            );
        }
    });
}

pub fn set_muted(wv: &Webview, muted: bool) {
    let _ = wv.with_webview(move |pv| unsafe {
        if let Ok(c8) = pv
            .controller()
            .CoreWebView2()
            .and_then(|c| c.cast::<ICoreWebView2_8>())
        {
            let _ = c8.SetIsMuted(muted);
        }
    });
}
