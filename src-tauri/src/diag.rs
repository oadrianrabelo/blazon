//! Diagnostics: a log file, crash reports and the "Report a problem" text.
//!
//! The log never records full addresses or page titles, only hosts, so a
//! report can be shared without exposing browsing history.

use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_LOG_BYTES: u64 = 1_000_000;
const RECENT_LINES: usize = 400;

struct Diag {
    dir: PathBuf,
    recent: Mutex<VecDeque<String>>,
}

static DIAG: OnceLock<Diag> = OnceLock::new();

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Civil date from days since 1970-01-01 (UTC).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Starts logging to `<root>/logs/blazon.log` and installs the crash hook.
pub fn init(root: &std::path::Path) {
    let dir = root.join("logs");
    let _ = fs::create_dir_all(&dir);
    let log = dir.join("blazon.log");
    if fs::metadata(&log).is_ok_and(|m| m.len() > MAX_LOG_BYTES) {
        let _ = fs::rename(&log, dir.join("blazon.old.log"));
    }
    let _ = DIAG.set(Diag {
        dir,
        recent: Mutex::new(VecDeque::new()),
    });
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        let backtrace = std::backtrace::Backtrace::force_capture();
        let text = format!("PANIC em {location}: {message}\n{backtrace}");
        error(&text);
        if let Some(d) = DIAG.get() {
            let _ = fs::write(d.dir.join("crash.txt"), format!("{}\n{text}", timestamp()));
        }
    }));
    info(&format!("Blazon {} iniciado", env!("CARGO_PKG_VERSION")));
}

fn write(level: &str, msg: &str) {
    let line = format!("[{}] {level} {msg}", timestamp());
    let Some(d) = DIAG.get() else {
        eprintln!("{line}");
        return;
    };
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(d.dir.join("blazon.log"))
    {
        let _ = writeln!(f, "{line}");
    }
    let mut recent = d.recent.lock().unwrap_or_else(|e| e.into_inner());
    recent.push_back(line);
    while recent.len() > RECENT_LINES {
        recent.pop_front();
    }
}

pub fn info(msg: &str) {
    write("INFO ", msg);
}

pub fn warn(msg: &str) {
    write("AVISO", msg);
}

pub fn error(msg: &str) {
    write("ERRO ", msg);
}

/// Only the host of an address, for logging.
pub fn host(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
        .unwrap_or_else(|| "(local)".into())
}

/// The report of the last crash, if the previous session crashed. Taking it clears it.
pub fn take_crash() -> Option<String> {
    let d = DIAG.get()?;
    let path = d.dir.join("crash.txt");
    let text = fs::read_to_string(&path).ok()?;
    let _ = fs::rename(&path, d.dir.join("crash.last.txt"));
    Some(text)
}

pub fn logs_dir() -> Option<PathBuf> {
    DIAG.get().map(|d| d.dir.clone())
}

pub fn recent_lines(n: usize) -> Vec<String> {
    let Some(d) = DIAG.get() else { return vec![] };
    let recent = d.recent.lock().unwrap_or_else(|e| e.into_inner());
    recent.iter().rev().take(n).rev().cloned().collect()
}

/// Windows version as reported by `ver`.
pub fn os_version() -> String {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new("cmd")
            .args(["/C", "ver"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "Windows (versão desconhecida)".into())
    }
    #[cfg(not(windows))]
    {
        std::env::consts::OS.to_string()
    }
}

/// Installed WebView2 runtime version.
pub fn webview2_version() -> String {
    #[cfg(windows)]
    {
        use webview2_com::{
            take_pwstr,
            Microsoft::Web::WebView2::Win32::GetAvailableCoreWebView2BrowserVersionString,
        };
        let mut v = windows_core::PWSTR::null();
        unsafe {
            if GetAvailableCoreWebView2BrowserVersionString(windows_core::PCWSTR::null(), &mut v)
                .is_ok()
            {
                return take_pwstr(v);
            }
        }
        "desconhecida".into()
    }
    #[cfg(not(windows))]
    {
        "n/d".into()
    }
}
