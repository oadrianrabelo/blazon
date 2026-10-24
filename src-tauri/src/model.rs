//! Data model: Spaces, tabs, folders, settings and their repair rules.

use crate::*;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Space {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) color: String,
    /// InPrivate: no history, no cookies on disk, tabs dropped on exit.
    #[serde(default)]
    pub(crate) private: bool,
    /// Crest Studio house (look template); empty means a custom color.
    #[serde(default)]
    pub(crate) house: String,
    /// Download folder for this Space; empty means the Windows default.
    #[serde(default)]
    pub(crate) downloads: String,
    /// Crest Studio arms and sidebar look, owned by the UI; null follows `house`.
    #[serde(default)]
    pub(crate) look: serde_json::Value,
    /// Needs Windows Hello before its tabs are shown.
    #[serde(default)]
    pub(crate) locked: bool,
    /// Search engine for this Space; empty uses the global one.
    #[serde(default)]
    pub(crate) search: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Tab {
    pub(crate) id: u64,
    pub(crate) space: String,
    /// Empty means the new tab page.
    pub(crate) url: String,
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) pinned: bool,
    #[serde(default)]
    pub(crate) folder: Option<u64>,
    #[serde(default, skip_deserializing)]
    pub(crate) loading: bool,
    /// Playing sound right now, and muted by the user.
    #[serde(default, skip_deserializing)]
    pub(crate) audio: bool,
    #[serde(default, skip_deserializing)]
    pub(crate) muted: bool,
    /// Whether the tab has a live webview; sleeping tabs reload on activation.
    #[serde(default, skip_deserializing)]
    pub(crate) awake: bool,
    #[serde(default)]
    pub(crate) last_active: u64,
    /// Recent addresses of this tab, so Back works after the app restarts
    /// (WebView2 can't restore a tab's own history).
    #[serde(default)]
    pub(crate) trail: Vec<String>,
    #[serde(skip)]
    pub(crate) trail_nav: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Folder {
    pub(crate) id: u64,
    pub(crate) space: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) open: bool,
    /// Custom folder color (hex) and icon (an emoji); empty uses the defaults.
    #[serde(default)]
    pub(crate) color: String,
    #[serde(default)]
    pub(crate) icon: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Visit {
    pub(crate) space: String,
    pub(crate) url: String,
    pub(crate) title: String,
    pub(crate) time: u64,
    #[serde(default = "one")]
    pub(crate) count: u32,
}

pub(crate) fn one() -> u32 {
    1
}

#[derive(Clone, Serialize)]
pub(crate) struct Suggestion {
    pub(crate) kind: &'static str,
    pub(crate) url: String,
    pub(crate) title: String,
    pub(crate) tab: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub(crate) sidebar_right: bool,
    pub(crate) search: String,
    /// Minutes before a background tab's webview is closed; 0 disables.
    pub(crate) sleep_minutes: u64,
    /// Use the standard Windows title bar instead of Crest's borderless frame.
    pub(crate) native_frame: bool,
    /// Sidebar hides to a thin edge and slides back when the pointer reaches it.
    pub(crate) floating_sidebar: bool,
    pub(crate) sidebar_width: f64,
    /// Page zoom for new tab webviews (1.0 = 100%).
    pub(crate) default_zoom: f64,
    /// Look & feel options owned by the UI (tab shape, pins, folders, colors).
    pub(crate) appearance: serde_json::Value,
    /// Modifier that opens a clicked link in Peek: shift, alt or off.
    pub(crate) peek_modifier: String,
    /// Space for links opened from other apps; empty means the current one.
    pub(crate) external_space: String,
    /// Links from other apps whose address contains `pattern` open in `space`.
    pub(crate) routes: Vec<Route>,
    /// Block ads and trackers, except on these sites.
    pub(crate) adblock: bool,
    pub(crate) adblock_allow: Vec<String>,
    /// Offer to save passwords and autofill forms (WebView2's own store).
    pub(crate) passwords: bool,
    /// Hours before an idle tab (not pinned, not in a folder) is archived; 0 disables.
    pub(crate) archive_hours: u64,
    /// Interface language: auto, pt or en.
    pub(crate) language: String,
    /// Look for a new version on GitHub once a day.
    pub(crate) auto_update: bool,
    /// Folder kept up to date with a backup file (e.g. inside OneDrive); empty disables.
    pub(crate) backup_folder: String,
    /// The welcome flow ran. Defaults to true so existing installs skip it;
    /// a fresh install sets it to false.
    pub(crate) welcomed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Route {
    pub(crate) pattern: String,
    pub(crate) space: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sidebar_right: false,
            search: "https://duckduckgo.com/?q=".into(),
            sleep_minutes: 30,
            native_frame: false,
            floating_sidebar: false,
            sidebar_width: SIDEBAR_WIDTH,
            default_zoom: 1.0,
            appearance: serde_json::json!({}),
            peek_modifier: "shift".into(),
            external_space: String::new(),
            routes: vec![],
            adblock: true,
            adblock_allow: vec![],
            passwords: true,
            archive_hours: 0,
            language: "auto".into(),
            auto_update: true,
            backup_folder: String::new(),
            welcomed: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct Data {
    pub(crate) spaces: Vec<Space>,
    pub(crate) tabs: Vec<Tab>,
    pub(crate) folders: Vec<Folder>,
    /// Split View pairs, shown side by side (left, right) when either is active.
    pub(crate) splits: Vec<[u64; 2]>,
    pub(crate) active_space: String,
    pub(crate) active_tab: HashMap<String, u64>,
    pub(crate) next_id: u64,
    pub(crate) settings: Settings,
    /// Closed and auto-archived tabs, newest last.
    pub(crate) archive: Vec<Archived>,
    /// Spaces shown in extra windows, reopened at startup.
    pub(crate) windows: Vec<String>,
    /// When the last update check ran (Unix seconds).
    pub(crate) last_update_check: u64,
    /// When the last automatic backup was written (Unix seconds).
    pub(crate) last_backup: u64,
    /// Locked Spaces opened with Windows Hello this session.
    #[serde(skip_deserializing)]
    pub(crate) unlocked: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Archived {
    pub(crate) space: String,
    pub(crate) url: String,
    pub(crate) title: String,
    pub(crate) time: u64,
}

pub(crate) const ARCHIVE_LIMIT: usize = 300;

impl Data {
    pub(crate) fn archive_tab(&mut self, tab: &Tab) {
        if tab.url.is_empty() || self.is_private(&tab.space) {
            return;
        }
        self.archive
            .retain(|a| !(a.space == tab.space && a.url == tab.url));
        self.archive.push(Archived {
            space: tab.space.clone(),
            url: tab.url.clone(),
            title: tab.title.clone(),
            time: now(),
        });
        if self.archive.len() > ARCHIVE_LIMIT {
            let extra = self.archive.len() - ARCHIVE_LIMIT;
            self.archive.drain(..extra);
        }
    }

    /// Search engine prefix for a Space.
    pub(crate) fn search_for(&self, space: &str) -> &str {
        self.spaces
            .iter()
            .find(|s| s.id == space && !s.search.is_empty())
            .map(|s| s.search.as_str())
            .unwrap_or(&self.settings.search)
    }

    pub(crate) fn is_locked(&self, space: &str) -> bool {
        self.spaces.iter().any(|s| s.id == space && s.locked)
            && !self.unlocked.iter().any(|u| u == space)
    }
}

impl Data {
    pub(crate) fn new_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    pub(crate) fn is_private(&self, space: &str) -> bool {
        self.spaces.iter().any(|s| s.id == space && s.private)
    }

    pub(crate) fn active_tab_id(&self) -> Option<u64> {
        self.active_tab.get(&self.active_space).copied()
    }

    /// Tabs on screen: the active tab, or its Split View pair.
    /// Tabs on screen in the window showing `space`.
    pub(crate) fn visible_in(&self, space: &str) -> Vec<u64> {
        let Some(id) = self.active_tab.get(space).copied() else {
            return vec![];
        };
        match self.splits.iter().find(|p| p.contains(&id)) {
            Some(pair) => pair.to_vec(),
            None => vec![id],
        }
    }

    pub(crate) fn unsplit(&mut self, id: u64) {
        self.splits.retain(|p| !p.contains(&id));
    }

    pub(crate) fn space_tabs(&self, space: &str) -> Vec<u64> {
        let mut tabs: Vec<&Tab> = self.tabs.iter().filter(|t| t.space == space).collect();
        tabs.sort_by_key(|t| (!t.pinned, t.folder.is_none()));
        tabs.iter().map(|t| t.id).collect()
    }

    pub(crate) fn repair(&mut self) {
        if self.spaces.is_empty() {
            self.spaces.push(Space {
                id: "personal".into(),
                name: "Pessoal".into(),
                color: "#86adc4".into(),
                private: false,
                house: "winter".into(),
                downloads: String::new(),
                look: serde_json::Value::Null,
                locked: false,
                search: String::new(),
            });
        }
        let spaces: Vec<String> = self
            .spaces
            .iter()
            .filter(|s| !s.private)
            .map(|s| s.id.clone())
            .collect();
        self.tabs.retain(|t| spaces.contains(&t.space));
        let spaces: Vec<String> = self.spaces.iter().map(|s| s.id.clone()).collect();
        self.folders.retain(|f| spaces.contains(&f.space));
        let folders: Vec<u64> = self.folders.iter().map(|f| f.id).collect();
        for t in self.tabs.iter_mut() {
            if t.folder.is_some_and(|f| !folders.contains(&f)) {
                t.folder = None;
            }
        }
        if !spaces.contains(&self.active_space) {
            self.active_space = spaces[0].clone();
        }
        let tabs = &self.tabs;
        let space_of = |id: u64| tabs.iter().find(|t| t.id == id).map(|t| t.space.clone());
        self.splits
            .retain(|[a, b]| a != b && space_of(*a).is_some() && space_of(*a) == space_of(*b));
        self.active_tab
            .retain(|s, id| tabs.iter().any(|t| t.id == *id && &t.space == s));
        let max_tab = self.tabs.iter().map(|t| t.id).max().unwrap_or(0);
        let max_folder = self.folders.iter().map(|f| f.id).max().unwrap_or(0);
        self.next_id = max_tab.max(max_folder).max(self.next_id);
    }
}

pub(crate) const HISTORY_LIMIT: usize = 5000;
pub(crate) const CLOSED_LIMIT: usize = 25;
