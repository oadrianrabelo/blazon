//! Page transitions: switching tabs cross-fades the page and switching Spaces
//! slides it, like Crest. Site pages are drawn by WebView2 in their own windows,
//! so the old page is frozen into a picture on a transparent layer ("fx") above
//! the page card, the switch happens underneath, and the picture animates away.

use crate::*;
use tauri::EventTarget;

pub(crate) fn fx_of(win: &str) -> String {
    format!("fx{}", sfx(win))
}

/// Signals that the fx layer has painted the picture.
static READY: Mutex<Option<std::sync::mpsc::Sender<()>>> = Mutex::new(None);

fn enabled(app: &AppHandle) -> bool {
    with_data(app, |d| {
        d.settings
            .appearance
            .get("pageTransitions")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    })
}

/// Covers the page card with a picture of the current page before a switch.
/// `kind` is "tab" or "space"; `dir` is the side the new Space comes from.
/// Returns true when the cover is up: call `reveal` once the switch is done.
pub(crate) async fn cover(app: &AppHandle, kind: &str, dir: i32) -> bool {
    #[cfg(windows)]
    {
        use base64::Engine;
        if !enabled(app) {
            return false;
        }
        let Some(win) = window(app) else { return false };
        let wl = win.label().to_string();
        let space = state(app).windows.lock().unwrap().get(&wl).cloned();
        let Some(space) = space else { return false };
        let shown = with_data(app, |d| d.visible_in(&space));
        // Split view and settings keep switching instantly.
        if shown.len() != 1 || state(app).panel.lock().unwrap().contains_key(&wl) {
            return false;
        }
        let Some(page) = app.get_webview(&label(shown[0])) else {
            return false;
        };
        let (Ok(pos), Ok(size)) = (page.position(), page.size()) else {
            return false;
        };
        let shot = tauri::async_runtime::spawn(async move { webview2::capture_jpeg(&page).await });
        let jpeg = match tokio_timeout(shot, 180).await {
            Some(Some(j)) => j,
            _ => return false,
        };
        let Some(fx) = app.get_webview(&fx_of(&wl)) else {
            // First switch in this window: make the layer; it is used from the next one.
            let _ = win.add_child(
                interface_webview(fx_of(&wl), WebviewUrl::App("fx.html".into()))
                    .background_color(tauri::window::Color(0, 0, 0, 0)),
                pos,
                size,
            );
            if let Some(fx) = app.get_webview(&fx_of(&wl)) {
                let _ = fx.hide();
            }
            return false;
        };
        let _ = fx.set_position(pos);
        let _ = fx.set_size(size);
        let (tx, rx) = std::sync::mpsc::channel();
        *READY.lock().unwrap() = Some(tx);
        let image = format!(
            "data:image/jpeg;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(jpeg)
        );
        let _ = app.emit_to(
            EventTarget::webview(fx_of(&wl)),
            "fx-cover",
            serde_json::json!({ "image": image, "kind": kind, "dir": dir }),
        );
        let painted = tauri::async_runtime::spawn_blocking(move || {
            rx.recv_timeout(std::time::Duration::from_millis(160))
                .is_ok()
        })
        .await
        .unwrap_or(false);
        if !painted {
            return false;
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

#[tauri::command]
pub(crate) async fn fx_ready() {
    if let Some(tx) = READY.lock().unwrap().take() {
        let _ = tx.send(());
    }
}

#[tauri::command]
pub(crate) async fn fx_done(app: AppHandle) {
    if let Some(fx) = app.get_webview(&fx_of(&focused_label(&app))) {
        let _ = fx.hide();
    }
}
