#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri::{
    async_runtime::spawn,
    webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder},
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, Window,
    WindowEvent,
};
use url::Url;

const WINDOW: &str = "main";
const UI: &str = "ui";
const SIDEBAR_WIDTH: f64 = 264.0;
/// Pages can't reach the IPC bridge, so the tab shortcut script asks for
/// commands by opening this host, which `on_new_window` intercepts.
const CMD_HOST: &str = "crest-cmd.invalid";
const NEWTAB_PAGE: &str = "newtab.html";

const SHORTCUTS_JS: &str = r#"(() => {
  if (window.top !== window) return;
  const keys = { t: 'newtab', w: 'close', l: 'address' };
  addEventListener('keydown', (e) => {
    if (!e.ctrlKey || e.altKey) return;
    const k = e.key.toLowerCase();
    const digit = /^Digit[1-9]$/.test(e.code) ? e.code.slice(5) : null;
    let cmd = null;
    if (k === 'tab') cmd = e.shiftKey ? 'prev' : 'next';
    else if (digit) cmd = (e.shiftKey ? 'space' : 'tab') + digit;
    else if (!e.shiftKey && keys[k]) cmd = keys[k];
    if (!cmd) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    window.open('https://crest-cmd.invalid/' + cmd);
  }, true);
})();"#;

#[derive(Clone, Serialize, Deserialize)]
struct Space {
    id: String,
    name: String,
    color: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Tab {
    id: u64,
    space: String,
    /// Empty means the new tab page.
    url: String,
    title: String,
    #[serde(default)]
    pinned: bool,
    #[serde(default, skip_deserializing)]
    loading: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    sidebar_right: bool,
    search: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sidebar_right: false,
            search: "https://duckduckgo.com/?q=".into(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct Data {
    spaces: Vec<Space>,
    tabs: Vec<Tab>,
    active_space: String,
    active_tab: HashMap<String, u64>,
    next_id: u64,
    settings: Settings,
}

impl Data {
    fn new_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    fn active_tab_id(&self) -> Option<u64> {
        self.active_tab.get(&self.active_space).copied()
    }

    fn space_tabs(&self, space: &str) -> Vec<u64> {
        let mut tabs: Vec<&Tab> = self.tabs.iter().filter(|t| t.space == space).collect();
        tabs.sort_by_key(|t| !t.pinned);
        tabs.iter().map(|t| t.id).collect()
    }

    fn repair(&mut self) {
        if self.spaces.is_empty() {
            self.spaces.push(Space {
                id: "personal".into(),
                name: "Pessoal".into(),
                color: "#3b82f6".into(),
            });
        }
        let spaces: Vec<String> = self.spaces.iter().map(|s| s.id.clone()).collect();
        self.tabs.retain(|t| spaces.contains(&t.space));
        if !spaces.contains(&self.active_space) {
            self.active_space = spaces[0].clone();
        }
        let tabs = &self.tabs;
        self.active_tab
            .retain(|s, id| tabs.iter().any(|t| t.id == *id && &t.space == s));
        self.next_id = self.tabs.iter().map(|t| t.id).max().unwrap_or(0).max(self.next_id);
    }
}

struct AppState {
    data: Mutex<Data>,
    file: PathBuf,
    root: PathBuf,
}

fn state(app: &AppHandle) -> &AppState {
    app.state::<AppState>().inner()
}

fn with_data<T>(app: &AppHandle, f: impl FnOnce(&mut Data) -> T) -> T {
    f(&mut state(app).data.lock().unwrap())
}

/// Persists the data and pushes it to the sidebar.
fn commit(app: &AppHandle) {
    let s = state(app);
    let data = s.data.lock().unwrap().clone();
    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let tmp = s.file.with_extension("tmp");
        if fs::write(&tmp, json).is_ok() {
            let _ = fs::rename(&tmp, &s.file);
        }
    }
    let _ = app.emit_to(UI, "state", data);
}

fn label(id: u64) -> String {
    format!("tab-{id}")
}

fn window(app: &AppHandle) -> Option<Window> {
    app.get_window(WINDOW)
}

fn is_newtab(url: &Url) -> bool {
    matches!(url.host_str(), Some("tauri.localhost") | Some("localhost"))
        && url.path().trim_start_matches('/') == NEWTAB_PAGE
}

fn to_url(input: &str, search: &str) -> String {
    let s = input.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Ok(u) = Url::parse(s) {
        if matches!(u.scheme(), "http" | "https" | "file" | "about" | "data" | "view-source") {
            return u.to_string();
        }
    }
    if !s.contains(char::is_whitespace) {
        let scheme = if s.starts_with("localhost") || s.starts_with("127.") { "http" } else { "https" };
        if s.contains('.') || s.starts_with("localhost") {
            if let Ok(u) = Url::parse(&format!("{scheme}://{s}")) {
                return u.to_string();
            }
        }
    }
    let q: String = url::form_urlencoded::byte_serialize(s.as_bytes()).collect();
    format!("{search}{q}")
}

/// Sidebar and content rectangles in physical pixels.
fn layout_rects(win: &Window, right: bool) -> Option<[(PhysicalPosition<i32>, PhysicalSize<u32>); 2]> {
    let size = win.inner_size().ok()?;
    let scale = win.scale_factor().ok()?;
    let sw = ((SIDEBAR_WIDTH * scale) as u32).min(size.width);
    let cw = size.width - sw;
    let (ui_x, content_x) = if right { (cw, 0) } else { (0, sw) };
    Some([
        (PhysicalPosition::new(ui_x as i32, 0), PhysicalSize::new(sw, size.height)),
        (PhysicalPosition::new(content_x as i32, 0), PhysicalSize::new(cw, size.height)),
    ])
}

fn update_tab(app: &AppHandle, id: u64, f: impl FnOnce(&mut Tab)) {
    let found = with_data(app, |d| d.tabs.iter_mut().find(|t| t.id == id).map(f).is_some());
    if found {
        commit(app);
    }
}

fn create_tab_webview(app: &AppHandle, win: &Window, tab: &Tab, right: bool) -> tauri::Result<()> {
    let id = tab.id;
    let url = match Url::parse(&tab.url) {
        Ok(u) if !tab.url.is_empty() => WebviewUrl::External(u),
        _ => WebviewUrl::App(NEWTAB_PAGE.into()),
    };
    let app_nw = app.clone();
    let builder = WebviewBuilder::new(label(id), url)
        .data_directory(state(app).root.join("spaces").join(&tab.space))
        .initialization_script(SHORTCUTS_JS)
        .zoom_hotkeys_enabled(true)
        .devtools(true)
        .on_page_load(move |wv, payload| {
            let url = payload.url();
            let url = if is_newtab(url) { String::new() } else { url.to_string() };
            let loading = matches!(payload.event(), PageLoadEvent::Started);
            update_tab(wv.app_handle(), id, |t| {
                t.url = url;
                t.loading = loading;
            });
        })
        .on_document_title_changed(move |wv, title| {
            update_tab(wv.app_handle(), id, |t| t.title = title);
        })
        .on_new_window(move |url, features| {
            let app = app_nw.clone();
            if url.host_str() == Some(CMD_HOST) {
                let cmd = url.path().trim_start_matches('/').to_string();
                spawn(async move { run_cmd(&app, &cmd).await });
                return NewWindowResponse::Deny;
            }
            // Sized popups (OAuth, payment flows) keep their opener; everything else becomes a tab.
            if features.size().is_some() {
                return NewWindowResponse::Allow;
            }
            let url = url.to_string();
            spawn(async move { open_tab(&app, url).await });
            NewWindowResponse::Deny
        });
    let [_, (pos, size)] = layout_rects(win, right).unwrap_or_default();
    win.add_child(builder, pos, size)?;
    Ok(())
}

/// Creates the active tab's webview if needed, shows it and hides the rest.
fn apply_layout(app: &AppHandle) {
    let Some(win) = window(app) else { return };
    let (active, right) = with_data(app, |d| {
        let tab = d.active_tab_id().and_then(|id| d.tabs.iter().find(|t| t.id == id).cloned());
        (tab, d.settings.sidebar_right)
    });
    let Some(rects) = layout_rects(&win, right) else { return };
    let active_label = active.as_ref().map(|t| label(t.id));
    if let Some(tab) = &active {
        if app.get_webview(&label(tab.id)).is_none() {
            let _ = create_tab_webview(app, &win, tab, right);
        }
    }
    for wv in win.webviews() {
        let l = wv.label().to_string();
        if l == UI {
            let _ = wv.set_position(rects[0].0);
            let _ = wv.set_size(rects[0].1);
        } else if Some(&l) == active_label.as_ref() {
            let _ = wv.set_position(rects[1].0);
            let _ = wv.set_size(rects[1].1);
            let _ = wv.show();
        } else {
            let _ = wv.hide();
        }
    }
}

fn focus_address(app: &AppHandle) {
    if let Some(ui) = app.get_webview(UI) {
        let _ = ui.set_focus();
        let _ = app.emit_to(UI, "focus-address", ());
    }
}

fn focus_active(app: &AppHandle) {
    if let Some(id) = with_data(app, |d| d.active_tab_id()) {
        if let Some(wv) = app.get_webview(&label(id)) {
            let _ = wv.set_focus();
        }
    }
}

async fn open_tab(app: &AppHandle, url: String) {
    with_data(app, |d| {
        let id = d.new_id();
        let space = d.active_space.clone();
        d.tabs.push(Tab { id, space: space.clone(), url, title: String::new(), pinned: false, loading: true });
        d.active_tab.insert(space, id);
    });
    commit(app);
    apply_layout(app);
}

async fn run_cmd(app: &AppHandle, cmd: &str) {
    match cmd {
        "newtab" => {
            open_tab(app, String::new()).await;
            focus_address(app);
        }
        "close" => {
            if let Some(id) = with_data(app, |d| d.active_tab_id()) {
                close_tab(app.clone(), id).await;
            }
        }
        "address" => focus_address(app),
        "next" | "prev" => {
            let next = with_data(app, |d| {
                let tabs = d.space_tabs(&d.active_space);
                let i = tabs.iter().position(|t| Some(*t) == d.active_tab_id())?;
                let n = tabs.len();
                Some(tabs[if cmd == "next" { (i + 1) % n } else { (i + n - 1) % n }])
            });
            if let Some(id) = next {
                activate_tab(app.clone(), id).await;
            }
        }
        c if c.starts_with("space") => {
            let n: usize = c[5..].parse().unwrap_or(1);
            if let Some(id) = with_data(app, |d| d.spaces.get(n - 1).map(|s| s.id.clone())) {
                switch_space(app.clone(), id).await;
            }
        }
        c if c.starts_with("tab") => {
            let n: usize = c[3..].parse().unwrap_or(1);
            let id = with_data(app, |d| {
                let tabs = d.space_tabs(&d.active_space);
                if n == 9 { tabs.last().copied() } else { tabs.get(n - 1).copied() }
            });
            if let Some(id) = id {
                activate_tab(app.clone(), id).await;
            }
        }
        _ => {}
    }
}

// Commands are async so webview creation never runs on the main thread (deadlocks on Windows).

#[tauri::command]
async fn get_state(app: AppHandle) -> Data {
    with_data(&app, |d| d.clone())
}

#[tauri::command]
async fn new_tab(app: AppHandle, url: Option<String>) {
    let url = with_data(&app, |d| to_url(url.as_deref().unwrap_or(""), &d.settings.search));
    let blank = url.is_empty();
    open_tab(&app, url).await;
    if blank {
        focus_address(&app);
    } else {
        focus_active(&app);
    }
}

#[tauri::command]
async fn activate_tab(app: AppHandle, id: u64) {
    let ok = with_data(&app, |d| {
        let space = d.tabs.iter().find(|t| t.id == id)?.space.clone();
        d.active_space = space.clone();
        d.active_tab.insert(space, id);
        Some(())
    });
    if ok.is_some() {
        commit(&app);
        apply_layout(&app);
        focus_active(&app);
    }
}

#[tauri::command]
async fn close_tab(app: AppHandle, id: u64) {
    let replace = with_data(&app, |d| {
        let space = d.tabs.iter().find(|t| t.id == id)?.space.clone();
        let order = d.space_tabs(&space);
        let i = order.iter().position(|t| *t == id).unwrap_or(0);
        d.tabs.retain(|t| t.id != id);
        if d.active_tab.get(&space) == Some(&id) {
            match order.get(i + 1).or(i.checked_sub(1).and_then(|j| order.get(j))) {
                Some(next) => {
                    d.active_tab.insert(space.clone(), *next);
                }
                None => {
                    d.active_tab.remove(&space);
                }
            }
        }
        Some(space == d.active_space && !d.active_tab.contains_key(&space))
    });
    if let Some(wv) = app.get_webview(&label(id)) {
        let _ = wv.close();
    }
    if replace == Some(true) {
        open_tab(&app, String::new()).await;
        focus_address(&app);
    } else {
        commit(&app);
        apply_layout(&app);
    }
}

#[tauri::command]
async fn navigate(app: AppHandle, input: String) {
    let (id, url) = with_data(&app, |d| (d.active_tab_id(), to_url(&input, &d.settings.search)));
    let Some(id) = id else {
        return new_tab(app, Some(input)).await;
    };
    let Some(wv) = app.get_webview(&label(id)) else { return };
    if let Ok(u) = Url::parse(&url) {
        let _ = wv.navigate(u);
        let _ = wv.set_focus();
    }
}

#[tauri::command]
async fn nav(app: AppHandle, action: String) {
    let Some(id) = with_data(&app, |d| d.active_tab_id()) else { return };
    let Some(wv) = app.get_webview(&label(id)) else { return };
    let _ = match action.as_str() {
        "back" => wv.eval("history.back()"),
        "forward" => wv.eval("history.forward()"),
        "reload" => wv.reload(),
        "stop" => wv.eval("window.stop()"),
        _ => Ok(()),
    };
}

#[tauri::command]
async fn toggle_pin(app: AppHandle, id: u64) {
    update_tab(&app, id, |t| t.pinned = !t.pinned);
}

/// Moves a tab before `before` (or to the end), adopting its pinned state.
#[tauri::command]
async fn move_tab(app: AppHandle, id: u64, before: Option<u64>, pinned: bool) {
    let moved = with_data(&app, |d| {
        let i = d.tabs.iter().position(|t| t.id == id)?;
        let mut tab = d.tabs.remove(i);
        tab.pinned = pinned;
        let at = before
            .and_then(|b| d.tabs.iter().position(|t| t.id == b))
            .unwrap_or(d.tabs.len());
        d.tabs.insert(at, tab);
        Some(())
    });
    if moved.is_some() {
        commit(&app);
    }
}

#[tauri::command]
async fn add_space(app: AppHandle, name: String, color: String) {
    with_data(&app, |d| {
        let id = format!("space-{}", d.new_id());
        d.spaces.push(Space { id: id.clone(), name, color });
        d.active_space = id;
    });
    open_tab(&app, String::new()).await;
    focus_address(&app);
}

#[tauri::command]
async fn switch_space(app: AppHandle, id: String) {
    let empty = with_data(&app, |d| {
        if !d.spaces.iter().any(|s| s.id == id) {
            return None;
        }
        d.active_space = id.clone();
        if d.active_tab_id().is_none() {
            if let Some(first) = d.space_tabs(&id).first() {
                d.active_tab.insert(id.clone(), *first);
            }
        }
        Some(d.active_tab_id().is_none())
    });
    match empty {
        Some(true) => {
            open_tab(&app, String::new()).await;
            focus_address(&app);
        }
        Some(false) => {
            commit(&app);
            apply_layout(&app);
            focus_active(&app);
        }
        None => {}
    }
}

#[tauri::command]
async fn update_space(app: AppHandle, id: String, name: String, color: String) {
    with_data(&app, |d| {
        if let Some(s) = d.spaces.iter_mut().find(|s| s.id == id) {
            s.name = name;
            s.color = color;
        }
    });
    commit(&app);
}

/// Deletes a Space, its tabs and its website data.
#[tauri::command]
async fn delete_space(app: AppHandle, id: String) {
    let closed = with_data(&app, |d| {
        if d.spaces.len() <= 1 {
            return None;
        }
        d.spaces.retain(|s| s.id != id);
        let closed: Vec<u64> = d.tabs.iter().filter(|t| t.space == id).map(|t| t.id).collect();
        d.tabs.retain(|t| t.space != id);
        d.active_tab.remove(&id);
        Some(closed)
    });
    let Some(closed) = closed else { return };
    for tab in closed {
        if let Some(wv) = app.get_webview(&label(tab)) {
            let _ = wv.close();
        }
    }
    let fallback = with_data(&app, |d| d.spaces[0].id.clone());
    if with_data(&app, |d| d.active_space == id) {
        switch_space(app.clone(), fallback).await;
    } else {
        commit(&app);
    }
    // The WebView2 browser process may still hold files for a moment.
    let dir = state(&app).root.join("spaces").join(&id);
    spawn(async move {
        for _ in 0..10 {
            if fs::remove_dir_all(&dir).is_ok() || !dir.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

#[tauri::command]
async fn set_settings(app: AppHandle, settings: Settings) {
    with_data(&app, |d| d.settings = settings);
    commit(&app);
    apply_layout(&app);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_state,
            new_tab,
            activate_tab,
            close_tab,
            navigate,
            nav,
            toggle_pin,
            move_tab,
            add_space,
            switch_space,
            update_space,
            delete_space,
            set_settings,
        ])
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            fs::create_dir_all(&root)?;
            let file = root.join("state.json");
            let mut data: Data = fs::read_to_string(&file)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            data.repair();
            if data.active_tab_id().is_none() {
                let id = data.new_id();
                let space = data.active_space.clone();
                data.tabs.push(Tab { id, space: space.clone(), url: String::new(), title: String::new(), pinned: false, loading: false });
                data.active_tab.insert(space, id);
            }
            let right = data.settings.sidebar_right;
            app.manage(AppState { data: Mutex::new(data), file, root });

            let win = tauri::window::WindowBuilder::new(app, WINDOW)
                .title("Crest")
                .inner_size(1280.0, 820.0)
                .min_inner_size(640.0, 400.0)
                .center()
                .build()?;
            let _ = win.set_size(LogicalSize::new(1280.0, 820.0));
            let [(pos, size), _] = layout_rects(&win, right).unwrap_or_default();
            win.add_child(
                WebviewBuilder::new(UI, WebviewUrl::App("index.html".into()))
                    // HTML5 drag and drop for tab reordering.
                    .disable_drag_drop_handler(),
                pos,
                size,
            )?;

            let handle = app.handle().clone();
            win.on_window_event(move |e| {
                if matches!(e, WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }) {
                    apply_layout(&handle);
                }
            });
            let handle = app.handle().clone();
            spawn(async move { apply_layout(&handle) });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Crest");
}
