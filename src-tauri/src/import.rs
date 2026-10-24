//! Reads tabs and bookmarks from other browsers installed on Windows: Arc's
//! sidebar (Spaces, pinned folders, favorites) and Chromium bookmark files.
//! The Arc reader follows Crest's `ArcSidebar` importer.

use std::{collections::HashMap, fs, path::PathBuf};

use serde::Serialize;
use serde_json::Value;

/// Upper bound on imported tabs, so a huge bookmark file can't flood the sidebar.
const MAX_ENTRIES: usize = 2000;

#[derive(Clone, Serialize)]
pub struct Source {
    pub id: String,
    pub name: String,
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

pub enum Import {
    /// Arc: one new Crest Space per Arc Space.
    Spaces(Vec<Space>),
    /// Bookmarks: folders added to the current Space.
    Bookmarks(Vec<Folder>),
}

const CHROMIUM: &[(&str, &str, &str)] = &[
    ("chrome", "Google Chrome", r"Google\Chrome\User Data"),
    ("edge", "Microsoft Edge", r"Microsoft\Edge\User Data"),
    ("brave", "Brave", r"BraveSoftware\Brave-Browser\User Data"),
    ("vivaldi", "Vivaldi", r"Vivaldi\User Data"),
];

fn local_app_data() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

fn arc_sidebar() -> Option<PathBuf> {
    let packages = local_app_data()?.join("Packages");
    fs::read_dir(packages).ok()?.flatten().find_map(|e| {
        let name = e.file_name();
        let path = e.path().join(r"LocalCache\Local\Arc\StorableSidebar.json");
        (name.to_string_lossy().starts_with("TheBrowserCompany.Arc") && path.is_file())
            .then_some(path)
    })
}

fn bookmarks_file(dir: &str) -> Option<PathBuf> {
    let path = local_app_data()?.join(dir).join(r"Default\Bookmarks");
    path.is_file().then_some(path)
}

pub fn sources() -> Vec<Source> {
    let mut out = vec![];
    if arc_sidebar().is_some() {
        out.push(Source {
            id: "arc".into(),
            name: "Arc (Spaces, pastas e favoritos)".into(),
        });
    }
    for (id, name, dir) in CHROMIUM {
        if bookmarks_file(dir).is_some() {
            out.push(Source {
                id: (*id).into(),
                name: format!("{name} (favoritos)"),
            });
        }
    }
    out
}

pub fn read(id: &str) -> Result<Import, String> {
    let path = if id == "arc" {
        arc_sidebar()
    } else {
        CHROMIUM
            .iter()
            .find(|(i, ..)| *i == id)
            .and_then(|(_, _, dir)| bookmarks_file(dir))
    }
    .ok_or("Navegador não encontrado.")?;
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let json: Value =
        serde_json::from_str(&text).map_err(|_| "Arquivo em formato desconhecido.")?;
    let mut budget = MAX_ENTRIES;
    if id == "arc" {
        Ok(Import::Spaces(read_arc(&json, &mut budget)))
    } else {
        Ok(Import::Bookmarks(read_bookmarks(&json, &mut budget)))
    }
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

// Chromium bookmarks

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
