//! Support: problem reports, crash reports, updates and Windows integration.

use crate::*;

/// Errors from the UI's own pages (sidebar, settings, overlay).
#[tauri::command]
pub(crate) async fn report_js_error(page: String, message: String, stack: Option<String>) {
    let stack = stack.unwrap_or_default();
    let stack: String = stack.lines().take(6).collect::<Vec<_>>().join(" | ");
    diag::error(&format!("JS [{page}] {message} {stack}"));
}

/// Last crash report, shown once after a crash.
#[tauri::command]
pub(crate) async fn take_crash_report() -> Option<String> {
    diag::take_crash()
}

#[tauri::command]
pub(crate) async fn open_logs_folder() {
    #[cfg(windows)]
    if let Some(dir) = diag::logs_dir() {
        let _ = std::process::Command::new("explorer").arg(dir).spawn();
    }
}

/// The text of a problem report: what happened, system, setup and recent log.
#[tauri::command]
pub(crate) async fn bug_report(app: AppHandle, description: String) -> String {
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
pub(crate) fn app_version() -> String {
    env!("CARGO_PKG_VERSION").into()
}

#[tauri::command]
pub(crate) async fn check_update(app: AppHandle) -> Result<Option<updater::Update>, String> {
    with_data(&app, |d| d.last_update_check = now());
    let found = tauri::async_runtime::spawn_blocking(updater::check)
        .await
        .map_err(|e| e.to_string())?;
    match &found {
        Ok(Some(u)) => diag::info(&format!("atualização disponível: {}", u.version)),
        Err(e) => diag::warn(&format!("verificação de atualização falhou: {e}")),
        _ => {}
    }
    found
}

/// Downloads, verifies and starts the installer, then closes Blazon.
#[tauri::command]
pub(crate) async fn install_update(app: AppHandle) -> Result<(), String> {
    // Looks the release up again here instead of trusting what the page sends.
    let update = tauri::async_runtime::spawn_blocking(updater::check)
        .await
        .map_err(|e| e.to_string())??
        .ok_or("Você já está na versão mais recente.")?;
    diag::info(&format!("instalando a versão {}", update.version));
    tauri::async_runtime::spawn_blocking(move || updater::install(&update))
        .await
        .map_err(|e| e.to_string())??;
    EXITING.store(true, Ordering::Relaxed);
    save_history(&app);
    commit(&app);
    app.exit(0);
    Ok(())
}

/// Daily background check; tells every window when a new version exists.
pub(crate) fn schedule_update_check(app: &AppHandle) {
    let app = app.clone();
    spawn(async move {
        tokio_sleep(Duration::from_secs(20)).await;
        let due = with_data(&app, |d| {
            d.settings.auto_update && now().saturating_sub(d.last_update_check) > 86_400
        });
        if !due {
            return;
        }
        if let Ok(Some(update)) = check_update(app.clone()).await {
            let _ = app.emit("update-available", update);
        }
    });
}

pub(crate) async fn tokio_sleep(d: Duration) {
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(d)).await;
}

/// Opens Windows' default apps page so Crest can be picked as the browser.
#[tauri::command]
pub(crate) async fn open_default_apps() {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer")
        .arg("ms-settings:defaultapps")
        .spawn();
}
