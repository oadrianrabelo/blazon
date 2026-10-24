//! Ad and tracker blocking with Brave's adblock engine and EasyList/EasyPrivacy.
//! Lists ship with the installer and refresh weekly through Windows' curl.exe.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock, RwLock,
    },
    time::{Duration, SystemTime},
};

use adblock::{
    lists::{FilterSet, ParseOptions},
    request::Request,
    Engine,
};

const LISTS: [(&str, &str); 2] = [
    ("easylist.txt", "https://easylist.to/easylist/easylist.txt"),
    (
        "easyprivacy.txt",
        "https://easylist.to/easylist/easyprivacy.txt",
    ),
];
const REFRESH_AFTER: Duration = Duration::from_secs(7 * 24 * 3600);

struct Blocker {
    engine: RwLock<Option<Engine>>,
    enabled: AtomicBool,
    /// Sites (hosts) where blocking is paused.
    allow: RwLock<Vec<String>>,
}

fn blocker() -> &'static Blocker {
    static B: OnceLock<Blocker> = OnceLock::new();
    B.get_or_init(|| Blocker {
        engine: RwLock::new(None),
        enabled: AtomicBool::new(true),
        allow: RwLock::new(vec![]),
    })
}

pub fn configure(enabled: bool, allow: &[String]) {
    let b = blocker();
    b.enabled.store(enabled, Ordering::Relaxed);
    *b.allow.write().unwrap() = allow
        .iter()
        .map(|h| h.trim().to_lowercase())
        .filter(|h| !h.is_empty())
        .collect();
}

fn build(paths: &[PathBuf]) -> Option<Engine> {
    let mut set = FilterSet::new(false);
    let mut any = false;
    for p in paths {
        if let Ok(text) = fs::read_to_string(p) {
            set.add_filter_list(text, ParseOptions::default());
            any = true;
        }
    }
    any.then(|| Engine::new_with_filter_set(set))
}

fn is_fresh(p: &Path) -> bool {
    fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < REFRESH_AFTER)
}

/// Loads the lists in the background: cached copies when present, otherwise
/// the bundled ones; then refreshes stale lists and rebuilds the engine.
pub fn load(bundled: Option<PathBuf>, cache: PathBuf) {
    std::thread::spawn(move || {
        let _ = fs::create_dir_all(&cache);
        let pick = |name: &str| {
            let cached = cache.join(name);
            if cached.is_file() {
                Some(cached)
            } else {
                bundled
                    .as_ref()
                    .map(|d| d.join(name))
                    .filter(|p| p.is_file())
            }
        };
        let paths: Vec<PathBuf> = LISTS.iter().filter_map(|(n, _)| pick(n)).collect();
        if let Some(engine) = build(&paths) {
            *blocker().engine.write().unwrap() = Some(engine);
        }
        let mut refreshed = false;
        for (name, url) in LISTS {
            let target = cache.join(name);
            if !is_fresh(&target) && download(url, &target) {
                refreshed = true;
            }
        }
        if refreshed {
            let paths: Vec<PathBuf> = LISTS.iter().filter_map(|(n, _)| pick(n)).collect();
            if let Some(engine) = build(&paths) {
                *blocker().engine.write().unwrap() = Some(engine);
            }
        }
    });
}

fn download(url: &str, target: &Path) -> bool {
    let tmp = target.with_extension("part");
    let mut cmd = std::process::Command::new("curl.exe");
    cmd.args(["-sSfL", "--max-time", "60", "-o"])
        .arg(&tmp)
        .arg(url);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let ok = cmd.status().is_ok_and(|s| s.success())
        && fs::metadata(&tmp).is_ok_and(|m| m.len() > 10_000)
        && fs::rename(&tmp, target).is_ok();
    let _ = fs::remove_file(&tmp);
    ok
}

fn host(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|h| h.trim_start_matches("www.").to_lowercase())
        })
        .unwrap_or_default()
}

fn paused_for(source: &str) -> bool {
    let b = blocker();
    if !b.enabled.load(Ordering::Relaxed) {
        return true;
    }
    let h = host(source);
    b.allow
        .read()
        .unwrap()
        .iter()
        .any(|a| h == *a || h.ends_with(&format!(".{a}")))
}

/// Whether a subresource request made by the page at `source` should be blocked.
pub fn should_block(url: &str, source: &str, kind: &str) -> bool {
    if paused_for(source) {
        return false;
    }
    let engine = blocker().engine.read().unwrap();
    let Some(engine) = engine.as_ref() else {
        return false;
    };
    Request::new(url, source, kind, "GET")
        .is_ok_and(|r| engine.check_network_request(&r).should_block())
}

/// CSS hiding page-specific ad elements, or empty.
pub fn cosmetic_css(url: &str) -> String {
    if paused_for(url) {
        return String::new();
    }
    let engine = blocker().engine.read().unwrap();
    let Some(engine) = engine.as_ref() else {
        return String::new();
    };
    let selectors: Vec<String> = engine
        .url_cosmetic_resources(url)
        .hide_selectors
        .into_iter()
        .collect();
    if selectors.is_empty() {
        return String::new();
    }
    format!("{}{{display:none!important}}", selectors.join(","))
}
