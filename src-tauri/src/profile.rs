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
    let name = webview2::add_extension(&wv, folder.to_string()).await?;
    diag::info(&format!("extensão instalada: {name}"));
    Ok(name)
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
