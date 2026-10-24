//! Keyboard shortcuts the person can change (Settings › Shortcuts). One table
//! of commands with their default keys; overrides live in Settings.shortcuts.
//! The sidebar and every site page read the same combo → command map, so a
//! shortcut works wherever the focus is. Ctrl+1…9 (tabs), Ctrl+Shift+1…9
//! (Spaces) and F6 stay fixed.

use crate::*;

/// Command id and default combo, as "Ctrl+Shift+T" (modifiers in that order).
pub(crate) const DEFAULTS: &[(&str, &str)] = &[
    ("newtab", "Ctrl+T"),
    ("close", "Ctrl+W"),
    ("reopen", "Ctrl+Shift+T"),
    ("palette", "Ctrl+K"),
    ("address", "Ctrl+L"),
    ("newwindow", "Ctrl+N"),
    ("history", "Ctrl+H"),
    ("next", "Ctrl+Tab"),
    ("prev", "Ctrl+Shift+Tab"),
    ("sidebar", "Ctrl+Shift+S"),
    ("tablayout", "Ctrl+Shift+L"),
    ("reader", "Ctrl+Shift+R"),
    ("pin", "Ctrl+Shift+P"),
    ("duplicate", "Ctrl+Shift+D"),
    ("mute", "Ctrl+M"),
    ("settings", "Ctrl+,"),
];

/// Every command's current combo ("" when turned off).
pub(crate) fn effective(settings: &Settings) -> Vec<(&'static str, String)> {
    DEFAULTS
        .iter()
        .map(|(id, def)| {
            (
                *id,
                settings
                    .shortcuts
                    .get(*id)
                    .cloned()
                    .unwrap_or_else(|| def.to_string()),
            )
        })
        .collect()
}

/// combo → command, for the page and sidebar key handlers.
pub(crate) fn map_json(settings: &Settings) -> String {
    let map: serde_json::Map<String, serde_json::Value> = effective(settings)
        .into_iter()
        .filter(|(_, c)| !c.is_empty())
        .map(|(id, c)| (c, serde_json::Value::String(id.to_string())))
        .collect();
    serde_json::Value::Object(map).to_string()
}

/// A combo a page can't swallow: it needs Ctrl or Alt, or is a function key.
pub(crate) fn valid(combo: &str) -> bool {
    let parts: Vec<&str> = combo.split('+').collect();
    let Some(key) = parts.last().filter(|k| !k.is_empty()) else {
        return false;
    };
    let fkey = key.len() > 1
        && key.starts_with('F')
        && key[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n));
    let modded = parts.contains(&"Ctrl") || parts.contains(&"Alt");
    let reserved = matches!(
        combo,
        "Ctrl+C" | "Ctrl+V" | "Ctrl+X" | "Ctrl+A" | "Ctrl+Z" | "Ctrl+Y" | "Alt+F4" | "F6"
    );
    (modded || fkey) && !reserved
}

#[derive(Serialize)]
pub(crate) struct Shortcut {
    id: &'static str,
    combo: String,
    default: &'static str,
}

#[tauri::command]
pub(crate) async fn shortcuts(app: AppHandle) -> Vec<Shortcut> {
    with_data(&app, |d| {
        effective(&d.settings)
            .into_iter()
            .map(|(id, combo)| Shortcut {
                id,
                combo,
                default: DEFAULTS
                    .iter()
                    .find(|(i, _)| *i == id)
                    .map(|(_, c)| *c)
                    .unwrap_or(""),
            })
            .collect()
    })
}

/// Sets a command's combo ("" turns it off). A combo already used by another
/// command moves here; the other one is left without a shortcut.
#[tauri::command]
pub(crate) async fn set_shortcut(
    app: AppHandle,
    id: String,
    combo: String,
) -> Result<String, String> {
    let Some((_, default)) = DEFAULTS.iter().find(|(i, _)| *i == id) else {
        return Err("Comando desconhecido.".into());
    };
    if !combo.is_empty() && !valid(&combo) {
        return Err("Use Ctrl ou Alt com outra tecla, ou uma tecla F (F1 a F24). Algumas, como Ctrl+C e F6, são do sistema.".into());
    }
    let taken = with_data(&app, |d| {
        let taken = effective(&d.settings)
            .into_iter()
            .find(|(other, c)| *other != id && !combo.is_empty() && *c == combo)
            .map(|(other, _)| other);
        if let Some(other) = taken {
            d.settings
                .shortcuts
                .insert(other.to_string(), String::new());
        }
        if combo == *default {
            d.settings.shortcuts.remove(&id);
        } else {
            d.settings.shortcuts.insert(id.clone(), combo.clone());
        }
        taken
    });
    commit(&app);
    push_to_pages(&app);
    Ok(taken.unwrap_or_default().to_string())
}

#[tauri::command]
pub(crate) async fn reset_shortcuts(app: AppHandle) {
    with_data(&app, |d| d.settings.shortcuts.clear());
    commit(&app);
    push_to_pages(&app);
}

/// Open pages learn the new keys without reloading.
pub(crate) fn push_to_pages(app: &AppHandle) {
    let json = with_data(app, |d| map_json(&d.settings));
    for wv in app.webviews().into_values() {
        let l = wv.label();
        if l.starts_with("tab-") || l.starts_with("peek") || l == "quick-page" {
            let _ = wv.eval(format!("window.__blazonKeys = {json};"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_combos_a_page_cannot_type_are_allowed() {
        assert!(valid("Ctrl+T"));
        assert!(valid("Ctrl+Shift+Tab"));
        assert!(valid("Alt+P"));
        assert!(valid("F2"));
        assert!(!valid("T"));
        assert!(!valid("Shift+T"));
        assert!(!valid("Ctrl+C"));
        assert!(!valid("F6"));
        assert!(!valid("Ctrl+"));
    }

    #[test]
    fn overrides_replace_defaults() {
        let mut s = Settings::default();
        s.shortcuts.insert("palette".into(), "Ctrl+P".into());
        s.shortcuts.insert("history".into(), String::new());
        let map: serde_json::Value = serde_json::from_str(&map_json(&s)).unwrap();
        assert_eq!(map["Ctrl+P"], "palette");
        assert!(map.get("Ctrl+K").is_none());
        assert!(map.get("Ctrl+H").is_none());
        assert_eq!(map["Ctrl+T"], "newtab");
    }
}
