const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();

// Crest Studio houses: sidebar palette (field, middle, lower) and banner pattern,
// from Crest's SpaceHouse/Tincture definitions.
const HOUSES = {
  winter: { title: 'Winter', colors: ['#1e2833', '#3e4e5e', '#86adc4'], pattern: 'diagonal' },
  lion: { title: 'Lion', colors: ['#3c0e1a', '#722030', '#c9a254'], pattern: 'chevron' },
  storm: { title: 'Storm', colors: ['#151824', '#313848', '#b08f4a'], pattern: 'quartered' },
  dragon: { title: 'Dragon', colors: ['#1a100d', '#7a1e12', '#be4438'], pattern: 'bands' },
  meadow: { title: 'Meadow', colors: ['#152318', '#345738', '#bca766'], pattern: 'lozenges' },
  iron: { title: 'Iron', colors: ['#0e1a1c', '#2c3a3c', '#9c9781'], pattern: 'stripes' },
  river: { title: 'River', colors: ['#0f1e2e', '#22486c', '#a85c41'], pattern: 'split' },
  sun: { title: 'Sun', colors: ['#35160b', '#8b3d1b', '#d09e65'], pattern: 'gradient' },
  vigil: { title: 'Vigil', colors: ['#101112', '#272a2e', '#7b838c'], pattern: 'checkered' },
};
const TILED = ['stripes', 'checkered', 'lozenges', 'gradient'];

const ICONS = {
  back: '<path d="m15 18-6-6 6-6"/>',
  forward: '<path d="m9 18 6-6-6-6"/>',
  reload: '<path d="M20 12a8 8 0 1 1-2.3-5.7L20 8.5"/><path d="M20 3.5v5h-5"/>',
  x: '<path d="M18 6 6 18M6 6l12 12"/>',
  min: '<path d="M5 12h14"/>',
  max: '<rect x="5" y="5" width="14" height="14" rx="1.5"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  more: '<circle cx="5" cy="12" r=".6"/><circle cx="12" cy="12" r=".6"/><circle cx="19" cy="12" r=".6"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  gear: '<circle cx="12" cy="12" r="3"/><path d="M12 2.5v3M12 18.5v3M4.6 4.6l2.1 2.1M17.3 17.3l2.1 2.1M2.5 12h3M18.5 12h3M4.6 19.4l2.1-2.1M17.3 6.7l2.1-2.1"/>',
  archive: '<rect x="3" y="4" width="18" height="4" rx="1"/><path d="M5 8v11a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V8M10 12h4"/>',
};
const SHIELD = 'M15.68 17.24L84.32 17.24L84.32 46.88Q84.32 78.08 50 87.44Q15.68 78.08 15.68 46.88Z';
const COLORS = ['#3b82f6', '#16a34a', '#e11d48', '#f59e0b', '#8b5cf6', '#0891b2', '#db2777', '#65a30d'];

const $ = (id) => document.getElementById(id);
let state = null;
let dragId = null;

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

function icon(name) {
  const t = document.createElement('template');
  t.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${ICONS[name]}</svg>`;
  return t.content.firstChild;
}
document.querySelectorAll('[data-i]').forEach((n) => n.append(icon(n.dataset.i)));

function setIcon(node, name) {
  node.replaceChildren(icon(name));
}

// Colors

function hexRgb(h) {
  const n = parseInt(h.slice(1), 16);
  return [n >> 16, (n >> 8) & 255, n & 255];
}
function mix(a, b, t) {
  const x = hexRgb(a);
  const y = hexRgb(b);
  return '#' + x.map((v, i) => Math.round(v * t + y[i] * (1 - t)).toString(16).padStart(2, '0')).join('');
}

function look(space) {
  const h = HOUSES[space.house];
  if (h) return h;
  const c = /^#[0-9a-f]{6}$/i.test(space.color) ? space.color : '#3b82f6';
  return { colors: [mix(c, '#0b0d10', 0.22), mix(c, '#0b0d10', 0.5), c], pattern: 'diagonal' };
}

function emblem(space) {
  if (HOUSES[space.house]) {
    const img = new Image();
    img.src = `crests/${space.house}.svg`;
    img.alt = '';
    return img;
  }
  const t = document.createElement('template');
  const c = look(space).colors;
  t.innerHTML = `<svg viewBox="10 12 80 80"><path d="${SHIELD}" fill="${c[2]}" stroke="${c[0]}" stroke-width="3"/></svg>`;
  return t.content.firstChild;
}

function applyLook(space) {
  const { colors, pattern } = look(space);
  const root = document.documentElement.style;
  colors.forEach((c, i) => root.setProperty(`--c${i}`, c));
  const banner = $('banner');
  banner.className = `banner ${pattern}` + (TILED.includes(pattern) ? ' tiles' : '');
  if (!banner.children.length) banner.append(el('div', 'p-mid'), el('div', 'p-low'));
}

// Rendering

function host(url) {
  try { return new URL(url).hostname; } catch { return ''; }
}

function tabTitle(t) {
  return t.title || host(t.url) || 'Nova aba';
}

function favicon(t) {
  const box = el('span', 'fav', (tabTitle(t)[0] || '•').toUpperCase());
  const h = host(t.url);
  if (h) {
    const img = new Image();
    img.onload = () => { box.textContent = ''; box.style.background = 'none'; box.append(img); };
    img.src = `https://icons.duckduckgo.com/ip3/${encodeURIComponent(h)}.ico`;
  }
  return box;
}

const activeSpace = () => state.spaces.find((s) => s.id === state.active_space);
const activeTab = () => state.tabs.find((t) => t.id === state.active_tab[state.active_space]);

function dragTarget(node, t, pinned) {
  node.draggable = true;
  node.addEventListener('dragstart', () => { dragId = t ? t.id : null; });
  node.addEventListener('dragover', (e) => { e.preventDefault(); node.classList.add('drop-before'); });
  node.addEventListener('dragleave', () => node.classList.remove('drop-before'));
  node.addEventListener('drop', (e) => {
    e.preventDefault();
    e.stopPropagation();
    node.classList.remove('drop-before');
    if (dragId != null && dragId !== t?.id) invoke('move_tab', { id: dragId, before: t ? t.id : null, pinned });
    dragId = null;
  });
}

function tabEvents(node, t) {
  node.onclick = () => invoke('activate_tab', { id: t.id });
  node.onauxclick = (e) => { if (e.button === 1) invoke('close_tab', { id: t.id }); };
  node.oncontextmenu = (e) => { e.preventDefault(); invoke('toggle_pin', { id: t.id }); };
}

function render() {
  const space = activeSpace();
  const current = activeTab();
  applyLook(space);
  document.body.classList.toggle('right', state.settings.sidebar_right);
  document.body.classList.toggle('native', state.settings.native_frame);
  document.title = current ? `${tabTitle(current)} — Crest` : 'Crest';

  const name = $('space-name');
  const label = el('span', 'label', space.name);
  if (space.private) label.append(el('span', 'tag', ' · privado'));
  name.replaceChildren(emblem(space), label);

  const address = $('address');
  if (document.activeElement !== address) address.value = current?.url || '';
  const loading = !!current?.loading;
  setIcon($('reload'), loading ? 'x' : 'reload');
  $('reload').dataset.nav = loading ? 'stop' : 'reload';
  $('reload').title = loading ? 'Parar' : 'Recarregar (F5)';

  const tabs = state.tabs.filter((t) => t.space === state.active_space);
  const pinned = $('pinned');
  const list = $('tabs');
  pinned.replaceChildren();
  list.replaceChildren();

  for (const t of tabs) {
    const active = current && t.id === current.id;
    if (t.pinned) {
      const b = el('button', 'pin' + (active ? ' active' : '') + (t.awake ? '' : ' asleep'));
      b.title = `${tabTitle(t)}\nClique direito para desafixar`;
      b.append(favicon(t));
      tabEvents(b, t);
      dragTarget(b, t, true);
      pinned.append(b);
      continue;
    }
    const li = el('li', 'tab' + (active ? ' active' : '') + (t.loading ? ' loading' : '') + (t.awake ? '' : ' asleep'));
    li.title = `${t.url || 'Nova aba'}\nClique direito para fixar`;
    const close = el('button', 'icon close');
    close.append(icon('x'));
    close.title = 'Fechar (Ctrl+W)';
    close.onclick = (e) => { e.stopPropagation(); invoke('close_tab', { id: t.id }); };
    li.append(favicon(t), el('span', 'title', tabTitle(t)), close);
    tabEvents(li, t);
    dragTarget(li, t, false);
    list.append(li);
  }

  const spaces = $('spaces');
  spaces.replaceChildren();
  state.spaces.forEach((s, i) => {
    const b = el('button', 'dot' + (s.id === state.active_space ? ' active' : ''));
    b.title = `${s.name} (Ctrl+Shift+${i + 1})`;
    b.append(emblem(s));
    b.onclick = () => invoke('switch_space', { id: s.id });
    b.ondblclick = () => openSpaceDialog(s);
    spaces.append(b);
  });
}

listen('state', (e) => { state = e.payload; render(); });
listen('focus-address', () => { $('address').focus(); $('address').select(); });

// Window controls (borderless frame)
$('win-min').onclick = () => appWindow.minimize();
$('win-max').onclick = () => appWindow.toggleMaximize();
$('win-close').onclick = () => appWindow.close();

// Address bar with suggestions from open tabs and history.
let suggestions = [];
let sel = -1;
let suggestSeq = 0;

function showSuggestions() {
  const ul = $('suggest');
  ul.replaceChildren();
  suggestions.forEach((s, i) => {
    const li = el('li', i === sel ? 'sel' : '');
    const text = el('span', 'text');
    text.append(el('span', 'title', s.title || host(s.url)), el('span', 'url', s.url));
    li.append(favicon(s), text);
    if (s.kind === 'tab') li.append(el('span', 'kind', 'aba aberta'));
    li.onmousedown = (e) => { e.preventDefault(); pick(s); };
    ul.append(li);
  });
  ul.hidden = suggestions.length === 0;
}

function hideSuggestions() {
  suggestions = [];
  sel = -1;
  suggestSeq++;
  showSuggestions();
}

function pick(s) {
  hideSuggestions();
  $('address').blur();
  if (s.tab != null) invoke('activate_tab', { id: s.tab });
  else invoke('navigate', { input: s.url });
}

$('address').addEventListener('input', async () => {
  const seq = ++suggestSeq;
  const res = await invoke('suggest', { query: $('address').value });
  if (seq !== suggestSeq) return;
  suggestions = res;
  sel = -1;
  showSuggestions();
});
$('address-form').addEventListener('submit', (e) => {
  e.preventDefault();
  if (sel >= 0 && suggestions[sel]) return pick(suggestions[sel]);
  const v = $('address').value.trim();
  hideSuggestions();
  if (v) invoke('navigate', { input: v });
  $('address').blur();
});
$('address').addEventListener('focus', () => $('address').select());
$('address').addEventListener('blur', hideSuggestions);
$('address').addEventListener('keydown', (e) => {
  if (e.key === 'Escape') { hideSuggestions(); $('address').value = activeTab()?.url || ''; $('address').blur(); }
  else if ((e.key === 'ArrowDown' || e.key === 'ArrowUp') && suggestions.length) {
    e.preventDefault();
    const n = suggestions.length;
    sel = e.key === 'ArrowDown' ? (sel + 1) % n : (sel <= 0 ? n - 1 : sel - 1);
    $('address').value = suggestions[sel].url;
    showSuggestions();
  }
});

document.querySelectorAll('[data-nav]').forEach((b) => {
  b.onclick = () => invoke('nav', { action: b.dataset.nav });
});
$('new-tab').onclick = () => invoke('new_tab', {});
dragTarget($('new-tab'), null, false);
$('new-tab').draggable = false;
$('pinned').addEventListener('dragover', (e) => e.preventDefault());
$('pinned').addEventListener('drop', (e) => {
  if (e.target === $('pinned') && dragId != null) invoke('move_tab', { id: dragId, before: null, pinned: true });
});

// History dialog
const DAY = new Intl.DateTimeFormat('pt-BR', { weekday: 'long', day: 'numeric', month: 'long' });
let historySeq = 0;
async function loadHistory() {
  const seq = ++historySeq;
  const items = await invoke('history_list', { query: $('history-q').value });
  if (seq !== historySeq) return;
  const ul = $('history-list');
  ul.replaceChildren();
  let lastDay = '';
  for (const v of items) {
    const day = DAY.format(new Date(v.time * 1000));
    if (day !== lastDay) { ul.append(el('li', 'day', day)); lastDay = day; }
    const li = el('li');
    li.title = v.url;
    li.append(el('span', 'title', v.title || host(v.url)), el('span', 'url', host(v.url)));
    li.onclick = () => { $('history').close(); invoke('new_tab', { url: v.url }); };
    ul.append(li);
  }
  if (!items.length) ul.append(el('li', 'day', 'Nada por aqui.'));
}
function openHistory() {
  if ($('history').open) return;
  $('history-q').value = '';
  loadHistory();
  $('history').showModal();
  $('history-q').focus();
}
$('history-btn').onclick = openHistory;
$('history-q').addEventListener('input', loadHistory);
$('history-clear').onclick = async () => {
  if (!confirm(`Limpar todo o histórico do Space "${activeSpace().name}"?`)) return;
  await invoke('clear_history');
  loadHistory();
};
listen('open-history', openHistory);

// Space dialog: create or edit a Space, Crest Studio style.
let editing = null;
let pickedHouse = 'winter';

function renderHouses() {
  const grid = $('sd-houses');
  grid.replaceChildren();
  const custom = { house: '', color: $('sd-color').value };
  for (const key of [...Object.keys(HOUSES), '']) {
    const b = el('button', 'house' + (key === pickedHouse ? ' sel' : ''));
    b.type = 'button';
    b.title = key ? HOUSES[key].title : 'Cor própria';
    b.append(emblem(key ? { house: key } : custom));
    b.onclick = () => { pickedHouse = key; renderHouses(); };
    grid.append(b);
  }
  $('sd-custom-row').hidden = pickedHouse !== '';
}

function openSpaceDialog(space) {
  editing = space || null;
  $('sd-title').textContent = space ? 'Editar Space' : 'Novo Space';
  $('sd-name').value = space ? space.name : '';
  pickedHouse = space ? space.house : Object.keys(HOUSES)[state.spaces.length % 9];
  $('sd-color').value = space && !space.house ? space.color : COLORS[state.spaces.length % COLORS.length];
  $('sd-private').checked = false;
  $('sd-private-row').hidden = !!space;
  $('sd-delete').hidden = !space || state.spaces.length <= 1;
  renderHouses();
  $('space-dialog').showModal();
  $('sd-name').focus();
}
$('sd-color').addEventListener('input', renderHouses);
$('sd-cancel').onclick = () => $('space-dialog').close('cancel');
$('space-dialog').addEventListener('close', () => {
  const name = $('sd-name').value.trim();
  if ($('space-dialog').returnValue !== 'save' || !name) return;
  const house = pickedHouse;
  const color = house ? HOUSES[house].colors[2] : $('sd-color').value;
  if (editing) invoke('update_space', { id: editing.id, name, color, house });
  else invoke('add_space', { name, color, house, private: $('sd-private').checked });
});
$('sd-delete').onclick = () => {
  const s = editing;
  if (!confirm(`Excluir o Space "${s.name}"? As abas, cookies e logins dele serão apagados.`)) return;
  $('space-dialog').close('cancel');
  invoke('delete_space', { id: s.id });
};
$('add-space').onclick = () => openSpaceDialog(null);
$('space-name').onclick = () => openSpaceDialog(activeSpace());
$('space-menu').onclick = () => openSpaceDialog(activeSpace());

// Settings dialog
$('settings-btn').onclick = () => {
  $('set-right').checked = state.settings.sidebar_right;
  $('set-frame').checked = state.settings.native_frame;
  $('set-search').value = state.settings.search;
  $('set-sleep').value = String(state.settings.sleep_minutes);
  $('settings').showModal();
};
$('settings').addEventListener('close', () => {
  const settings = {
    sidebar_right: $('set-right').checked,
    native_frame: $('set-frame').checked,
    search: $('set-search').value,
    sleep_minutes: Number($('set-sleep').value),
  };
  if (JSON.stringify(settings) !== JSON.stringify(state.settings)) invoke('set_settings', { settings });
});

// Shortcuts while the sidebar itself has focus.
document.addEventListener('keydown', (e) => {
  if (!e.ctrlKey || e.altKey || !state) return;
  const k = e.key.toLowerCase();
  const tabs = state.tabs.filter((t) => t.space === state.active_space).sort((a, b) => b.pinned - a.pinned);
  const digit = /^Digit[1-9]$/.test(e.code) ? e.code.slice(5) : null;
  if (e.shiftKey && digit) {
    const s = state.spaces[Number(digit) - 1];
    if (s) invoke('switch_space', { id: s.id });
  } else if (k === 't' && e.shiftKey) invoke('reopen_tab');
  else if (k === 't') invoke('new_tab', {});
  else if (k === 'h') openHistory();
  else if (k === 'w' && activeTab()) invoke('close_tab', { id: activeTab().id });
  else if (k === 'l') { $('address').focus(); $('address').select(); }
  else if (k === 'tab' && tabs.length) {
    const i = tabs.findIndex((t) => t.id === activeTab()?.id);
    const n = tabs[(i + (e.shiftKey ? tabs.length - 1 : 1)) % tabs.length];
    invoke('activate_tab', { id: n.id });
  } else if (digit && tabs.length) {
    const t = digit === '9' ? tabs[tabs.length - 1] : tabs[Number(digit) - 1];
    if (t) invoke('activate_tab', { id: t.id });
  } else return;
  e.preventDefault();
});

invoke('get_state').then((s) => { state = s; render(); });
