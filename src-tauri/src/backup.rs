//! Backups: everything Blazon knows (Spaces, Studio looks, tabs, folders,
//! archive, history and settings) in one JSON file, exported on demand or
//! kept up to date in a folder of the person's choice (e.g. OneDrive), and
//! restored by replacing everything or merging it in. Website data (cookies,
//! logins) stays in each Space's WebView2 profile and is not included.

use crate::*;

const FORMAT: &str = "blazon-backup";
const AUTO_FILE: &str = "blazon-backup.json";
/// Dated copies kept next to the automatic backup.
const KEEP_COPIES: usize = 5;
/// Minimum time between automatic backups.
const AUTO_EVERY: u64 = 10 * 60;

#[derive(Serialize, Deserialize)]
pub(crate) struct Backup {
    format: String,
    version: u32,
    app_version: String,
    created: u64,
    data: Data,
    #[serde(default)]
    history: Vec<Visit>,
}

/// Set while a restore replaces the files, so nothing writes over them.
pub(crate) static RESTORING: AtomicBool = AtomicBool::new(false);

static LAST_AUTO: Mutex<(u64, u64)> = Mutex::new((0, 0));

fn snapshot(app: &AppHandle) -> Backup {
    let data = with_data(app, |d| {
        let mut d = d.clone();
        // Private Spaces keep no tabs, and unlocks are per session.
        let private: Vec<String> = d
            .spaces
            .iter()
            .filter(|s| s.private)
            .map(|s| s.id.clone())
            .collect();
        d.tabs.retain(|t| !private.contains(&t.space));
        d.unlocked.clear();
        d
    });
    let history = state(app).history.lock().unwrap().clone();
    Backup {
        format: FORMAT.into(),
        version: 1,
        app_version: env!("CARGO_PKG_VERSION").into(),
        created: now(),
        data,
        history,
    }
}

fn today() -> String {
    // yyyy-mm-dd from diag's timestamp ("yyyy-mm-dd hh:mm:ssZ").
    diag::timestamp().chars().take(10).collect()
}

/// Saves a backup file wherever the person chooses.
#[tauri::command]
pub(crate) async fn export_backup(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let Some(path) = app
        .dialog()
        .file()
        .set_file_name(format!("blazon-backup-{}.json", today()))
        .add_filter("Backup do Blazon", &["json"])
        .blocking_save_file()
    else {
        return Err("Nenhum arquivo escolhido.".into());
    };
    let path = path.into_path().map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&snapshot(&app)).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    diag::info("backup exportado");
    Ok(path.display().to_string())
}

fn read_backup(path: &std::path::Path) -> Result<Backup, String> {
    let text = fs::read_to_string(path).map_err(|_| "Não deu para ler o arquivo.".to_string())?;
    let b: Backup = serde_json::from_str(&text)
        .map_err(|_| "Este arquivo não é um backup do Blazon.".to_string())?;
    if b.format != FORMAT {
        return Err("Este arquivo não é um backup do Blazon.".into());
    }
    Ok(b)
}

/// Restores a backup. "replace" swaps everything and restarts Blazon;
/// "merge" adds the backup's Spaces (with their tabs and history) to the current ones.
#[tauri::command]
pub(crate) async fn restore_backup(app: AppHandle, mode: String) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let Some(file) = app
        .dialog()
        .file()
        .add_filter("Backup do Blazon", &["json"])
        .blocking_pick_file()
    else {
        return Err("Nenhum arquivo escolhido.".into());
    };
    let backup = read_backup(&file.into_path().map_err(|e| e.to_string())?)?;
    if mode == "replace" {
        let s = state(&app);
        let mut data = backup.data;
        data.repair();
        RESTORING.store(true, Ordering::Relaxed);
        fs::write(
            &s.file,
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            s.root.join("history.json"),
            serde_json::to_string(&backup.history).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        s.history_dirty.store(false, Ordering::Relaxed);
        diag::info("backup restaurado (substituir); reiniciando");
        app.restart();
    }
    let (spaces, tabs, ids) = merge(&app, backup.data);
    let mut by_space: HashMap<String, Vec<import::Visit>> = HashMap::new();
    for v in backup.history {
        if let Some(space) = ids.get(&v.space) {
            by_space
                .entry(space.clone())
                .or_default()
                .push(import::Visit {
                    url: v.url,
                    title: v.title,
                    time: v.time,
                    count: v.count,
                });
        }
    }
    let visits: usize = by_space
        .into_iter()
        .map(|(space, v)| merge_history(&app, &space, v))
        .sum();
    commit(&app);
    diag::info("backup mesclado");
    Ok(format!(
        "Mesclado: {spaces} Spaces, {tabs} abas e {visits} páginas do histórico."
    ))
}

/// Adds a backup's Spaces under fresh ids; returns counts and the id map.
fn merge(app: &AppHandle, other: Data) -> (usize, usize, HashMap<String, String>) {
    with_data(app, |d| {
        let mut ids = HashMap::new();
        for mut s in other.spaces.into_iter().filter(|s| !s.private) {
            let new = format!("space-{}", d.new_id());
            ids.insert(s.id.clone(), new.clone());
            s.id = new;
            s.locked = false;
            d.spaces.push(s);
        }
        let mut folders = HashMap::new();
        for mut f in other.folders.into_iter() {
            let Some(space) = ids.get(&f.space) else {
                continue;
            };
            let new = d.new_id();
            folders.insert(f.id, new);
            f.id = new;
            f.space = space.clone();
            d.folders.push(f);
        }
        for f in d.folders.iter_mut() {
            if folders.values().any(|v| *v == f.id) {
                f.parent = f.parent.and_then(|p| folders.get(&p).copied());
            }
        }
        let mut tabs = 0;
        for mut t in other.tabs.into_iter() {
            let Some(space) = ids.get(&t.space) else {
                continue;
            };
            t.id = d.new_id();
            t.space = space.clone();
            t.folder = t.folder.and_then(|f| folders.get(&f).copied());
            t.awake = false;
            d.tabs.push(t);
            tabs += 1;
        }
        for mut a in other.archive.into_iter() {
            if let Some(space) = ids.get(&a.space) {
                a.space = space.clone();
                d.archive.push(a);
            }
        }
        (ids.len(), tabs, ids)
    })
}

/// Keeps `<folder>/blazon-backup.json` current (at most every 10 minutes,
/// only when something changed) plus a few dated copies.
pub(crate) fn auto_backup(app: &AppHandle) {
    let folder = with_data(app, |d| d.settings.backup_folder.clone());
    if folder.is_empty() || RESTORING.load(Ordering::Relaxed) {
        return;
    }
    let mut snap = snapshot(app);
    snap.created = 0;
    let Ok(json) = serde_json::to_string_pretty(&snap) else {
        return;
    };
    let hash = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        json.hash(&mut h);
        h.finish()
    };
    let mut last = LAST_AUTO.lock().unwrap();
    if last.1 == hash || now().saturating_sub(last.0) < AUTO_EVERY {
        return;
    }
    snap.created = now();
    let Ok(json) = serde_json::to_string_pretty(&snap) else {
        return;
    };
    let dir = PathBuf::from(&folder);
    if fs::create_dir_all(&dir).is_err() {
        diag::warn("pasta de backup automático indisponível");
        return;
    }
    let tmp = dir.join(format!("{AUTO_FILE}.tmp"));
    if fs::write(&tmp, &json).is_err() || fs::rename(&tmp, dir.join(AUTO_FILE)).is_err() {
        diag::warn("backup automático falhou");
        return;
    }
    let _ = fs::write(dir.join(format!("blazon-backup-{}.json", today())), &json);
    // Keep only the newest dated copies.
    let mut dated: Vec<PathBuf> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("blazon-backup-") && n.ends_with(".json"))
        })
        .collect();
    dated.sort();
    while dated.len() > KEEP_COPIES {
        let _ = fs::remove_file(dated.remove(0));
    }
    *last = (now(), hash);
    with_data(app, |d| d.last_backup = now());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backups_round_trip() {
        let b = Backup {
            format: FORMAT.into(),
            version: 1,
            app_version: "0.1.0".into(),
            created: 1,
            data: Data::default(),
            history: vec![],
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: Backup = serde_json::from_str(&json).unwrap();
        assert_eq!(back.format, FORMAT);
    }
}
