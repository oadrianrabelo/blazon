//! Spaces: creating, switching, ordering, locking with Windows Hello and link routing.

use crate::*;

/// The Space a link from another app opens in: a matching route, the
/// configured Space, or the current one.
pub(crate) fn route_space(d: &Data, url: &str) -> String {
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

/// Asks for Windows Hello when `space` is locked; true when it may be shown.
pub(crate) async fn ensure_unlocked(app: &AppHandle, space: &str) -> bool {
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

pub(crate) async fn verify_hello(app: &AppHandle, message: &str) -> bool {
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

pub(crate) const HOUSES: [&str; 9] = [
    "winter", "lion", "storm", "dragon", "meadow", "iron", "river", "sun", "vigil",
];
pub(crate) const HOUSE_ACCENTS: [&str; 9] = [
    "#86adc4", "#c9a254", "#b08f4a", "#be4438", "#bca766", "#9c9781", "#a85c41", "#d09e65",
    "#7b838c",
];

#[tauri::command]
pub(crate) async fn set_space_search(app: AppHandle, id: String, search: String) {
    with_data(&app, |d| {
        if let Some(s) = d.spaces.iter_mut().find(|s| s.id == id) {
            s.search = search;
        }
    });
    commit(&app);
}

/// Moves a Space to position `to` in the switcher.
#[tauri::command]
pub(crate) async fn move_space(app: AppHandle, id: String, to: usize) {
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
pub(crate) async fn add_space(
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
pub(crate) async fn switch_space(app: AppHandle, id: String) {
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
pub(crate) async fn update_space(
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
pub(crate) async fn delete_space(app: AppHandle, id: String) {
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

/// Native folder picker for a Space's downloads.
#[tauri::command]
pub(crate) async fn pick_folder(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.to_string())
}

#[tauri::command]
pub(crate) async fn set_space_downloads(app: AppHandle, id: String, path: String) {
    with_data(&app, |d| {
        if let Some(s) = d.spaces.iter_mut().find(|s| s.id == id) {
            s.downloads = path;
        }
    });
    commit(&app);
}

/// Locks or unlocks a Space; either way needs Windows Hello first.
#[tauri::command]
pub(crate) async fn set_space_lock(app: AppHandle, id: String, locked: bool) -> Result<(), String> {
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
pub(crate) async fn lock_now(app: AppHandle) {
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
