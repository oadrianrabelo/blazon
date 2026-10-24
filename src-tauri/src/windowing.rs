//! Windows and layout: which Space each window shows and where every webview sits.

use crate::*;

/// Label suffix of a window's own webviews ("" for the main window).
pub(crate) fn sfx(win: &str) -> String {
    if win == WINDOW {
        String::new()
    } else {
        format!("-{win}")
    }
}
pub(crate) fn ui_of(win: &str) -> String {
    format!("{UI}{}", sfx(win))
}
pub(crate) fn panel_of(win: &str) -> String {
    format!("{PANEL}{}", sfx(win))
}

pub(crate) fn focused_label(app: &AppHandle) -> String {
    let f = state(app).focused.lock().unwrap().clone();
    if app.get_window(&f).is_some() {
        f
    } else {
        WINDOW.to_string()
    }
}

/// The window the person is using.
pub(crate) fn window(app: &AppHandle) -> Option<Window> {
    app.get_window(&focused_label(app))
        .or_else(|| app.get_window(WINDOW))
}

/// Tabs visible in any window.
pub(crate) fn shown_tabs(app: &AppHandle) -> Vec<u64> {
    let spaces: Vec<String> = state(app)
        .windows
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect();
    with_data(app, |d| {
        spaces.iter().flat_map(|s| d.visible_in(s)).collect()
    })
}

/// Keeps the focused window on the active Space. When that Space is already
/// open in another window, that window comes forward instead.
pub(crate) fn sync_windows(app: &AppHandle) {
    let focused = focused_label(app);
    let want = with_data(app, |d| d.active_space.clone());
    let s = state(app);
    let mut wins = s.windows.lock().unwrap();
    if wins.get(&focused) == Some(&want) {
        return;
    }
    let other = wins
        .iter()
        .find(|(l, sp)| **sp == want && **l != focused)
        .map(|(l, _)| l.clone());
    match other {
        Some(other) => {
            drop(wins);
            *s.focused.lock().unwrap() = other.clone();
            if let Some(w) = app.get_window(&other) {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }
        None => {
            wins.insert(focused, want);
        }
    }
}

/// Corner radius of the page card (Crest's BrowserChromeLayout.pageCornerRadius)
/// and of Peek's card, in logical pixels.
pub(crate) const PAGE_RADIUS: f64 = 13.0;
pub(crate) const PEEK_RADIUS: f64 = 12.0;

/// Height of the top bar when tabs are horizontal (Appearance › Tabs: top):
/// the toolbar row and the tab row, as in Safari.
pub(crate) const TOPBAR_HEIGHT: f64 = 84.0;

/// Appearance › Tabs: on the side (Crest's sidebar) or along the top.
pub(crate) fn horizontal(settings: &Settings) -> bool {
    settings
        .appearance
        .get("tabLayout")
        .and_then(|v| v.as_str())
        == Some("horizontal")
}

/// UI and page rectangles in physical pixels. The UI covers the whole window
/// (sidebar or top bar plus the Space background); the page sits on it as an
/// inset card.
pub(crate) fn layout_rects(
    win: &Window,
    settings: &Settings,
    collapsed: bool,
) -> Option<[(PhysicalPosition<i32>, PhysicalSize<u32>); 2]> {
    let size = win.inner_size().ok()?;
    let scale = win.scale_factor().ok()?;
    let m = (CARD_MARGIN * scale) as u32;
    if horizontal(settings) {
        let bar = if collapsed { FLOAT_EDGE } else { TOPBAR_HEIGHT };
        let bh = ((bar * scale) as u32).min(size.height);
        return Some([
            (PhysicalPosition::new(0, 0), size),
            (
                PhysicalPosition::new(m as i32, bh as i32),
                PhysicalSize::new(
                    size.width.saturating_sub(2 * m),
                    size.height.saturating_sub(bh + m),
                ),
            ),
        ]);
    }
    let right = settings.sidebar_right;
    let width = if collapsed {
        FLOAT_EDGE
    } else {
        settings.sidebar_width.clamp(200.0, 420.0)
    };
    let sw = ((width * scale) as u32).min(size.width);
    let cw = size.width.saturating_sub(sw + m);
    let ch = size.height.saturating_sub(2 * m);
    let content_x = if right { m } else { sw };
    Some([
        (PhysicalPosition::new(0, 0), size),
        (
            PhysicalPosition::new(content_x as i32, m as i32),
            PhysicalSize::new(cw, ch),
        ),
    ])
}

/// Windows whose page is hidden behind a picture of itself, so a menu or
/// dialog of the top bar can spread over the page card (see `set_page_hidden`).
static COVERED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The top bar is too short for its menus and dialogs: while one is open the
/// UI shows a picture of the page (page_snapshot) and the live page steps aside.
#[tauri::command]
pub(crate) async fn set_page_hidden(app: AppHandle, webview: tauri::Webview, hidden: bool) {
    let wl = webview.window().label().to_string();
    {
        let mut c = COVERED.lock().unwrap();
        c.retain(|l| *l != wl);
        if hidden {
            c.push(wl);
        }
    }
    layout(&app, false);
}

/// Creates the visible tabs' webviews if needed, shows them and hides the rest.
pub(crate) fn apply_layout(app: &AppHandle) {
    layout(app, true);
}

/// `create` must be false on the main thread: creating a webview there deadlocks on Windows.
pub(crate) fn layout(app: &AppHandle, create: bool) {
    sync_windows(app);
    let wins = state(app).windows.lock().unwrap().clone();
    for (win, space) in wins {
        layout_window(app, &win, &space, create);
    }
}

pub(crate) fn layout_window(app: &AppHandle, wl: &str, space: &str, create: bool) {
    let Some(win) = app.get_window(wl) else {
        return;
    };
    let (visible, settings) = with_data(app, |d| {
        let ids = d.visible_in(space);
        let tabs: Vec<Tab> = d
            .tabs
            .iter_mut()
            .filter(|t| ids.contains(&t.id))
            .map(|t| {
                t.last_active = now();
                t.clone()
            })
            .collect();
        let ordered: Vec<Tab> = ids
            .iter()
            .filter_map(|id| tabs.iter().find(|t| t.id == *id).cloned())
            .collect();
        (ordered, d.settings.clone())
    });
    let s = state(app);
    let focused = focused_label(app) == wl;
    let collapsed = settings.floating_sidebar && !(focused && s.peek.load(Ordering::Relaxed));
    let Some(rects) = layout_rects(&win, &settings, collapsed) else {
        return;
    };
    let panel = s.panel.lock().unwrap().get(wl).cloned();
    let covered = COVERED.lock().unwrap().iter().any(|l| l == wl);
    let (ui_l, panel_l, overlay_l, peek_l) = (
        ui_of(wl),
        panel_of(wl),
        overlay::overlay_of(wl),
        overlay::peek_of(wl),
    );
    let scale = win.scale_factor().unwrap_or(1.0);
    // An extension side panel takes a column beside the page.
    let side_panel = sidepanel::panel_for(wl, space);
    let side_l = sidepanel::side_of(wl);
    let (page_rect, side_rect) = match &side_panel {
        Some(_) => {
            let (p, s) = sidepanel::split(rects[1], scale);
            (p, Some(s))
        }
        None => (rects[1], None),
    };
    sidepanel::tell_ui(app, wl, side_panel.as_ref(), side_rect, scale);
    let (pos, size) = page_rect;
    // Split View: equal halves with a card gap between them.
    let n = visible.len().max(1) as u32;
    let gap = (CARD_MARGIN * win.scale_factor().unwrap_or(1.0)) as u32;
    let w = size.width.saturating_sub(gap * (n - 1)) / n;
    let slots: Vec<(String, PhysicalPosition<i32>)> = visible
        .iter()
        .enumerate()
        .map(|(i, t)| {
            (
                label(t.id),
                PhysicalPosition::new(pos.x + (i as u32 * (w + gap)) as i32, pos.y),
            )
        })
        .collect();
    for tab in &visible {
        match app.get_webview(&label(tab.id)) {
            // The Space moved to this window: bring its pages along.
            Some(wv) if wv.window().label() != wl => {
                let _ = wv.reparent(&win);
            }
            Some(_) => {}
            None if create && panel.is_none() => {
                if let Err(e) = create_tab_webview(app, &win, tab) {
                    diag::error(&format!(
                        "falha ao criar a aba {} ({}): {e}",
                        tab.id,
                        diag::host(&tab.url)
                    ));
                }
            }
            None => {}
        }
    }
    if create && panel.is_some() && app.get_webview(&panel_l).is_none() {
        let section = panel.clone().unwrap_or_default();
        let url = WebviewUrl::App(format!("settings.html#{section}").into());
        if let Err(e) = win.add_child(
            interface_webview(&panel_l, url).disable_drag_drop_handler(),
            pos,
            size,
        ) {
            diag::error(&format!("falha ao abrir as configurações: {e}"));
        }
    }
    for wv in win.webviews() {
        let l = wv.label().to_string();
        if l == ui_l || l == overlay_l {
            let _ = wv.set_position(rects[0].0);
            let _ = wv.set_size(rects[0].1);
        } else if l == panel_l {
            let _ = wv.set_position(pos);
            let _ = wv.set_size(size);
            #[cfg(windows)]
            webview2::round_corners(&wv, PAGE_RADIUS);
        } else if l == peek_l {
            let (p, s) = overlay::peek_rect(rects[1], win.scale_factor().unwrap_or(1.0));
            let _ = wv.set_position(p);
            let _ = wv.set_size(s);
            #[cfg(windows)]
            webview2::round_corners(&wv, PEEK_RADIUS);
        } else if l == transition::fx_of(wl) {
            // Shown and hidden by transition.rs around a switch.
        } else if l == side_l {
            match side_rect {
                Some((p, s)) if panel.is_none() => {
                    let _ = wv.set_position(p);
                    let _ = wv.set_size(s);
                    #[cfg(windows)]
                    webview2::round_corners(&wv, PEEK_RADIUS);
                    let _ = wv.show();
                }
                Some(_) => {
                    let _ = wv.hide();
                }
                // The window now shows another Space: the panel goes away.
                None => {
                    let _ = wv.close();
                }
            }
        } else if panel.is_some() {
            let _ = wv.hide();
        } else if let Some((_, p)) = slots.iter().find(|(s, _)| *s == l) {
            let _ = wv.set_position(*p);
            let _ = wv.set_size(PhysicalSize::new(w, size.height));
            #[cfg(windows)]
            webview2::round_corners(&wv, PAGE_RADIUS);
            if covered {
                let _ = wv.hide();
            } else {
                let _ = wv.show();
            }
        } else {
            let _ = wv.hide();
        }
    }
}

#[tauri::command]
pub(crate) async fn sidebar_peek(app: AppHandle, on: bool) {
    let s = state(&app);
    if s.peek.swap(on, Ordering::Relaxed) != on && with_data(&app, |d| d.settings.floating_sidebar)
    {
        apply_layout(&app);
    }
}

#[tauri::command]
pub(crate) async fn open_panel(app: AppHandle, section: String) {
    let win = focused_label(&app);
    let panel_l = panel_of(&win);
    let previous = state(&app)
        .panel
        .lock()
        .unwrap()
        .insert(win, section.clone());
    match app.get_webview(&panel_l) {
        Some(panel) if previous.is_some() => {
            let _ = app.emit_to(panel_l.as_str(), "section", section);
            let _ = panel.set_focus();
        }
        _ => {
            apply_layout(&app);
            if let Some(panel) = app.get_webview(&panel_l) {
                let _ = panel.set_focus();
            }
        }
    }
}

/// Builds a browser window with its sidebar webview and event handlers.
/// Must not run on the main thread after startup (see `layout`).
/// Appearance › Translucent sidebar (stored in the UI's appearance options).
pub(crate) fn mica_on(settings: &Settings) -> bool {
    settings
        .appearance
        .get("mica")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Puts Windows 11's Mica material (Acrylic on Windows 10) behind the window,
/// which the sidebar shows through when it is translucent.
pub(crate) fn apply_material(win: &Window, on: bool) {
    use tauri::window::{Effect, EffectsBuilder};
    let _ = if on {
        let effect = if diag::windows_build() >= 22000 {
            Effect::Mica
        } else {
            Effect::Acrylic
        };
        win.set_effects(EffectsBuilder::new().effect(effect).build())
    } else {
        win.set_effects(None)
    };
}

pub(crate) fn build_window(app: &AppHandle, wl: &str) -> tauri::Result<Window> {
    let (native_frame, mica) = with_data(app, |d| (d.settings.native_frame, mica_on(&d.settings)));
    let win = tauri::window::WindowBuilder::new(app, wl)
        .title("Blazon")
        .inner_size(1280.0, 820.0)
        .min_inner_size(640.0, 400.0)
        .decorations(native_frame)
        // A see-through window only when the material is on (shadows stay otherwise).
        .transparent(mica)
        .build()?;
    if mica {
        apply_material(&win, true);
    }
    let size = win.inner_size()?;
    win.add_child(
        interface_webview(ui_of(wl), WebviewUrl::App("index.html".into()))
            // Transparent so the Mica material can show through the sidebar.
            .background_color(tauri::window::Color(0, 0, 0, 0))
            // HTML5 drag and drop for tab reordering.
            .disable_drag_drop_handler(),
        PhysicalPosition::new(0, 0),
        size,
    )?;
    let handle = app.clone();
    let label = wl.to_string();
    win.on_window_event(move |e| match e {
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => layout(&handle, false),
        WindowEvent::Focused(true) => {
            *state(&handle).focused.lock().unwrap() = label.clone();
            let space = state(&handle).windows.lock().unwrap().get(&label).cloned();
            if let Some(space) = space {
                with_data(&handle, |d| d.active_space = space);
            }
        }
        WindowEvent::CloseRequested { .. } if label == WINDOW => {
            EXITING.store(true, Ordering::Relaxed);
            save_history(&handle);
            // The main window closes the whole browser.
            handle.exit(0);
        }
        WindowEvent::Destroyed if label != WINDOW && !EXITING.load(Ordering::Relaxed) => {
            let space = state(&handle).windows.lock().unwrap().remove(&label);
            if *state(&handle).focused.lock().unwrap() == label {
                *state(&handle).focused.lock().unwrap() = WINDOW.to_string();
            }
            with_data(&handle, |d| {
                if let Some(space) = &space {
                    // Its pages were destroyed with the window.
                    for t in d.tabs.iter_mut().filter(|t| &t.space == space) {
                        t.awake = false;
                    }
                }
                if let Some(main) = state(&handle).windows.lock().unwrap().get(WINDOW) {
                    d.active_space = main.clone();
                }
            });
            save_windows(&handle);
            commit(&handle);
        }
        _ => {}
    });
    Ok(win)
}

/// Set while the browser shuts down, so closing windows don't forget themselves.
pub(crate) static EXITING: AtomicBool = AtomicBool::new(false);

pub(crate) fn save_windows(app: &AppHandle) {
    let extra: Vec<String> = state(app)
        .windows
        .lock()
        .unwrap()
        .iter()
        .filter(|(l, _)| *l != WINDOW)
        .map(|(_, s)| s.clone())
        .collect();
    with_data(app, |d| d.windows = extra);
    commit(app);
}

/// Adds an empty Space without switching to it.
pub(crate) fn quiet_space(app: &AppHandle) -> String {
    with_data(app, |d| {
        let id = format!("space-{}", d.new_id());
        let h = d.spaces.len() % HOUSES.len();
        d.spaces.push(Space {
            id: id.clone(),
            name: "Novo Space".into(),
            color: HOUSE_ACCENTS[h].into(),
            private: false,
            house: HOUSES[h].into(),
            downloads: String::new(),
            look: serde_json::Value::Null,
            locked: false,
            search: String::new(),
        });
        id
    })
}

/// Opens a window showing `space`. With no Space, uses one that isn't open
/// anywhere, or creates a new one.
#[tauri::command]
pub(crate) async fn new_window(app: AppHandle, space: Option<String>) -> Result<(), String> {
    let shown: Vec<String> = state(&app)
        .windows
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect();
    let space = match space {
        Some(s) => s,
        None => match with_data(&app, |d| {
            d.spaces
                .iter()
                .find(|s| !shown.contains(&s.id))
                .map(|s| s.id.clone())
        }) {
            Some(s) => s,
            None => quiet_space(&app),
        },
    };
    if !ensure_unlocked(&app, &space).await {
        return Err("Space bloqueado.".into());
    }
    // A Space shows in one window at a time: the window it leaves picks another.
    let from = state(&app)
        .windows
        .lock()
        .unwrap()
        .iter()
        .find(|(_, s)| **s == space)
        .map(|(l, _)| l.clone());
    if let Some(from) = from {
        let shown: Vec<String> = state(&app)
            .windows
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect();
        let other = with_data(&app, |d| {
            d.spaces
                .iter()
                .find(|s| !shown.contains(&s.id))
                .map(|s| s.id.clone())
        });
        let other = match other {
            Some(o) => o,
            None => quiet_space(&app),
        };
        state(&app).windows.lock().unwrap().insert(from, other);
    }
    let wl = format!("win-{}", with_data(&app, |d| d.new_id()));
    build_window(&app, &wl).map_err(|e| e.to_string())?;
    state(&app)
        .windows
        .lock()
        .unwrap()
        .insert(wl.clone(), space.clone());
    *state(&app).focused.lock().unwrap() = wl;
    let empty = with_data(&app, |d| {
        d.active_space = space.clone();
        if !d.active_tab.contains_key(&space) {
            if let Some(first) = d.space_tabs(&space).first() {
                d.active_tab.insert(space.clone(), *first);
            }
        }
        !d.active_tab.contains_key(&space)
    });
    if empty {
        open_tab(&app, String::new()).await;
    }
    diag::info("nova janela aberta");
    save_windows(&app);
    apply_layout(&app);
    Ok(())
}

#[tauri::command]
pub(crate) async fn close_panel(app: AppHandle) {
    let win = focused_label(&app);
    state(&app).panel.lock().unwrap().remove(&win);
    if let Some(panel) = app.get_webview(&panel_of(&win)) {
        let _ = panel.close();
    }
    apply_layout(&app);
    focus_active(&app);
}
