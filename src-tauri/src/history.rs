//! Browsing history: recording visits, suggestions, the start page and imports.

use crate::*;

pub(crate) fn record_visit(app: &AppHandle, space: &str, url: &str, title: &str) {
    if url.is_empty() || url.starts_with("about:") {
        return;
    }
    let s = state(app);
    let mut h = s.history.lock().unwrap();
    let (title, count) = match h.iter().position(|v| v.space == space && v.url == url) {
        Some(i) => {
            let old = h.remove(i);
            (
                if title.is_empty() {
                    old.title
                } else {
                    title.to_string()
                },
                old.count.saturating_add(1),
            )
        }
        None => (title.to_string(), 1),
    };
    h.push(Visit {
        space: space.into(),
        url: url.into(),
        title,
        time: now(),
        count,
    });
    if h.len() > HISTORY_LIMIT {
        let extra = h.len() - HISTORY_LIMIT;
        h.drain(..extra);
    }
    s.history_dirty.store(true, Ordering::Relaxed);
}

pub(crate) fn set_visit_title(app: &AppHandle, space: &str, url: &str, title: &str) {
    let s = state(app);
    if let Some(v) = s
        .history
        .lock()
        .unwrap()
        .iter_mut()
        .rev()
        .find(|v| v.space == space && v.url == url)
    {
        v.title = title.into();
        s.history_dirty.store(true, Ordering::Relaxed);
    }
}

/// What the start page shows: the Space's look and its most visited sites.
pub(crate) fn start_page_data(app: &AppHandle, space: &str) -> String {
    let (sp, search, lang) = with_data(app, |d| {
        (
            d.spaces.iter().find(|s| s.id == space).cloned(),
            d.search_for(space).to_string(),
            d.settings.language.clone(),
        )
    });
    let mut by_host: HashMap<String, (u32, Visit)> = HashMap::new();
    for v in state(app)
        .history
        .lock()
        .unwrap()
        .iter()
        .filter(|v| v.space == space)
    {
        let Some(host) = Url::parse(&v.url)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
        else {
            continue;
        };
        let e = by_host.entry(host).or_insert((0, v.clone()));
        e.0 += v.count;
        if v.count > e.1.count {
            e.1 = v.clone();
        }
    }
    let mut top: Vec<(u32, Visit)> = by_host.into_values().collect();
    top.sort_by_key(|t| std::cmp::Reverse(t.0));
    let sites: Vec<_> = top
        .into_iter()
        .take(8)
        .map(|(_, v)| serde_json::json!({ "url": v.url, "title": v.title }))
        .collect();
    serde_json::json!({ "space": sp, "search": search, "sites": sites, "lang": lang }).to_string()
}

pub(crate) fn save_history(app: &AppHandle) {
    let s = state(app);
    if !s.history_dirty.swap(false, Ordering::Relaxed) {
        return;
    }
    let json = serde_json::to_string(&*s.history.lock().unwrap()).unwrap_or_default();
    let file = s.root.join("history.json");
    let tmp = file.with_extension("tmp");
    if fs::write(&tmp, json).is_ok() {
        let _ = fs::rename(&tmp, &file);
    }
}

/// Address bar suggestions: open tabs first, then history, newest first.
#[tauri::command]
pub(crate) async fn suggest(app: AppHandle, query: String) -> Vec<Suggestion> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return vec![];
    }
    let words: Vec<&str> = q.split_whitespace().collect();
    let hit = |url: &str, title: &str| {
        let hay = format!("{} {}", url.to_lowercase(), title.to_lowercase());
        words.iter().all(|w| hay.contains(w))
    };
    let (space, tabs) = with_data(&app, |d| (d.active_space.clone(), d.tabs.clone()));
    let active = with_data(&app, |d| d.active_tab_id());
    let mut out: Vec<Suggestion> = tabs
        .iter()
        .filter(|t| {
            t.space == space && Some(t.id) != active && !t.url.is_empty() && hit(&t.url, &t.title)
        })
        .take(3)
        .map(|t| Suggestion {
            kind: "tab",
            url: t.url.clone(),
            title: t.title.clone(),
            tab: Some(t.id),
        })
        .collect();
    let history = state(&app).history.lock().unwrap();
    for v in history.iter().rev() {
        if out.len() >= 8 {
            break;
        }
        if v.space == space && hit(&v.url, &v.title) && !out.iter().any(|s| s.url == v.url) {
            out.push(Suggestion {
                kind: "history",
                url: v.url.clone(),
                title: v.title.clone(),
                tab: None,
            });
        }
    }
    out
}

#[tauri::command]
pub(crate) async fn history_list(app: AppHandle, query: String) -> Vec<Visit> {
    let q = query.to_lowercase();
    let space = with_data(&app, |d| d.active_space.clone());
    state(&app)
        .history
        .lock()
        .unwrap()
        .iter()
        .rev()
        .filter(|v| {
            v.space == space
                && (q.is_empty()
                    || v.url.to_lowercase().contains(&q)
                    || v.title.to_lowercase().contains(&q))
        })
        .take(300)
        .cloned()
        .collect()
}

#[tauri::command]
pub(crate) async fn clear_history(app: AppHandle) {
    let space = with_data(&app, |d| d.active_space.clone());
    let s = state(&app);
    s.history.lock().unwrap().retain(|v| v.space != space);
    s.history_dirty.store(true, Ordering::Relaxed);
    save_history(&app);
}

/// Adds imported visits to a Space's history, merging repeated addresses.
pub(crate) fn merge_history(app: &AppHandle, space: &str, visits: Vec<import::Visit>) -> usize {
    let n = visits.len();
    let s = state(app);
    let mut h = s.history.lock().unwrap();
    let mut index: HashMap<String, usize> = h
        .iter()
        .enumerate()
        .filter(|(_, v)| v.space == space)
        .map(|(i, v)| (v.url.clone(), i))
        .collect();
    for v in visits {
        match index.get(&v.url) {
            Some(&i) => {
                let e = &mut h[i];
                e.count = e.count.saturating_add(v.count);
                e.time = e.time.max(v.time);
                if e.title.is_empty() {
                    e.title = v.title;
                }
            }
            None => {
                index.insert(v.url.clone(), h.len());
                h.push(Visit {
                    space: space.into(),
                    url: v.url,
                    title: v.title,
                    time: v.time,
                    count: v.count,
                });
            }
        }
    }
    h.sort_by_key(|v| v.time);
    if h.len() > HISTORY_LIMIT {
        let extra = h.len() - HISTORY_LIMIT;
        h.drain(..extra);
    }
    s.history_dirty.store(true, Ordering::Relaxed);
    drop(h);
    save_history(app);
    n
}
