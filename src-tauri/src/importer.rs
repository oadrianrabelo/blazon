//! Applying imports from other browsers (the parsing lives in `import`).

use crate::*;

#[tauri::command]
pub(crate) async fn import_sources() -> Vec<import::Source> {
    import::sources()
}

pub(crate) fn push_tab(
    d: &mut Data,
    space: &str,
    e: import::Entry,
    pinned: bool,
    folder: Option<u64>,
) {
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

pub(crate) fn push_folders(d: &mut Data, space: &str, folders: Vec<import::Folder>) {
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

/// Puts imported data in the current Space or a new one named after the source.
/// Imported tabs start asleep. Returns a summary for the person.
pub(crate) fn apply_import(
    app: &AppHandle,
    imported: import::Imported,
    new_space: Option<String>,
) -> String {
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
pub(crate) async fn import_browser(
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
pub(crate) async fn import_html(
    app: AppHandle,
    new_space: Option<String>,
) -> Result<String, String> {
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
