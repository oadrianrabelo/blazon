#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod import;

use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

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
/// Gap around the page card, in logical pixels.
const CARD_MARGIN: f64 = 8.0;
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
    else if (e.shiftKey && k === 't') cmd = 'reopen';
    else if (!e.shiftKey && k === 'h') cmd = 'history';
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
    /// InPrivate: no history, no cookies on disk, tabs dropped on exit.
    #[serde(default)]
    private: bool,
    /// Crest Studio house (look template); empty means a custom color.
    #[serde(default)]
    house: String,
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
    #[serde(default)]
    folder: Option<u64>,
    #[serde(default, skip_deserializing)]
    loading: bool,
    /// Whether the tab has a live webview; sleeping tabs reload on activation.
    #[serde(default, skip_deserializing)]
    awake: bool,
    #[serde(skip)]
    last_active: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Folder {
    id: u64,
    space: String,
    name: String,
    #[serde(default)]
    open: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct Visit {
    space: String,
    url: String,
    title: String,
    time: u64,
    #[serde(default = "one")]
    count: u32,
}

fn one() -> u32 {
    1
}

#[derive(Clone, Serialize)]
struct Suggestion {
    kind: &'static str,
    url: String,
    title: String,
    tab: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    sidebar_right: bool,
    search: String,
    /// Minutes before a background tab's webview is closed; 0 disables.
    sleep_minutes: u64,
    /// Use the standard Windows title bar instead of Crest's borderless frame.
    native_frame: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sidebar_right: false,
            search: "https://duckduckgo.com/?q=".into(),
            sleep_minutes: 30,
            native_frame: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct Data {
    spaces: Vec<Space>,
    tabs: Vec<Tab>,
    folders: Vec<Folder>,
    /// Split View pairs, shown side by side (left, right) when either is active.
    splits: Vec<[u64; 2]>,
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

    fn is_private(&self, space: &str) -> bool {
        self.spaces.iter().any(|s| s.id == space && s.private)
    }

    fn active_tab_id(&self) -> Option<u64> {
        self.active_tab.get(&self.active_space).copied()
    }

    /// Tabs on screen: the active tab, or its Split View pair.
    fn visible(&self) -> Vec<u64> {
        let Some(id) = self.active_tab_id() else { return vec![] };
        match self.splits.iter().find(|p| p.contains(&id)) {
            Some(pair) => pair.to_vec(),
            None => vec![id],
        }
    }

    fn unsplit(&mut self, id: u64) {
        self.splits.retain(|p| !p.contains(&id));
    }

    fn space_tabs(&self, space: &str) -> Vec<u64> {
        let mut tabs: Vec<&Tab> = self.tabs.iter().filter(|t| t.space == space).collect();
        tabs.sort_by_key(|t| (!t.pinned, t.folder.is_none()));
        tabs.iter().map(|t| t.id).collect()
    }

    fn repair(&mut self) {
        if self.spaces.is_empty() {
            self.spaces.push(Space {
                id: "personal".into(),
                name: "Pessoal".into(),
                color: "#86adc4".into(),
                private: false,
                house: "winter".into(),
            });
        }
        let spaces: Vec<String> = self.spaces.iter().filter(|s| !s.private).map(|s| s.id.clone()).collect();
        self.tabs.retain(|t| spaces.contains(&t.space));
        let spaces: Vec<String> = self.spaces.iter().map(|s| s.id.clone()).collect();
        self.folders.retain(|f| spaces.contains(&f.space));
        let folders: Vec<u64> = self.folders.iter().map(|f| f.id).collect();
        for t in self.tabs.iter_mut() {
            if t.folder.is_some_and(|f| !folders.contains(&f)) {
                t.folder = None;
            }
        }
        if !spaces.contains(&self.active_space) {
            self.active_space = spaces[0].clone();
        }
        let tabs = &self.tabs;
        let space_of = |id: u64| tabs.iter().find(|t| t.id == id).map(|t| t.space.clone());
        self.splits.retain(|[a, b]| a != b && space_of(*a).is_some() && space_of(*a) == space_of(*b));
        self.active_tab
            .retain(|s, id| tabs.iter().any(|t| t.id == *id && &t.space == s));
        let max_tab = self.tabs.iter().map(|t| t.id).max().unwrap_or(0);
        let max_folder = self.folders.iter().map(|f| f.id).max().unwrap_or(0);
        self.next_id = max_tab.max(max_folder).max(self.next_id);
    }
}

const HISTORY_LIMIT: usize = 5000;
const CLOSED_LIMIT: usize = 25;

struct AppState {
    data: Mutex<Data>,
    file: PathBuf,
    root: PathBuf,
    history: Mutex<Vec<Visit>>,
    history_dirty: AtomicBool,
    /// Recently closed tabs as (space, url), newest last.
    closed: Mutex<Vec<(String, String)>>,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn record_visit(app: &AppHandle, space: &str, url: &str, title: &str) {
    if url.is_empty() || url.starts_with("about:") {
        return;
    }
    let s = state(app);
    let mut h = s.history.lock().unwrap();
    let (title, count) = match h.iter().position(|v| v.space == space && v.url == url) {
        Some(i) => {
            let old = h.remove(i);
            (if title.is_empty() { old.title } else { title.to_string() }, old.count.saturating_add(1))
        }
        None => (title.to_string(), 1),
    };
    h.push(Visit { space: space.into(), url: url.into(), title, time: now(), count });
    if h.len() > HISTORY_LIMIT {
        let extra = h.len() - HISTORY_LIMIT;
        h.drain(..extra);
    }
    s.history_dirty.store(true, Ordering::Relaxed);
}

fn set_visit_title(app: &AppHandle, space: &str, url: &str, title: &str) {
    let s = state(app);
    if let Some(v) = s.history.lock().unwrap().iter_mut().rev().find(|v| v.space == space && v.url == url) {
        v.title = title.into();
        s.history_dirty.store(true, Ordering::Relaxed);
    }
}

/// What the start page shows: the Space's look and its most visited sites.
fn start_page_data(app: &AppHandle, space: &str) -> String {
    let (sp, search) = with_data(app, |d| (d.spaces.iter().find(|s| s.id == space).cloned(), d.settings.search.clone()));
    let mut by_host: HashMap<String, (u32, Visit)> = HashMap::new();
    for v in state(app).history.lock().unwrap().iter().filter(|v| v.space == space) {
        let Some(host) = Url::parse(&v.url).ok().and_then(|u| u.host_str().map(String::from)) else { continue };
        let e = by_host.entry(host).or_insert((0, v.clone()));
        e.0 += v.count;
        if v.count > e.1.count {
            e.1 = v.clone();
        }
    }
    let mut top: Vec<(u32, Visit)> = by_host.into_values().collect();
    top.sort_by(|a, b| b.0.cmp(&a.0));
    let sites: Vec<_> = top
        .into_iter()
        .take(8)
        .map(|(_, v)| serde_json::json!({ "url": v.url, "title": v.title }))
        .collect();
    serde_json::json!({ "space": sp, "search": search, "sites": sites }).to_string()
}

fn save_history(app: &AppHandle) {
    let s = state(app);
    if !s.history_dirty.swap(false, Ordering::Relaxed) {
        return;
    }
    let json = serde_json::to_string(&*s.history.lock().unwrap()).unwrap_or_default();
    let file = s.root.join("history.json");
    let tmp = file.with_extension("tmp");
    if fs::write(&tmp, json).is_ok() {
        let _ = fs::rename(&tmp, &file);
    }
}

/// Closes the webviews of background tabs idle longer than the sleep setting.
fn sleep_idle_tabs(app: &AppHandle) {
    let asleep = with_data(app, |d| {
        let limit = d.settings.sleep_minutes * 60;
        let visible = d.visible();
        let cutoff = now().saturating_sub(limit);
        let mut ids = vec![];
        for t in d.tabs.iter_mut() {
            if limit > 0 && t.awake && !t.pinned && !visible.contains(&t.id) && t.last_active < cutoff {
                t.awake = false;
                t.loading = false;
                ids.push(t.id);
            }
        }
        ids
    });
    for id in &asleep {
        if let Some(wv) = app.get_webview(&label(*id)) {
            let _ = wv.close();
        }
    }
    if !asleep.is_empty() {
        commit(app);
    }
}

/// Turns a command-line argument (URL or file path) into a URL.
fn arg_to_url(arg: &str) -> Option<String> {
    if arg.starts_with('-') {
        return None;
    }
    if let Ok(u) = Url::parse(arg) {
        if matches!(u.scheme(), "http" | "https" | "file") {
            return Some(u.to_string());
        }
    }
    let path = PathBuf::from(arg);
    if path.is_file() {
        return Url::from_file_path(fs::canonicalize(&path).ok()?).ok().map(|u| u.to_string());
    }
    None
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

/// UI and page rectangles in physical pixels. The UI covers the whole window
/// (sidebar plus the Space background); the page sits on it as an inset card.
fn layout_rects(win: &Window, right: bool) -> Option<[(PhysicalPosition<i32>, PhysicalSize<u32>); 2]> {
    let size = win.inner_size().ok()?;
    let scale = win.scale_factor().ok()?;
    let sw = ((SIDEBAR_WIDTH * scale) as u32).min(size.width);
    let m = (CARD_MARGIN * scale) as u32;
    let cw = size.width.saturating_sub(sw + m);
    let ch = size.height.saturating_sub(2 * m);
    let content_x = if right { m } else { sw };
    Some([
        (PhysicalPosition::new(0, 0), size),
        (PhysicalPosition::new(content_x as i32, m as i32), PhysicalSize::new(cw, ch)),
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
    let private = with_data(app, |d| d.is_private(&tab.space));
    let space = tab.space.clone();
    let space_t = tab.space.clone();
    let builder = WebviewBuilder::new(label(id), url)
        .incognito(private)
        .data_directory(state(app).root.join("spaces").join(&tab.space))
        .initialization_script(SHORTCUTS_JS)
        .zoom_hotkeys_enabled(true)
        .devtools(true)
        .on_page_load(move |wv, payload| {
            let url = payload.url();
            let newtab = is_newtab(url);
            let url = if newtab { String::new() } else { url.to_string() };
            let loading = matches!(payload.event(), PageLoadEvent::Started);
            let app = wv.app_handle();
            if newtab && !loading {
                let _ = wv.eval(format!("window.crestStart && window.crestStart({})", start_page_data(app, &space)));
            }
            let mut title = String::new();
            update_tab(app, id, |t| {
                t.url = url.clone();
                t.loading = loading;
                title = t.title.clone();
            });
            if !loading && !private {
                record_visit(app, &space, &url, &title);
            }
        })
        .on_document_title_changed(move |wv, title| {
            let app = wv.app_handle();
            let mut url = String::new();
            update_tab(app, id, |t| {
                t.title = title.clone();
                url = t.url.clone();
            });
            if !private {
                set_visit_title(app, &space_t, &url, &title);
            }
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
    with_data(app, |d| {
        if let Some(t) = d.tabs.iter_mut().find(|t| t.id == id) {
            t.awake = true;
        }
    });
    Ok(())
}

/// Creates the active tab's webview if needed, shows it and hides the rest.
fn apply_layout(app: &AppHandle) {
    let Some(win) = window(app) else { return };
    let (visible, right) = with_data(app, |d| {
        let ids = d.visible();
        let tabs: Vec<Tab> = d
            .tabs
            .iter_mut()
            .filter(|t| ids.contains(&t.id))
            .map(|t| {
                t.last_active = now();
                t.clone()
            })
            .collect();
        let ordered: Vec<Tab> = ids.iter().filter_map(|id| tabs.iter().find(|t| t.id == *id).cloned()).collect();
        (ordered, d.settings.sidebar_right)
    });
    let Some(rects) = layout_rects(&win, right) else { return };
    let (pos, size) = rects[1];
    // Split View: equal halves with a card gap between them.
    let n = visible.len().max(1) as u32;
    let gap = (CARD_MARGIN * win.scale_factor().unwrap_or(1.0)) as u32;
    let w = size.width.saturating_sub(gap * (n - 1)) / n;
    let slots: Vec<(String, PhysicalPosition<i32>)> = visible
        .iter()
        .enumerate()
        .map(|(i, t)| (label(t.id), PhysicalPosition::new(pos.x + (i as u32 * (w + gap)) as i32, pos.y)))
        .collect();
    for tab in &visible {
        if app.get_webview(&label(tab.id)).is_none() {
            let _ = create_tab_webview(app, &win, tab, right);
        }
    }
    for wv in win.webviews() {
        let l = wv.label().to_string();
        if l == UI {
            let _ = wv.set_position(rects[0].0);
            let _ = wv.set_size(rects[0].1);
        } else if let Some((_, p)) = slots.iter().find(|(s, _)| *s == l) {
            let _ = wv.set_position(*p);
            let _ = wv.set_size(PhysicalSize::new(w, size.height));
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
        d.tabs.push(Tab { id, space: space.clone(), url, title: String::new(), pinned: false, folder: None, loading: true, awake: false, last_active: now() });
        d.active_tab.insert(space, id);
    });
    apply_layout(app);
    commit(app);
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
        "reopen" => reopen_tab(app.clone()).await,
        "history" => {
            if let Some(ui) = app.get_webview(UI) {
                let _ = ui.set_focus();
                let _ = app.emit_to(UI, "open-history", ());
            }
        }
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
        apply_layout(&app);
        commit(&app);
        focus_active(&app);
    }
}

#[tauri::command]
async fn close_tab(app: AppHandle, id: u64) {
    let replace = with_data(&app, |d| {
        let space = d.tabs.iter().find(|t| t.id == id)?.space.clone();
        let order = d.space_tabs(&space);
        let i = order.iter().position(|t| *t == id).unwrap_or(0);
        let url = d.tabs.iter().find(|t| t.id == id).map(|t| t.url.clone()).unwrap_or_default();
        d.unsplit(id);
        if !url.is_empty() {
            let mut closed = state(&app).closed.lock().unwrap();
            closed.push((space.clone(), url));
            if closed.len() > CLOSED_LIMIT {
                closed.remove(0);
            }
        }
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
        apply_layout(&app);
        commit(&app);
    }
}

#[tauri::command]
async fn reopen_tab(app: AppHandle) {
    let Some((space, url)) = state(&app).closed.lock().unwrap().pop() else { return };
    let exists = with_data(&app, |d| {
        let ok = d.spaces.iter().any(|s| s.id == space);
        if ok {
            d.active_space = space;
        }
        ok
    });
    if exists {
        open_tab(&app, url).await;
        focus_active(&app);
    }
}

/// Address bar suggestions: open tabs first, then history, newest first.
#[tauri::command]
async fn suggest(app: AppHandle, query: String) -> Vec<Suggestion> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return vec![];
    }
    let words: Vec<&str> = q.split_whitespace().collect();
    let hit = |url: &str, title: &str| {
        let hay = format!("{} {}", url.to_lowercase(), title.to_lowercase());
        words.iter().all(|w| hay.contains(w))
    };
    let (space, tabs) = with_data(&app, |d| (d.active_space.clone(), d.tabs.clone()));
    let active = with_data(&app, |d| d.active_tab_id());
    let mut out: Vec<Suggestion> = tabs
        .iter()
        .filter(|t| t.space == space && Some(t.id) != active && !t.url.is_empty() && hit(&t.url, &t.title))
        .take(3)
        .map(|t| Suggestion { kind: "tab", url: t.url.clone(), title: t.title.clone(), tab: Some(t.id) })
        .collect();
    let history = state(&app).history.lock().unwrap();
    for v in history.iter().rev() {
        if out.len() >= 8 {
            break;
        }
        if v.space == space && hit(&v.url, &v.title) && !out.iter().any(|s| s.url == v.url) {
            out.push(Suggestion { kind: "history", url: v.url.clone(), title: v.title.clone(), tab: None });
        }
    }
    out
}

#[tauri::command]
async fn history_list(app: AppHandle, query: String) -> Vec<Visit> {
    let q = query.to_lowercase();
    let space = with_data(&app, |d| d.active_space.clone());
    state(&app)
        .history
        .lock()
        .unwrap()
        .iter()
        .rev()
        .filter(|v| v.space == space && (q.is_empty() || v.url.to_lowercase().contains(&q) || v.title.to_lowercase().contains(&q)))
        .take(300)
        .cloned()
        .collect()
}

#[tauri::command]
async fn clear_history(app: AppHandle) {
    let space = with_data(&app, |d| d.active_space.clone());
    let s = state(&app);
    s.history.lock().unwrap().retain(|v| v.space != space);
    s.history_dirty.store(true, Ordering::Relaxed);
    save_history(&app);
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

/// Moves a tab before `before` (or to the end), into the given pinned row or folder.
#[tauri::command]
async fn move_tab(app: AppHandle, id: u64, before: Option<u64>, pinned: bool, folder: Option<u64>) {
    let moved = with_data(&app, |d| {
        let i = d.tabs.iter().position(|t| t.id == id)?;
        let mut tab = d.tabs.remove(i);
        tab.pinned = pinned && folder.is_none();
        tab.folder = folder;
        if let Some(f) = d.folders.iter_mut().find(|f| Some(f.id) == folder) {
            f.open = true;
        }
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

/// Shows `id` side by side with `with`, or with a new tab when `with` is None.
#[tauri::command]
async fn split_tab(app: AppHandle, id: u64, with: Option<u64>) {
    let space = with_data(&app, |d| d.tabs.iter().find(|t| t.id == id).map(|t| t.space.clone()));
    let Some(space) = space else { return };
    let other = match with {
        Some(o) => o,
        None => {
            let o = with_data(&app, |d| {
                let o = d.new_id();
                d.tabs.push(Tab { id: o, space: space.clone(), url: String::new(), title: String::new(), pinned: false, folder: None, loading: false, awake: false, last_active: now() });
                o
            });
            o
        }
    };
    let ok = with_data(&app, |d| {
        if id == other || d.tabs.iter().find(|t| t.id == other).map(|t| &t.space) != Some(&space) {
            return false;
        }
        d.unsplit(id);
        d.unsplit(other);
        d.splits.push([id, other]);
        d.active_space = space.clone();
        d.active_tab.insert(space, if with.is_some() { id } else { other });
        true
    });
    if ok {
        apply_layout(&app);
        commit(&app);
        if with.is_some() {
            focus_active(&app);
        } else {
            focus_address(&app);
        }
    }
}

#[tauri::command]
async fn unsplit_tab(app: AppHandle, id: u64) {
    with_data(&app, |d| d.unsplit(id));
    apply_layout(&app);
    commit(&app);
}

#[tauri::command]
async fn import_sources() -> Vec<import::Source> {
    import::sources()
}

const HOUSES: [&str; 9] = ["winter", "lion", "storm", "dragon", "meadow", "iron", "river", "sun", "vigil"];
const HOUSE_ACCENTS: [&str; 9] = ["#86adc4", "#c9a254", "#b08f4a", "#be4438", "#bca766", "#9c9781", "#a85c41", "#d09e65", "#7b838c"];

fn push_tab(d: &mut Data, space: &str, e: import::Entry, pinned: bool, folder: Option<u64>) {
    let id = d.new_id();
    d.tabs.push(Tab { id, space: space.into(), url: e.url, title: e.title, pinned, folder, loading: false, awake: false, last_active: 0 });
}

fn push_folders(d: &mut Data, space: &str, folders: Vec<import::Folder>) {
    for f in folders {
        let fid = d.new_id();
        d.folders.push(Folder { id: fid, space: space.into(), name: f.name, open: false });
        for e in f.tabs {
            push_tab(d, space, e, false, Some(fid));
        }
    }
}

/// Imports from another browser; imported tabs start asleep. Returns a summary.
#[tauri::command]
async fn import_browser(app: AppHandle, id: String) -> Result<String, String> {
    let imported = import::read(&id)?;
    let summary = with_data(&app, |d| {
        let before = d.tabs.len();
        let what = match imported {
            import::Import::Bookmarks(folders) => {
                let space = d.active_space.clone();
                let n = folders.len();
                push_folders(d, &space, folders);
                format!("{n} pastas")
            }
            import::Import::Spaces(spaces) => {
                let n = spaces.len();
                for sp in spaces {
                    let sid = format!("space-{}", d.new_id());
                    let h = d.spaces.len() % HOUSES.len();
                    d.spaces.push(Space { id: sid.clone(), name: sp.name, color: HOUSE_ACCENTS[h].into(), private: false, house: HOUSES[h].into() });
                    for e in sp.pinned {
                        push_tab(d, &sid, e, true, None);
                    }
                    push_folders(d, &sid, sp.folders);
                    for e in sp.tabs {
                        push_tab(d, &sid, e, false, None);
                    }
                }
                format!("{n} Spaces")
            }
        };
        format!("Importado: {what} e {} abas.", d.tabs.len() - before)
    });
    commit(&app);
    Ok(summary)
}

#[tauri::command]
async fn add_folder(app: AppHandle, name: String, tab: Option<u64>) {
    with_data(&app, |d| {
        let id = d.new_id();
        let space = d.active_space.clone();
        d.folders.push(Folder { id, space, name, open: true });
        if let Some(t) = d.tabs.iter_mut().find(|t| Some(t.id) == tab) {
            t.folder = Some(id);
            t.pinned = false;
        }
    });
    commit(&app);
}

#[tauri::command]
async fn update_folder(app: AppHandle, id: u64, name: Option<String>, open: Option<bool>) {
    with_data(&app, |d| {
        if let Some(f) = d.folders.iter_mut().find(|f| f.id == id) {
            if let Some(name) = name {
                f.name = name;
            }
            if let Some(open) = open {
                f.open = open;
            }
        }
    });
    commit(&app);
}

/// Removes a folder, keeping its tabs as loose tabs.
#[tauri::command]
async fn delete_folder(app: AppHandle, id: u64) {
    with_data(&app, |d| {
        d.folders.retain(|f| f.id != id);
        for t in d.tabs.iter_mut().filter(|t| t.folder == Some(id)) {
            t.folder = None;
        }
    });
    commit(&app);
}

#[tauri::command]
async fn duplicate_tab(app: AppHandle, id: u64) {
    let url = with_data(&app, |d| d.tabs.iter().find(|t| t.id == id).map(|t| t.url.clone()));
    if let Some(url) = url {
        open_tab(&app, url).await;
        focus_active(&app);
    }
}

#[tauri::command]
async fn add_space(app: AppHandle, name: String, color: String, private: bool, house: String) {
    with_data(&app, |d| {
        let id = format!("space-{}", d.new_id());
        d.spaces.push(Space { id: id.clone(), name, color, private, house });
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
            apply_layout(&app);
            commit(&app);
            focus_active(&app);
        }
        None => {}
    }
}

#[tauri::command]
async fn update_space(app: AppHandle, id: String, name: String, color: String, house: String) {
    with_data(&app, |d| {
        if let Some(s) = d.spaces.iter_mut().find(|s| s.id == id) {
            s.name = name;
            s.color = color;
            s.house = house;
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
        d.folders.retain(|f| f.space != id);
        d.active_tab.remove(&id);
        Some(closed)
    });
    {
        let s = state(&app);
        s.history.lock().unwrap().retain(|v| v.space != id);
        s.history_dirty.store(true, Ordering::Relaxed);
        s.closed.lock().unwrap().retain(|(sp, _)| *sp != id);
    }
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
    if let Some(win) = window(&app) {
        let _ = win.set_decorations(settings.native_frame);
    }
    with_data(&app, |d| d.settings = settings);
    commit(&app);
    apply_layout(&app);
}

fn main() {
    tauri::Builder::default()
        // A second launch (e.g. a link clicked in another app) opens its URLs here.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let app = app.clone();
            let urls: Vec<String> = args.iter().skip(1).filter_map(|a| arg_to_url(a)).collect();
            spawn(async move {
                for url in urls {
                    open_tab(&app, url).await;
                }
                if let Some(win) = window(&app) {
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
                focus_active(&app);
            });
        }))
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
            reopen_tab,
            suggest,
            history_list,
            clear_history,
            add_folder,
            update_folder,
            delete_folder,
            duplicate_tab,
            split_tab,
            unsplit_tab,
            import_sources,
            import_browser,
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
                data.tabs.push(Tab { id, space: space.clone(), url: String::new(), title: String::new(), pinned: false, folder: None, loading: false, awake: false, last_active: 0 });
                data.active_tab.insert(space, id);
            }
            for url in std::env::args().skip(1).filter_map(|a| arg_to_url(&a)) {
                let id = data.new_id();
                let space = data.active_space.clone();
                data.tabs.push(Tab { id, space: space.clone(), url, title: String::new(), pinned: false, folder: None, loading: false, awake: false, last_active: 0 });
                data.active_tab.insert(space, id);
            }
            let history: Vec<Visit> = fs::read_to_string(root.join("history.json"))
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let right = data.settings.sidebar_right;
            let native_frame = data.settings.native_frame;
            app.manage(AppState {
                data: Mutex::new(data),
                file,
                root,
                history: Mutex::new(history),
                history_dirty: AtomicBool::new(false),
                closed: Mutex::new(vec![]),
            });

            let win = tauri::window::WindowBuilder::new(app, WINDOW)
                .title("Crest")
                .inner_size(1280.0, 820.0)
                .min_inner_size(640.0, 400.0)
                .center()
                .decorations(native_frame)
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
            win.on_window_event(move |e| match e {
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => apply_layout(&handle),
                WindowEvent::CloseRequested { .. } => save_history(&handle),
                _ => {}
            });
            let handle = app.handle().clone();
            spawn(async move {
                apply_layout(&handle);
                commit(&handle);
            });
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(30));
                save_history(&handle);
                sleep_idle_tabs(&handle);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Crest");
}
