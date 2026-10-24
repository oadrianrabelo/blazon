//! Updates from GitHub Releases. The latest release's NSIS installer is
//! downloaded with Windows' curl.exe, checked against the SHA-256 digest
//! GitHub publishes for it, and run in passive mode, which reopens Blazon.

use serde::Serialize;
use serde_json::Value;

/// GitHub follows renames, so this keeps working if the repository moves.
pub const REPO: &str = "oadrianrabelo/crest-windows-vibe-coded";

#[derive(Clone, Serialize)]
pub struct Update {
    pub version: String,
    pub notes: String,
    pub url: String,
    pub page: String,
    /// "sha256:<hex>" when GitHub provides it.
    pub digest: Option<String>,
}

fn version_parts(v: &str) -> Vec<u64> {
    v.trim_start_matches('v')
        .split(['.', '-'])
        .take(3)
        .map(|p| p.parse().unwrap_or(0))
        .collect()
}

/// Whether `latest` is newer than `current` (semantic version, numeric parts).
pub fn is_newer(latest: &str, current: &str) -> bool {
    version_parts(latest) > version_parts(current)
}

#[cfg(windows)]
fn curl(args: &[&str]) -> Result<Vec<u8>, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let out = std::process::Command::new("curl.exe")
        .args(["-sSfL", "--max-time", "600", "-H", "User-Agent: Blazon"])
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// The latest release, when it is newer than this build.
#[cfg(windows)]
pub fn check() -> Result<Option<Update>, String> {
    let body = curl(&[
        "-H",
        "Accept: application/vnd.github+json",
        &format!("https://api.github.com/repos/{REPO}/releases/latest"),
    ])
    .map_err(|e| {
        if e.contains("404") {
            "Nenhuma versão publicada ainda.".to_string()
        } else {
            format!("Sem conexão com o GitHub ({e}).")
        }
    })?;
    let json: Value = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    let tag = json["tag_name"].as_str().unwrap_or_default();
    if !is_newer(tag, env!("CARGO_PKG_VERSION")) {
        return Ok(None);
    }
    let asset = json["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| {
            a["name"]
                .as_str()
                .is_some_and(|n| n.ends_with("-setup.exe"))
        })
        .ok_or("A versão nova não tem instalador para Windows.")?;
    Ok(Some(Update {
        version: tag.trim_start_matches('v').to_string(),
        notes: json["body"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .take(2000)
            .collect(),
        url: asset["browser_download_url"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        page: json["html_url"].as_str().unwrap_or_default().to_string(),
        digest: asset["digest"].as_str().map(String::from),
    }))
}

/// Downloads and verifies the installer, then starts it. The caller exits
/// the app so the installer can replace it.
#[cfg(windows)]
pub fn install(update: &Update) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    if !update.url.starts_with("https://github.com/") {
        return Err("Endereço de download inesperado.".into());
    }
    let dir = std::env::temp_dir().join("blazon-update");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = dir.join(format!("Blazon_{}_setup.exe", update.version));
    curl(&["-o", &file.to_string_lossy(), &update.url])
        .map_err(|e| format!("O download falhou ({e})."))?;
    if let Some(expected) = update
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
    {
        let out = std::process::Command::new("certutil")
            .args(["-hashfile", &file.to_string_lossy(), "SHA256"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&out.stdout)
            .to_lowercase()
            .replace(' ', "");
        if !text.contains(&expected.to_lowercase()) {
            let _ = std::fs::remove_file(&file);
            return Err(
                "O instalador baixado não confere com o publicado. Nada foi instalado.".into(),
            );
        }
    }
    std::process::Command::new(&file)
        .args(["/P", "/R", "/UPDATE"])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
pub fn check() -> Result<Option<Update>, String> {
    Err("Atualizações só no Windows.".into())
}

#[cfg(not(windows))]
pub fn install(_: &Update) -> Result<(), String> {
    Err("Atualizações só no Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("v0.2.0", "0.1.9"));
        assert!(is_newer("v0.10.0", "0.9.0"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(!is_newer("v0.1.0", "0.1.0"));
        assert!(!is_newer("v0.1.0", "0.2.0"));
    }
}
