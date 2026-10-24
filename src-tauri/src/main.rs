#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backup;
mod blocker;
mod diag;
mod downloads;
mod fonts;
#[cfg(windows)]
mod hello;
mod history;
mod import;
mod importer;
mod keys;
mod model;
mod overlay;
mod passwords;
mod profile;
mod quick;
mod sidepanel;
mod spaces;
#[cfg(windows)]
mod sqlite;
mod support;
mod tabs;
mod transition;
mod updater;
#[cfg(windows)]
mod webview2;
mod windowing;

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

use downloads::*;
use history::*;
use model::*;
use serde::{Deserialize, Serialize};
use spaces::*;
use support::*;
use tabs::*;
use tauri::{
    async_runtime::spawn,
    webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder},
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, Window, WindowEvent,
};
use url::Url;
use windowing::*;

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
/// Key handler injected into every site page. The combo → command map comes
/// from Settings › Shortcuts (keys.rs) and is replaced live by `push_to_pages`.
fn shortcuts_js(settings: &Settings) -> String {
    let peek_modifier = settings.peek_modifier.as_str();
    r#"(() => {
  if (window.top !== window) return;
  window.__blazonKeys = KEYMAP;
  const send = (cmd) => window.open('https://blazon-cmd.invalid/' + cmd);
  const combo = (e) => {
    let k = e.key;
    if (/^Key[A-Z]$/.test(e.code)) k = e.code.slice(3);
    else if (/^Digit[0-9]$/.test(e.code)) k = e.code.slice(5);
    else if (k.length === 1) k = k.toUpperCase();
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(k)) return '';
    return (e.ctrlKey ? 'Ctrl+' : '') + (e.altKey ? 'Alt+' : '') + (e.shiftKey ? 'Shift+' : '') + k;
  };
  addEventListener('keydown', (e) => {
    if (e.key === 'F6' && !e.ctrlKey && !e.altKey) { e.preventDefault(); send('address'); return; }
    const digit = /^Digit[1-9]$/.test(e.code) ? e.code.slice(5) : null;
    let cmd = null;
    if (digit && e.ctrlKey && !e.altKey) cmd = (e.shiftKey ? 'space' : 'tab') + digit;
    else cmd = (window.__blazonKeys || {})[combo(e)];
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
        .replace("KEYMAP", &keys::map_json(settings))
        .replace(
            "MODIFIER",
            if matches!(peek_modifier, "shift" | "alt" | "off") {
                peek_modifier
            } else {
                "shift"
            },
        )
}

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

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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
    if backup::RESTORING.load(Ordering::Relaxed) {
        return;
    }
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

/// Builder for Blazon's own interface webviews (sidebar, panel, overlays).
/// When BLAZON_DEVTOOLS_PORT is set, as in the CI smoke test, they expose
/// WebView2's DevTools protocol on that port; site pages never do.
pub(crate) fn interface_webview(
    label: impl Into<String>,
    url: WebviewUrl,
) -> WebviewBuilder<tauri::Wry> {
    let mut args = browser_args();
    if let Some(port) = std::env::var("BLAZON_DEVTOOLS_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
    {
        args.push_str(&format!(" --remote-debugging-port={port}"));
    }
    WebviewBuilder::new(label, url).additional_browser_args(&args)
}

/// Appearance › Text: Mac asks WebView2 for grayscale text smoothing, as on
/// the Mac, instead of ClearType. Webviews that share a profile folder must
/// be created with the same arguments, so this is read once at startup and a
/// change applies after a restart.
static GRAYSCALE_TEXT: AtomicBool = AtomicBool::new(true);

/// Browser arguments for every webview (wry's defaults plus the text smoothing).
pub(crate) fn browser_args() -> String {
    let mut args = String::from("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection");
    if GRAYSCALE_TEXT.load(Ordering::Relaxed) {
        args.push_str(" --disable-lcd-text");
    }
    args
}

pub(crate) fn mac_text(settings: &Settings) -> bool {
    settings
        .appearance
        .get("fontRendering")
        .and_then(|v| v.as_str())
        != Some("windows")
}

fn label(id: u64) -> String {
    format!("tab-{id}")
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
        "focuspage" => focus_active(app),
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
        "reader" => tabs::reader_mode(app.clone()).await,
        "pin" | "duplicate" | "mute" => {
            if let Some(id) = with_data(app, |d| d.active_tab_id()) {
                match cmd {
                    "pin" => tabs::toggle_pin(app.clone(), id).await,
                    "duplicate" => tabs::duplicate_tab(app.clone(), id).await,
                    _ => tabs::toggle_mute(app.clone(), id).await,
                }
            }
        }
        "settings" => windowing::open_panel(app.clone(), "look".into()).await,
        "tablayout" => {
            with_data(app, |d| {
                let horizontal = d
                    .settings
                    .appearance
                    .get("tabLayout")
                    .and_then(|v| v.as_str())
                    == Some("horizontal");
                if !d.settings.appearance.is_object() {
                    d.settings.appearance = serde_json::json!({});
                }
                d.settings.appearance["tabLayout"] =
                    serde_json::Value::from(if horizontal { "vertical" } else { "horizontal" });
            });
            apply_layout(app);
            commit(app);
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
    let mica = windowing::mica_on(&settings);
    let (frame_changed, mica_changed, moved) = with_data(&app, |d| {
        let old = &d.settings;
        let look = |s: &Settings, k: &str| s.appearance.get(k).cloned();
        (
            old.native_frame != settings.native_frame,
            windowing::mica_on(old) != mica,
            old.sidebar_right != settings.sidebar_right
                || old.floating_sidebar != settings.floating_sidebar
                || (old.sidebar_width - settings.sidebar_width).abs() > f64::EPSILON
                || look(old, "tabLayout") != look(&settings, "tabLayout")
                || look(old, "controlsSide") != look(&settings, "controlsSide"),
        )
    });
    for win in app.windows().into_values() {
        if frame_changed {
            let _ = win.set_decorations(settings.native_frame);
        }
        if mica_changed {
            windowing::apply_material(&win, mica);
        }
    }
    blocker::configure(settings.adblock, &settings.adblock_allow);
    with_data(&app, |d| d.settings = settings);
    commit(&app);
    apply_layout(&app);
    if moved {
        windowing::repaint_interface(&app);
    }
}

fn main() {
    tauri::Builder::default()
        // A second launch (e.g. a link clicked in another app) opens its URLs here.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let app = app.clone();
            let urls: Vec<String> = args.iter().skip(1).filter_map(|a| arg_to_url(a)).collect();
            spawn(async move {
                // A single link with no matching route can open in the Quick
                // Window (its own small window, like Crest's) or in Peek.
                let mode = with_data(&app, |d| d.settings.external_space.clone());
                let routed = urls.len() != 1
                    || with_data(&app, |d| {
                        d.settings
                            .routes
                            .iter()
                            .any(|r| !r.pattern.is_empty() && urls[0].contains(&r.pattern))
                    });
                if mode == "quick" && !routed {
                    quick::open(&app, &urls[0]);
                    return;
                }
                if let Some(win) = window(&app) {
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
                if mode == "peek" && !routed {
                    overlay::open_peek(&app, &urls[0]);
                    return;
                }
                for url in urls {
                    with_data(&app, |d| d.active_space = route_space(d, &url));
                    open_tab(&app, url).await;
                }
                focus_active(&app);
            });
        }))
        .invoke_handler(tauri::generate_handler![
            get_state,
            run_command,
            keys::shortcuts,
            fonts::system_fonts,
            keys::set_shortcut,
            keys::reset_shortcuts,
            tabs::new_tab,
            tabs::activate_tab,
            tabs::close_tab,
            tabs::navigate,
            tabs::nav,
            tabs::toggle_pin,
            tabs::move_tab,
            spaces::add_space,
            spaces::switch_space,
            spaces::update_space,
            spaces::delete_space,
            set_settings,
            windowing::sidebar_peek,
            windowing::set_page_hidden,
            windowing::open_panel,
            windowing::close_panel,
            support::open_default_apps,
            backup::export_backup,
            tabs::sleep_space,
            tabs::translate_page,
            profile::memory_usage,
            backup::restore_backup,
            support::app_version,
            support::check_update,
            support::install_update,
            windowing::new_window,
            downloads::downloads_list,
            downloads::download_action,
            downloads::clear_downloads,
            profile::list_permissions,
            profile::set_permission,
            profile::list_extensions,
            profile::add_extension,
            profile::change_extension,
            support::report_js_error,
            support::take_crash_report,
            support::open_logs_folder,
            support::bug_report,
            tabs::toggle_mute,
            tabs::toggle_site_blocking,
            spaces::pick_folder,
            spaces::set_space_downloads,
            spaces::set_space_lock,
            spaces::lock_now,
            tabs::restore_archived,
            tabs::clear_archive,
            tabs::reader_mode,
            tabs::reopen_tab,
            history::suggest,
            history::history_list,
            history::clear_history,
            tabs::add_folder,
            tabs::update_folder,
            tabs::delete_folder,
            tabs::move_folder,
            tabs::duplicate_tab,
            tabs::split_tab,
            spaces::set_space_search,
            spaces::move_space,
            tabs::unsplit_tab,
            importer::import_sources,
            importer::import_browser,
            importer::import_html,
            overlay::open_overlay,
            overlay::close_overlay,
            overlay::peek_to_tab,
            overlay::peek_url,
            overlay::page_snapshot,
            overlay::peek_geometry,
            overlay::peek_show,
            passwords::password_sources,
            passwords::import_passwords,
            passwords::import_passwords_csv,
            passwords::list_logins,
            passwords::reveal_login,
            passwords::delete_login,
            passwords::clear_logins,
            passwords::logins_for_tab,
            passwords::fill_login,
            transition::fx_ready,
            transition::fx_done,
            sidepanel::extension_panels,
            sidepanel::open_side_panel,
            sidepanel::close_side_panel,
            quick::quick_state,
            quick::quick_to_tab,
            quick::quick_close,
            quick::quick_nav,
            overlay::peek_snapshot,
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
                // First run: show the welcome flow.
                Err(_) => {
                    let mut d = Data::default();
                    d.settings.welcomed = false;
                    d
                }
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
            GRAYSCALE_TEXT.store(mac_text(&data.settings), Ordering::Relaxed);
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
            schedule_update_check(app.handle());
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(30));
                save_history(&handle);
                sleep_idle_tabs(&handle);
                archive_idle_tabs(&handle);
                backup::auto_backup(&handle);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Blazon");
}

#[cfg(test)]
mod tests {
    use super::*;

    const DDG: &str = "https://duckduckgo.com/?q=";

    #[test]
    fn address_bar_input() {
        assert_eq!(to_url("example.com", DDG), "https://example.com/");
        assert_eq!(to_url("https://a.com/x?y=1", DDG), "https://a.com/x?y=1");
        assert_eq!(to_url("localhost:3000", DDG), "http://localhost:3000/");
        assert_eq!(
            to_url("rust lang", DDG),
            "https://duckduckgo.com/?q=rust+lang"
        );
        assert_eq!(to_url("c++", DDG), "https://duckduckgo.com/?q=c%2B%2B");
        assert_eq!(to_url("  ", DDG), "");
    }

    fn space(id: &str) -> Space {
        Space {
            id: id.into(),
            name: id.into(),
            color: "#000000".into(),
            private: false,
            house: String::new(),
            downloads: String::new(),
            look: serde_json::Value::Null,
            locked: false,
            search: String::new(),
        }
    }

    fn tab(id: u64, space: &str) -> Tab {
        Tab {
            id,
            space: space.into(),
            url: format!("https://{id}.com/"),
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
        }
    }

    #[test]
    fn link_routes_pick_the_right_space() {
        let mut d = Data {
            spaces: vec![space("home"), space("work")],
            active_space: "home".into(),
            ..Default::default()
        };
        d.settings.routes = vec![Route {
            pattern: "figma.com".into(),
            space: "work".into(),
        }];
        assert_eq!(route_space(&d, "https://www.figma.com/file/1"), "work");
        assert_eq!(route_space(&d, "https://example.com/"), "home");
        d.settings.external_space = "work".into();
        assert_eq!(route_space(&d, "https://example.com/"), "work");
    }

    #[test]
    fn repair_drops_orphans_and_private_tabs() {
        let mut private = space("secret");
        private.private = true;
        let mut d = Data {
            spaces: vec![space("home"), private],
            tabs: vec![tab(1, "home"), tab(2, "gone"), tab(3, "secret")],
            splits: vec![[1, 2]],
            active_space: "gone".into(),
            ..Default::default()
        };
        d.repair();
        assert_eq!(d.tabs.iter().map(|t| t.id).collect::<Vec<_>>(), [1]);
        assert!(d.splits.is_empty());
        assert_eq!(d.active_space, "home");
        assert!(d.next_id >= 1);
    }

    #[test]
    fn repair_creates_a_space_when_empty() {
        let mut d = Data::default();
        d.repair();
        assert_eq!(d.spaces.len(), 1);
        assert_eq!(d.active_space, d.spaces[0].id);
    }

    #[test]
    fn split_view_shows_both_tabs() {
        let mut d = Data {
            spaces: vec![space("home")],
            tabs: vec![tab(1, "home"), tab(2, "home")],
            active_space: "home".into(),
            ..Default::default()
        };
        d.active_tab.insert("home".into(), 2);
        assert_eq!(d.visible_in("home"), [2]);
        d.splits.push([1, 2]);
        assert_eq!(d.visible_in("home"), [1, 2]);
        d.unsplit(1);
        assert_eq!(d.visible_in("home"), [2]);
    }

    #[test]
    fn space_search_overrides_global() {
        let mut work = space("work");
        work.search = "https://kagi.com/search?q=".into();
        let d = Data {
            spaces: vec![space("home"), work],
            ..Default::default()
        };
        assert_eq!(d.search_for("work"), "https://kagi.com/search?q=");
        assert_eq!(d.search_for("home"), d.settings.search);
    }

    #[test]
    fn old_state_files_still_load() {
        // A state.json from the first versions, without the newer fields.
        let old = r##"{"spaces":[{"id":"personal","name":"Pessoal","color":"#3b82f6"}],
            "tabs":[{"id":1,"space":"personal","url":"https://a.com/","title":"A"}],
            "active_space":"personal","active_tab":{"personal":1},"next_id":1,
            "settings":{"sidebar_right":false,"search":"https://duckduckgo.com/?q="}}"##;
        let d: Data = serde_json::from_str(old).expect("old state loads");
        assert_eq!(d.tabs.len(), 1);
        assert_eq!(d.settings.sleep_minutes, 30);
        assert!(d.settings.adblock);
    }
}
