//! Quick Window, after Crest's: a link from another app opens in a small
//! window of its own instead of the browser. Its toolbar picks the Space the
//! page belongs to and moves it there as a tab, or the window just closes.

use crate::*;
use tauri::{EventTarget, Webview};

const QUICK: &str = "quick";
const QUICK_UI: &str = "quick-ui";
const QUICK_PAGE: &str = "quick-page";
/// Height of the toolbar above the page, in logical pixels.
const BAR: f64 = 46.0;
const INSET: f64 = 8.0;

/// The Space whose profile the Quick Window's page uses.
static QUICK_SPACE: Mutex<String> = Mutex::new(String::new());

fn page_rect(win: &Window) -> Option<(PhysicalPosition<i32>, PhysicalSize<u32>)> {
    let size = win.inner_size().ok()?;
    let scale = win.scale_factor().unwrap_or(1.0);
    let inset = (INSET * scale) as i32;
    let top = (BAR * scale) as i32;
    Some((
        PhysicalPosition::new(inset, top),
        PhysicalSize::new(
            (size.width as i32 - 2 * inset).max(100) as u32,
            (size.height as i32 - top - inset).max(100) as u32,
        ),
    ))
}

fn layout_quick(app: &AppHandle) {
    let Some(win) = app.get_window(QUICK) else {
        return;
    };
    if let Ok(size) = win.inner_size() {
        if let Some(ui) = app.get_webview(QUICK_UI) {
            let _ = ui.set_position(PhysicalPosition::new(0, 0));
            let _ = ui.set_size(size);
        }
    }
    if let (Some(page), Some((pos, size))) = (app.get_webview(QUICK_PAGE), page_rect(&win)) {
        let _ = page.set_position(pos);
        let _ = page.set_size(size);
        #[cfg(windows)]
        webview2::round_corners(
            &page,
            windowing::page_radius(&with_data(app, |d| d.settings.clone())),
        );
    }
}

fn tell_ui(app: &AppHandle, wv: &Webview) {
    let url = wv.url().map(|u| u.to_string()).unwrap_or_default();
    let _ = app.emit_to(
        EventTarget::webview(QUICK_UI),
        "quick-page",
        serde_json::json!({ "url": url }),
    );
}

/// Opens `url` in the Quick Window, creating it when needed.
pub(crate) fn open(app: &AppHandle, url: &str) {
    let Ok(parsed) = Url::parse(url) else { return };
    if let Some(page) = app.get_webview(QUICK_PAGE) {
        let _ = page.navigate(parsed);
        if let Some(w) = app.get_window(QUICK) {
            let _ = w.unminimize();
            let _ = w.set_focus();
        }
        return;
    }
    let (space, private, passwords) = with_data(app, |d| {
        let space = route_space(d, url);
        (space.clone(), d.is_private(&space), d.settings.passwords)
    });
    *QUICK_SPACE.lock().unwrap() = space.clone();
    let built = (|| -> tauri::Result<()> {
        let win = tauri::window::WindowBuilder::new(app, QUICK)
            .title("Blazon")
            .inner_size(960.0, 680.0)
            .min_inner_size(480.0, 320.0)
            .center()
            .decorations(false)
            .build()?;
        let size = win.inner_size()?;
        win.add_child(
            interface_webview(QUICK_UI, WebviewUrl::App("quick.html".into())),
            PhysicalPosition::new(0, 0),
            size,
        )?;
        let app_nw = app.clone();
        let app_pl = app.clone();
        let builder = WebviewBuilder::new(QUICK_PAGE, WebviewUrl::External(parsed))
            .incognito(private)
            .data_directory(state(app).root.join("spaces").join(&space))
            .additional_browser_args(&browser_args())
            .browser_extensions_enabled(true)
            .zoom_hotkeys_enabled(true)
            .on_new_window(move |url, _| {
                // Links that want a new window stay in the Quick Window.
                if let Some(page) = app_nw.get_webview(QUICK_PAGE) {
                    let _ = page.navigate(url);
                }
                NewWindowResponse::Deny
            })
            .on_page_load(move |wv, _| tell_ui(&app_pl, &wv));
        let (pos, psize) = page_rect(&win).unwrap_or((PhysicalPosition::new(0, 0), size));
        let page = win.add_child(builder, pos, psize)?;
        #[cfg(windows)]
        {
            webview2::install(app, &page, None, None, passwords);
            webview2::round_corners(
                &page,
                windowing::page_radius(&with_data(app, |d| d.settings.clone())),
            );
        }
        let _ = page.set_focus();
        let handle = app.clone();
        win.on_window_event(move |e| {
            if matches!(
                e,
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }
            ) {
                layout_quick(&handle);
            }
        });
        Ok(())
    })();
    if let Err(e) = built {
        diag::error(&format!("janela rápida: {e}"));
        let url = url.to_string();
        let app = app.clone();
        spawn(async move { open_tab(&app, url).await });
    }
}

#[derive(Serialize)]
pub(crate) struct QuickState {
    url: String,
    space: String,
}

#[tauri::command]
pub(crate) async fn quick_state(app: AppHandle) -> QuickState {
    QuickState {
        url: app
            .get_webview(QUICK_PAGE)
            .and_then(|w| w.url().ok())
            .map(|u| u.to_string())
            .unwrap_or_default(),
        space: QUICK_SPACE.lock().unwrap().clone(),
    }
}

/// Moves the page into `space` as a tab of the browser and closes the Quick Window.
#[tauri::command]
pub(crate) async fn quick_to_tab(app: AppHandle, space: String) {
    let url = app
        .get_webview(QUICK_PAGE)
        .and_then(|w| w.url().ok())
        .map(|u| u.to_string());
    if let Some(w) = app.get_window(QUICK) {
        let _ = w.close();
    }
    let Some(url) = url else { return };
    let exists = with_data(&app, |d| d.spaces.iter().any(|s| s.id == space));
    if exists {
        with_data(&app, |d| d.active_space = space);
    }
    open_tab(&app, url).await;
    if let Some(win) = window(&app) {
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
    focus_active(&app);
}

#[tauri::command]
pub(crate) async fn quick_close(app: AppHandle) {
    if let Some(w) = app.get_window(QUICK) {
        let _ = w.close();
    }
}

#[tauri::command]
pub(crate) async fn quick_nav(app: AppHandle, action: String) {
    let Some(page) = app.get_webview(QUICK_PAGE) else {
        return;
    };
    let script = match action.as_str() {
        "back" => "history.back()",
        "forward" => "history.forward()",
        _ => "location.reload()",
    };
    let _ = page.eval(script);
}
