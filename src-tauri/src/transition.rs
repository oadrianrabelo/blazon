//! Page transitions: switching tabs cross-fades the page and switching Spaces
//! slides it, like Crest. Site pages are drawn by WebView2 in their own windows,
//! so the old page is frozen into a picture on a transparent layer ("fx") above
//! the page card, the switch happens underneath, and the picture animates away.
//!
//! Taking the picture costs a few frames, so the sidebar asks for it as soon as
//! the pointer goes down on a tab or a Space (`prepare_switch`): by the time the
//! click lands the picture is already painted on the hidden layer, and the
//! switch starts at once.

use crate::*;
use std::time::Instant;
use tauri::{EventTarget, Webview};

pub(crate) fn fx_of(win: &str) -> String {
    format!("fx{}", sfx(win))
}

/// Signals that the fx layer has painted the picture a `cover` waits for.
static READY: Mutex<Option<std::sync::mpsc::Sender<()>>> = Mutex::new(None);

/// A picture painted ahead of a switch (see `prepare_switch`).
struct Preload {
    window: String,
    tab: u64,
    url: String,
    at: Instant,
    painted: bool,
}
static PRELOAD: Mutex<Option<Preload>> = Mutex::new(None);

/// A preloaded picture is used for this long after the pointer went down.
const PRELOAD_FRESH_MS: u128 = 1500;

fn enabled(app: &AppHandle) -> bool {
    with_data(app, |d| {
        d.settings
            .appearance
            .get("pageTransitions")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    })
}

/// The one page the focused window shows, when a transition applies to it:
/// transitions on, a single page (not Split View) and no settings panel.
fn shown_page(app: &AppHandle) -> Option<(Window, Tab)> {
    if !enabled(app) {
        return None;
    }
    let win = window(app)?;
    let wl = win.label().to_string();
    let space = state(app).windows.lock().unwrap().get(&wl).cloned()?;
    let shown = with_data(app, |d| d.visible_in(&space));
    if shown.len() != 1 || state(app).panel.lock().unwrap().contains_key(&wl) {
        return None;
    }
    let tab = with_data(app, |d| d.tabs.iter().find(|t| t.id == shown[0]).cloned())?;
    Some((win, tab))
}

/// Takes the picture of a page as a data URL.
#[cfg(windows)]
async fn picture(page: Webview, ms: u64) -> Option<String> {
    use base64::Engine;
    let shot = tauri::async_runtime::spawn(async move { webview2::capture_jpeg(&page).await });
    let jpeg = tokio_timeout(shot, ms).await.flatten()?;
    Some(format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(jpeg)
    ))
}

/// The fx layer of a window, made (hidden) the first time it is needed.
fn fx_layer(app: &AppHandle, win: &Window, page: &Webview) -> Option<Webview> {
    let wl = win.label().to_string();
    if let Some(fx) = app.get_webview(&fx_of(&wl)) {
        return Some(fx);
    }
    let (Ok(pos), Ok(size)) = (page.position(), page.size()) else {
        return None;
    };
    let _ = win.add_child(
        interface_webview(fx_of(&wl), WebviewUrl::App("fx.html".into()))
            .background_color(tauri::window::Color(0, 0, 0, 0)),
        pos,
        size,
    );
    let fx = app.get_webview(&fx_of(&wl))?;
    let _ = fx.hide();
    // Its first picture comes with the next switch: the page is still loading.
    None
}

/// Paints a picture of the current page on the hidden fx layer, ahead of a
/// switch the person is about to make (pointer down on a tab or a Space).
#[tauri::command]
pub(crate) async fn prepare_switch(app: AppHandle) {
    #[cfg(windows)]
    {
        let Some((win, tab)) = shown_page(&app) else {
            return;
        };
        if let Some(p) = PRELOAD.lock().unwrap().as_ref() {
            // A picture of this page from a moment ago is still good.
            if p.tab == tab.id && p.url == tab.url && p.at.elapsed().as_millis() < 400 {
                return;
            }
        }
        let Some(page) = app.get_webview(&label(tab.id)) else {
            return;
        };
        let Some(fx) = fx_layer(&app, &win, &page) else {
            return;
        };
        webview2::set_memory_low(&fx, false);
        let Some(image) = picture(page, 250).await else {
            return;
        };
        *PRELOAD.lock().unwrap() = Some(Preload {
            window: win.label().to_string(),
            tab: tab.id,
            url: tab.url.clone(),
            at: Instant::now(),
            painted: false,
        });
        let radius = with_data(&app, |d| page_radius(&d.settings));
        let _ = app.emit_to(
            EventTarget::webview(fx.label()),
            "fx-preload",
            serde_json::json!({ "image": image, "radius": radius }),
        );
    }
    #[cfg(not(windows))]
    let _ = app;
}

/// Covers the page card with a picture of the current page before a switch.
/// `kind` is "tab" or "space"; `dir` is the side the new Space comes from.
/// Returns true when the cover is up: call `reveal` once the switch is done.
pub(crate) async fn cover(app: &AppHandle, kind: &str, dir: i32) -> bool {
    #[cfg(windows)]
    {
        let Some((win, tab)) = shown_page(app) else {
            return false;
        };
        let wl = win.label().to_string();
        let Some(page) = app.get_webview(&label(tab.id)) else {
            return false;
        };
        let (Ok(pos), Ok(size)) = (page.position(), page.size()) else {
            return false;
        };
        let Some(fx) = fx_layer(app, &win, &page) else {
            return false;
        };
        let _ = fx.set_position(pos);
        let _ = fx.set_size(size);
        webview2::set_memory_low(&fx, false);
        let preloaded = PRELOAD.lock().unwrap().take().is_some_and(|p| {
            p.painted
                && p.window == wl
                && p.tab == tab.id
                && p.url == tab.url
                && p.at.elapsed().as_millis() < PRELOAD_FRESH_MS
        });
        if preloaded {
            // Already painted: show it and switch right away.
            let _ = app.emit_to(
                EventTarget::webview(fx_of(&wl)),
                "fx-arm",
                serde_json::json!({ "kind": kind, "dir": dir }),
            );
        } else {
            // Keyboard switches and the like: take the picture now, but never
            // hold the switch up for long.
            let Some(image) = picture(page, 120).await else {
                return false;
            };
            let (tx, rx) = std::sync::mpsc::channel();
            *READY.lock().unwrap() = Some(tx);
            let radius = with_data(app, |d| page_radius(&d.settings));
            let _ = app.emit_to(
                EventTarget::webview(fx_of(&wl)),
                "fx-cover",
                serde_json::json!({ "image": image, "kind": kind, "dir": dir, "radius": radius }),
            );
            let painted = tauri::async_runtime::spawn_blocking(move || {
                rx.recv_timeout(std::time::Duration::from_millis(120))
                    .is_ok()
            })
            .await
            .unwrap_or(false);
            if !painted {
                return false;
            }
        }
        webview2::bring_to_front(&fx);
        let _ = fx.show();
        true
    }
    #[cfg(not(windows))]
    {
        let _ = (app, kind, dir);
        false
    }
}

#[cfg(windows)]
async fn tokio_timeout<T: Send + 'static>(
    task: tauri::async_runtime::JoinHandle<T>,
    ms: u64,
) -> Option<T> {
    let (tx, rx) = std::sync::mpsc::channel();
    tauri::async_runtime::spawn(async move {
        if let Ok(v) = task.await {
            let _ = tx.send(v);
        }
    });
    tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(std::time::Duration::from_millis(ms)).ok()
    })
    .await
    .ok()
    .flatten()
}

/// Lets the picture animate away now that the new page is in place.
pub(crate) fn reveal(app: &AppHandle) {
    let wl = focused_label(app);
    let _ = app.emit_to(EventTarget::webview(fx_of(&wl)), "fx-reveal", ());
}

/// The fx layer painted a picture: the one a `cover` waits for, or a preload.
#[tauri::command]
pub(crate) async fn fx_ready(preload: Option<bool>) {
    if preload == Some(true) {
        if let Some(p) = PRELOAD.lock().unwrap().as_mut() {
            p.painted = true;
        }
    } else if let Some(tx) = READY.lock().unwrap().take() {
        let _ = tx.send(());
    }
}

#[tauri::command]
pub(crate) async fn fx_done(app: AppHandle) {
    if let Some(fx) = app.get_webview(&fx_of(&focused_label(&app))) {
        let _ = fx.hide();
        // Between switches the layer only holds a picture: keep it light.
        #[cfg(windows)]
        webview2::set_memory_low(&fx, true);
    }
}
