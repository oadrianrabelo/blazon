//! Extension side panel, after Crest's ExtensionSidePanel: an extension's
//! panel, popup or options page shown in a column beside the page, in the
//! Space's own WebView2 profile so it talks to the installed extension.
//! (WebView2 has no toolbar for extension buttons, so this is how they open.)

use crate::*;
use tauri::EventTarget;

/// Width of the panel and height of the header the sidebar draws above it,
/// in logical pixels.
pub(crate) const SIDE_WIDTH: f64 = 360.0;
pub(crate) const SIDE_HEADER: f64 = 40.0;

#[derive(Clone)]
pub(crate) struct Panel {
    space: String,
    name: String,
}

/// A rectangle in physical pixels.
pub(crate) type Rect = (PhysicalPosition<i32>, PhysicalSize<u32>);

/// Open panel per window label.
static PANELS: Mutex<Option<HashMap<String, Panel>>> = Mutex::new(None);

pub(crate) fn side_of(win: &str) -> String {
    format!("side{}", sfx(win))
}

/// The panel open in window `win`, if it belongs to the Space shown there.
pub(crate) fn panel_for(win: &str, space: &str) -> Option<Panel> {
    PANELS
        .lock()
        .unwrap()
        .as_ref()?
        .get(win)
        .filter(|p| p.space == space)
        .cloned()
}

/// Splits the page card into the page and the panel column beside it.
pub(crate) fn split(page: Rect, scale: f64) -> (Rect, Rect) {
    let (pos, size) = page;
    let gap = (CARD_MARGIN * scale) as u32;
    let w = ((SIDE_WIDTH * scale) as u32).min(size.width / 2);
    let header = (SIDE_HEADER * scale) as u32;
    let page_w = size.width.saturating_sub(w + gap);
    (
        (pos, PhysicalSize::new(page_w, size.height)),
        (
            PhysicalPosition::new(pos.x + (page_w + gap) as i32, pos.y + header as i32),
            PhysicalSize::new(w, size.height.saturating_sub(header)),
        ),
    )
}

/// Tells the sidebar where to draw the panel's header (CSS pixels), or that
/// there is no panel.
pub(crate) fn tell_ui(
    app: &AppHandle,
    win: &str,
    panel: Option<&Panel>,
    rect: Option<Rect>,
    scale: f64,
) {
    let payload = match (panel, rect) {
        (Some(p), Some((pos, size))) => serde_json::json!({
            "name": p.name,
            "x": pos.x as f64 / scale,
            "y": pos.y as f64 / scale - SIDE_HEADER,
            "w": size.width as f64 / scale,
        }),
        _ => serde_json::Value::Null,
    };
    let _ = app.emit_to(EventTarget::webview(ui_of(win)), "side-panel", payload);
}

#[derive(Serialize)]
pub(crate) struct PanelPage {
    id: String,
    name: String,
    page: String,
    /// "panel", "popup" or "options".
    kind: &'static str,
}

/// The page an extension can show in the panel, from its manifest.json.
pub(crate) fn panel_page(manifest: &serde_json::Value) -> Option<(String, &'static str)> {
    let s = |p: &str| {
        manifest
            .pointer(p)
            .and_then(|v| v.as_str())
            .filter(|v| !v.is_empty())
            .map(|v| v.trim_start_matches('/').to_string())
    };
    s("/side_panel/default_path")
        .map(|p| (p, "panel"))
        .or_else(|| s("/action/default_popup").map(|p| (p, "popup")))
        .or_else(|| s("/browser_action/default_popup").map(|p| (p, "popup")))
        .or_else(|| s("/options_ui/page").map(|p| (p, "options")))
        .or_else(|| s("/options_page").map(|p| (p, "options")))
}

/// Enabled extensions of a Space that have a page to show in the panel.
#[cfg(windows)]
#[tauri::command]
pub(crate) async fn extension_panels(
    app: AppHandle,
    space: String,
) -> Result<Vec<PanelPage>, String> {
    let wv = profile::space_webview(&app, &space)?;
    let installed = webview2::list_extensions(&wv).await;
    let dirs = with_data(&app, |d| d.extension_dirs.clone());
    Ok(installed
        .into_iter()
        .filter(|e| e.enabled)
        .filter_map(|e| {
            let dir = dirs.get(&e.id)?;
            let manifest: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(PathBuf::from(dir).join("manifest.json")).ok()?,
            )
            .ok()?;
            let (page, kind) = panel_page(&manifest)?;
            Some(PanelPage {
                id: e.id,
                name: e.name,
                page,
                kind,
            })
        })
        .collect())
}

#[cfg(not(windows))]
#[tauri::command]
pub(crate) async fn extension_panels() -> Result<Vec<PanelPage>, String> {
    Err("Extensões só no Windows.".into())
}

/// Opens an extension page in the panel of the focused window.
#[tauri::command]
pub(crate) async fn open_side_panel(
    app: AppHandle,
    id: String,
    page: String,
    name: String,
) -> Result<(), String> {
    let win = window(&app).ok_or("Nenhuma janela.")?;
    let wl = win.label().to_string();
    let space = state(&app)
        .windows
        .lock()
        .unwrap()
        .get(&wl)
        .cloned()
        .unwrap_or_else(|| with_data(&app, |d| d.active_space.clone()));
    let url: Url = format!("chrome-extension://{id}/{page}")
        .parse()
        .map_err(|_| "Endereço da extensão inválido.".to_string())?;
    if let Some(old) = app.get_webview(&side_of(&wl)) {
        let _ = old.close();
    }
    PANELS
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(
            wl.clone(),
            Panel {
                space: space.clone(),
                name,
            },
        );
    let private = with_data(&app, |d| d.is_private(&space));
    // Same profile options as the Space's tabs, so it shares their extensions.
    let builder = WebviewBuilder::new(side_of(&wl), WebviewUrl::External(url))
        .incognito(private)
        .data_directory(state(&app).root.join("spaces").join(&space))
        .browser_extensions_enabled(true);
    let settings = with_data(&app, |d| d.settings.clone());
    let rects = layout_rects(&win, &settings, false).ok_or("Janela indisponível.")?;
    let (_, side) = split(rects[1], win.scale_factor().unwrap_or(1.0));
    win.add_child(builder, side.0, side.1)
        .map_err(|e| e.to_string())?;
    layout(&app, false);
    Ok(())
}

#[tauri::command]
pub(crate) async fn close_side_panel(app: AppHandle) {
    let wl = focused_label(&app);
    if let Some(p) = PANELS.lock().unwrap().as_mut() {
        p.remove(&wl);
    }
    if let Some(wv) = app.get_webview(&side_of(&wl)) {
        let _ = wv.close();
    }
    tell_ui(&app, &wl, None, None, 1.0);
    layout(&app, false);
}

#[cfg(test)]
mod tests {
    use super::panel_page;

    #[test]
    fn picks_the_best_page_from_the_manifest() {
        let m = serde_json::json!({ "side_panel": { "default_path": "panel.html" }, "action": { "default_popup": "popup.html" } });
        assert_eq!(panel_page(&m), Some(("panel.html".into(), "panel")));
        let m = serde_json::json!({ "browser_action": { "default_popup": "/ui/popup.html" } });
        assert_eq!(panel_page(&m), Some(("ui/popup.html".into(), "popup")));
        let m = serde_json::json!({ "options_ui": { "page": "options.html" } });
        assert_eq!(panel_page(&m), Some(("options.html".into(), "options")));
        assert_eq!(panel_page(&serde_json::json!({ "name": "x" })), None);
    }
}
