//! Downloads shown in Blazon's own list.

use crate::*;

#[derive(Clone, Serialize)]
pub(crate) struct Download {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) host: String,
    pub(crate) received: i64,
    pub(crate) total: i64,
    /// progress | done | failed
    pub(crate) state: &'static str,
    pub(crate) time: u64,
    #[serde(skip)]
    pub(crate) last_emit: u128,
}

pub(crate) fn emit_downloads(app: &AppHandle) {
    let list = state(app).downloads.lock().unwrap().clone();
    let _ = app.emit("downloads", list);
}

pub(crate) fn millis() -> u128 {
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

#[tauri::command]
pub(crate) async fn downloads_list(app: AppHandle) -> Vec<Download> {
    state(&app).downloads.lock().unwrap().clone()
}

/// open | folder | cancel | remove
#[tauri::command]
pub(crate) async fn download_action(app: AppHandle, id: u64, action: String) {
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
pub(crate) async fn clear_downloads(app: AppHandle) {
    state(&app)
        .downloads
        .lock()
        .unwrap()
        .retain(|d| d.state == "progress");
    emit_downloads(&app);
}
