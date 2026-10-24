//! Overlays drawn above the page: the command palette and Peek (a link
//! previewed in a floating card). Both use a transparent UI webview over the
//! whole window; Peek adds a web content webview on top of it.

use super::*;

const OVERLAY: &str = "overlay";
const PEEK: &str = "peek";

/// Each window has its own overlay and Peek webviews.
pub fn overlay_of(win: &str) -> String {
    format!("{OVERLAY}{}", sfx(win))
}
pub fn peek_of(win: &str) -> String {
    format!("{PEEK}{}", sfx(win))
}

/// Esc closes Peek from inside the previewed page.
const PEEK_JS: &str = r#"addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && window.top === window) { e.preventDefault(); window.open('https://blazon-cmd.invalid/closepeek'); }
}, true);"#;

/// Peek's card: inset from the page card, leaving room for its toolbar.
pub fn peek_rect(
    content: (PhysicalPosition<i32>, PhysicalSize<u32>),
    scale: f64,
) -> (PhysicalPosition<i32>, PhysicalSize<u32>) {
    let (pos, size) = content;
    let side = (size.width as f64 * 0.07) as i32;
    let top = (48.0 * scale) as i32;
    let bottom = (24.0 * scale) as i32;
    (
        PhysicalPosition::new(pos.x + side, pos.y + top),
        PhysicalSize::new(
            (size.width as i32 - 2 * side).max(100) as u32,
            (size.height as i32 - top - bottom).max(100) as u32,
        ),
    )
}

fn close_existing(app: &AppHandle) {
    let win = focused_label(app);
    for l in [peek_of(&win), overlay_of(&win)] {
        if let Some(wv) = app.get_webview(&l) {
            let _ = wv.close();
        }
    }
}

fn add_overlay(win: &Window, mode: &str) -> tauri::Result<()> {
    let size = win.inner_size()?;
    let wv = win.add_child(
        WebviewBuilder::new(
            overlay_of(win.label()),
            WebviewUrl::App(format!("overlay.html#{mode}").into()),
        )
        .background_color(tauri::window::Color(0, 0, 0, 0)),
        PhysicalPosition::new(0, 0),
        size,
    )?;
    let _ = wv.set_focus();
    Ok(())
}

pub fn open_palette(app: &AppHandle) {
    let Some(win) = window(app) else { return };
    close_existing(app);
    let _ = add_overlay(&win, "palette");
}

pub fn open_peek(app: &AppHandle, url: &str) {
    let Ok(url) = Url::parse(url) else { return };
    let Some(win) = window(app) else { return };
    close_existing(app);
    if add_overlay(&win, "peek").is_err() {
        return;
    }
    let (space, settings) = with_data(app, |d| (d.active_space.clone(), d.settings.clone()));
    let private = with_data(app, |d| d.is_private(&space));
    let app_nw = app.clone();
    let builder = WebviewBuilder::new(peek_of(win.label()), WebviewUrl::External(url))
        .incognito(private)
        .data_directory(state(app).root.join("spaces").join(&space))
        .initialization_script(PEEK_JS)
        .browser_extensions_enabled(true)
        .zoom_hotkeys_enabled(true)
        .on_new_window(move |url, _| {
            let app = app_nw.clone();
            if url.host_str() == Some(CMD_HOST) {
                spawn(async move { close_overlay(app).await });
            } else {
                let url = url.to_string();
                spawn(async move { open_tab(&app, url).await });
            }
            NewWindowResponse::Deny
        });
    let collapsed = settings.floating_sidebar && !state(app).peek.load(Ordering::Relaxed);
    let Some(rects) = layout_rects(&win, &settings, collapsed) else {
        return;
    };
    let (pos, size) = peek_rect(rects[1], win.scale_factor().unwrap_or(1.0));
    if let Ok(wv) = win.add_child(builder, pos, size) {
        #[cfg(windows)]
        webview2::install(app, &wv, None, None, settings.passwords);
        let _ = wv.set_focus();
    }
}

#[tauri::command]
pub async fn open_overlay(app: AppHandle, mode: String, url: Option<String>) {
    match mode.as_str() {
        "peek" => open_peek(&app, url.as_deref().unwrap_or("")),
        _ => open_palette(&app),
    }
}

#[tauri::command]
pub async fn close_overlay(app: AppHandle) {
    close_existing(&app);
    focus_active(&app);
}

/// Promotes the previewed page to a regular tab.
#[tauri::command]
pub async fn peek_to_tab(app: AppHandle) {
    let url = app
        .get_webview(&peek_of(&focused_label(&app)))
        .and_then(|wv| wv.url().ok());
    close_existing(&app);
    if let Some(url) = url {
        open_tab(&app, url.to_string()).await;
        focus_active(&app);
    }
}

/// Address of the page in Peek, for the toolbar.
#[tauri::command]
pub async fn peek_url(app: AppHandle) -> Option<String> {
    app.get_webview(&peek_of(&focused_label(&app)))
        .and_then(|wv| wv.url().ok())
        .map(|u| u.to_string())
}
