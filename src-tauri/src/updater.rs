//! Updates from GitHub Releases. The latest release's NSIS installer is
//! downloaded with Windows' curl.exe and only run (in passive mode, which
//! reopens Blazon) when its Ed25519 signature checks out against the key built
//! into this app: the CI signs each release with the private half, kept as the
//! BLAZON_UPDATE_KEY secret (scripts/sign-update.mjs). A release someone
//! uploaded without that key, even through the repository, is refused.

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
    /// The installer's signature file (`<installer>.sig`).
    pub signature: Option<String>,
}

/// Public half of the release signing key (Ed25519).
pub const UPDATE_KEY: &str = "ee7e94697c2d981c74ddbe34d4ce0dd66382ca6c251301329e417e4aec4dd031";

/// What the CI signs: the version and the installer's SHA-256, so a signature
/// can't be moved to another file or replayed for another version.
fn signed_message(version: &str, installer: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hash: String = Sha256::digest(installer)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("blazon-update:{}:{hash}", version.trim_start_matches('v'))
}

/// Checks a base64 Ed25519 signature of an installer against `key` (hex).
pub fn verify(
    version: &str,
    installer: &[u8],
    signature_b64: &str,
    key: &str,
) -> Result<(), String> {
    use base64::Engine;
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let refuse = || "A assinatura da atualização não confere. Nada foi instalado.".to_string();
    let key: Vec<u8> = (0..key.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(key.get(i..i + 2)?, 16).ok())
        .collect();
    let key: [u8; 32] = key.try_into().map_err(|_| refuse())?;
    let key = VerifyingKey::from_bytes(&key).map_err(|_| refuse())?;
    let sig = base64::engine::general_purpose::STANDARD
        .decode(signature_b64.trim())
        .map_err(|_| refuse())?;
    let sig: [u8; 64] = sig.try_into().map_err(|_| refuse())?;
    key.verify(
        signed_message(version, installer).as_bytes(),
        &Signature::from_bytes(&sig),
    )
    .map_err(|_| refuse())
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
    let installer = asset["name"].as_str().unwrap_or_default();
    let signature = json["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| a["name"].as_str() == Some(&format!("{installer}.sig")))
        .and_then(|a| a["browser_download_url"].as_str())
        .map(String::from);
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
        signature,
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
    let Some(sig_url) = update
        .signature
        .as_deref()
        .filter(|u| u.starts_with("https://github.com/"))
    else {
        return Err("A versão nova não veio assinada. Por segurança ela não foi instalada; baixe pela página de versões se confiar nela.".into());
    };
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
    let signature =
        curl(&[sig_url]).map_err(|e| format!("O download da assinatura falhou ({e})."))?;
    let installer = std::fs::read(&file).map_err(|e| e.to_string())?;
    if let Err(e) = verify(
        &update.version,
        &installer,
        &String::from_utf8_lossy(&signature),
        UPDATE_KEY,
    ) {
        let _ = std::fs::remove_file(&file);
        return Err(e);
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
    use super::*;

    #[test]
    fn only_a_signature_by_the_release_key_passes() {
        use base64::Engine;
        use ed25519_dalek::{Signer, SigningKey};
        let signer = SigningKey::from_bytes(&[7u8; 32]);
        let public: String = signer
            .verifying_key()
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let file = b"installer bytes";
        let sig = base64::engine::general_purpose::STANDARD.encode(
            signer
                .sign(signed_message("0.1.7", file).as_bytes())
                .to_bytes(),
        );
        assert!(verify("v0.1.7", file, &sig, &public).is_ok());
        assert!(
            verify("0.1.8", file, &sig, &public).is_err(),
            "other version"
        );
        assert!(
            verify("0.1.7", b"tampered", &sig, &public).is_err(),
            "other file"
        );
        assert!(
            verify("0.1.7", file, &sig, UPDATE_KEY).is_err(),
            "other key"
        );
        assert!(verify("0.1.7", file, "garbage", &public).is_err());
    }

    #[test]
    fn release_key_is_well_formed() {
        assert_eq!(UPDATE_KEY.len(), 64);
        assert!(UPDATE_KEY.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("v0.2.0", "0.1.9"));
        assert!(is_newer("v0.10.0", "0.9.0"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(!is_newer("v0.1.0", "0.1.0"));
        assert!(!is_newer("v0.1.0", "0.2.0"));
    }
}
