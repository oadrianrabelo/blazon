//! WebView2 features Tauri doesn't expose: request blocking, audio state and
//! muting, downloads, saved passwords, site permissions and extensions.

use std::{cell::RefCell, collections::HashMap, path::PathBuf, sync::mpsc, time::Duration};

use tauri::{AppHandle, Emitter, Webview};
use webview2_com::{
    take_pwstr, BrowserExtensionEnableCompletedHandler, BrowserExtensionRemoveCompletedHandler,
    BytesReceivedChangedEventHandler, CapturePreviewCompletedHandler, DownloadStartingEventHandler,
    GetNonDefaultPermissionSettingsCompletedHandler, IsDocumentPlayingAudioChangedEventHandler,
    Microsoft::Web::WebView2::Win32::*, NavigationCompletedEventHandler, ProcessFailedEventHandler,
    ProfileAddBrowserExtensionCompletedHandler, ProfileGetBrowserExtensionsCompletedHandler,
    SetPermissionStateCompletedHandler, StateChangedEventHandler, WebResourceRequestedEventHandler,
};
use windows_core::{w, Interface, BOOL, HSTRING, PWSTR};

use crate::{blocker, update_tab};

const ERROR_JS: &str = include_str!("error.js");

/// Network failures that get Blazon's own page; None leaves WebView2's page.
fn failure_code(status: COREWEBVIEW2_WEB_ERROR_STATUS) -> Option<&'static str> {
    Some(match status {
        COREWEBVIEW2_WEB_ERROR_STATUS_HOST_NAME_NOT_RESOLVED => "HOST_NAME_NOT_RESOLVED",
        COREWEBVIEW2_WEB_ERROR_STATUS_TIMEOUT => "TIMEOUT",
        COREWEBVIEW2_WEB_ERROR_STATUS_DISCONNECTED => "DISCONNECTED",
        COREWEBVIEW2_WEB_ERROR_STATUS_SERVER_UNREACHABLE => "SERVER_UNREACHABLE",
        COREWEBVIEW2_WEB_ERROR_STATUS_CANNOT_CONNECT => "CANNOT_CONNECT",
        COREWEBVIEW2_WEB_ERROR_STATUS_CONNECTION_RESET => "CONNECTION_RESET",
        COREWEBVIEW2_WEB_ERROR_STATUS_ERROR_HTTP_INVALID_SERVER_RESPONSE => {
            "INVALID_SERVER_RESPONSE"
        }
        _ => return None,
    })
}

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

        // A site that can't be reached gets Crest's failure page instead of
        // Edge's. Server error pages (404, 500…) and certificate warnings stay.
        let app_nc = app.clone();
        let _ = core.add_NavigationCompleted(
            &NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
                let (Some(sender), Some(args)) = (sender, args) else {
                    return Ok(());
                };
                let mut ok = BOOL::default();
                args.IsSuccess(&mut ok)?;
                if ok.as_bool() {
                    return Ok(());
                }
                let mut status = COREWEBVIEW2_WEB_ERROR_STATUS::default();
                args.WebErrorStatus(&mut status)?;
                let Some(code) = failure_code(status) else {
                    return Ok(());
                };
                let mut uri = PWSTR::null();
                sender.Source(&mut uri)?;
                let url = take_pwstr(uri);
                let lang = crate::with_data(&app_nc, |d| d.settings.language.clone());
                let data = serde_json::json!({ "url": url, "code": code, "lang": lang });
                let script = format!("const BLAZON_ERROR = {data};\n{}", ERROR_JS);
                let _ = sender.ExecuteScript(&HSTRING::from(script), None);
                Ok(())
            })),
            &mut token,
        );

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
                    let _ = app_pf.emit("tab-crashed", id);
                }
                Ok(())
            })),
            &mut token,
        );

        // Downloads: saved to the Space's folder and shown in Blazon's own list.
        if let Ok(c4) = core.cast::<ICoreWebView2_4>() {
            let app_dl = app.clone();
            let _ = c4.add_DownloadStarting(
                &DownloadStartingEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    if let Some(dir) = &downloads {
                        let mut path = PWSTR::null();
                        args.ResultFilePath(&mut path)?;
                        let path = PathBuf::from(take_pwstr(path));
                        if let Some(name) = path.file_name() {
                            let _ = std::fs::create_dir_all(dir);
                            args.SetResultFilePath(&HSTRING::from(dir.join(name).as_os_str()))?;
                        }
                    }
                    args.SetHandled(true)?;
                    track_download(&app_dl, &args.DownloadOperation()?);
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

/// Whether the page has its own history to go back to.
pub async fn can_go_back(wv: &Webview) -> bool {
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = wv.with_webview(move |pv| unsafe {
        let mut b = BOOL::default();
        let ok = pv
            .controller()
            .CoreWebView2()
            .and_then(|c| c.CanGoBack(&mut b))
            .is_ok();
        let _ = tx.send(ok && b.as_bool());
    });
    tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(std::time::Duration::from_secs(2))
            .unwrap_or(true)
    })
    .await
    .unwrap_or(true)
}

// Downloads

thread_local! {
    /// Live download operations by id; COM objects stay on the UI thread.
    static OPERATIONS: RefCell<HashMap<u64, ICoreWebView2DownloadOperation>> = RefCell::new(HashMap::new());
}

fn download_numbers(op: &ICoreWebView2DownloadOperation) -> (i64, i64) {
    let (mut received, mut total) = (0i64, 0i64);
    unsafe {
        let _ = op.BytesReceived(&mut received);
        let _ = op.TotalBytesToReceive(&mut total);
    }
    (received, total)
}

fn track_download(app: &AppHandle, op: &ICoreWebView2DownloadOperation) {
    let mut path = PWSTR::null();
    let path = unsafe {
        op.ResultFilePath(&mut path)
            .map(|_| take_pwstr(path))
            .unwrap_or_default()
    };
    let mut uri = PWSTR::null();
    let uri = unsafe {
        op.Uri(&mut uri)
            .map(|_| take_pwstr(uri))
            .unwrap_or_default()
    };
    let (_, total) = download_numbers(op);
    let id = crate::download_started(app, &path, &uri, total);
    OPERATIONS.with(|m| m.borrow_mut().insert(id, op.clone()));
    let mut token = 0i64;
    let app_p = app.clone();
    unsafe {
        let _ = op.add_BytesReceivedChanged(
            &BytesReceivedChangedEventHandler::create(Box::new(move |op, _| {
                if let Some(op) = op {
                    let (received, total) = download_numbers(&op);
                    crate::download_progress(&app_p, id, received, total);
                }
                Ok(())
            })),
            &mut token,
        );
        let app_s = app.clone();
        let _ = op.add_StateChanged(
            &StateChangedEventHandler::create(Box::new(move |op, _| {
                let Some(op) = op else { return Ok(()) };
                let mut state = COREWEBVIEW2_DOWNLOAD_STATE::default();
                op.State(&mut state)?;
                let name = match state {
                    COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED => "done",
                    COREWEBVIEW2_DOWNLOAD_STATE_INTERRUPTED => "failed",
                    _ => "progress",
                };
                if name != "progress" {
                    OPERATIONS.with(|m| m.borrow_mut().remove(&id));
                }
                let (received, total) = download_numbers(&op);
                crate::download_state(&app_s, id, name, received, total);
                Ok(())
            })),
            &mut token,
        );
    }
}

pub fn cancel_download(app: &AppHandle, id: u64) {
    let _ = app.run_on_main_thread(move || {
        OPERATIONS.with(|m| {
            if let Some(op) = m.borrow().get(&id) {
                let _ = unsafe { op.Cancel() };
            }
        })
    });
}

// Profile features: site permissions and extensions. Each Space has its own
// profile (data folder), reached through one of its webviews.

fn profile(pv: &tauri::webview::PlatformWebview) -> Option<ICoreWebView2Profile> {
    unsafe {
        pv.controller()
            .CoreWebView2()
            .ok()?
            .cast::<ICoreWebView2_13>()
            .ok()?
            .Profile()
            .ok()
    }
}

/// Runs `start` on the UI thread and waits for the value it sends back.
async fn ask<T: Send + 'static>(
    wv: &Webview,
    start: impl FnOnce(tauri::webview::PlatformWebview, mpsc::Sender<T>) + Send + 'static,
) -> Option<T> {
    let (tx, rx) = mpsc::channel();
    wv.with_webview(move |pv| start(pv, tx)).ok()?;
    tauri::async_runtime::spawn_blocking(move || rx.recv_timeout(Duration::from_secs(8)).ok())
        .await
        .ok()
        .flatten()
}

#[derive(serde::Serialize)]
pub struct Permission {
    pub kind: i32,
    pub origin: String,
    pub state: &'static str,
}

pub async fn list_permissions(wv: &Webview) -> Vec<Permission> {
    ask(wv, |pv, tx| unsafe {
        let Some(p4) = profile(&pv).and_then(|p| p.cast::<ICoreWebView2Profile4>().ok()) else {
            let _ = tx.send(vec![]);
            return;
        };
        let tx2 = tx.clone();
        let started = p4.GetNonDefaultPermissionSettings(
            &GetNonDefaultPermissionSettingsCompletedHandler::create(Box::new(move |_, list| {
                let mut out = vec![];
                if let Some(list) = list {
                    let mut n = 0u32;
                    list.Count(&mut n)?;
                    for i in 0..n {
                        let item = list.GetValueAtIndex(i)?;
                        let (mut kind, mut state, mut origin) = (
                            COREWEBVIEW2_PERMISSION_KIND::default(),
                            COREWEBVIEW2_PERMISSION_STATE::default(),
                            PWSTR::null(),
                        );
                        item.PermissionKind(&mut kind)?;
                        item.PermissionState(&mut state)?;
                        item.PermissionOrigin(&mut origin)?;
                        out.push(Permission {
                            kind: kind.0,
                            origin: take_pwstr(origin),
                            state: if state == COREWEBVIEW2_PERMISSION_STATE_ALLOW {
                                "allow"
                            } else {
                                "deny"
                            },
                        });
                    }
                }
                let _ = tx2.send(out);
                Ok(())
            })),
        );
        if started.is_err() {
            let _ = tx.send(vec![]);
        }
    })
    .await
    .unwrap_or_default()
}

/// Sets a site permission; "default" makes the site ask again.
pub async fn set_permission(wv: &Webview, kind: i32, origin: String, state: String) -> bool {
    ask(wv, move |pv, tx| unsafe {
        let Some(p4) = profile(&pv).and_then(|p| p.cast::<ICoreWebView2Profile4>().ok()) else {
            let _ = tx.send(false);
            return;
        };
        let state = match state.as_str() {
            "allow" => COREWEBVIEW2_PERMISSION_STATE_ALLOW,
            "deny" => COREWEBVIEW2_PERMISSION_STATE_DENY,
            _ => COREWEBVIEW2_PERMISSION_STATE_DEFAULT,
        };
        let tx2 = tx.clone();
        let r = p4.SetPermissionState(
            COREWEBVIEW2_PERMISSION_KIND(kind),
            &HSTRING::from(origin),
            state,
            &SetPermissionStateCompletedHandler::create(Box::new(move |hr| {
                let _ = tx2.send(hr.is_ok());
                Ok(())
            })),
        );
        if r.is_err() {
            let _ = tx.send(false);
        }
    })
    .await
    .unwrap_or(false)
}

#[derive(serde::Serialize)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}

fn extension_info(e: &ICoreWebView2BrowserExtension) -> Extension {
    let (mut id, mut name, mut enabled) = (PWSTR::null(), PWSTR::null(), BOOL::default());
    unsafe {
        let _ = e.Id(&mut id);
        let _ = e.Name(&mut name);
        let _ = e.IsEnabled(&mut enabled);
    }
    Extension {
        id: take_pwstr(id),
        name: take_pwstr(name),
        enabled: enabled.as_bool(),
    }
}

fn with_extensions(
    pv: &tauri::webview::PlatformWebview,
    f: impl FnOnce(Vec<ICoreWebView2BrowserExtension>) + 'static,
) -> bool {
    let Some(p7) = profile(pv).and_then(|p| p.cast::<ICoreWebView2Profile7>().ok()) else {
        return false;
    };
    let mut f = Some(f);
    unsafe {
        p7.GetBrowserExtensions(&ProfileGetBrowserExtensionsCompletedHandler::create(
            Box::new(move |_, list| {
                let mut out = vec![];
                if let Some(list) = list {
                    let mut n = 0u32;
                    list.Count(&mut n)?;
                    for i in 0..n {
                        out.push(list.GetValueAtIndex(i)?);
                    }
                }
                if let Some(f) = f.take() {
                    f(out);
                }
                Ok(())
            }),
        ))
        .is_ok()
    }
}

pub async fn list_extensions(wv: &Webview) -> Vec<Extension> {
    ask(wv, |pv, tx| {
        let tx2 = tx.clone();
        if !with_extensions(&pv, move |list| {
            let _ = tx2.send(list.iter().map(extension_info).collect());
        }) {
            let _ = tx.send(vec![]);
        }
    })
    .await
    .unwrap_or_default()
}

/// Installs an unpacked extension folder; returns its name or an error.
pub async fn add_extension(wv: &Webview, folder: String) -> Result<Extension, String> {
    ask(wv, move |pv, tx| unsafe {
        let Some(p7) = profile(&pv).and_then(|p| p.cast::<ICoreWebView2Profile7>().ok()) else {
            let _ = tx.send(Err("Esta versão do WebView2 não suporta extensões.".to_string()));
            return;
        };
        let tx2 = tx.clone();
        let r = p7.AddBrowserExtension(
            &HSTRING::from(folder),
            &ProfileAddBrowserExtensionCompletedHandler::create(Box::new(move |hr, ext| {
                let _ = tx2.send(match (hr.is_ok(), ext) {
                    (true, Some(e)) => Ok(extension_info(&e)),
                    _ => Err(format!("A extensão não pôde ser instalada ({hr:?}). Ela precisa ser uma pasta descompactada com manifest.json.")),
                });
                Ok(())
            })),
        );
        if let Err(e) = r {
            let _ = tx.send(Err(e.to_string()));
        }
    })
    .await
    .unwrap_or_else(|| Err("Tempo esgotado.".into()))
}

/// Enables, disables ("enable"/"disable") or removes ("remove") an extension.
pub async fn change_extension(wv: &Webview, id: String, action: String) -> bool {
    ask(wv, move |pv, tx| {
        let tx2 = tx.clone();
        let started = with_extensions(&pv, move |list| unsafe {
            let Some(e) = list.into_iter().find(|e| extension_info(e).id == id) else {
                let _ = tx2.send(false);
                return;
            };
            let tx3 = tx2.clone();
            let r = if action == "remove" {
                e.Remove(&BrowserExtensionRemoveCompletedHandler::create(Box::new(
                    move |hr| {
                        let _ = tx3.send(hr.is_ok());
                        Ok(())
                    },
                )))
            } else {
                e.Enable(
                    action == "enable",
                    &BrowserExtensionEnableCompletedHandler::create(Box::new(move |hr| {
                        let _ = tx3.send(hr.is_ok());
                        Ok(())
                    })),
                )
            };
            if r.is_err() {
                let _ = tx2.send(false);
            }
        });
        if !started {
            let _ = tx.send(false);
        }
    })
    .await
    .unwrap_or(false)
}

/// Working set, in MB, of all processes of a webview's WebView2 environment.
pub async fn memory_mb(wv: &Webview) -> u64 {
    let pids = ask(wv, |pv, tx| unsafe {
        let mut pids = vec![];
        if let Ok(infos) = pv
            .environment()
            .cast::<ICoreWebView2Environment8>()
            .and_then(|e| e.GetProcessInfos())
        {
            let mut n = 0u32;
            let _ = infos.Count(&mut n);
            for i in 0..n {
                if let Ok(info) = infos.GetValueAtIndex(i) {
                    let mut pid = 0i32;
                    if info.ProcessId(&mut pid).is_ok() {
                        pids.push(pid as u32);
                    }
                }
            }
        }
        let _ = tx.send(pids);
    })
    .await
    .unwrap_or_default();
    let mut bytes = 0u64;
    for pid in pids {
        use windows::Win32::{
            Foundation::CloseHandle,
            System::{
                ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
                Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
            },
        };
        unsafe {
            if let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut c = PROCESS_MEMORY_COUNTERS::default();
                if K32GetProcessMemoryInfo(
                    h,
                    &mut c,
                    std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                )
                .as_bool()
                {
                    bytes += c.WorkingSetSize as u64;
                }
                let _ = CloseHandle(h);
            }
        }
    }
    bytes / 1_048_576
}

/// A JPEG snapshot of what the webview shows, used to blur the page behind
/// the palette and to animate Peek away.
pub async fn capture_jpeg(wv: &Webview) -> Option<Vec<u8>> {
    use windows::Win32::{
        System::Com::{IStream, STATFLAG_NONAME, STATSTG, STREAM_SEEK_SET},
        UI::Shell::SHCreateMemStream,
    };
    ask(wv, |pv, tx| unsafe {
        let (Ok(core), Some(stream)) = (pv.controller().CoreWebView2(), SHCreateMemStream(None))
        else {
            let _ = tx.send(None);
            return;
        };
        let read: IStream = stream.clone();
        let tx2 = tx.clone();
        let r = core.CapturePreview(
            COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_JPEG,
            &stream,
            &CapturePreviewCompletedHandler::create(Box::new(move |hr| {
                let mut bytes = None;
                if hr.is_ok() {
                    let mut stat = STATSTG::default();
                    if read.Stat(&mut stat, STATFLAG_NONAME).is_ok()
                        && read.Seek(0, STREAM_SEEK_SET, None).is_ok()
                    {
                        let mut buf = vec![0u8; stat.cbSize as usize];
                        let mut n = 0u32;
                        if read
                            .Read(buf.as_mut_ptr().cast(), buf.len() as u32, Some(&mut n))
                            .is_ok()
                        {
                            buf.truncate(n as usize);
                            bytes = Some(buf);
                        }
                    }
                }
                let _ = tx2.send(bytes);
                Ok(())
            })),
        );
        if r.is_err() {
            let _ = tx.send(None);
        }
    })
    .await
    .flatten()
}

/// Rounds a page card's corners (Crest's 13pt page radius) by clipping the
/// window that hosts the webview. Called after every resize.
pub fn round_corners(wv: &Webview, radius: f64) {
    use windows::Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn},
        UI::WindowsAndMessaging::GetClientRect,
    };
    let scale = wv.window().scale_factor().unwrap_or(1.0);
    let d = (radius * 2.0 * scale).round() as i32;
    let _ = wv.with_webview(move |pv| unsafe {
        let mut host = HWND::default();
        if pv.controller().ParentWindow(&mut host).is_err() || host.is_invalid() {
            return;
        }
        let mut r = RECT::default();
        if GetClientRect(host, &mut r).is_err() {
            return;
        }
        // The window owns the region from here on.
        let rgn = CreateRoundRectRgn(0, 0, r.right + 1, r.bottom + 1, d, d);
        let _ = SetWindowRgn(host, Some(rgn), true);
    });
}
