const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

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
    node.classList.remove('drop-before');
    if (dragId != null && dragId !== t?.id) invoke('move_tab', { id: dragId, before: t ? t.id : null, pinned });
    dragId = null;
  });
}

function render() {
  const space = activeSpace();
  const current = activeTab();
  document.documentElement.style.setProperty('--accent', space.color);
  $('space-name').textContent = space.name;
  document.title = current ? `${tabTitle(current)} — Crest` : 'Crest';

  const address = $('address');
  if (document.activeElement !== address) address.value = current?.url || '';
  $('reload').textContent = current?.loading ? '✕' : '⟳';
  $('reload').dataset.nav = current?.loading ? 'stop' : 'reload';

  const tabs = state.tabs.filter((t) => t.space === state.active_space);
  const pinned = $('pinned');
  const list = $('tabs');
  pinned.replaceChildren();
  list.replaceChildren();

  for (const t of tabs) {
    const active = current && t.id === current.id;
    if (t.pinned) {
      const b = el('button', 'pin' + (active ? ' active' : ''));
      b.title = tabTitle(t);
      b.append(favicon(t));
      b.onclick = () => invoke('activate_tab', { id: t.id });
      b.onauxclick = (e) => { if (e.button === 1) invoke('close_tab', { id: t.id }); };
      b.oncontextmenu = (e) => { e.preventDefault(); invoke('toggle_pin', { id: t.id }); };
      dragTarget(b, t, true);
      pinned.append(b);
      continue;
    }
    const li = el('li', 'tab' + (active ? ' active' : '') + (t.loading ? ' loading' : ''));
    li.title = t.url || 'Nova aba';
    const close = el('button', 'close', '✕');
    close.title = 'Fechar (Ctrl+W)';
    close.onclick = (e) => { e.stopPropagation(); invoke('close_tab', { id: t.id }); };
    li.append(favicon(t), el('span', 'title', tabTitle(t)), close);
    li.onclick = () => invoke('activate_tab', { id: t.id });
    li.onauxclick = (e) => { if (e.button === 1) invoke('close_tab', { id: t.id }); };
    li.oncontextmenu = (e) => { e.preventDefault(); invoke('toggle_pin', { id: t.id }); };
    dragTarget(li, t, false);
    list.append(li);
  }

  const spaces = $('spaces');
  spaces.replaceChildren();
  state.spaces.forEach((s, i) => {
    const b = el('button', 'dot' + (s.id === state.active_space ? ' active' : ''));
    b.title = `${s.name} (Ctrl+Shift+${i + 1})`;
    const sw = el('span');
    sw.style.background = s.color;
    b.append(sw);
    b.onclick = () => invoke('switch_space', { id: s.id });
    spaces.append(b);
  });
}

listen('state', (e) => { state = e.payload; render(); });
listen('focus-address', () => { $('address').focus(); $('address').select(); });

$('address-form').addEventListener('submit', (e) => {
  e.preventDefault();
  const v = $('address').value.trim();
  if (v) invoke('navigate', { input: v });
  $('address').blur();
});
$('address').addEventListener('focus', () => $('address').select());
$('address').addEventListener('keydown', (e) => {
  if (e.key === 'Escape') { $('address').value = activeTab()?.url || ''; $('address').blur(); }
});
document.querySelectorAll('[data-nav]').forEach((b) => {
  b.onclick = () => invoke('nav', { action: b.dataset.nav });
});
$('new-tab').onclick = () => invoke('new_tab', {});
dragTarget($('tabs'), null, false);
$('tabs').draggable = false;
$('pinned').addEventListener('dragover', (e) => e.preventDefault());
$('pinned').addEventListener('drop', (e) => {
  if (e.target === $('pinned') && dragId != null) invoke('move_tab', { id: dragId, before: null, pinned: true });
});

$('add-space').onclick = () => {
  const name = prompt('Nome do novo Space:', 'Trabalho');
  if (!name) return;
  invoke('add_space', { name, color: COLORS[state.spaces.length % COLORS.length] });
};
$('space-name').ondblclick = () => openSettings();
$('settings-btn').onclick = () => openSettings();

function openSettings() {
  const s = activeSpace();
  $('set-right').checked = state.settings.sidebar_right;
  $('set-search').value = state.settings.search;
  $('space-edit-name').value = s.name;
  $('space-edit-color').value = s.color;
  $('space-delete').disabled = state.spaces.length <= 1;
  $('settings').showModal();
}
$('settings').addEventListener('close', () => {
  const s = activeSpace();
  const settings = { sidebar_right: $('set-right').checked, search: $('set-search').value };
  if (JSON.stringify(settings) !== JSON.stringify(state.settings)) invoke('set_settings', { settings });
  const name = $('space-edit-name').value.trim() || s.name;
  const color = $('space-edit-color').value;
  if (name !== s.name || color !== s.color) invoke('update_space', { id: s.id, name, color });
});
$('space-delete').onclick = () => {
  const s = activeSpace();
  if (!confirm(`Excluir o Space "${s.name}"? As abas, cookies e logins dele serão apagados.`)) return;
  $('settings').close();
  invoke('delete_space', { id: s.id });
};

// Shortcuts while the sidebar itself has focus.
document.addEventListener('keydown', (e) => {
  if (!e.ctrlKey || e.altKey) return;
  const k = e.key.toLowerCase();
  const tabs = state ? state.tabs.filter((t) => t.space === state.active_space).sort((a, b) => b.pinned - a.pinned) : [];
  const digit = /^Digit[1-9]$/.test(e.code) ? e.code.slice(5) : null;
  if (e.shiftKey && digit) {
    const s = state.spaces[Number(digit) - 1];
    if (s) invoke('switch_space', { id: s.id });
  } else if (k === 't') invoke('new_tab', {});
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
