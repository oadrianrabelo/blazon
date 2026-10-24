//! Imports from other browsers installed on Windows:
//!
//! - **Arc**: Spaces, pinned folders, favorites and history (Arc's sidebar
//!   reader follows Crest's `ArcSidebar` importer).
//! - **Chromium family** (Chrome, Edge, Brave, Vivaldi, Opera, Opera GX,
//!   Chromium), every profile: bookmarks and history.
//! - **Firefox family** (Firefox, Zen, LibreWolf, Waterfox, Floorp), every
//!   profile: bookmarks and history.
//! - **Bookmarks HTML** exported by any browser.
//!
//! Passwords can't be imported: WebView2 has no API to add saved passwords.

use std::{collections::HashMap, fs, path::PathBuf};

use serde::Serialize;
use serde_json::Value;

/// Upper bounds, so a huge profile can't flood the sidebar or the history.
const MAX_ENTRIES: usize = 3000;
const MAX_HISTORY: usize = 5000;

#[derive(Clone, Serialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    /// What this source can provide: "spaces", "bookmarks", "history".
    pub kinds: Vec<&'static str>,
}

pub struct Entry {
    pub title: String,
    pub url: String,
}

pub struct Folder {
    pub name: String,
    pub tabs: Vec<Entry>,
}

#[derive(Default)]
pub struct Space {
    pub name: String,
    pub pinned: Vec<Entry>,
    pub folders: Vec<Folder>,
    pub tabs: Vec<Entry>,
}

pub struct Visit {
    pub url: String,
    pub title: String,
    /// Unix seconds.
    pub time: u64,
    pub count: u32,
}

#[derive(Default)]
pub struct Imported {
    /// Arc: one new Space per Arc Space.
    pub spaces: Vec<Space>,
    /// Bookmarks, as folders named by their path.
    pub folders: Vec<Folder>,
    pub history: Vec<Visit>,
}

/// Chromium-based browsers: key, name, base folder, and whether profiles
/// live directly in it (Opera) instead of in Default / Profile N folders.
const CHROMIUM: &[(&str, &str, Base, &str, bool)] = &[
    (
        "chrome",
        "Google Chrome",
        Base::Local,
        r"Google\Chrome\User Data",
        false,
    ),
    (
        "edge",
        "Microsoft Edge",
        Base::Local,
        r"Microsoft\Edge\User Data",
        false,
    ),
    (
        "brave",
        "Brave",
        Base::Local,
        r"BraveSoftware\Brave-Browser\User Data",
        false,
    ),
    (
        "vivaldi",
        "Vivaldi",
        Base::Local,
        r"Vivaldi\User Data",
        false,
    ),
    (
        "chromium",
        "Chromium",
        Base::Local,
        r"Chromium\User Data",
        false,
    ),
    (
        "opera",
        "Opera",
        Base::Roaming,
        r"Opera Software\Opera Stable",
        true,
    ),
    (
        "operagx",
        "Opera GX",
        Base::Roaming,
        r"Opera Software\Opera GX Stable",
        true,
    ),
];

const FIREFOX: &[(&str, &str, &str)] = &[
    ("firefox", "Firefox", r"Mozilla\Firefox"),
    ("zen", "Zen", r"zen"),
    ("librewolf", "LibreWolf", r"librewolf"),
    ("waterfox", "Waterfox", r"Waterfox"),
    ("floorp", "Floorp", r"Floorp"),
];

#[derive(Clone, Copy)]
enum Base {
    Local,
    Roaming,
}

fn base_dir(base: Base) -> Option<PathBuf> {
    let var = match base {
        Base::Local => "LOCALAPPDATA",
        Base::Roaming => "APPDATA",
    };
    std::env::var_os(var).map(PathBuf::from)
}

fn arc_dir() -> Option<PathBuf> {
    let packages = base_dir(Base::Local)?.join("Packages");
    fs::read_dir(packages).ok()?.flatten().find_map(|e| {
        let dir = e.path().join(r"LocalCache\Local\Arc");
        (e.file_name()
            .to_string_lossy()
            .starts_with("TheBrowserCompany.Arc")
            && dir.is_dir())
        .then_some(dir)
    })
}

/// Profile folders of a Chromium browser with their display names.
fn chromium_profiles(root: &std::path::Path, flat: bool) -> Vec<(String, String)> {
    if flat {
        return if root.join("Bookmarks").is_file() || root.join("History").is_file() {
            vec![(String::new(), String::new())]
        } else {
            vec![]
        };
    }
    let names: HashMap<String, String> = fs::read_to_string(root.join("Local State"))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| {
            v.pointer("/profile/info_cache")
                .and_then(Value::as_object)
                .cloned()
        })
        .map(|m| {
            m.into_iter()
                .map(|(k, v)| (k, str_of(&v, "name").to_string()))
                .collect()
        })
        .unwrap_or_default();
    let mut out: Vec<(String, String)> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let dir = e.file_name().to_string_lossy().to_string();
            let p = e.path();
            let is_profile = dir == "Default" || dir.starts_with("Profile ");
            (is_profile && (p.join("Bookmarks").is_file() || p.join("History").is_file()))
                .then(|| (dir.clone(), names.get(&dir).cloned().unwrap_or(dir)))
        })
        .collect();
    out.sort_by_key(|(dir, _)| (dir != "Default", dir.clone()));
    out
}

fn firefox_profiles(root: &std::path::Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(root.join("Profiles"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().join("places.sqlite").is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    out.sort();
    out
}

pub fn sources() -> Vec<Source> {
    let mut out = vec![];
    if let Some(arc) = arc_dir() {
        let mut kinds = vec![];
        if arc.join("StorableSidebar.json").is_file() {
            kinds.push("spaces");
        }
        if arc.join(r"User Data\Default\History").is_file() {
            kinds.push("history");
        }
        if !kinds.is_empty() {
            out.push(Source {
                id: "arc".into(),
                name: "Arc".into(),
                kinds,
            });
        }
    }
    for (key, name, base, dir, flat) in CHROMIUM {
        let Some(root) = base_dir(*base).map(|b| b.join(dir)) else {
            continue;
        };
        let profiles = chromium_profiles(&root, *flat);
        let many = profiles.len() > 1;
        for (dir, label) in profiles {
            let p = if dir.is_empty() {
                root.clone()
            } else {
                root.join(&dir)
            };
            let mut kinds = vec![];
            if p.join("Bookmarks").is_file() {
                kinds.push("bookmarks");
            }
            if p.join("History").is_file() {
                kinds.push("history");
            }
            let name = if many {
                format!("{name} · {label}")
            } else {
                name.to_string()
            };
            out.push(Source {
                id: format!("cr:{key}:{dir}"),
                name,
                kinds,
            });
        }
    }
    for (key, name, dir) in FIREFOX {
        let Some(root) = base_dir(Base::Roaming).map(|b| b.join(dir)) else {
            continue;
        };
        let profiles = firefox_profiles(&root);
        let many = profiles.len() > 1;
        for p in profiles {
            let label = p
                .split_once('.')
                .map(|(_, l)| l.to_string())
                .unwrap_or(p.clone());
            let name = if many {
                format!("{name} · {label}")
            } else {
                name.to_string()
            };
            out.push(Source {
                id: format!("ff:{key}:{p}"),
                name,
                kinds: vec!["bookmarks", "history"],
            });
        }
    }
    out
}

/// Reads what was asked for from source `id` (see [`sources`]).
pub fn read(id: &str, bookmarks: bool, history: bool) -> Result<Imported, String> {
    let mut out = Imported::default();
    let mut budget = MAX_ENTRIES;
    let not_found = || "Navegador não encontrado.".to_string();
    if id == "arc" {
        let dir = arc_dir().ok_or_else(not_found)?;
        if bookmarks {
            if let Ok(text) = fs::read_to_string(dir.join("StorableSidebar.json")) {
                let json: Value = serde_json::from_str(&text)
                    .map_err(|_| "Arquivo do Arc em formato desconhecido.")?;
                out.spaces = read_arc(&json, &mut budget);
            }
        }
        if history {
            out.history = chromium_history(&dir.join(r"User Data\Default\History"))?;
        }
    } else if let Some(rest) = id.strip_prefix("cr:") {
        let (key, dir) = rest.split_once(':').ok_or_else(not_found)?;
        let (_, _, base, root, _) = CHROMIUM.iter().find(|c| c.0 == key).ok_or_else(not_found)?;
        let mut p = base_dir(*base).ok_or_else(not_found)?.join(root);
        if !dir.is_empty() {
            p = p.join(dir);
        }
        if bookmarks {
            if let Ok(text) = fs::read_to_string(p.join("Bookmarks")) {
                let json: Value = serde_json::from_str(&text)
                    .map_err(|_| "Favoritos em formato desconhecido.")?;
                out.folders = read_bookmarks(&json, &mut budget);
            }
        }
        if history && p.join("History").is_file() {
            out.history = chromium_history(&p.join("History"))?;
        }
    } else if let Some(rest) = id.strip_prefix("ff:") {
        let (key, profile) = rest.split_once(':').ok_or_else(not_found)?;
        let (_, _, root) = FIREFOX.iter().find(|f| f.0 == key).ok_or_else(not_found)?;
        let places = base_dir(Base::Roaming)
            .ok_or_else(not_found)?
            .join(root)
            .join("Profiles")
            .join(profile)
            .join("places.sqlite");
        if bookmarks {
            out.folders = firefox_bookmarks(&places, &mut budget)?;
        }
        if history {
            out.history = firefox_history(&places)?;
        }
    } else {
        return Err(not_found());
    }
    Ok(out)
}

fn entry(title: &str, url: &str, budget: &mut usize) -> Option<Entry> {
    if *budget == 0 || !(url.starts_with("http://") || url.starts_with("https://")) {
        return None;
    }
    *budget -= 1;
    Some(Entry {
        title: title.into(),
        url: url.into(),
    })
}

fn str_of<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

// Chromium

fn read_bookmarks(json: &Value, budget: &mut usize) -> Vec<Folder> {
    let mut folders = vec![];
    for (root, name) in [
        ("bookmark_bar", "Barra de favoritos"),
        ("other", "Outros favoritos"),
        ("synced", "Favoritos do celular"),
    ] {
        if let Some(node) = json.get("roots").and_then(|r| r.get(root)) {
            walk_bookmarks(node, name, &mut folders, budget);
        }
    }
    folders.retain(|f| !f.tabs.is_empty());
    folders
}

/// Flattens nested bookmark folders into "Parent / Child" folders.
fn walk_bookmarks(node: &Value, path: &str, out: &mut Vec<Folder>, budget: &mut usize) {
    let mut folder = Folder {
        name: path.into(),
        tabs: vec![],
    };
    let mut subfolders = vec![];
    for child in node
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match str_of(child, "type") {
            "url" => folder
                .tabs
                .extend(entry(str_of(child, "name"), str_of(child, "url"), budget)),
            "folder" => subfolders.push(child),
            _ => {}
        }
    }
    out.push(folder);
    for sub in subfolders {
        walk_bookmarks(
            sub,
            &format!("{path} / {}", str_of(sub, "name")),
            out,
            budget,
        );
    }
}

#[cfg(windows)]
fn chromium_history(path: &std::path::Path) -> Result<Vec<Visit>, String> {
    // Chromium stores microseconds since 1601-01-01.
    let rows = crate::sqlite::query(
        path,
        &format!("SELECT url, title, visit_count, last_visit_time FROM urls WHERE hidden = 0 AND last_visit_time > 0 ORDER BY last_visit_time DESC LIMIT {MAX_HISTORY}"),
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            visit(
                &r[0].text(),
                &r[1].text(),
                (r[3].int() / 1_000_000 - 11_644_473_600) as u64,
                r[2].int(),
            )
        })
        .collect())
}

// Firefox

#[cfg(windows)]
fn firefox_history(places: &std::path::Path) -> Result<Vec<Visit>, String> {
    // Firefox stores microseconds since the Unix epoch.
    let rows = crate::sqlite::query(
        places,
        &format!("SELECT url, IFNULL(title, ''), visit_count, last_visit_date FROM moz_places WHERE hidden = 0 AND last_visit_date IS NOT NULL ORDER BY last_visit_date DESC LIMIT {MAX_HISTORY}"),
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            visit(
                &r[0].text(),
                &r[1].text(),
                (r[3].int() / 1_000_000) as u64,
                r[2].int(),
            )
        })
        .collect())
}

#[cfg(windows)]
fn firefox_bookmarks(places: &std::path::Path, budget: &mut usize) -> Result<Vec<Folder>, String> {
    let rows = crate::sqlite::query(
        places,
        "SELECT b.id, b.type, b.parent, IFNULL(b.title, ''), IFNULL(p.url, ''), IFNULL(b.guid, '') \
         FROM moz_bookmarks b LEFT JOIN moz_places p ON b.fk = p.id ORDER BY b.parent, b.position",
    )?;
    let roots = [
        ("toolbar_____", "Barra de favoritos"),
        ("menu________", "Menu de favoritos"),
        ("unfiled_____", "Outros favoritos"),
        ("mobile______", "Favoritos do celular"),
    ];
    // Folders: id -> (title, parent, guid).
    let folders: HashMap<i64, (String, i64, String)> = rows
        .iter()
        .filter(|r| r[1].int() == 2)
        .map(|r| (r[0].int(), (r[3].text(), r[2].int(), r[5].text())))
        .collect();
    let path_of = |mut id: i64| {
        let mut parts = vec![];
        for _ in 0..32 {
            let Some((title, parent, guid)) = folders.get(&id) else {
                break;
            };
            if let Some((_, name)) = roots.iter().find(|(g, _)| g == guid) {
                parts.push(name.to_string());
                break;
            }
            if guid == "root________" {
                break;
            }
            parts.push(if title.is_empty() {
                "Pasta sem nome".into()
            } else {
                title.clone()
            });
            id = *parent;
        }
        parts.reverse();
        parts.join(" / ")
    };
    let mut out: Vec<Folder> = vec![];
    for r in rows.iter().filter(|r| r[1].int() == 1) {
        let Some(e) = entry(&r[3].text(), &r[4].text(), budget) else {
            continue;
        };
        let path = path_of(r[2].int());
        match out.iter_mut().find(|f| f.name == path) {
            Some(f) => f.tabs.push(e),
            None => out.push(Folder {
                name: path,
                tabs: vec![e],
            }),
        }
    }
    Ok(out)
}

#[cfg(not(windows))]
fn chromium_history(_: &std::path::Path) -> Result<Vec<Visit>, String> {
    Err("Importar histórico só funciona no Windows.".into())
}
#[cfg(not(windows))]
fn firefox_history(_: &std::path::Path) -> Result<Vec<Visit>, String> {
    Err("Importar histórico só funciona no Windows.".into())
}
#[cfg(not(windows))]
fn firefox_bookmarks(_: &std::path::Path, _: &mut usize) -> Result<Vec<Folder>, String> {
    Err("Importar do Firefox só funciona no Windows.".into())
}

#[cfg_attr(not(windows), allow(dead_code))]
fn visit(url: &str, title: &str, time: u64, count: i64) -> Option<Visit> {
    (url.starts_with("http://") || url.starts_with("https://")).then(|| Visit {
        url: url.into(),
        title: title.into(),
        time,
        count: count.clamp(1, u32::MAX as i64) as u32,
    })
}

// Bookmarks HTML (the Netscape format every browser exports)

pub fn read_html(path: &std::path::Path) -> Result<Imported, String> {
    let text = fs::read_to_string(path).map_err(|_| "Não deu para ler o arquivo.")?;
    let mut budget = MAX_ENTRIES;
    let folders = parse_bookmarks_html(&text, &mut budget);
    if folders.is_empty() {
        return Err("Nenhum favorito encontrado nesse arquivo.".into());
    }
    Ok(Imported {
        folders,
        ..Default::default()
    })
}

fn decode_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
}

fn parse_bookmarks_html(text: &str, budget: &mut usize) -> Vec<Folder> {
    let lower = text.to_ascii_lowercase();
    let mut out: Vec<Folder> = vec![];
    let mut stack: Vec<String> = vec![];
    let mut pending: Option<String> = None;
    let mut i = 0;
    while let Some(off) = lower[i..].find('<') {
        let at = i + off;
        let rest = &lower[at..];
        if rest.starts_with("<h3") {
            let start = at + rest.find('>').map_or(3, |p| p + 1);
            let end = lower[start..]
                .find("</h3>")
                .map_or(lower.len(), |p| start + p);
            pending = Some(decode_entities(text[start..end].trim()));
            i = end;
        } else if rest.starts_with("<dl") {
            stack.push(pending.take().unwrap_or_default());
            i = at + 3;
        } else if rest.starts_with("</dl") {
            stack.pop();
            i = at + 4;
        } else if rest.starts_with("<a ") {
            let tag_end = at + rest.find('>').unwrap_or(rest.len() - 1);
            let tag = &text[at..tag_end];
            let href = tag.to_ascii_lowercase().find("href=\"").and_then(|p| {
                let v = &tag[p + 6..];
                v.find('"').map(|q| decode_entities(&v[..q]))
            });
            let end = lower[tag_end..]
                .find("</a>")
                .map_or(lower.len(), |p| tag_end + p);
            let title = decode_entities(text[(tag_end + 1).min(end)..end].trim());
            if let Some(e) = href.and_then(|h| entry(&title, &h, budget)) {
                let path = stack
                    .iter()
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" / ");
                let path = if path.is_empty() {
                    "Favoritos importados".to_string()
                } else {
                    path
                };
                match out.iter_mut().find(|f| f.name == path) {
                    Some(f) => f.tabs.push(e),
                    None => out.push(Folder {
                        name: path,
                        tabs: vec![e],
                    }),
                }
            }
            i = end;
        } else {
            i = at + 1;
        }
    }
    out
}

// Arc sidebar

/// Arc stores lists as alternating `id, object` pairs (or objects carrying `id`).
fn objects(v: &Value) -> Vec<(String, &Value)> {
    let mut out = vec![];
    if let Some(arr) = v.as_array() {
        let mut pending = None;
        for item in arr {
            if let Some(s) = item.as_str() {
                pending = Some(s.to_string());
            } else if item.is_object() {
                let id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .map(String::from)
                    .or(pending.take());
                if let Some(id) = id {
                    out.push((id, item));
                }
            }
        }
    }
    out
}

fn profile_key(v: &Value) -> String {
    if v.get("default").is_some() {
        "default".into()
    } else {
        v.to_string()
    }
}

fn children(item: &Value) -> Vec<String> {
    item.get("childrenIds")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Collects the tabs under `id`; folders become Crest folders named by path.
fn collect(
    id: &str,
    items: &HashMap<String, &Value>,
    path: &str,
    tabs: &mut Vec<Entry>,
    folders: &mut Vec<Folder>,
    depth: usize,
    budget: &mut usize,
) {
    let Some(item) = items.get(id) else { return };
    if depth > 16 {
        return;
    }
    let data = item.get("data");
    if let Some(tab) = data.and_then(|d| d.get("tab")) {
        let title = match str_of(tab, "savedTitle") {
            "" => str_of(item, "title"),
            t => t,
        };
        tabs.extend(entry(title, str_of(tab, "savedURL"), budget));
        return;
    }
    if data.and_then(|d| d.get("list")).is_some() {
        let name = match str_of(item, "title") {
            "" => "Pasta sem nome",
            t => t,
        };
        let name = if path.is_empty() {
            name.to_string()
        } else {
            format!("{path} / {name}")
        };
        let mut inner = vec![];
        let mut nested = vec![];
        for c in children(item) {
            collect(&c, items, &name, &mut inner, &mut nested, depth + 1, budget);
        }
        folders.push(Folder { name, tabs: inner });
        folders.extend(nested);
        return;
    }
    // Containers and split views: their children belong to the same place.
    for c in children(item) {
        collect(&c, items, path, tabs, folders, depth + 1, budget);
    }
}

fn read_arc(json: &Value, budget: &mut usize) -> Vec<Space> {
    let mut spaces = vec![];
    let containers = json
        .get("sidebar")
        .and_then(|s| s.get("containers"))
        .and_then(Value::as_array);
    for container in containers.into_iter().flatten() {
        let items: HashMap<String, &Value> =
            objects(container.get("items").unwrap_or(&Value::Null))
                .into_iter()
                .collect();
        let mut favorites = HashMap::new();
        if let Some(pairs) = container
            .get("topAppsContainerIDs")
            .and_then(Value::as_array)
        {
            for pair in pairs.chunks(2) {
                if let [profile, Value::String(id)] = pair {
                    favorites.insert(profile_key(profile), id.clone());
                }
            }
        }
        for (_, space) in objects(container.get("spaces").unwrap_or(&Value::Null)) {
            let mut out = Space {
                name: match str_of(space, "title") {
                    "" => format!("Arc {}", spaces.len() + 1),
                    t => t.into(),
                },
                ..Default::default()
            };
            if let Some(fav) =
                favorites.get(&profile_key(space.get("profile").unwrap_or(&Value::Null)))
            {
                let mut nested = vec![];
                collect(fav, &items, "", &mut out.pinned, &mut nested, 0, budget);
                out.pinned.extend(nested.into_iter().flat_map(|f| f.tabs));
            }
            let sections = space
                .get("containerIDs")
                .or_else(|| space.get("newContainerIDs"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut pinned_section = false;
            for section in sections.iter().filter_map(Value::as_str) {
                match section {
                    "pinned" => pinned_section = true,
                    "unpinned" => pinned_section = false,
                    id if pinned_section => {
                        let mut loose = vec![];
                        collect(id, &items, "", &mut loose, &mut out.folders, 0, budget);
                        if !loose.is_empty() {
                            out.folders.insert(
                                0,
                                Folder {
                                    name: "Fixadas".into(),
                                    tabs: loose,
                                },
                            );
                        }
                    }
                    id => {
                        let mut nested = vec![];
                        collect(id, &items, "", &mut out.tabs, &mut nested, 0, budget);
                        out.folders.extend(nested);
                    }
                }
            }
            out.folders.retain(|f| !f.tabs.is_empty());
            spaces.push(out);
        }
    }
    spaces
}
