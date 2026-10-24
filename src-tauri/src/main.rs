#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocker;
mod diag;
#[cfg(windows)]
mod hello;
mod import;
mod overlay;
#[cfg(windows)]
mod sqlite;
#[cfg(windows)]
mod webview2;

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
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, Window, WindowEvent,
};
use url::Url;

const WINDOW: &str = "main";
const UI: &str = "ui";
/// Settings and Crest Studio, shown over the page card.
const PANEL: &str = "panel";
/// Width of the edge left for a hidden floating sidebar.
const FLOAT_EDGE: f64 = 8.0;
const SIDEBAR_WIDTH: f64 = 264.0;
/// Gap around the page card, in logical pixels.
const CARD_MARGIN: f64 = 8.0;
/// Pages can't reach the IPC bridge, so the tab shortcut script asks for
/// commands by opening this host, which `on_new_window` intercepts.
const CMD_HOST: &str = "blazon-cmd.invalid";
const NEWTAB_PAGE: &str = "newtab.html";

/// Keyboard shortcuts inside pages, plus Peek on modifier+click of a link.
fn shortcuts_js(peek_modifier: &str) -> String {
    r#"(() => {
  if (window.top !== window) return;
  const keys = { t: 'newtab', w: 'close', l: 'address', k: 'palette', n: 'newwindow' };
  const send = (cmd) => window.open('https://blazon-cmd.invalid/' + cmd);
  addEventListener('keydown', (e) => {
    if (!e.ctrlKey || e.altKey) return;
    const k = e.key.toLowerCase();
    const digit = /^Digit[1-9]$/.test(e.code) ? e.code.slice(5) : null;
    let cmd = null;
    if (k === 'tab') cmd = e.shiftKey ? 'prev' : 'next';
    else if (digit) cmd = (e.shiftKey ? 'space' : 'tab') + digit;
    else if (!e.shiftKey && keys[k]) cmd = keys[k];
    else if (e.shiftKey && k === 's') cmd = 'sidebar';
    else if (e.shiftKey && k === 't') cmd = 'reopen';
    else if (!e.shiftKey && k === 'h') cmd = 'history';
    if (!cmd) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    send(cmd);
  }, true);
  const mod = 'MODIFIER';
  addEventListener('click', (e) => {
    if (mod === 'off' || !e[mod + 'Key'] || e.button !== 0) return;
    const a = e.target.closest && e.target.closest('a[href]');
    if (!a || !/^https?:/.test(a.href)) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    send('peek?u=' + encodeURIComponent(a.href));
  }, true);
})();"#
        .replace(
            "MODIFIER",
            if matches!(peek_modifier, "shift" | "alt" | "off") {
                peek_modifier
            } else {
                "shift"
            },
        )
}

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
    /// Download folder for this Space; empty means the Windows default.
    #[serde(default)]
    downloads: String,
    /// Crest Studio arms and sidebar look, owned by the UI; null follows `house`.
    #[serde(default)]
    look: serde_json::Value,
    /// Needs Windows Hello before its tabs are shown.
    #[serde(default)]
    locked: bool,
    /// Search engine for this Space; empty uses the global one.
    #[serde(default)]
    search: String,
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
    /// Playing sound right now, and muted by the user.
    #[serde(default, skip_deserializing)]
    audio: bool,
    #[serde(default, skip_deserializing)]
    muted: bool,
    /// Whether the tab has a live webview; sleeping tabs reload on activation.
    #[serde(default, skip_deserializing)]
    awake: bool,
    #[serde(default)]
    last_active: u64,
    /// Recent addresses of this tab, so Back works after the app restarts
    /// (WebView2 can't restore a tab's own history).
    #[serde(default)]
    trail: Vec<String>,
    #[serde(skip)]
    trail_nav: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct Folder {
    id: u64,
    space: String,
    name: String,
    #[serde(default)]
    open: bool,
    /// Custom folder color (hex) and icon (an emoji); empty uses the defaults.
    #[serde(default)]
    color: String,
    #[serde(default)]
    icon: String,
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
    /// Sidebar hides to a thin edge and slides back when the pointer reaches it.
    floating_sidebar: bool,
    sidebar_width: f64,
    /// Page zoom for new tab webviews (1.0 = 100%).
    default_zoom: f64,
    /// Look & feel options owned by the UI (tab shape, pins, folders, colors).
    appearance: serde_json::Value,
    /// Modifier that opens a clicked link in Peek: shift, alt or off.
    peek_modifier: String,
    /// Space for links opened from other apps; empty means the current one.
    external_space: String,
    /// Links from other apps whose address contains `pattern` open in `space`.
    routes: Vec<Route>,
    /// Block ads and trackers, except on these sites.
    adblock: bool,
    adblock_allow: Vec<String>,
    /// Offer to save passwords and autofill forms (WebView2's own store).
    passwords: bool,
    /// Hours before an idle tab (not pinned, not in a folder) is archived; 0 disables.
    archive_hours: u64,
    /// Interface language: auto, pt or en.
    language: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Route {
    pattern: String,
    space: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sidebar_right: false,
            search: "https://duckduckgo.com/?q=".into(),
            sleep_minutes: 30,
            native_frame: false,
            floating_sidebar: false,
            sidebar_width: SIDEBAR_WIDTH,
            default_zoom: 1.0,
            appearance: serde_json::json!({}),
            peek_modifier: "shift".into(),
            external_space: String::new(),
            routes: vec![],
            adblock: true,
            adblock_allow: vec![],
            passwords: true,
            archive_hours: 0,
            language: "auto".into(),
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
    /// Closed and auto-archived tabs, newest last.
    archive: Vec<Archived>,
    /// Spaces shown in extra windows, reopened at startup.
    windows: Vec<String>,
    /// Locked Spaces opened with Windows Hello this session.
    #[serde(skip_deserializing)]
    unlocked: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Archived {
    space: String,
    url: String,
    title: String,
    time: u64,
}

const ARCHIVE_LIMIT: usize = 300;

impl Data {
    fn archive_tab(&mut self, tab: &Tab) {
        if tab.url.is_empty() || self.is_private(&tab.space) {
            return;
        }
        self.archive
            .retain(|a| !(a.space == tab.space && a.url == tab.url));
        self.archive.push(Archived {
            space: tab.space.clone(),
            url: tab.url.clone(),
            title: tab.title.clone(),
            time: now(),
        });
        if self.archive.len() > ARCHIVE_LIMIT {
            let extra = self.archive.len() - ARCHIVE_LIMIT;
            self.archive.drain(..extra);
        }
    }

    /// Search engine prefix for a Space.
    fn search_for(&self, space: &str) -> &str {
        self.spaces
            .iter()
            .find(|s| s.id == space && !s.search.is_empty())
            .map(|s| s.search.as_str())
            .unwrap_or(&self.settings.search)
    }

    fn is_locked(&self, space: &str) -> bool {
        self.spaces.iter().any(|s| s.id == space && s.locked)
            && !self.unlocked.iter().any(|u| u == space)
    }
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
    /// Tabs on screen in the window showing `space`.
    fn visible_in(&self, space: &str) -> Vec<u64> {
        let Some(id) = self.active_tab.get(space).copied() else {
            return vec![];
        };
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
                downloads: String::new(),
                look: serde_json::Value::Null,
                locked: false,
                search: String::new(),
            });
        }
        let spaces: Vec<String> = self
            .spaces
            .iter()
            .filter(|s| !s.private)
            .map(|s| s.id.clone())
            .collect();
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
        self.splits
            .retain(|[a, b]| a != b && space_of(*a).is_some() && space_of(*a) == space_of(*b));
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
    /// Floating sidebar currently revealed.
    peek: AtomicBool,
    /// Section shown in each window's settings panel, when open.
    panel: Mutex<HashMap<String, String>>,
    /// Space shown by each window (window label -> Space id).
    windows: Mutex<HashMap<String, String>>,
    /// Label of the window the person is using.
    focused: Mutex<String>,
    /// Downloads of this session, newest last.
    downloads: Mutex<Vec<Download>>,
}

#[derive(Clone, Serialize)]
pub struct Download {
    id: u64,
    name: String,
    path: String,
    host: String,
    received: i64,
    total: i64,
    /// progress | done | failed
    state: &'static str,
    time: u64,
    #[serde(skip)]
    last_emit: u128,
}

fn emit_downloads(app: &AppHandle) {
    let list = state(app).downloads.lock().unwrap().clone();
    let _ = app.emit("downloads", list);
}

fn millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub(crate) fn download_started(app: &AppHandle, path: &str, uri: &str, total: i64) -> u64 {
    let id = with_data(app, |d| d.new_id());
    let name = PathBuf::from(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".into());
    diag::info(&format!("download iniciado de {}", diag::host(uri)));
    {
        let mut list = state(app).downloads.lock().unwrap();
        list.push(Download {
            id,
            name,
            path: path.into(),
            host: diag::host(uri),
            received: 0,
            total,
            state: "progress",
            time: now(),
            last_emit: 0,
        });
        if list.len() > 50 {
            list.remove(0);
        }
    }
    emit_downloads(app);
    id
}

pub(crate) fn download_progress(app: &AppHandle, id: u64, received: i64, total: i64) {
    let emit = {
        let mut list = state(app).downloads.lock().unwrap();
        let Some(d) = list.iter_mut().find(|d| d.id == id) else {
            return;
        };
        d.received = received;
        d.total = total;
        let now = millis();
        let emit = now - d.last_emit > 300;
        if emit {
            d.last_emit = now;
        }
        emit
    };
    if emit {
        emit_downloads(app);
    }
}

pub(crate) fn download_state(
    app: &AppHandle,
    id: u64,
    new_state: &'static str,
    received: i64,
    total: i64,
) {
    if let Some(d) = state(app)
        .downloads
        .lock()
        .unwrap()
        .iter_mut()
        .find(|d| d.id == id)
    {
        d.state = new_state;
        d.received = received;
        d.total = total;
    }
    if new_state == "failed" {
        diag::warn("download interrompido");
    }
    emit_downloads(app);
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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
            (
                if title.is_empty() {
                    old.title
                } else {
                    title.to_string()
                },
                old.count.saturating_add(1),
            )
        }
        None => (title.to_string(), 1),
    };
    h.push(Visit {
        space: space.into(),
        url: url.into(),
        title,
        time: now(),
        count,
    });
    if h.len() > HISTORY_LIMIT {
        let extra = h.len() - HISTORY_LIMIT;
        h.drain(..extra);
    }
    s.history_dirty.store(true, Ordering::Relaxed);
}

fn set_visit_title(app: &AppHandle, space: &str, url: &str, title: &str) {
    let s = state(app);
    if let Some(v) = s
        .history
        .lock()
        .unwrap()
        .iter_mut()
        .rev()
        .find(|v| v.space == space && v.url == url)
    {
        v.title = title.into();
        s.history_dirty.store(true, Ordering::Relaxed);
    }
}

/// What the start page shows: the Space's look and its most visited sites.
fn start_page_data(app: &AppHandle, space: &str) -> String {
    let (sp, search, lang) = with_data(app, |d| {
        (
            d.spaces.iter().find(|s| s.id == space).cloned(),
            d.search_for(space).to_string(),
            d.settings.language.clone(),
        )
    });
    let mut by_host: HashMap<String, (u32, Visit)> = HashMap::new();
    for v in state(app)
        .history
        .lock()
        .unwrap()
        .iter()
        .filter(|v| v.space == space)
    {
        let Some(host) = Url::parse(&v.url)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
        else {
            continue;
        };
        let e = by_host.entry(host).or_insert((0, v.clone()));
        e.0 += v.count;
        if v.count > e.1.count {
            e.1 = v.clone();
        }
    }
    let mut top: Vec<(u32, Visit)> = by_host.into_values().collect();
    top.sort_by_key(|t| std::cmp::Reverse(t.0));
    let sites: Vec<_> = top
        .into_iter()
        .take(8)
        .map(|(_, v)| serde_json::json!({ "url": v.url, "title": v.title }))
        .collect();
    serde_json::json!({ "space": sp, "search": search, "sites": sites, "lang": lang }).to_string()
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
    let visible = shown_tabs(app);
    let asleep = with_data(app, |d| {
        let limit = d.settings.sleep_minutes * 60;
        let cutoff = now().saturating_sub(limit);
        let mut ids = vec![];
        for t in d.tabs.iter_mut() {
            if limit > 0
                && t.awake
                && !t.pinned
                && !visible.contains(&t.id)
                && t.last_active < cutoff
            {
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

/// The Space a link from another app opens in: a matching route, the
/// configured Space, or the current one.
fn route_space(d: &Data, url: &str) -> String {
    let url = url.to_lowercase();
    let exists = |id: &str| d.spaces.iter().any(|s| s.id == id);
    d.settings
        .routes
        .iter()
        .find(|r| {
            !r.pattern.trim().is_empty()
                && url.contains(&r.pattern.trim().to_lowercase())
                && exists(&r.space)
        })
        .map(|r| r.space.clone())
        .or_else(|| exists(&d.settings.external_space).then(|| d.settings.external_space.clone()))
        .unwrap_or_else(|| d.active_space.clone())
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
        return Url::from_file_path(fs::canonicalize(&path).ok()?)
            .ok()
            .map(|u| u.to_string());
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
    // Each window's sidebar sees its own Space as the active one.
    let wins = s.windows.lock().unwrap().clone();
    for (win, space) in wins {
        let mut view = data.clone();
        view.active_space = space;
        let _ = app.emit_to(ui_of(&win), "state", view.clone());
        let _ = app.emit_to(panel_of(&win), "state", view);
    }
}

fn label(id: u64) -> String {
    format!("tab-{id}")
}

/// Label suffix of a window's own webviews ("" for the main window).
fn sfx(win: &str) -> String {
    if win == WINDOW {
        String::new()
    } else {
        format!("-{win}")
    }
}
fn ui_of(win: &str) -> String {
    format!("{UI}{}", sfx(win))
}
fn panel_of(win: &str) -> String {
    format!("{PANEL}{}", sfx(win))
}

fn focused_label(app: &AppHandle) -> String {
    let f = state(app).focused.lock().unwrap().clone();
    if app.get_window(&f).is_some() {
        f
    } else {
        WINDOW.to_string()
    }
}

/// The window the person is using.
fn window(app: &AppHandle) -> Option<Window> {
    app.get_window(&focused_label(app))
        .or_else(|| app.get_window(WINDOW))
}

/// Tabs visible in any window.
fn shown_tabs(app: &AppHandle) -> Vec<u64> {
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
fn sync_windows(app: &AppHandle) {
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
        if matches!(
            u.scheme(),
            "http" | "https" | "file" | "about" | "data" | "view-source"
        ) {
            return u.to_string();
        }
    }
    if !s.contains(char::is_whitespace) {
        let scheme = if s.starts_with("localhost") || s.starts_with("127.") {
            "http"
        } else {
            "https"
        };
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
fn layout_rects(
    win: &Window,
    settings: &Settings,
    collapsed: bool,
) -> Option<[(PhysicalPosition<i32>, PhysicalSize<u32>); 2]> {
    let right = settings.sidebar_right;
    let size = win.inner_size().ok()?;
    let scale = win.scale_factor().ok()?;
    let width = if collapsed {
        FLOAT_EDGE
    } else {
        settings.sidebar_width.clamp(200.0, 420.0)
    };
    let sw = ((width * scale) as u32).min(size.width);
    let m = (CARD_MARGIN * scale) as u32;
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

fn update_tab(app: &AppHandle, id: u64, f: impl FnOnce(&mut Tab)) {
    let found = with_data(app, |d| {
        d.tabs.iter_mut().find(|t| t.id == id).map(f).is_some()
    });
    if found {
        commit(app);
    }
}

/// Creates a tab's webview; `layout` positions it right after.
fn create_tab_webview(app: &AppHandle, win: &Window, tab: &Tab) -> tauri::Result<()> {
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
        .initialization_script(shortcuts_js(&with_data(app, |d| {
            d.settings.peek_modifier.clone()
        })))
        .zoom_hotkeys_enabled(true)
        .browser_extensions_enabled(true)
        .devtools(true)
        .on_page_load(move |wv, payload| {
            let url = payload.url();
            let newtab = is_newtab(url);
            let url = if newtab {
                String::new()
            } else {
                url.to_string()
            };
            let loading = matches!(payload.event(), PageLoadEvent::Started);
            let app = wv.app_handle();
            if !newtab && !loading {
                let css = blocker::cosmetic_css(&url);
                if !css.is_empty() {
                    let css = serde_json::to_string(&css).unwrap_or_default();
                    let _ = wv.eval(format!(
                        "(() => {{ const s = document.createElement('style'); s.textContent = {css}; document.documentElement.append(s); }})()"
                    ));
                }
            }
            if newtab && !loading {
                let _ = wv.eval(format!(
                    "window.crestStart && window.crestStart({})",
                    start_page_data(app, &space)
                ));
            }
            let mut title = String::new();
            update_tab(app, id, |t| {
                if !loading && !t.url.is_empty() && t.url != url {
                    if t.trail_nav {
                        t.trail_nav = false;
                    } else {
                        t.trail.push(t.url.clone());
                        if t.trail.len() > 15 {
                            t.trail.remove(0);
                        }
                    }
                }
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
                let arg = url
                    .query_pairs()
                    .find(|(k, _)| k == "u")
                    .map(|(_, v)| v.into_owned());
                spawn(async move { run_cmd(&app, &cmd, arg).await });
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
    let zoom = with_data(app, |d| d.settings.default_zoom);
    let wv = win.add_child(
        builder,
        PhysicalPosition::new(0, 0),
        PhysicalSize::new(1, 1),
    )?;
    if (zoom - 1.0).abs() > f64::EPSILON {
        let _ = wv.set_zoom(zoom);
    }
    #[cfg(windows)]
    {
        let (downloads, passwords) = with_data(app, |d| {
            let dir = d
                .spaces
                .iter()
                .find(|s| s.id == tab.space)
                .map(|s| s.downloads.clone())
                .unwrap_or_default();
            (
                (!dir.is_empty()).then(|| PathBuf::from(dir)),
                d.settings.passwords,
            )
        });
        webview2::install(app, &wv, Some(id), downloads, passwords);
        if tab.muted {
            webview2::set_muted(&wv, true);
        }
    }
    with_data(app, |d| {
        if let Some(t) = d.tabs.iter_mut().find(|t| t.id == id) {
            t.awake = true;
        }
    });
    Ok(())
}

/// Creates the visible tabs' webviews if needed, shows them and hides the rest.
fn apply_layout(app: &AppHandle) {
    layout(app, true);
}

/// `create` must be false on the main thread: creating a webview there deadlocks on Windows.
fn layout(app: &AppHandle, create: bool) {
    sync_windows(app);
    let wins = state(app).windows.lock().unwrap().clone();
    for (win, space) in wins {
        layout_window(app, &win, &space, create);
    }
}

fn layout_window(app: &AppHandle, wl: &str, space: &str, create: bool) {
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
    let (ui_l, panel_l, overlay_l, peek_l) = (
        ui_of(wl),
        panel_of(wl),
        overlay::overlay_of(wl),
        overlay::peek_of(wl),
    );
    let (pos, size) = rects[1];
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
            WebviewBuilder::new(&panel_l, url).disable_drag_drop_handler(),
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
        } else if l == peek_l {
            let (p, s) = overlay::peek_rect(rects[1], win.scale_factor().unwrap_or(1.0));
            let _ = wv.set_position(p);
            let _ = wv.set_size(s);
        } else if panel.is_some() {
            let _ = wv.hide();
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
    if with_data(app, |d| d.settings.floating_sidebar)
        && !state(app).peek.swap(true, Ordering::Relaxed)
    {
        layout(app, false);
    }
    let ui = ui_of(&focused_label(app));
    if let Some(wv) = app.get_webview(&ui) {
        let _ = wv.set_focus();
        let _ = app.emit_to(ui.as_str(), "focus-address", ());
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
        d.tabs.push(Tab {
            id,
            space: space.clone(),
            url,
            title: String::new(),
            pinned: false,
            folder: None,
            loading: true,
            audio: false,
            muted: false,
            awake: false,
            last_active: now(),
            trail: vec![],
            trail_nav: false,
        });
        d.active_tab.insert(space, id);
    });
    apply_layout(app);
    commit(app);
}

async fn run_cmd(app: &AppHandle, cmd: &str, arg: Option<String>) {
    match cmd {
        "palette" => overlay::open_palette(app),
        "newwindow" => {
            let _ = new_window(app.clone(), None).await;
        }
        "peek" => overlay::open_peek(app, arg.as_deref().unwrap_or("")),
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
        "sidebar" => {
            with_data(app, |d| {
                d.settings.floating_sidebar = !d.settings.floating_sidebar
            });
            state(app).peek.store(false, Ordering::Relaxed);
            apply_layout(app);
            commit(app);
        }
        "reopen" => reopen_tab(app.clone()).await,
        "history" => {
            let ui = ui_of(&focused_label(app));
            if let Some(wv) = app.get_webview(&ui) {
                let _ = wv.set_focus();
                let _ = app.emit_to(ui.as_str(), "open-history", ());
            }
        }
        "next" | "prev" => {
            let next = with_data(app, |d| {
                let tabs = d.space_tabs(&d.active_space);
                let i = tabs.iter().position(|t| Some(*t) == d.active_tab_id())?;
                let n = tabs.len();
                Some(
                    tabs[if cmd == "next" {
                        (i + 1) % n
                    } else {
                        (i + n - 1) % n
                    }],
                )
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
                if n == 9 {
                    tabs.last().copied()
                } else {
                    tabs.get(n - 1).copied()
                }
            });
            if let Some(id) = id {
                activate_tab(app.clone(), id).await;
            }
        }
        _ => {}
    }
}

// Commands are async so webview creation never runs on the main thread (deadlocks on Windows).

/// Runs a named command (the same ones pages send) from the UI.
#[tauri::command]
async fn run_command(app: AppHandle, cmd: String) {
    run_cmd(&app, &cmd, None).await;
}

#[tauri::command]
async fn get_state(app: AppHandle, webview: tauri::Webview) -> Data {
    let space = state(&app)
        .windows
        .lock()
        .unwrap()
        .get(webview.window().label())
        .cloned();
    let mut data = with_data(&app, |d| d.clone());
    if let Some(space) = space {
        data.active_space = space;
    }
    data
}

#[tauri::command]
async fn new_tab(app: AppHandle, url: Option<String>) {
    let url = with_data(&app, |d| {
        to_url(url.as_deref().unwrap_or(""), d.search_for(&d.active_space))
    });
    let blank = url.is_empty();
    open_tab(&app, url).await;
    if blank {
        focus_address(&app);
    } else {
        focus_active(&app);
    }
}

/// Asks for Windows Hello when `space` is locked; true when it may be shown.
async fn ensure_unlocked(app: &AppHandle, space: &str) -> bool {
    if !with_data(app, |d| d.is_locked(space)) {
        return true;
    }
    let name = with_data(app, |d| {
        d.spaces
            .iter()
            .find(|s| s.id == space)
            .map(|s| s.name.clone())
            .unwrap_or_default()
    });
    if !verify_hello(app, &format!("Abrir o Space \"{name}\" no Blazon")).await {
        return false;
    }
    with_data(app, |d| d.unlocked.push(space.to_string()));
    true
}

async fn verify_hello(app: &AppHandle, message: &str) -> bool {
    #[cfg(windows)]
    {
        let Some(hwnd) = window(app)
            .and_then(|w| w.hwnd().ok())
            .map(|h| h.0 as isize)
        else {
            return false;
        };
        let message = message.to_string();
        tauri::async_runtime::spawn_blocking(move || hello::verify(hwnd, &message))
            .await
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, message);
        false
    }
}

#[tauri::command]
async fn activate_tab(app: AppHandle, id: u64) {
    let Some(space) = with_data(&app, |d| {
        d.tabs.iter().find(|t| t.id == id).map(|t| t.space.clone())
    }) else {
        return;
    };
    if !ensure_unlocked(&app, &space).await {
        return;
    }
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
        let url = d
            .tabs
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.url.clone())
            .unwrap_or_default();
        d.unsplit(id);
        if let Some(tab) = d.tabs.iter().find(|t| t.id == id).cloned() {
            d.archive_tab(&tab);
        }
        if !url.is_empty() {
            let mut closed = state(&app).closed.lock().unwrap();
            closed.push((space.clone(), url));
            if closed.len() > CLOSED_LIMIT {
                closed.remove(0);
            }
        }
        d.tabs.retain(|t| t.id != id);
        if d.active_tab.get(&space) == Some(&id) {
            match order
                .get(i + 1)
                .or(i.checked_sub(1).and_then(|j| order.get(j)))
            {
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
    let Some((space, url)) = state(&app).closed.lock().unwrap().pop() else {
        return;
    };
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
        .filter(|t| {
            t.space == space && Some(t.id) != active && !t.url.is_empty() && hit(&t.url, &t.title)
        })
        .take(3)
        .map(|t| Suggestion {
            kind: "tab",
            url: t.url.clone(),
            title: t.title.clone(),
            tab: Some(t.id),
        })
        .collect();
    let history = state(&app).history.lock().unwrap();
    for v in history.iter().rev() {
        if out.len() >= 8 {
            break;
        }
        if v.space == space && hit(&v.url, &v.title) && !out.iter().any(|s| s.url == v.url) {
            out.push(Suggestion {
                kind: "history",
                url: v.url.clone(),
                title: v.title.clone(),
                tab: None,
            });
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
        .filter(|v| {
            v.space == space
                && (q.is_empty()
                    || v.url.to_lowercase().contains(&q)
                    || v.title.to_lowercase().contains(&q))
        })
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
    let (id, url) = with_data(&app, |d| {
        (
            d.active_tab_id(),
            to_url(&input, d.search_for(&d.active_space)),
        )
    });
    let Some(id) = id else {
        return new_tab(app, Some(input)).await;
    };
    let Some(wv) = app.get_webview(&label(id)) else {
        return;
    };
    if let Ok(u) = Url::parse(&url) {
        let _ = wv.navigate(u);
        let _ = wv.set_focus();
    }
}

async fn can_go_back(wv: &tauri::Webview) -> bool {
    #[cfg(windows)]
    {
        webview2::can_go_back(wv).await
    }
    #[cfg(not(windows))]
    {
        let _ = wv;
        true
    }
}

#[tauri::command]
async fn nav(app: AppHandle, action: String) {
    let Some(id) = with_data(&app, |d| d.active_tab_id()) else {
        return;
    };
    let Some(wv) = app.get_webview(&label(id)) else {
        return;
    };
    if action == "back" && !can_go_back(&wv).await {
        // Fresh webview (e.g. after a restart): go back through the saved trail.
        let prev = with_data(&app, |d| {
            let t = d.tabs.iter_mut().find(|t| t.id == id)?;
            let prev = t.trail.pop()?;
            t.trail_nav = true;
            Some(prev)
        });
        if let Some(u) = prev.and_then(|p| Url::parse(&p).ok()) {
            let _ = wv.navigate(u);
        }
        return;
    }
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
    let space = with_data(&app, |d| {
        d.tabs.iter().find(|t| t.id == id).map(|t| t.space.clone())
    });
    let Some(space) = space else { return };
    let other = match with {
        Some(o) => o,
        None => with_data(&app, |d| {
            let o = d.new_id();
            d.tabs.push(Tab {
                id: o,
                space: space.clone(),
                url: String::new(),
                title: String::new(),
                pinned: false,
                folder: None,
                loading: false,
                audio: false,
                muted: false,
                awake: false,
                last_active: now(),
                trail: vec![],
                trail_nav: false,
            });
            o
        }),
    };
    let ok = with_data(&app, |d| {
        if id == other || d.tabs.iter().find(|t| t.id == other).map(|t| &t.space) != Some(&space) {
            return false;
        }
        d.unsplit(id);
        d.unsplit(other);
        d.splits.push([id, other]);
        d.active_space = space.clone();
        d.active_tab
            .insert(space, if with.is_some() { id } else { other });
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

const HOUSES: [&str; 9] = [
    "winter", "lion", "storm", "dragon", "meadow", "iron", "river", "sun", "vigil",
];
const HOUSE_ACCENTS: [&str; 9] = [
    "#86adc4", "#c9a254", "#b08f4a", "#be4438", "#bca766", "#9c9781", "#a85c41", "#d09e65",
    "#7b838c",
];

fn push_tab(d: &mut Data, space: &str, e: import::Entry, pinned: bool, folder: Option<u64>) {
    let id = d.new_id();
    d.tabs.push(Tab {
        id,
        space: space.into(),
        url: e.url,
        title: e.title,
        pinned,
        folder,
        loading: false,
        audio: false,
        muted: false,
        awake: false,
        last_active: 0,
        trail: vec![],
        trail_nav: false,
    });
}

fn push_folders(d: &mut Data, space: &str, folders: Vec<import::Folder>) {
    for f in folders {
        let fid = d.new_id();
        d.folders.push(Folder {
            id: fid,
            space: space.into(),
            name: f.name,
            open: false,
            color: String::new(),
            icon: String::new(),
        });
        for e in f.tabs {
            push_tab(d, space, e, false, Some(fid));
        }
    }
}

/// Adds imported visits to a Space's history, merging repeated addresses.
fn merge_history(app: &AppHandle, space: &str, visits: Vec<import::Visit>) -> usize {
    let n = visits.len();
    let s = state(app);
    let mut h = s.history.lock().unwrap();
    let mut index: HashMap<String, usize> = h
        .iter()
        .enumerate()
        .filter(|(_, v)| v.space == space)
        .map(|(i, v)| (v.url.clone(), i))
        .collect();
    for v in visits {
        match index.get(&v.url) {
            Some(&i) => {
                let e = &mut h[i];
                e.count = e.count.saturating_add(v.count);
                e.time = e.time.max(v.time);
                if e.title.is_empty() {
                    e.title = v.title;
                }
            }
            None => {
                index.insert(v.url.clone(), h.len());
                h.push(Visit {
                    space: space.into(),
                    url: v.url,
                    title: v.title,
                    time: v.time,
                    count: v.count,
                });
            }
        }
    }
    h.sort_by_key(|v| v.time);
    if h.len() > HISTORY_LIMIT {
        let extra = h.len() - HISTORY_LIMIT;
        h.drain(..extra);
    }
    s.history_dirty.store(true, Ordering::Relaxed);
    drop(h);
    save_history(app);
    n
}

/// Puts imported data in the current Space or a new one named after the source.
/// Imported tabs start asleep. Returns a summary for the person.
fn apply_import(app: &AppHandle, imported: import::Imported, new_space: Option<String>) -> String {
    let (target, summary) = with_data(app, |d| {
        let target = match new_space {
            Some(name) => {
                let sid = format!("space-{}", d.new_id());
                let h = d.spaces.len() % HOUSES.len();
                d.spaces.push(Space {
                    id: sid.clone(),
                    name,
                    color: HOUSE_ACCENTS[h].into(),
                    private: false,
                    house: HOUSES[h].into(),
                    downloads: String::new(),
                    look: serde_json::Value::Null,
                    locked: false,
                    search: String::new(),
                });
                sid
            }
            None => d.active_space.clone(),
        };
        let before = d.tabs.len();
        let mut parts = vec![];
        if !imported.spaces.is_empty() {
            parts.push(format!("{} Spaces", imported.spaces.len()));
        }
        for sp in imported.spaces {
            let sid = format!("space-{}", d.new_id());
            let h = d.spaces.len() % HOUSES.len();
            d.spaces.push(Space {
                id: sid.clone(),
                name: sp.name,
                color: HOUSE_ACCENTS[h].into(),
                private: false,
                house: HOUSES[h].into(),
                downloads: String::new(),
                look: serde_json::Value::Null,
                locked: false,
                search: String::new(),
            });
            for e in sp.pinned {
                push_tab(d, &sid, e, true, None);
            }
            push_folders(d, &sid, sp.folders);
            for e in sp.tabs {
                push_tab(d, &sid, e, false, None);
            }
        }
        if !imported.folders.is_empty() {
            parts.push(format!("{} pastas de favoritos", imported.folders.len()));
        }
        push_folders(d, &target, imported.folders);
        let tabs = d.tabs.len() - before;
        if tabs > 0 {
            parts.push(format!("{tabs} abas"));
        }
        (target, parts)
    });
    let mut parts = summary;
    if !imported.history.is_empty() {
        let n = merge_history(app, &target, imported.history);
        parts.push(format!("{n} páginas do histórico"));
    }
    commit(app);
    if parts.is_empty() {
        "Nada para importar.".into()
    } else {
        format!("Importado: {}.", parts.join(", "))
    }
}

/// Imports bookmarks and/or history from a detected browser profile.
#[tauri::command]
async fn import_browser(
    app: AppHandle,
    id: String,
    bookmarks: bool,
    history: bool,
    new_space: Option<String>,
) -> Result<String, String> {
    let imported =
        tauri::async_runtime::spawn_blocking(move || import::read(&id, bookmarks, history))
            .await
            .map_err(|e| e.to_string())?
            .inspect_err(|e| diag::warn(&format!("importação falhou: {e}")))?;
    let summary = apply_import(&app, imported, new_space);
    diag::info(&summary);
    Ok(summary)
}

/// Imports a bookmarks HTML file exported by any browser.
#[tauri::command]
async fn import_html(app: AppHandle, new_space: Option<String>) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let Some(file) = app
        .dialog()
        .file()
        .add_filter("Favoritos (HTML)", &["html", "htm"])
        .blocking_pick_file()
    else {
        return Err("Nenhum arquivo escolhido.".into());
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let imported = import::read_html(&path)?;
    Ok(apply_import(&app, imported, new_space))
}

#[tauri::command]
async fn set_space_search(app: AppHandle, id: String, search: String) {
    with_data(&app, |d| {
        if let Some(s) = d.spaces.iter_mut().find(|s| s.id == id) {
            s.search = search;
        }
    });
    commit(&app);
}

/// Moves a Space to position `to` in the switcher.
#[tauri::command]
async fn move_space(app: AppHandle, id: String, to: usize) {
    with_data(&app, |d| {
        if let Some(i) = d.spaces.iter().position(|s| s.id == id) {
            let s = d.spaces.remove(i);
            let to = to.min(d.spaces.len());
            d.spaces.insert(to, s);
        }
    });
    commit(&app);
}

#[tauri::command]
async fn add_folder(app: AppHandle, name: String, tab: Option<u64>) {
    with_data(&app, |d| {
        let id = d.new_id();
        let space = d.active_space.clone();
        d.folders.push(Folder {
            id,
            space,
            name,
            open: true,
            color: String::new(),
            icon: String::new(),
        });
        if let Some(t) = d.tabs.iter_mut().find(|t| Some(t.id) == tab) {
            t.folder = Some(id);
            t.pinned = false;
        }
    });
    commit(&app);
}

#[tauri::command]
async fn update_folder(
    app: AppHandle,
    id: u64,
    name: Option<String>,
    open: Option<bool>,
    color: Option<String>,
    icon: Option<String>,
) {
    with_data(&app, |d| {
        if let Some(f) = d.folders.iter_mut().find(|f| f.id == id) {
            if let Some(color) = color {
                f.color = color;
            }
            if let Some(icon) = icon {
                f.icon = icon.chars().take(4).collect();
            }
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
    let url = with_data(&app, |d| {
        d.tabs.iter().find(|t| t.id == id).map(|t| t.url.clone())
    });
    if let Some(url) = url {
        open_tab(&app, url).await;
        focus_active(&app);
    }
}

#[tauri::command]
async fn add_space(
    app: AppHandle,
    name: String,
    color: String,
    private: bool,
    house: String,
    look: Option<serde_json::Value>,
) -> String {
    let id = with_data(&app, |d| {
        let id = format!("space-{}", d.new_id());
        d.spaces.push(Space {
            id: id.clone(),
            name,
            color,
            private,
            house,
            downloads: String::new(),
            look: look.unwrap_or_default(),
            locked: false,
            search: String::new(),
        });
        d.active_space = id.clone();
        id
    });
    open_tab(&app, String::new()).await;
    id
}

#[tauri::command]
async fn switch_space(app: AppHandle, id: String) {
    if !ensure_unlocked(&app, &id).await {
        return;
    }
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
async fn update_space(
    app: AppHandle,
    id: String,
    name: String,
    color: String,
    house: String,
    look: Option<serde_json::Value>,
) {
    with_data(&app, |d| {
        if let Some(s) = d.spaces.iter_mut().find(|s| s.id == id) {
            s.name = name;
            s.color = color;
            s.house = house;
            if let Some(look) = look {
                s.look = look;
            }
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
        let closed: Vec<u64> = d
            .tabs
            .iter()
            .filter(|t| t.space == id)
            .map(|t| t.id)
            .collect();
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
async fn sidebar_peek(app: AppHandle, on: bool) {
    let s = state(&app);
    if s.peek.swap(on, Ordering::Relaxed) != on && with_data(&app, |d| d.settings.floating_sidebar)
    {
        apply_layout(&app);
    }
}

#[tauri::command]
async fn open_panel(app: AppHandle, section: String) {
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

#[tauri::command]
async fn toggle_mute(app: AppHandle, id: u64) {
    let mut muted = false;
    update_tab(&app, id, |t| {
        t.muted = !t.muted;
        muted = t.muted;
    });
    #[cfg(windows)]
    if let Some(wv) = app.get_webview(&label(id)) {
        webview2::set_muted(&wv, muted);
    }
    #[cfg(not(windows))]
    let _ = muted;
}

/// Pauses or resumes blocking on the active tab's site.
#[tauri::command]
async fn toggle_site_blocking(app: AppHandle) -> Option<bool> {
    let host = with_data(&app, |d| {
        let id = d.active_tab_id()?;
        let url = d.tabs.iter().find(|t| t.id == id)?.url.clone();
        Url::parse(&url)
            .ok()?
            .host_str()
            .map(|h| h.trim_start_matches("www.").to_string())
    })?;
    let paused = with_data(&app, |d| {
        let list = &mut d.settings.adblock_allow;
        if let Some(i) = list.iter().position(|h| *h == host) {
            list.remove(i);
            false
        } else {
            list.push(host);
            true
        }
    });
    with_data(&app, |d| {
        blocker::configure(d.settings.adblock, &d.settings.adblock_allow)
    });
    commit(&app);
    nav(app, "reload".into()).await;
    Some(paused)
}

/// Native folder picker for a Space's downloads.
#[tauri::command]
async fn pick_folder(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.to_string())
}

#[tauri::command]
async fn set_space_downloads(app: AppHandle, id: String, path: String) {
    with_data(&app, |d| {
        if let Some(s) = d.spaces.iter_mut().find(|s| s.id == id) {
            s.downloads = path;
        }
    });
    commit(&app);
}

/// Locks or unlocks a Space; either way needs Windows Hello first.
#[tauri::command]
async fn set_space_lock(app: AppHandle, id: String, locked: bool) -> Result<(), String> {
    #[cfg(windows)]
    if !tauri::async_runtime::spawn_blocking(hello::available)
        .await
        .unwrap_or(false)
    {
        return Err(
            "O Windows Hello (PIN, rosto ou digital) não está configurado neste PC.".into(),
        );
    }
    if !verify_hello(
        &app,
        if locked {
            "Bloquear este Space no Blazon"
        } else {
            "Desbloquear este Space no Blazon"
        },
    )
    .await
    {
        return Err("Verificação cancelada.".into());
    }
    with_data(&app, |d| {
        if let Some(s) = d.spaces.iter_mut().find(|s| s.id == id) {
            s.locked = locked;
        }
        if !d.unlocked.contains(&id) {
            d.unlocked.push(id.clone());
        }
    });
    commit(&app);
    Ok(())
}

/// Locks every locked Space again and leaves them if one is showing.
#[tauri::command]
async fn lock_now(app: AppHandle) {
    let fallback = with_data(&app, |d| {
        d.unlocked.clear();
        if !d.is_locked(&d.active_space) {
            return None;
        }
        d.spaces.iter().find(|s| !s.locked).map(|s| s.id.clone())
    });
    match fallback {
        Some(id) => switch_space(app, id).await,
        None => commit(&app),
    }
}

#[tauri::command]
async fn restore_archived(app: AppHandle, url: String, space: String) {
    if !ensure_unlocked(&app, &space).await {
        return;
    }
    let ok = with_data(&app, |d| {
        d.archive.retain(|a| !(a.space == space && a.url == url));
        let ok = d.spaces.iter().any(|s| s.id == space);
        if ok {
            d.active_space = space.clone();
        }
        ok
    });
    if ok {
        open_tab(&app, url).await;
        focus_active(&app);
    }
}

#[tauri::command]
async fn clear_archive(app: AppHandle) {
    with_data(&app, |d| {
        let space = d.active_space.clone();
        d.archive.retain(|a| a.space != space);
    });
    commit(&app);
}

const READER_JS: &str = include_str!("reader.js");
const READABILITY_JS: &str = include_str!("vendor/Readability.js");

/// Toggles reader mode on the active tab.
#[tauri::command]
async fn reader_mode(app: AppHandle) {
    let Some(id) = with_data(&app, |d| d.active_tab_id()) else {
        return;
    };
    if let Some(wv) = app.get_webview(&label(id)) {
        let _ = wv.eval(READER_JS.replace("/*READABILITY*/", READABILITY_JS));
    }
}

/// Archives tabs idle longer than the archive setting.
fn archive_idle_tabs(app: &AppHandle) {
    let visible = shown_tabs(app);
    let gone = with_data(app, |d| {
        let limit = d.settings.archive_hours * 3600;
        if limit == 0 {
            return vec![];
        }
        let cutoff = now().saturating_sub(limit);
        let idle: Vec<Tab> = d
            .tabs
            .iter()
            .filter(|t| {
                !t.pinned
                    && t.folder.is_none()
                    && !visible.contains(&t.id)
                    && t.last_active > 0
                    && t.last_active < cutoff
                    && !t.audio
            })
            .cloned()
            .collect();
        for t in &idle {
            d.archive_tab(t);
            d.unsplit(t.id);
            if d.active_tab.get(&t.space) == Some(&t.id) {
                d.active_tab.remove(&t.space);
            }
        }
        d.tabs.retain(|t| !idle.iter().any(|i| i.id == t.id));
        idle.iter().map(|t| t.id).collect::<Vec<_>>()
    });
    for id in &gone {
        if let Some(wv) = app.get_webview(&label(*id)) {
            let _ = wv.close();
        }
    }
    if !gone.is_empty() {
        commit(app);
    }
}

/// Errors from the UI's own pages (sidebar, settings, overlay).
#[tauri::command]
async fn report_js_error(page: String, message: String, stack: Option<String>) {
    let stack = stack.unwrap_or_default();
    let stack: String = stack.lines().take(6).collect::<Vec<_>>().join(" | ");
    diag::error(&format!("JS [{page}] {message} {stack}"));
}

/// Last crash report, shown once after a crash.
#[tauri::command]
async fn take_crash_report() -> Option<String> {
    diag::take_crash()
}

#[tauri::command]
async fn open_logs_folder() {
    #[cfg(windows)]
    if let Some(dir) = diag::logs_dir() {
        let _ = std::process::Command::new("explorer").arg(dir).spawn();
    }
}

/// The text of a problem report: what happened, system, setup and recent log.
#[tauri::command]
async fn bug_report(app: AppHandle, description: String) -> String {
    let summary = with_data(&app, |d| {
        let s = &d.settings;
        format!(
            "- Spaces: {} ({} privados, {} bloqueados) · Abas: {} ({} acordadas) · Pastas: {}\n\
             - Barra: {}{}{} · Bloqueador: {} · Senhas: {} · Dormir: {} min · Arquivar: {} h · Zoom: {}%",
            d.spaces.len(),
            d.spaces.iter().filter(|x| x.private).count(),
            d.spaces.iter().filter(|x| x.locked).count(),
            d.tabs.len(),
            d.tabs.iter().filter(|t| t.awake).count(),
            d.folders.len(),
            if s.sidebar_right { "direita" } else { "esquerda" },
            if s.floating_sidebar { ", flutuante" } else { "" },
            if s.native_frame { ", moldura do Windows" } else { "" },
            if s.adblock { "ligado" } else { "desligado" },
            if s.passwords { "sim" } else { "não" },
            s.sleep_minutes,
            s.archive_hours,
            (s.default_zoom * 100.0).round(),
        )
    });
    let description = if description.trim().is_empty() {
        "(não descrito)".to_string()
    } else {
        description.trim().to_string()
    };
    let windows = tauri::async_runtime::spawn_blocking(diag::os_version)
        .await
        .unwrap_or_default();
    format!(
        "## Relatório de problema · Blazon {}\n\n**O que aconteceu:** {description}\n\n\
         - Sistema: {windows}\n- WebView2: {}\n{summary}\n\n## Log recente\n\n```\n{}\n```\n",
        env!("CARGO_PKG_VERSION"),
        diag::webview2_version(),
        diag::recent_lines(120).join("\n"),
    )
}

#[tauri::command]
async fn downloads_list(app: AppHandle) -> Vec<Download> {
    state(&app).downloads.lock().unwrap().clone()
}

/// open | folder | cancel | remove
#[tauri::command]
async fn download_action(app: AppHandle, id: u64, action: String) {
    let path = state(&app)
        .downloads
        .lock()
        .unwrap()
        .iter()
        .find(|d| d.id == id)
        .map(|d| d.path.clone());
    match action.as_str() {
        #[cfg(windows)]
        "open" => {
            if let Some(p) = path {
                let _ = std::process::Command::new("explorer").arg(p).spawn();
            }
        }
        #[cfg(windows)]
        "folder" => {
            if let Some(p) = path {
                let _ = std::process::Command::new("explorer")
                    .arg(format!("/select,{p}"))
                    .spawn();
            }
        }
        #[cfg(windows)]
        "cancel" => webview2::cancel_download(&app, id),
        "remove" => state(&app).downloads.lock().unwrap().retain(|d| d.id != id),
        _ => {}
    }
    emit_downloads(&app);
}

#[tauri::command]
async fn clear_downloads(app: AppHandle) {
    state(&app)
        .downloads
        .lock()
        .unwrap()
        .retain(|d| d.state == "progress");
    emit_downloads(&app);
}

/// A live webview of a Space, to reach its WebView2 profile.
fn space_webview(app: &AppHandle, space: &str) -> Result<tauri::Webview, String> {
    let ids = with_data(app, |d| {
        d.tabs
            .iter()
            .filter(|t| t.space == space && t.awake)
            .map(|t| t.id)
            .collect::<Vec<_>>()
    });
    ids.into_iter()
        .find_map(|id| app.get_webview(&label(id)))
        .ok_or_else(|| {
            "Abra uma aba neste Space primeiro (as configurações ficam no perfil do Space)."
                .to_string()
        })
}

#[cfg(windows)]
#[tauri::command]
async fn list_permissions(
    app: AppHandle,
    space: String,
) -> Result<Vec<webview2::Permission>, String> {
    Ok(webview2::list_permissions(&space_webview(&app, &space)?).await)
}

#[cfg(windows)]
#[tauri::command]
async fn set_permission(
    app: AppHandle,
    space: String,
    kind: i32,
    origin: String,
    value: String,
) -> Result<bool, String> {
    Ok(webview2::set_permission(&space_webview(&app, &space)?, kind, origin, value).await)
}

#[cfg(windows)]
#[tauri::command]
async fn list_extensions(
    app: AppHandle,
    space: String,
) -> Result<Vec<webview2::Extension>, String> {
    Ok(webview2::list_extensions(&space_webview(&app, &space)?).await)
}

/// Picks an unpacked extension folder and installs it in a Space.
#[cfg(windows)]
#[tauri::command]
async fn add_extension(app: AppHandle, space: String) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let wv = space_webview(&app, &space)?;
    let Some(folder) = app.dialog().file().blocking_pick_folder() else {
        return Err("Nenhuma pasta escolhida.".into());
    };
    let name = webview2::add_extension(&wv, folder.to_string()).await?;
    diag::info(&format!("extensão instalada: {name}"));
    Ok(name)
}

#[cfg(windows)]
#[tauri::command]
async fn change_extension(
    app: AppHandle,
    space: String,
    id: String,
    action: String,
) -> Result<bool, String> {
    Ok(webview2::change_extension(&space_webview(&app, &space)?, id, action).await)
}

#[cfg(not(windows))]
#[tauri::command]
async fn list_permissions() -> Result<Vec<()>, String> {
    Err("Só no Windows.".into())
}
#[cfg(not(windows))]
#[tauri::command]
async fn set_permission() -> Result<bool, String> {
    Err("Só no Windows.".into())
}
#[cfg(not(windows))]
#[tauri::command]
async fn list_extensions() -> Result<Vec<()>, String> {
    Err("Só no Windows.".into())
}
#[cfg(not(windows))]
#[tauri::command]
async fn add_extension() -> Result<String, String> {
    Err("Só no Windows.".into())
}
#[cfg(not(windows))]
#[tauri::command]
async fn change_extension() -> Result<bool, String> {
    Err("Só no Windows.".into())
}

/// Builds a browser window with its sidebar webview and event handlers.
/// Must not run on the main thread after startup (see `layout`).
fn build_window(app: &AppHandle, wl: &str) -> tauri::Result<Window> {
    let native_frame = with_data(app, |d| d.settings.native_frame);
    let win = tauri::window::WindowBuilder::new(app, wl)
        .title("Blazon")
        .inner_size(1280.0, 820.0)
        .min_inner_size(640.0, 400.0)
        .decorations(native_frame)
        .build()?;
    let size = win.inner_size()?;
    win.add_child(
        WebviewBuilder::new(ui_of(wl), WebviewUrl::App("index.html".into()))
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
static EXITING: AtomicBool = AtomicBool::new(false);

fn save_windows(app: &AppHandle) {
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
fn quiet_space(app: &AppHandle) -> String {
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
async fn new_window(app: AppHandle, space: Option<String>) -> Result<(), String> {
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

/// Opens Windows' default apps page so Crest can be picked as the browser.
#[tauri::command]
async fn open_default_apps() {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer")
        .arg("ms-settings:defaultapps")
        .spawn();
}

#[tauri::command]
async fn close_panel(app: AppHandle) {
    let win = focused_label(&app);
    state(&app).panel.lock().unwrap().remove(&win);
    if let Some(panel) = app.get_webview(&panel_of(&win)) {
        let _ = panel.close();
    }
    apply_layout(&app);
    focus_active(&app);
}

#[tauri::command]
async fn set_settings(app: AppHandle, settings: Settings) {
    let zoom_changed = with_data(&app, |d| {
        (d.settings.default_zoom - settings.default_zoom).abs() > f64::EPSILON
    });
    if zoom_changed {
        for wv in app.webviews().into_values() {
            if wv.label().starts_with("tab-") {
                let _ = wv.set_zoom(settings.default_zoom);
            }
        }
    }
    for win in app.windows().into_values() {
        let _ = win.set_decorations(settings.native_frame);
    }
    blocker::configure(settings.adblock, &settings.adblock_allow);
    with_data(&app, |d| d.settings = settings);
    commit(&app);
    apply_layout(&app);
}

fn main() {
    tauri::Builder::default()
        // A second launch (e.g. a link clicked in another app) opens its URLs here.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let app = app.clone();
            let urls: Vec<String> = args.iter().skip(1).filter_map(|a| arg_to_url(a)).collect();
            spawn(async move {
                for url in urls {
                    with_data(&app, |d| d.active_space = route_space(d, &url));
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
            run_command,
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
            sidebar_peek,
            open_panel,
            close_panel,
            open_default_apps,
            new_window,
            downloads_list,
            download_action,
            clear_downloads,
            list_permissions,
            set_permission,
            list_extensions,
            add_extension,
            change_extension,
            report_js_error,
            take_crash_report,
            open_logs_folder,
            bug_report,
            toggle_mute,
            toggle_site_blocking,
            pick_folder,
            set_space_downloads,
            set_space_lock,
            lock_now,
            restore_archived,
            clear_archive,
            reader_mode,
            reopen_tab,
            suggest,
            history_list,
            clear_history,
            add_folder,
            update_folder,
            delete_folder,
            duplicate_tab,
            split_tab,
            set_space_search,
            move_space,
            unsplit_tab,
            import_sources,
            import_browser,
            import_html,
            overlay::open_overlay,
            overlay::close_overlay,
            overlay::peek_to_tab,
            overlay::peek_url,
        ])
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            // Keep the data of installs from before the rename to Blazon.
            if !root.exists() {
                if let Some(old) = root.parent().map(|p| p.join("app.crest.windows")) {
                    if old.is_dir() {
                        let _ = fs::rename(&old, &root);
                    }
                }
            }
            fs::create_dir_all(&root)?;
            diag::init(&root);
            let file = root.join("state.json");
            let mut data: Data = match fs::read_to_string(&file) {
                Err(_) => Data::default(),
                Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
                    // Never overwrite unreadable data: keep a copy for recovery.
                    let backup = root.join(format!("state.broken-{}.json", now()));
                    let _ = fs::copy(&file, &backup);
                    diag::error(&format!(
                        "state.json ilegível ({e}); cópia salva em {}",
                        backup.display()
                    ));
                    Data::default()
                }),
            };
            data.repair();
            if data.active_tab_id().is_none() {
                let id = data.new_id();
                let space = data.active_space.clone();
                data.tabs.push(Tab {
                    id,
                    space: space.clone(),
                    url: String::new(),
                    title: String::new(),
                    pinned: false,
                    folder: None,
                    loading: false,
                    audio: false,
                    muted: false,
                    awake: false,
                    last_active: 0,
                    trail: vec![],
                    trail_nav: false,
                });
                data.active_tab.insert(space, id);
            }
            for url in std::env::args().skip(1).filter_map(|a| arg_to_url(&a)) {
                let id = data.new_id();
                let space = route_space(&data, &url);
                data.active_space = space.clone();
                data.tabs.push(Tab {
                    id,
                    space: space.clone(),
                    url,
                    title: String::new(),
                    pinned: false,
                    folder: None,
                    loading: false,
                    audio: false,
                    muted: false,
                    awake: false,
                    last_active: 0,
                    trail: vec![],
                    trail_nav: false,
                });
                data.active_tab.insert(space, id);
            }
            let history: Vec<Visit> = fs::read_to_string(root.join("history.json"))
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let active_space = data.active_space.clone();
            let extra_windows: Vec<String> = std::mem::take(&mut data.windows);
            let started = now();
            for t in data.tabs.iter_mut().filter(|t| t.last_active == 0) {
                t.last_active = started;
            }
            if data.is_locked(&data.active_space.clone()) {
                if let Some(open) = data.spaces.iter().find(|s| !s.locked).map(|s| s.id.clone()) {
                    data.active_space = open;
                }
            }
            blocker::configure(data.settings.adblock, &data.settings.adblock_allow);
            blocker::load(
                app.path().resource_dir().ok().map(|d| d.join("adblock")),
                root.join("adblock"),
            );
            app.manage(AppState {
                data: Mutex::new(data),
                file,
                root,
                history: Mutex::new(history),
                history_dirty: AtomicBool::new(false),
                closed: Mutex::new(vec![]),
                peek: AtomicBool::new(false),
                panel: Mutex::new(HashMap::new()),
                windows: Mutex::new(HashMap::from([(WINDOW.to_string(), active_space.clone())])),
                focused: Mutex::new(WINDOW.to_string()),
                downloads: Mutex::new(vec![]),
            });

            let win = build_window(app.handle(), WINDOW)?;
            let _ = win.center();
            let handle = app.handle().clone();
            spawn(async move {
                apply_layout(&handle);
                commit(&handle);
                for space in extra_windows {
                    if with_data(&handle, |d| {
                        d.spaces.iter().any(|s| s.id == space && !s.locked)
                    }) {
                        let _ = new_window(handle.clone(), Some(space)).await;
                    }
                }
                if let Some(main) = handle.get_window(WINDOW) {
                    let _ = main.set_focus();
                }
            });
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(30));
                save_history(&handle);
                sleep_idle_tabs(&handle);
                archive_idle_tabs(&handle);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Blazon");
}
