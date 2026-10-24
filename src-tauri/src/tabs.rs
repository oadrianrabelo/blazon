//! Tabs: their webviews, navigation, pinning, folders, Split View, sleep and archive.

use crate::*;

/// Closes the webviews of background tabs idle longer than the sleep setting.
/// Closes the webviews of idle background tabs. Tabs playing sound stay;
/// pinned tabs stay unless their Space isn't open in any window, so a Space
/// left alone releases its whole WebView2 process.
pub(crate) fn sleep_idle_tabs(app: &AppHandle) {
    let visible = shown_tabs(app);
    let shown_spaces: Vec<String> = state(app)
        .windows
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect();
    let asleep = with_data(app, |d| {
        let limit = d.settings.sleep_minutes * 60;
        let cutoff = now().saturating_sub(limit);
        let mut ids = vec![];
        for t in d.tabs.iter_mut() {
            if limit > 0
                && t.awake
                && !t.audio
                && (!t.pinned || !shown_spaces.contains(&t.space))
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

pub(crate) fn is_newtab(url: &Url) -> bool {
    matches!(url.host_str(), Some("tauri.localhost") | Some("localhost"))
        && url.path().trim_start_matches('/') == NEWTAB_PAGE
}

pub(crate) fn to_url(input: &str, search: &str) -> String {
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

pub(crate) fn update_tab(app: &AppHandle, id: u64, f: impl FnOnce(&mut Tab)) {
    let found = with_data(app, |d| {
        d.tabs.iter_mut().find(|t| t.id == id).map(f).is_some()
    });
    if found {
        commit(app);
    }
}

/// Creates a tab's webview; `layout` positions it right after.
pub(crate) fn create_tab_webview(app: &AppHandle, win: &Window, tab: &Tab) -> tauri::Result<()> {
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

pub(crate) fn focus_address(app: &AppHandle) {
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

pub(crate) fn focus_active(app: &AppHandle) {
    if let Some(id) = with_data(app, |d| d.active_tab_id()) {
        if let Some(wv) = app.get_webview(&label(id)) {
            let _ = wv.set_focus();
        }
    }
}

pub(crate) async fn open_tab(app: &AppHandle, url: String) {
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

#[tauri::command]
pub(crate) async fn new_tab(app: AppHandle, url: Option<String>) {
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

#[tauri::command]
pub(crate) async fn activate_tab(app: AppHandle, id: u64) {
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
pub(crate) async fn close_tab(app: AppHandle, id: u64) {
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
pub(crate) async fn reopen_tab(app: AppHandle) {
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

#[tauri::command]
pub(crate) async fn navigate(app: AppHandle, input: String) {
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

pub(crate) async fn can_go_back(wv: &tauri::Webview) -> bool {
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
pub(crate) async fn nav(app: AppHandle, action: String) {
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
pub(crate) async fn toggle_pin(app: AppHandle, id: u64) {
    update_tab(&app, id, |t| t.pinned = !t.pinned);
}

/// Moves a tab before `before` (or to the end), into the given pinned row or folder.
#[tauri::command]
pub(crate) async fn move_tab(
    app: AppHandle,
    id: u64,
    before: Option<u64>,
    pinned: bool,
    folder: Option<u64>,
) {
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
pub(crate) async fn split_tab(app: AppHandle, id: u64, with: Option<u64>) {
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
pub(crate) async fn unsplit_tab(app: AppHandle, id: u64) {
    with_data(&app, |d| d.unsplit(id));
    apply_layout(&app);
    commit(&app);
}

#[tauri::command]
pub(crate) async fn add_folder(app: AppHandle, name: String, tab: Option<u64>) {
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
pub(crate) async fn update_folder(
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
pub(crate) async fn delete_folder(app: AppHandle, id: u64) {
    with_data(&app, |d| {
        d.folders.retain(|f| f.id != id);
        for t in d.tabs.iter_mut().filter(|t| t.folder == Some(id)) {
            t.folder = None;
        }
    });
    commit(&app);
}

#[tauri::command]
pub(crate) async fn duplicate_tab(app: AppHandle, id: u64) {
    let url = with_data(&app, |d| {
        d.tabs.iter().find(|t| t.id == id).map(|t| t.url.clone())
    });
    if let Some(url) = url {
        open_tab(&app, url).await;
        focus_active(&app);
    }
}

#[tauri::command]
pub(crate) async fn toggle_mute(app: AppHandle, id: u64) {
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
pub(crate) async fn toggle_site_blocking(app: AppHandle) -> Option<bool> {
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

#[tauri::command]
pub(crate) async fn restore_archived(app: AppHandle, url: String, space: String) {
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
pub(crate) async fn clear_archive(app: AppHandle) {
    with_data(&app, |d| {
        let space = d.active_space.clone();
        d.archive.retain(|a| a.space != space);
    });
    commit(&app);
}

pub(crate) const READER_JS: &str = include_str!("reader.js");
pub(crate) const READABILITY_JS: &str = include_str!("vendor/Readability.js");

/// Toggles reader mode on the active tab.
#[tauri::command]
pub(crate) async fn reader_mode(app: AppHandle) {
    let Some(id) = with_data(&app, |d| d.active_tab_id()) else {
        return;
    };
    if let Some(wv) = app.get_webview(&label(id)) {
        let _ = wv.eval(READER_JS.replace("/*READABILITY*/", READABILITY_JS));
    }
}

/// Archives tabs idle longer than the archive setting.
pub(crate) fn archive_idle_tabs(app: &AppHandle) {
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

/// Puts every hidden tab of a Space to sleep now.
#[tauri::command]
pub(crate) async fn sleep_space(app: AppHandle, space: String) -> usize {
    let visible = shown_tabs(&app);
    let ids = with_data(&app, |d| {
        let mut ids = vec![];
        for t in d
            .tabs
            .iter_mut()
            .filter(|t| t.space == space && t.awake && !visible.contains(&t.id))
        {
            t.awake = false;
            t.loading = false;
            ids.push(t.id);
        }
        ids
    });
    for id in &ids {
        if let Some(wv) = app.get_webview(&label(*id)) {
            let _ = wv.close();
        }
    }
    commit(&app);
    ids.len()
}

/// Opens the active page translated by Google Translate into `target`.
#[tauri::command]
pub(crate) async fn translate_page(app: AppHandle, target: String) {
    let Some(url) = with_data(&app, |d| {
        let id = d.active_tab_id()?;
        d.tabs.iter().find(|t| t.id == id).map(|t| t.url.clone())
    }) else {
        return;
    };
    if !url.starts_with("http") {
        return;
    }
    let target = if target == "en" { "en" } else { "pt" };
    let q: String = url::form_urlencoded::byte_serialize(url.as_bytes()).collect();
    navigate(
        app,
        format!("https://translate.google.com/translate?sl=auto&tl={target}&u={q}"),
    )
    .await;
}
