const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();

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
  folder: '<path d="M3 6.5A1.5 1.5 0 0 1 4.5 5H9l2 2.5h8.5A1.5 1.5 0 0 1 21 9v8.5a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 17.5Z"/>',
  chev: '<path d="m9 6 6 6-6 6"/>',
  pin: '<path d="M12 17v5M9 3h6l-1 6 3 4H7l3-4Z"/>',
  copy: '<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V5a1 1 0 0 0-1-1H5a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h3"/>',
  edit: '<path d="M4 20h4L19 9l-4-4L4 16Z"/>',
  split: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M12 4v16"/>',
  archive: '<rect x="3" y="4" width="18" height="4" rx="1"/><path d="M5 8v11a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V8M10 12h4"/>',
};
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
  banner.style.backgroundImage = tileImage(pattern, colors);
  if (!banner.children.length) banner.append(el('div', 'p-mid'), el('div', 'p-low'));
}

// Look & feel options (settings.appearance)
function applyAppearance(settings) {
  const a = lookOf(settings);
  const root = document.documentElement.style;
  root.setProperty('--tab-scale', a.tabScale);
  root.setProperty('--radius', { square: '3px', rounded: '9px', pill: '999px' }[a.tabShape] || '9px');
  root.setProperty('--sidebar-w', `${Math.min(420, Math.max(200, settings.sidebar_width || 264))}px`);
  if (a.accent) root.setProperty('--c2', a.accent);
  const b = document.body.classList;
  for (const k of ['sel-outline', 'sel-glow']) b.remove(k);
  if (a.selected !== 'fill') b.add(`sel-${a.selected}`);
  b.toggle('follow-accent', a.followAccent);
  b.toggle('no-dim', !a.dimUnloaded);
  b.toggle('no-pin-fill', !a.pinFill);
  b.toggle('no-stretch', !a.stretchPins);
  b.toggle('folder-plain', !a.folderAccent);
  b.toggle('no-address-outline', !a.addressOutline);
  b.toggle('no-banner', !a.banner);
  b.toggle('floating', !!settings.floating_sidebar);
  if (!settings.floating_sidebar) b.remove('collapsed');
  else if (!b.contains('collapsed') && !$('sidebar').matches(':hover') && document.activeElement !== $('address')) b.add('collapsed');
  const pins = state.tabs.filter((t) => t.space === state.active_space && t.pinned && t.folder == null).length;
  let cols = a.pinsPerRow;
  if (!cols) cols = Math.ceil(pins / Math.ceil(pins / 4 || 1)) || 4;
  root.setProperty('--pin-cols', a.stretchPins ? `repeat(${cols}, 1fr)` : `repeat(${cols}, 52px)`);
  window.folderCounts = a.folderCounts;
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
    if (dragId != null && dragId !== t?.id) invoke('move_tab', { id: dragId, before: t ? t.id : null, pinned, folder: t ? t.folder ?? null : null });
    dragId = null;
  });
}

function tabEvents(node, t) {
  node.onclick = () => invoke('activate_tab', { id: t.id });
  node.onauxclick = (e) => { if (e.button === 1) invoke('close_tab', { id: t.id }); };
  node.oncontextmenu = (e) => { e.preventDefault(); e.stopPropagation(); tabMenu(e, t); };
}

// Context menu

function showMenu(e, items) {
  const m = $('ctx');
  m.replaceChildren();
  for (const it of items) {
    if (it === '-') { m.append(el('hr')); continue; }
    if (it.sub) { m.append(el('div', 'sub', it.sub)); continue; }
    const b = el('button');
    if (it.icon) b.append(icon(it.icon));
    b.append(document.createTextNode(it.label));
    b.onclick = () => { hideMenu(); it.run(); };
    m.append(b);
  }
  m.hidden = false;
  // Stay inside the sidebar: the page card covers the rest of the window.
  const bar = $('sidebar').getBoundingClientRect();
  const x = Math.max(bar.left + 6, Math.min(e.clientX, bar.right - m.offsetWidth - 6));
  const y = Math.min(e.clientY, innerHeight - m.offsetHeight - 6);
  m.style.left = `${x}px`;
  m.style.top = `${Math.max(6, y)}px`;
}
function hideMenu() { $('ctx').hidden = true; }
addEventListener('mousedown', (e) => { if (!$('ctx').contains(e.target)) hideMenu(); });
addEventListener('blur', hideMenu);
addEventListener('keydown', (e) => { if (e.key === 'Escape') hideMenu(); });

function askFolderName(initial) {
  const name = prompt('Nome da pasta:', initial || 'Nova pasta');
  return name && name.trim();
}

function tabMenu(e, t) {
  const folders = state.folders.filter((f) => f.space === state.active_space);
  const items = [
    { label: t.pinned ? 'Desafixar' : 'Fixar', icon: 'pin', run: () => invoke('toggle_pin', { id: t.id }) },
    { label: 'Nova pasta com esta aba', icon: 'folder', run: () => { const name = askFolderName(); if (name) invoke('add_folder', { name, tab: t.id }); } },
  ];
  if (folders.length || t.folder != null) items.push('-', { sub: 'Mover para' });
  for (const f of folders) {
    if (f.id !== t.folder) items.push({ label: f.name, icon: 'folder', run: () => invoke('move_tab', { id: t.id, before: null, pinned: false, folder: f.id }) });
  }
  if (t.folder != null) items.push({ label: 'Fora da pasta', icon: 'chev', run: () => invoke('move_tab', { id: t.id, before: null, pinned: false, folder: null }) });
  const pair = state.splits.find((p) => p.includes(t.id));
  items.push('-');
  if (pair) items.push({ label: 'Desfazer lado a lado', icon: 'split', run: () => invoke('unsplit_tab', { id: t.id }) });
  else {
    items.push({ label: 'Lado a lado com nova aba', icon: 'split', run: () => invoke('split_tab', { id: t.id, with: null }) });
    const others = state.tabs.filter((o) => o.space === t.space && o.id !== t.id && !state.splits.some((p) => p.includes(o.id))).slice(0, 6);
    if (others.length) items.push({ sub: 'Lado a lado com' });
    for (const o of others) items.push({ label: tabTitle(o), icon: 'split', run: () => invoke('split_tab', { id: t.id, with: o.id }) });
  }
  items.push('-',
    { label: 'Duplicar', icon: 'plus', run: () => invoke('duplicate_tab', { id: t.id }) },
    { label: 'Copiar link', icon: 'copy', run: () => navigator.clipboard.writeText(t.url) },
    { label: 'Fechar', icon: 'x', run: () => invoke('close_tab', { id: t.id }) });
  showMenu(e, items);
}

function folderMenu(e, f) {
  showMenu(e, [
    { label: 'Renomear', icon: 'edit', run: () => { const name = askFolderName(f.name); if (name) invoke('update_folder', { id: f.id, name }); } },
    { label: 'Excluir pasta (mantém as abas)', icon: 'x', run: () => invoke('delete_folder', { id: f.id }) },
  ]);
}

$('scroll').addEventListener('contextmenu', (e) => {
  e.preventDefault();
  showMenu(e, [{ label: 'Nova pasta', icon: 'folder', run: () => { const name = askFolderName(); if (name) invoke('add_folder', { name, tab: null }); } }]);
});

function render() {
  const space = activeSpace();
  const current = activeTab();
  applyLook(space);
  applyAppearance(state.settings);
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

  const tabRow = (t, extra) => {
    const active = current && t.id === current.id;
    const li = el('li', 'tab' + extra + (active ? ' active' : '') + (t.loading ? ' loading' : '') + (t.awake ? '' : ' asleep'));
    li.title = t.url || 'Nova aba';
    const close = el('button', 'icon close');
    close.append(icon('x'));
    close.title = 'Fechar (Ctrl+W)';
    close.onclick = (e) => { e.stopPropagation(); invoke('close_tab', { id: t.id }); };
    li.append(favicon(t), el('span', 'title', tabTitle(t)));
    if (state.splits.some((p) => p.includes(t.id))) {
      const mark = icon('split');
      mark.classList.add('split-mark');
      li.append(mark);
    }
    li.append(close);
    tabEvents(li, t);
    dragTarget(li, t, false);
    return li;
  };

  const folders = $('folders');
  folders.replaceChildren();
  for (const f of state.folders.filter((x) => x.space === state.active_space)) {
    const head = el('li', 'folder' + (f.open ? ' open' : ''));
    const chev = icon('chev');
    chev.classList.add('chev');
    head.append(icon('folder'), el('span', 'title', f.name));
    if (window.folderCounts) head.append(el('span', 'count', String(tabs.filter((x) => x.folder === f.id).length)));
    head.append(chev);
    head.onclick = () => invoke('update_folder', { id: f.id, open: !f.open });
    head.ondblclick = () => { const name = askFolderName(f.name); if (name) invoke('update_folder', { id: f.id, name }); };
    head.oncontextmenu = (e) => { e.preventDefault(); e.stopPropagation(); folderMenu(e, f); };
    head.addEventListener('dragover', (e) => { e.preventDefault(); head.classList.add('drop-before'); });
    head.addEventListener('dragleave', () => head.classList.remove('drop-before'));
    head.addEventListener('drop', (e) => {
      e.preventDefault();
      head.classList.remove('drop-before');
      if (dragId != null) invoke('move_tab', { id: dragId, before: null, pinned: false, folder: f.id });
      dragId = null;
    });
    folders.append(head);
    if (!f.open) continue;
    for (const t of tabs.filter((x) => x.folder === f.id)) folders.append(tabRow(t, ' in-folder'));
  }

  for (const t of tabs) {
    const active = current && t.id === current.id;
    if (t.folder != null) continue;
    if (t.pinned) {
      const b = el('button', 'pin' + (active ? ' active' : '') + (t.awake ? '' : ' asleep'));
      b.title = tabTitle(t);
      b.append(favicon(t));
      tabEvents(b, t);
      dragTarget(b, t, true);
      pinned.append(b);
      continue;
    }
    list.append(tabRow(t, ''));
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
listen('focus-address', () => { document.body.classList.remove('collapsed'); $('address').focus(); $('address').select(); });

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
  if (e.target === $('pinned') && dragId != null) invoke('move_tab', { id: dragId, before: null, pinned: true, folder: null });
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

// Settings panel (shown over the page card)
$('settings-btn').onclick = () => invoke('open_panel', { section: 'look' });

// Floating sidebar: hidden to an edge strip, revealed on hover.
let peekTimer = null;
function peek(on) {
  clearTimeout(peekTimer);
  if (!state?.settings.floating_sidebar) return;
  document.body.classList.toggle('collapsed', !on);
  invoke('sidebar_peek', { on });
}
$('edge').addEventListener('mouseenter', () => peek(true));
$('sidebar').addEventListener('mouseenter', () => clearTimeout(peekTimer));
$('sidebar').addEventListener('mouseleave', () => {
  const busy = document.activeElement === $('address') || !$('ctx').hidden || document.querySelector('dialog[open]');
  if (!busy) peekTimer = setTimeout(() => peek(false), 350);
});

// Shortcuts while the sidebar itself has focus.
document.addEventListener('keydown', (e) => {
  if (!e.ctrlKey || e.altKey || !state) return;
  const k = e.key.toLowerCase();
  const tabs = state.tabs.filter((t) => t.space === state.active_space).sort((a, b) => (b.pinned - a.pinned) || ((b.folder != null) - (a.folder != null)));
  const digit = /^Digit[1-9]$/.test(e.code) ? e.code.slice(5) : null;
  if (e.shiftKey && digit) {
    const s = state.spaces[Number(digit) - 1];
    if (s) invoke('switch_space', { id: s.id });
  } else if (k === 't' && e.shiftKey) invoke('reopen_tab');
  else if (k === 't') invoke('new_tab', {});
  else if (k === 'h') openHistory();
  else if (k === 'k') invoke('open_overlay', { mode: 'palette', url: null });
  else if (k === 's' && e.shiftKey) invoke('set_settings', { settings: { ...state.settings, floating_sidebar: !state.settings.floating_sidebar } });
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
