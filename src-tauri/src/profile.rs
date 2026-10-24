//! Per-Space WebView2 profile features: site permissions and extensions.

use crate::*;

/// A live webview of a Space, to reach its WebView2 profile.
pub(crate) fn space_webview(app: &AppHandle, space: &str) -> Result<tauri::Webview, String> {
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
pub(crate) async fn list_permissions(
    app: AppHandle,
    space: String,
) -> Result<Vec<webview2::Permission>, String> {
    Ok(webview2::list_permissions(&space_webview(&app, &space)?).await)
}

#[cfg(windows)]
#[tauri::command]
pub(crate) async fn set_permission(
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
pub(crate) async fn list_extensions(
    app: AppHandle,
    space: String,
) -> Result<Vec<webview2::Extension>, String> {
    Ok(webview2::list_extensions(&space_webview(&app, &space)?).await)
}

/// Picks an unpacked extension folder and installs it in a Space.
#[cfg(windows)]
#[tauri::command]
pub(crate) async fn add_extension(app: AppHandle, space: String) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let wv = space_webview(&app, &space)?;
    let Some(folder) = app.dialog().file().blocking_pick_folder() else {
        return Err("Nenhuma pasta escolhida.".into());
    };
    let ext = webview2::add_extension(&wv, folder.to_string()).await?;
    with_data(&app, |d| {
        d.extension_dirs.insert(ext.id.clone(), folder.to_string())
    });
    commit(&app);
    diag::info(&format!("extensão instalada: {}", ext.name));
    Ok(ext.name)
}

#[cfg(windows)]
#[tauri::command]
pub(crate) async fn change_extension(
    app: AppHandle,
    space: String,
    id: String,
    action: String,
) -> Result<bool, String> {
    Ok(webview2::change_extension(&space_webview(&app, &space)?, id, action).await)
}

#[cfg(not(windows))]
#[tauri::command]
pub(crate) async fn list_permissions() -> Result<Vec<()>, String> {
    Err("Só no Windows.".into())
}
#[cfg(not(windows))]
#[tauri::command]
pub(crate) async fn set_permission() -> Result<bool, String> {
    Err("Só no Windows.".into())
}
#[cfg(not(windows))]
#[tauri::command]
pub(crate) async fn list_extensions() -> Result<Vec<()>, String> {
    Err("Só no Windows.".into())
}
#[cfg(not(windows))]
#[tauri::command]
pub(crate) async fn add_extension() -> Result<String, String> {
    Err("Só no Windows.".into())
}
#[cfg(not(windows))]
#[tauri::command]
pub(crate) async fn change_extension() -> Result<bool, String> {
    Err("Só no Windows.".into())
}

#[derive(Serialize)]
pub(crate) struct SpaceMemory {
    space: String,
    name: String,
    mb: u64,
    awake: usize,
}

/// Memory used by each Space's WebView2 processes (and by Blazon's own UI).
#[tauri::command]
pub(crate) async fn memory_usage(app: AppHandle) -> Vec<SpaceMemory> {
    #[cfg(windows)]
    {
        let spaces = with_data(&app, |d| {
            d.spaces
                .iter()
                .map(|s| (s.id.clone(), s.name.clone()))
                .collect::<Vec<_>>()
        });
        let mut out = vec![];
        for (id, name) in spaces {
            let awake = with_data(&app, |d| {
                d.tabs.iter().filter(|t| t.space == id && t.awake).count()
            });
            let Ok(wv) = space_webview(&app, &id) else {
                continue;
            };
            let mb = webview2::memory_mb(&wv).await;
            out.push(SpaceMemory {
                space: id,
                name,
                mb,
                awake,
            });
        }
        if let Some(ui) = app.get_webview(UI) {
            out.push(SpaceMemory {
                space: String::new(),
                name: "Interface do Blazon".into(),
                mb: webview2::memory_mb(&ui).await,
                awake: 0,
            });
        }
        out
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        vec![]
    }
}
