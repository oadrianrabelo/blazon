//! Blazon's own password vault, like Crest's Credentials: logins imported from
//! other browsers or from a CSV export, kept encrypted with Windows DPAPI (only
//! this Windows account can read them), listed in Settings › Passwords and
//! filled into a page only when the person picks a login from the key button.
//!
//! WebView2 keeps saving new passwords in each Space's own store; this vault is
//! for what comes from elsewhere, since WebView2 has no API to add passwords.

use crate::*;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Login {
    id: u64,
    origin: String,
    host: String,
    username: String,
    password: String,
    /// Where it came from: "Google Chrome", "CSV"…
    source: String,
    created: u64,
}

/// A login without its password, for the interface.
#[derive(Serialize)]
pub(crate) struct LoginInfo {
    id: u64,
    origin: String,
    host: String,
    username: String,
    source: String,
}

impl From<&Login> for LoginInfo {
    fn from(l: &Login) -> Self {
        LoginInfo {
            id: l.id,
            origin: l.origin.clone(),
            host: l.host.clone(),
            username: l.username.clone(),
            source: l.source.clone(),
        }
    }
}

static VAULT: Mutex<Option<Vec<Login>>> = Mutex::new(None);

fn vault_file(app: &AppHandle) -> PathBuf {
    state(app).root.join("passwords.bin")
}

/// Runs `f` on the vault, loading it on first use; saves when `f` returns true.
fn with_vault<T>(app: &AppHandle, f: impl FnOnce(&mut Vec<Login>) -> (T, bool)) -> T {
    let mut guard = VAULT.lock().unwrap();
    let list = guard.get_or_insert_with(|| {
        fs::read(vault_file(app))
            .ok()
            .and_then(|b| crypto::unprotect(&b))
            .and_then(|plain| serde_json::from_slice(&plain).ok())
            .unwrap_or_default()
    });
    let (out, changed) = f(list);
    if changed {
        if let Some(sealed) = serde_json::to_vec(list)
            .ok()
            .and_then(|plain| crypto::protect(&plain))
        {
            let file = vault_file(app);
            let tmp = file.with_extension("tmp");
            if fs::write(&tmp, sealed).is_ok() {
                let _ = fs::rename(&tmp, &file);
            }
        } else {
            diag::error("cofre de senhas: não foi possível criptografar");
        }
    }
    out
}

/// "accounts.google.com" from "https://accounts.google.com/signin".
pub(crate) fn host_of(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|h| h.trim_start_matches("www.").to_lowercase())
        })
        .unwrap_or_default()
}

/// A login saved for `login` works on `page` when one is the other or a subdomain of it.
pub(crate) fn host_matches(login: &str, page: &str) -> bool {
    !login.is_empty()
        && !page.is_empty()
        && (login == page
            || page.ends_with(&format!(".{login}"))
            || login.ends_with(&format!(".{page}")))
}

/// Adds logins, replacing the password of one already saved for the same site and user.
fn add(app: &AppHandle, incoming: Vec<(String, String, String)>, source: &str) -> usize {
    with_vault(app, |list| {
        let mut added = 0;
        let mut next = list.iter().map(|l| l.id).max().unwrap_or(0) + 1;
        for (origin, username, password) in incoming {
            let host = host_of(&origin);
            if host.is_empty() || password.is_empty() {
                continue;
            }
            if let Some(l) = list
                .iter_mut()
                .find(|l| l.host == host && l.username == username)
            {
                l.password = password;
                l.source = source.to_string();
            } else {
                list.push(Login {
                    id: next,
                    origin,
                    host,
                    username,
                    password,
                    source: source.to_string(),
                    created: now(),
                });
                next += 1;
            }
            added += 1;
        }
        (added, added > 0)
    })
}

/// Splits CSV text into rows, honoring quotes and doubled quotes.
pub(crate) fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = vec![];
    let mut row = vec![];
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = text.trim_start_matches('\u{feff}').chars().peekable();
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            ('"', true) => quoted = false,
            ('"', false) if cell.is_empty() => quoted = true,
            (',', false) => row.push(std::mem::take(&mut cell)),
            ('\r', false) => {}
            ('\n', false) => {
                row.push(std::mem::take(&mut cell));
                if row.iter().any(|c| !c.is_empty()) {
                    rows.push(std::mem::take(&mut row));
                } else {
                    row.clear();
                }
            }
            _ => cell.push(c),
        }
    }
    row.push(cell);
    if row.iter().any(|c| !c.is_empty()) {
        rows.push(row);
    }
    rows
}

/// Logins from a password CSV exported by Chrome, Edge, Brave, Firefox,
/// Safari or Bitwarden: (origin, username, password).
pub(crate) fn logins_from_csv(text: &str) -> Vec<(String, String, String)> {
    let rows = parse_csv(text);
    let Some((head, body)) = rows.split_first() else {
        return vec![];
    };
    let col = |names: &[&str]| {
        head.iter()
            .position(|h| names.contains(&h.trim().to_lowercase().as_str()))
    };
    let (Some(url), Some(user), Some(pass)) = (
        col(&["url", "login_uri", "website", "origin"]),
        col(&["username", "login_username", "user", "email"]),
        col(&["password", "login_password"]),
    ) else {
        return vec![];
    };
    body.iter()
        .filter_map(|r| {
            Some((
                r.get(url)?.trim().to_string(),
                r.get(user).cloned().unwrap_or_default(),
                r.get(pass)?.clone(),
            ))
        })
        .collect()
}

#[derive(Serialize)]
pub(crate) struct PasswordSource {
    id: String,
    name: String,
}

/// Chromium browsers on this PC that keep saved passwords.
#[tauri::command]
pub(crate) async fn password_sources() -> Vec<PasswordSource> {
    import::CHROMIUM
        .iter()
        .filter_map(|(id, name, base, dir, flat)| {
            let root = import::base_dir(*base)?.join(dir);
            let any = import::chromium_profiles(&root, *flat)
                .iter()
                .any(|(p, _)| root.join(p).join("Login Data").is_file());
            any.then(|| PasswordSource {
                id: id.to_string(),
                name: name.to_string(),
            })
        })
        .collect()
}

/// Imports every profile's saved passwords from a Chromium browser.
#[tauri::command]
pub(crate) async fn import_passwords(app: AppHandle, id: String) -> Result<String, String> {
    let (_, name, base, dir, flat) = *import::CHROMIUM
        .iter()
        .find(|(k, ..)| *k == id)
        .ok_or("Navegador desconhecido.")?;
    let root = import::base_dir(base)
        .ok_or("Pasta do navegador não encontrada.")?
        .join(dir);
    let read = tauri::async_runtime::spawn_blocking(move || crypto::read_chromium(&root, flat))
        .await
        .map_err(|e| e.to_string())??;
    let added = add(&app, read.logins, name);
    diag::info(&format!(
        "senhas importadas de {name}: {added} (protegidas: {})",
        read.locked
    ));
    let mut msg = format!("{added} senhas importadas do {name}.");
    if read.locked > 0 {
        msg.push_str(&format!(
            " {} estão protegidas pelo próprio {name} e só saem pela exportação: abra as senhas do {name}, exporte em CSV e use \"Importar arquivo CSV\".",
            read.locked
        ));
    }
    Ok(msg)
}

/// Imports a password CSV picked by the person.
#[tauri::command]
pub(crate) async fn import_passwords_csv(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let Some(file) = app
        .dialog()
        .file()
        .add_filter("Senhas exportadas (CSV)", &["csv"])
        .blocking_pick_file()
    else {
        return Err("Nenhum arquivo escolhido.".into());
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let text = fs::read_to_string(&path).map_err(|_| "Não deu para ler o arquivo.".to_string())?;
    let logins = logins_from_csv(&text);
    if logins.is_empty() {
        return Err(
            "Nenhuma senha encontrada. O arquivo precisa ter as colunas url, username e password."
                .into(),
        );
    }
    let added = add(&app, logins, "CSV");
    Ok(format!(
        "{added} senhas importadas. Apague o arquivo CSV: ele guarda as senhas sem proteção."
    ))
}

#[tauri::command]
pub(crate) async fn list_logins(app: AppHandle) -> Vec<LoginInfo> {
    with_vault(&app, |list| {
        let mut out: Vec<LoginInfo> = list.iter().map(LoginInfo::from).collect();
        out.sort_by(|a, b| a.host.cmp(&b.host).then(a.username.cmp(&b.username)));
        (out, false)
    })
}

/// Shows a password after Windows Hello confirms it's the owner.
#[tauri::command]
pub(crate) async fn reveal_login(app: AppHandle, id: u64) -> Result<String, String> {
    #[cfg(windows)]
    {
        let hello = tauri::async_runtime::spawn_blocking(hello::available)
            .await
            .unwrap_or(false);
        if hello && !spaces::verify_hello(&app, "Mostrar a senha salva no Blazon").await {
            return Err("Não confirmado.".into());
        }
    }
    with_vault(&app, |list| {
        (
            list.iter()
                .find(|l| l.id == id)
                .map(|l| l.password.clone())
                .ok_or_else(|| "Senha não encontrada.".to_string()),
            false,
        )
    })
}

#[tauri::command]
pub(crate) async fn delete_login(app: AppHandle, id: u64) {
    with_vault(&app, |list| {
        let before = list.len();
        list.retain(|l| l.id != id);
        ((), list.len() != before)
    });
}

#[tauri::command]
pub(crate) async fn clear_logins(app: AppHandle) {
    with_vault(&app, |list| {
        list.clear();
        ((), true)
    });
}

/// Saved logins for the site open in tab `tab`.
#[tauri::command]
pub(crate) async fn logins_for_tab(app: AppHandle, tab: u64) -> Vec<LoginInfo> {
    let Some(host) = app
        .get_webview(&label(tab))
        .and_then(|w| w.url().ok())
        .map(|u| host_of(u.as_str()))
    else {
        return vec![];
    };
    with_vault(&app, |list| {
        (
            list.iter()
                .filter(|l| host_matches(&l.host, &host))
                .map(LoginInfo::from)
                .collect(),
            false,
        )
    })
}

/// Fills a saved login into the page, only if the page is still on its site.
#[tauri::command]
pub(crate) async fn fill_login(app: AppHandle, tab: u64, id: u64) -> Result<(), String> {
    let wv = app
        .get_webview(&label(tab))
        .ok_or("A aba não está aberta.")?;
    let page = wv.url().map(|u| host_of(u.as_str())).unwrap_or_default();
    let login = with_vault(&app, |list| {
        (list.iter().find(|l| l.id == id).cloned(), false)
    })
    .ok_or("Senha não encontrada.")?;
    if !host_matches(&login.host, &page) {
        return Err("Esta senha é de outro site.".into());
    }
    let data = serde_json::json!({ "u": login.username, "p": login.password });
    wv.eval(format!("({FILL_JS})({data})"))
        .map_err(|e| e.to_string())?;
    let _ = wv.set_focus();
    Ok(())
}

/// Fills the visible password field and the username field before it,
/// firing the events sites listen to.
const FILL_JS: &str = r#"function (c) {
  const visible = (e) => e.offsetParent !== null && !e.disabled && !e.readOnly;
  const pass = [...document.querySelectorAll('input[type=password]')].find(visible);
  if (!pass) return;
  const fields = [...(pass.form || document).querySelectorAll('input')].filter(visible);
  const user = fields.slice(0, fields.indexOf(pass)).reverse()
    .find((e) => /^(text|email|tel)$/i.test(e.type) || e.autocomplete === 'username');
  const set = (e, v) => {
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    setter.call(e, v);
    e.dispatchEvent(new Event('input', { bubbles: true }));
    e.dispatchEvent(new Event('change', { bubbles: true }));
  };
  if (user && c.u) set(user, c.u);
  set(pass, c.p);
  pass.focus();
}"#;

pub(crate) struct ChromiumLogins {
    logins: Vec<(String, String, String)>,
    /// Passwords sealed with the browser's app-bound key (Chrome 127+), which
    /// only the browser itself can open.
    locked: usize,
}

#[cfg(windows)]
mod crypto {
    use super::ChromiumLogins;
    use crate::sqlite;
    use std::path::Path;
    use windows::Win32::{
        Foundation::{LocalFree, HLOCAL},
        Security::Cryptography::*,
    };

    fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        }
    }

    unsafe fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out.pbData as _)));
        v
    }

    /// Encrypts with DPAPI for the current Windows account.
    pub fn protect(data: &[u8]) -> Option<Vec<u8>> {
        unsafe {
            let mut out = CRYPT_INTEGER_BLOB::default();
            CryptProtectData(
                &blob(data),
                windows::core::w!("Blazon"),
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
            .ok()?;
            Some(take(out))
        }
    }

    pub fn unprotect(data: &[u8]) -> Option<Vec<u8>> {
        unsafe {
            let mut out = CRYPT_INTEGER_BLOB::default();
            CryptUnprotectData(
                &blob(data),
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
            .ok()?;
            Some(take(out))
        }
    }

    /// AES-256-GCM decryption with Windows' CNG.
    fn aes_gcm(key: &[u8], nonce: &[u8], data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < 16 {
            return None;
        }
        let (cipher, tag) = data.split_at(data.len() - 16);
        unsafe {
            let mut alg = BCRYPT_ALG_HANDLE::default();
            if BCryptOpenAlgorithmProvider(
                &mut alg,
                BCRYPT_AES_ALGORITHM,
                None,
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
            .is_err()
            {
                return None;
            }
            let mode: Vec<u8> = "ChainingModeGCM\0"
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect();
            let mut hkey = BCRYPT_KEY_HANDLE::default();
            let mut result = None;
            if BCryptSetProperty(BCRYPT_HANDLE(alg.0), BCRYPT_CHAINING_MODE, &mode, 0).is_ok()
                && BCryptGenerateSymmetricKey(alg, &mut hkey, None, key, 0).is_ok()
            {
                let mut nonce = nonce.to_vec();
                let mut tag = tag.to_vec();
                let info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
                    cbSize: std::mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
                    dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
                    pbNonce: nonce.as_mut_ptr(),
                    cbNonce: nonce.len() as u32,
                    pbTag: tag.as_mut_ptr(),
                    cbTag: tag.len() as u32,
                    ..Default::default()
                };
                let mut out = vec![0u8; cipher.len()];
                let mut n = 0u32;
                if BCryptDecrypt(
                    hkey,
                    Some(cipher),
                    Some(&info as *const _ as *const _),
                    None,
                    Some(&mut out),
                    &mut n,
                    BCRYPT_FLAGS(0),
                )
                .is_ok()
                {
                    out.truncate(n as usize);
                    result = Some(out);
                }
                let _ = BCryptDestroyKey(hkey);
            }
            let _ = BCryptCloseAlgorithmProvider(alg, 0);
            result
        }
    }

    /// The browser's AES key from "Local State", sealed with DPAPI.
    fn master_key(root: &Path) -> Option<Vec<u8>> {
        use base64::Engine;
        let state: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("Local State")).ok()?).ok()?;
        let sealed = base64::engine::general_purpose::STANDARD
            .decode(state.pointer("/os_crypt/encrypted_key")?.as_str()?)
            .ok()?;
        unprotect(sealed.strip_prefix(b"DPAPI")?)
    }

    pub fn read_chromium(root: &Path, flat: bool) -> Result<ChromiumLogins, String> {
        let key = master_key(root);
        let mut out = ChromiumLogins {
            logins: vec![],
            locked: 0,
        };
        for (dir, _) in crate::import::chromium_profiles(root, flat) {
            let db = root.join(&dir).join("Login Data");
            if !db.is_file() {
                continue;
            }
            let rows = sqlite::query(&db, "SELECT origin_url, username_value, password_value FROM logins WHERE blacklisted_by_user = 0")?;
            for r in rows {
                let (origin, user, sealed) = (r[0].text(), r[1].text(), r[2].bytes());
                let password = match sealed.get(..3) {
                    Some(b"v10") | Some(b"v11") if sealed.len() > 15 => key
                        .as_deref()
                        .and_then(|k| aes_gcm(k, &sealed[3..15], &sealed[15..])),
                    Some(b"v20") => {
                        out.locked += 1;
                        continue;
                    }
                    // Very old profiles sealed each password with DPAPI directly.
                    _ => unprotect(&sealed),
                };
                match password.and_then(|p| String::from_utf8(p).ok()) {
                    Some(p) if !p.is_empty() => out.logins.push((origin, user, p)),
                    _ => out.locked += 1,
                }
            }
        }
        Ok(out)
    }
}

#[cfg(not(windows))]
mod crypto {
    use super::ChromiumLogins;
    use std::path::Path;
    pub fn protect(_: &[u8]) -> Option<Vec<u8>> {
        None
    }
    pub fn unprotect(_: &[u8]) -> Option<Vec<u8>> {
        None
    }
    pub fn read_chromium(_: &Path, _: bool) -> Result<ChromiumLogins, String> {
        Err("Só no Windows.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_browser_csv_exports() {
        let chrome = "name,url,username,password,note\nexample.com,https://example.com/login,ana,\"s3,cr\"\"et\",\n";
        assert_eq!(
            logins_from_csv(chrome),
            vec![(
                "https://example.com/login".into(),
                "ana".into(),
                "s3,cr\"et".into()
            )]
        );
        let firefox = "\"url\",\"username\",\"password\",\"httpRealm\"\r\n\"https://site.org\",\"bia\",\"x\",\"\"\r\n";
        assert_eq!(logins_from_csv(firefox).len(), 1);
        let bitwarden = "folder,favorite,type,name,notes,fields,reprompt,login_uri,login_username,login_password,login_totp\n,,login,Site,,,0,https://a.io,cai,pw,\n";
        assert_eq!(logins_from_csv(bitwarden)[0].2, "pw");
        assert!(logins_from_csv("a,b\n1,2\n").is_empty());
    }

    #[test]
    fn matches_site_and_subdomains_only() {
        assert!(host_matches("google.com", "accounts.google.com"));
        assert!(host_matches("accounts.google.com", "google.com"));
        assert!(host_matches("github.com", "github.com"));
        assert!(!host_matches("google.com", "notgoogle.com"));
        assert!(!host_matches("google.com", "google.com.evil.io"));
        assert_eq!(host_of("https://www.Example.com/a"), "example.com");
    }
}
