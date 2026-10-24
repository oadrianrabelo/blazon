const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();

// Icons drawn after SF Symbols: thin rounded strokes on a 24 grid.
const ICONS = {
  back: '<path d="M14.5 5.5 8 12l6.5 6.5"/>',
  forward: '<path d="m9.5 5.5 6.5 6.5-6.5 6.5"/>',
  reload: '<path d="M19 12.5a7 7 0 1 1-2.05-5.45"/><path d="M17.5 3.8v3.6h-3.6" />',
  x: '<path d="m7 7 10 10M17 7 7 17"/>',
  puzzle: '<path d="M9.2 4.5a1.8 1.8 0 0 1 3.6 0v1.3h3.4a1.2 1.2 0 0 1 1.2 1.2v3.4h1.3a1.8 1.8 0 0 1 0 3.6h-1.3v3.8a1.2 1.2 0 0 1-1.2 1.2h-3.6v-1.4a1.8 1.8 0 0 0-3.6 0V19H5.8a1.2 1.2 0 0 1-1.2-1.2V14h1.3a1.8 1.8 0 0 0 0-3.6H4.6V7a1.2 1.2 0 0 1 1.2-1.2h3.4Z"/>',
  key: '<circle cx="8" cy="15.5" r="3.8"/><path d="m10.8 12.8 7.7-7.7M16 7.6l2 2M13.9 9.7l1.6 1.6"/>',
  min: '<path d="M6 12h12"/>',
  max: '<rect x="6" y="6" width="12" height="12" rx="2.5"/>',
  restore: '<rect x="6" y="9" width="9" height="9" rx="2"/><path d="M9 9V7.8C9 6.8 9.8 6 10.8 6h5.4c1 0 1.8.8 1.8 1.8v5.4c0 1-.8 1.8-1.8 1.8H15"/>',
  search: '<circle cx="10.5" cy="10.5" r="5.8"/><path d="m15 15 4.5 4.5"/>',
  more: '<circle cx="6" cy="12" r="1.15" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.15" fill="currentColor" stroke="none"/><circle cx="18" cy="12" r="1.15" fill="currentColor" stroke="none"/>',
  plus: '<path d="M12 5.5v13M5.5 12h13"/>',
  gear: '<path d="M10.4 3.6h3.2l.5 2.3 1.7.9 2.2-.8 1.6 2.8-1.7 1.6v1.9l1.7 1.6-1.6 2.8-2.2-.8-1.7.9-.5 2.3h-3.2l-.5-2.3-1.7-.9-2.2.8-1.6-2.8 1.7-1.6v-1.9L4.4 8.8 6 6l2.2.8 1.7-.9Z"/><circle cx="12" cy="12" r="2.6"/>',
  folder: '<path d="M3.8 7.2a2 2 0 0 1 2-2h3.4l1.8 2h7.2a2 2 0 0 1 2 2v7.6a2 2 0 0 1-2 2H5.8a2 2 0 0 1-2-2Z"/><path d="M3.8 9.4h16.4"/>',
  chev: '<path d="m10 6.5 5.5 5.5-5.5 5.5"/>',
  pin: '<path d="M9.5 4h5l-.8 5.2 2.8 3.3h-9l2.8-3.3Z"/><path d="M12 12.5V20"/>',
  copy: '<rect x="8.5" y="8.5" width="10.5" height="10.5" rx="2.4"/><path d="M15.5 8.5V7a2 2 0 0 0-2-2H7a2 2 0 0 0-2 2v6.5a2 2 0 0 0 2 2h1.5"/>',
  edit: '<path d="M5 19l1-4 9.6-9.6a1.9 1.9 0 0 1 2.7 2.7L8.7 17.7Z"/><path d="m14 7 3 3"/>',
  split: '<rect x="3.5" y="5" width="17" height="14" rx="2.8"/><path d="M12 5v14"/>',
  shield: '<path d="M12 3.6c2.2 1.5 4.6 2.3 7 2.4v5.2c0 4.4-2.9 7.6-7 9.2-4.1-1.6-7-4.8-7-9.2V6c2.4-.1 4.8-.9 7-2.4Z"/><path d="m9.2 12 2 2 3.8-4"/>',
  shieldOff: '<path d="M12 3.6c2.2 1.5 4.6 2.3 7 2.4v5.2c0 4.4-2.9 7.6-7 9.2-4.1-1.6-7-4.8-7-9.2V6c2.4-.1 4.8-.9 7-2.4Z"/><path d="m5 5 14 14"/>',
  sound: '<path d="M4.5 9.5h2.8L11.5 6v12l-4.2-3.5H4.5Z" fill="currentColor" stroke-linejoin="round"/><path d="M14.8 9.2a4 4 0 0 1 0 5.6M17.4 6.8a7.4 7.4 0 0 1 0 10.4"/>',
  muted: '<path d="M4.5 9.5h2.8L11.5 6v12l-4.2-3.5H4.5Z" fill="currentColor" stroke-linejoin="round"/><path d="m15 9.5 5 5M20 9.5l-5 5"/>',
  book: '<path d="M5.5 5.8A2.3 2.3 0 0 1 7.8 3.5H18.5v14H7.8a2.3 2.3 0 0 0-2.3 2.3Z"/><path d="M5.5 19.8a2.3 2.3 0 0 1 2.3-2.3h10.7v3H7.8"/>',
  lock: '<rect x="5.5" y="10.5" width="13" height="9.5" rx="2.4"/><path d="M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5"/>',
  download: '<path d="M12 4.5v10M8 10.8l4 4 4-4"/><path d="M5 15.5v1.8a2.2 2.2 0 0 0 2.2 2.2h9.6a2.2 2.2 0 0 0 2.2-2.2v-1.8"/>',
  archive: '<rect x="3.8" y="4.8" width="16.4" height="4.4" rx="1.4"/><path d="M5.3 9.2v8.3a2 2 0 0 0 2 2h9.4a2 2 0 0 0 2-2V9.2M10 12.8h4"/>',
};
const COLORS = ['#3b82f6', '#16a34a', '#e11d48', '#f59e0b', '#8b5cf6', '#0891b2', '#db2777', '#65a30d'];

const $ = (id) => document.getElementById(id);
let state = null;

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

function icon(name) {
  const t = document.createElement('template');
  t.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">${ICONS[name]}</svg>`;
  return t.content.firstChild;
}
document.querySelectorAll('[data-i]').forEach((n) => n.append(icon(n.dataset.i)));

function setIcon(node, name) {
  node.replaceChildren(icon(name));
}

function emblem(space) {
  const t = document.createElement('template');
  t.innerHTML = renderCrest(crestOf(space));
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
  if (window.Glass) Glass.setEnabled(a.liquidGlass !== false);
  const mac = a.windowControls === 'mac' || a.windowControls === 'graphite';
  b.toggle('mac-controls', mac);
  b.toggle('graphite', a.windowControls === 'graphite');
  b.toggle('wc-fluent', a.windowControls === 'fluent');
  const side = a.controlsSide === 'left' || a.controlsSide === 'right' ? a.controlsSide : mac ? 'left' : 'right';
  b.toggle('wc-left', side === 'left');
  b.toggle('wc-right', side === 'right');
  const off = a.hiddenButtons || [];
  for (const [key, , sel] of TOOLBAR_BUTTONS) document.querySelectorAll(sel).forEach((n) => n.classList.toggle('btn-off', off.includes(key)));
  applyLookVars(settings);
  b.toggle('mica', !!a.mica);
  b.toggle('horizontal', a.tabLayout === 'horizontal');
  // Text: read by type.js before every interface page draws.
  document.documentElement.classList.toggle('win-type', a.fontRendering === 'windows');
  try { localStorage.setItem('blazon.type', a.fontRendering === 'windows' ? 'windows' : 'mac'); } catch {}
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

// Drag and drop, Crest style: the tab lifts under the pointer, the others
// make room for it, and it settles into its new place when dropped.
let drag = null;
function dragTarget(node, t) {
  node.addEventListener('pointerdown', (e) => {
    if (e.button !== 0 || e.target.closest('.close, .sound')) return;
    drag = { t, node, x: e.clientX, y: e.clientY, pointer: e.pointerId, started: false };
  });
}

const horizontal = () => document.body.classList.contains('horizontal');

function dragRoots() { return [$('pinned'), $('folders'), $('tabs')]; }

function startDrag(e) {
  drag.started = true;
  hideMenu();
  const r = drag.node.getBoundingClientRect();
  drag.dx = e.clientX - r.left;
  drag.dy = e.clientY - r.top;
  const ghost = drag.node.cloneNode(true);
  ghost.removeAttribute('data-key');
  ghost.classList.add('drag-ghost');
  Object.assign(ghost.style, { width: `${r.width}px`, height: `${r.height}px`, left: `${r.left}px`, top: `${r.top}px` });
  document.body.append(ghost);
  drag.ghost = ghost;
  drag.kind = drag.node.classList.contains('pin') ? 'pin' : 'row';
  drag.placeholder = makePlaceholder(drag.kind, drag.t.folder ?? null);
  drag.node.after(drag.placeholder);
  drag.node.hidden = true;
  document.body.classList.add('dragging');
  try { $('sidebar').setPointerCapture(drag.pointer); } catch { /* pointer already gone */ }
}

function makePlaceholder(kind, folder) {
  const p = el(kind === 'pin' ? 'button' : 'li', (kind === 'pin' ? 'pin' : 'tab') + ' placeholder' + (folder != null ? ' in-folder' : ''));
  p.dataset.key = drag.node.dataset.key;
  if (folder != null) p.dataset.folder = folder;
  if (kind === 'row' && horizontal()) p.style.width = `${drag.kind === 'row' ? drag.node.offsetWidth || 196 : 196}px`;
  else if (kind === 'row') p.style.height = `${drag.kind === 'row' ? drag.node.offsetHeight || 34 : 34}px`;
  return p;
}

// Where the dragged tab would land under the pointer.
function dropSpot(e) {
  const hit = document.elementFromPoint(e.clientX, e.clientY);
  if (!hit || !$('sidebar').contains(hit) || hit.closest('.placeholder')) return null;
  const pin = hit.closest('#pinned .pin');
  if (pin && pin !== drag.node) {
    const r = pin.getBoundingClientRect();
    return { kind: 'pin', parent: $('pinned'), ref: e.clientX < r.left + r.width / 2 ? pin : pin.nextElementSibling, folder: null };
  }
  if (hit.closest('#pinned')) return { kind: 'pin', parent: $('pinned'), ref: null, folder: null };
  const row = hit.closest('#folders .tab, #tabs .tab');
  if (row && row !== drag.node) {
    const r = row.getBoundingClientRect();
    const folder = row.dataset.folder != null ? Number(row.dataset.folder) : null;
    const first = horizontal() ? e.clientX < r.left + r.width / 2 : e.clientY < r.top + r.height / 2;
    return { kind: 'row', parent: row.parentNode, ref: first ? row : row.nextElementSibling, folder };
  }
  const head = hit.closest('.folder');
  if (head) {
    const folder = Number(head.dataset.key.slice(1));
    if (!head.classList.contains('open')) return { into: head, folder };
    return { kind: 'row', parent: $('folders'), ref: head.nextElementSibling, folder };
  }
  if (hit.closest('.new-tab')) return { kind: 'row', parent: $('tabs'), ref: $('tabs').firstElementChild, folder: null };
  if (hit.closest('#scroll')) return { kind: 'row', parent: $('tabs'), ref: null, folder: null };
  return null;
}

function moveDrag(e) {
  Object.assign(drag.ghost.style, { left: `${e.clientX - drag.dx}px`, top: `${e.clientY - drag.dy}px` });
  const spot = dropSpot(e);
  if (!spot) return;
  document.querySelectorAll('.folder.drop-into').forEach((n) => n.classList.remove('drop-into'));
  drag.into = null;
  if (spot.into) {
    spot.into.classList.add('drop-into');
    drag.into = spot.folder;
    return;
  }
  const ph = drag.placeholder;
  let ref = spot.ref;
  while (ref === drag.node || ref === ph) ref = ref.nextElementSibling;
  const sameKind = (ph.classList.contains('pin') ? 'pin' : 'row') === spot.kind;
  const inPlace = sameKind && ph.parentNode === spot.parent && (ph.nextElementSibling === ref
    || (ph.nextElementSibling === drag.node && drag.node.nextElementSibling === ref));
  if (inPlace && String(ph.dataset.folder ?? '') === String(spot.folder ?? '')) return;
  // The others slide out of the way (CrestMotion.collection).
  const before = Motion.snapshot(...dragRoots());
  const next = sameKind ? ph : makePlaceholder(spot.kind, spot.folder);
  if (!sameKind) ph.remove();
  next.classList.toggle('in-folder', spot.folder != null);
  if (spot.folder != null) next.dataset.folder = spot.folder; else delete next.dataset.folder;
  spot.parent.insertBefore(next, ref);
  drag.placeholder = next;
  before.delete(next.dataset.key);
  Motion.play(before, dragRoots());
}

function endDrag(drop) {
  const d = drag;
  drag = null;
  if (!d?.started) return;
  document.body.classList.remove('dragging');
  document.querySelectorAll('.folder.drop-into').forEach((n) => n.classList.remove('drop-into'));
  let args = null;
  if (drop && d.into != null) args = { before: null, pinned: false, folder: d.into };
  else if (drop) {
    const ph = d.placeholder;
    let next = ph.nextElementSibling;
    while (next === d.node) next = next.nextElementSibling;
    args = { before: next?.dataset.tab ? Number(next.dataset.tab) : null, pinned: ph.parentNode === $('pinned'), folder: ph.dataset.folder != null ? Number(ph.dataset.folder) : null };
  }
  // The lifted tab settles onto its slot (CrestMotion.dragPreview), or back home.
  document.body.classList.add('just-dragged');
  setTimeout(() => document.body.classList.remove('just-dragged'), 0);
  if (!args) d.node.after(d.placeholder);
  const target = d.into != null ? document.querySelector(`.folder[data-key="f${d.into}"]`) : d.placeholder;
  const home = (target || d.placeholder).getBoundingClientRect();
  const g = d.ghost;
  const from = g.getBoundingClientRect();
  g.animate([{ left: `${from.left}px`, top: `${from.top}px` }, { left: `${home.left}px`, top: `${home.top}px`, scale: 1, opacity: d.into != null ? 0 : 1 }],
    { duration: Motion.reduced.matches ? 0 : Motion.ms('--drag-preview'), easing: Motion.ease('--smooth'), fill: 'forwards' }).onfinish = () => {
    g.remove();
    if (args) invoke('move_tab', { id: d.t.id, ...args }).then(() => { if (d.placeholder.isConnected) render(); });
    else render();
  };
}

addEventListener('pointermove', (e) => {
  if (!drag || e.pointerId !== drag.pointer) return;
  if (!drag.started) {
    if (Math.hypot(e.clientX - drag.x, e.clientY - drag.y) < 5) return;
    startDrag(e);
  }
  moveDrag(e);
});
addEventListener('pointerup', (e) => { if (drag && e.pointerId === drag.pointer) endDrag(true); });
addEventListener('pointercancel', () => endDrag(false));
addEventListener('keydown', (e) => { if (e.key === 'Escape' && drag?.started) { e.preventDefault(); endDrag(false); } }, true);
// A click right after a drag must not activate the tab underneath.
addEventListener('click', (e) => { if (document.body.classList.contains('just-dragged')) { e.stopPropagation(); e.preventDefault(); } }, true);

function tabEvents(node, t) {
  // Keyboard: reachable with Tab/arrows, Enter opens, Delete closes, menu key opens the menu.
  node.tabIndex = -1;
  node.dataset.tab = t.id;
  node.setAttribute('role', node.tagName === 'LI' ? 'option' : 'button');
  node.setAttribute('aria-label', tabTitle(t));
  node.setAttribute('aria-selected', String(t.id === activeTab()?.id));
  node.onkeydown = (e) => {
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); invoke('activate_tab', { id: t.id }); }
    else if (e.key === 'Delete') { e.preventDefault(); invoke('close_tab', { id: t.id }); }
    else if (e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10')) {
      e.preventDefault();
      const r = node.getBoundingClientRect();
      tabMenu({ clientX: r.left + 20, clientY: r.bottom, preventDefault() {}, stopPropagation() {} }, t);
    }
  };
  node.onclick = () => invoke('activate_tab', { id: t.id });
  node.onauxclick = (e) => { if (e.button === 1) invoke('close_tab', { id: t.id }); };
  node.oncontextmenu = (e) => { e.preventDefault(); e.stopPropagation(); tabMenu(e, t); };
}

// Tabs on top: the bar is too short for its menus and dialogs, so while one
// is open the page steps aside for a picture of itself (set_page_hidden).
const Cover = (() => {
  let holds = 0;
  const shot = $('page-shot');
  async function hold() {
    if (!horizontal() || holds++ > 0) return;
    const s = await invoke('page_snapshot').catch(() => null);
    if (!holds) return;
    if (s) {
      Object.assign(shot.style, { left: `${s.rect.x}px`, top: `${s.rect.y}px`, width: `${s.rect.w}px`, height: `${s.rect.h}px` });
      shot.src = s.image;
      await shot.decode().catch(() => {});
      if (!holds) return;
      shot.hidden = false;
    }
    await invoke('set_page_hidden', { hidden: true });
  }
  function release() {
    if (!holds || --holds > 0) return;
    invoke('set_page_hidden', { hidden: false }).then(() => setTimeout(() => { if (!holds) shot.hidden = true; }, 60));
  }
  return { hold, release };
})();
{
  const showModal = HTMLDialogElement.prototype.showModal;
  HTMLDialogElement.prototype.showModal = function () {
    if (horizontal() && !this.open) {
      Cover.hold();
      this.addEventListener('close', () => Cover.release(), { once: true });
    }
    showModal.call(this);
  };
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
  m.classList.remove('m-leaving');
  m.getAnimations().forEach((x) => x.cancel());
  if (horizontal() && !menuHeld) {
    // The page steps aside first so the menu can spread over the page card.
    menuHeld = true;
    m.hidden = true;
    const seq = ++menuSeq;
    Cover.hold().then(() => { if (seq === menuSeq && menuHeld) placeMenu(e); });
    return;
  }
  placeMenu(e);
}
let menuHeld = false;
let menuSeq = 0;
function placeMenu(e) {
  const m = $('ctx');
  m.hidden = false;
  // Stay inside the sidebar: the page card covers the rest of the window
  // (tabs on top: anywhere, the page is a picture while the menu is open).
  const bar = horizontal() ? { left: 0, right: innerWidth } : $('sidebar').getBoundingClientRect();
  const x = Math.max(bar.left + 6, Math.min(e.clientX, bar.right - m.offsetWidth - 6));
  const y = e.above ? e.clientY - m.offsetHeight : Math.min(e.clientY, innerHeight - m.offsetHeight - 6);
  m.style.left = `${x}px`;
  m.style.top = `${Math.max(6, y)}px`;
  // Grows out of the button that opened it, or the spot that was clicked.
  const from = e.anchor ? e.anchor.getBoundingClientRect() : { left: e.clientX - 4, top: e.clientY - 4, width: 8, height: 8 };
  Motion.morphFrom(m, from);
}
function hideMenu() {
  if (menuHeld) { menuHeld = false; menuSeq++; setTimeout(() => Cover.release(), Motion.ms('--dismissal')); }
  if (!$('ctx').hidden) Motion.hide($('ctx'));
}
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
    { label: 'Traduzir página', icon: 'book', run: () => invoke('activate_tab', { id: t.id }).then(() => invoke('translate_page', { target: I18N.lang() })) },
    { label: 'Duplicar', icon: 'plus', run: () => invoke('duplicate_tab', { id: t.id }) },
    { label: 'Copiar link', icon: 'copy', run: () => navigator.clipboard.writeText(t.url) },
    { label: 'Fechar', icon: 'x', run: () => invoke('close_tab', { id: t.id }) });
  showMenu(e, items);
}

const FOLDER_COLORS = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#14b8a6', '#3b82f6', '#6366f1', '#a855f7', '#ec4899', '#a3a3a3', '#f5f5f4', '#78716c'];
let folderEditing = null;
let folderColor = '';
function openFolderDialog(f) {
  folderEditing = f;
  folderColor = f.color || '';
  const box = $('fd-colors');
  box.replaceChildren();
  for (const c of FOLDER_COLORS) {
    const b = el('button', c === folderColor ? 'sel' : '');
    b.type = 'button';
    b.style.background = c;
    b.onclick = () => { folderColor = c; [...box.children].forEach((x) => x.classList.toggle('sel', x === b)); };
    box.append(b);
  }
  $('fd-icon').value = f.icon || '';
  $('folder-dialog').showModal();
}
$('fd-reset').onclick = () => { invoke('update_folder', { id: folderEditing.id, color: '', icon: '' }); $('folder-dialog').close(); };
$('folder-dialog').addEventListener('close', () => {
  if ($('folder-dialog').returnValue !== 'save' || !folderEditing) return;
  invoke('update_folder', { id: folderEditing.id, color: folderColor, icon: $('fd-icon').value.trim() });
});

function folderMenu(e, f) {
  showMenu(e, [
    { label: 'Cor e ícone…', icon: 'edit', run: () => openFolderDialog(f) },
    { label: 'Renomear', icon: 'edit', run: () => { const name = askFolderName(f.name); if (name) invoke('update_folder', { id: f.id, name }); } },
    { label: 'Excluir pasta (mantém as abas)', icon: 'x', run: () => invoke('delete_folder', { id: f.id }) },
  ]);
}

$('scroll').addEventListener('contextmenu', (e) => {
  e.preventDefault();
  showMenu(e, [{ label: 'Nova pasta', icon: 'folder', run: () => { const name = askFolderName(); if (name) invoke('add_folder', { name, tab: null }); } }]);
});

// Saved logins (Settings › Passwords) for the site in the active tab: the key
// button fills one in, and only on the site it belongs to.
let loginsFor = null;
let logins = [];
function refreshLogins(tab) {
  const key = tab?.url ? `${tab.id}|${tab.url}` : '';
  if (key === loginsFor) return;
  loginsFor = key;
  logins = [];
  $('logins').hidden = true;
  if (!key) return;
  invoke('logins_for_tab', { tab: tab.id }).then((list) => {
    if (loginsFor !== key) return;
    logins = list;
    $('logins').hidden = !list.length;
  }).catch(() => {});
}
$('logins').onclick = (e) => {
  const tab = activeTab()?.id;
  const fill = (l) => invoke('fill_login', { tab, id: l.id }).catch((err) => toast(String(err)));
  if (logins.length === 1) return fill(logins[0]);
  const r = $('logins').getBoundingClientRect();
  showMenu({ clientX: r.left, clientY: r.bottom + 4, anchor: $('logins') }, [{ sub: 'Preencher com' },
    ...logins.map((l) => ({ label: l.username || l.host, icon: 'key', run: () => fill(l) }))]);
};

// Motion: what was on screen before this render (see motion.js).
let shownSpace = null;
let shownActive = null;
let shownLoading = null;

function render() {
  const space = activeSpace();
  const current = activeTab();
  const spaceChanged = shownSpace !== null && shownSpace !== state.active_space;
  const listRoots = [$('pinned'), $('folders'), $('tabs'), $('spaces')];
  const before = spaceChanged ? null : Motion.snapshot(...listRoots);
  const activeNode = document.querySelector('#sidebar [aria-selected="true"]');
  const activeChanged = shownActive !== (current?.id ?? null);
  const fromActive = activeChanged && activeNode ? activeNode.getBoundingClientRect() : null;
  const fromDot = spaceChanged ? document.querySelector('#spaces .dot.active')?.getBoundingClientRect() : null;
  I18N.sync(state.settings.language);
  applyLook(space);
  applyAppearance(state.settings);
  document.body.classList.toggle('right', !!state.settings.sidebar_right);
  document.body.classList.toggle('native', !!state.settings.native_frame);
  document.title = current ? `${tabTitle(current)} — Blazon` : 'Blazon';

  const name = $('space-name');
  const label = el('span', 'label', space.name);
  if (space.private) label.append(el('span', 'tag', ' · privado'));
  name.replaceChildren(emblem(space), label);

  const address = $('address');
  if (document.activeElement !== address) address.value = current?.url || '';
  const site = host(current?.url || '').replace(/^www\./, '');
  const paused = !state.settings.adblock || (state.settings.adblock_allow || []).includes(site);
  setIcon($('shield'), paused ? 'shieldOff' : 'shield');
  $('shield').title = !state.settings.adblock ? 'Bloqueador desligado (Configurações → Privacidade)'
    : paused ? `Bloqueador pausado em ${site}. Clique para retomar.` : `Bloqueando anúncios${site ? ` em ${site}` : ''}. Clique para pausar neste site.`;
  $('shield').disabled = !site || !state.settings.adblock;
  const loading = !!current?.loading;
  const form = $('address-form');
  form.classList.toggle('loading', loading);
  if (!loading && shownLoading === current?.id) {
    form.classList.add('done');
    setTimeout(() => form.classList.remove('done'), 600);
  }
  shownLoading = loading ? current.id : null;
  setIcon($('reload'), loading ? 'x' : 'reload');
  $('reload').dataset.nav = loading ? 'stop' : 'reload';
  $('reload').title = loading ? 'Parar' : 'Recarregar (F5)';

  refreshLogins(current);

  const tabs = state.tabs.filter((t) => t.space === state.active_space);
  const pinned = $('pinned');
  const list = $('tabs');
  pinned.replaceChildren();
  list.replaceChildren();

  const tabRow = (t, extra) => {
    const active = current && t.id === current.id;
    const li = el('li', 'tab' + extra + (active ? ' active' : '') + (t.loading ? ' loading' : '') + (t.awake ? '' : ' asleep'));
    li.dataset.key = `t${t.id}`;
    li.title = t.url || 'Nova aba';
    const close = el('button', 'icon close');
    close.append(icon('x'));
    close.title = 'Fechar (Ctrl+W)';
    close.onclick = (e) => { e.stopPropagation(); invoke('close_tab', { id: t.id }); };
    li.append(favicon(t), el('span', 'title', tabTitle(t)));
    if (t.audio || t.muted) {
      const snd = el('button', 'icon sound');
      snd.append(icon(t.muted ? 'muted' : 'sound'));
      snd.title = t.muted ? 'Ativar som' : 'Silenciar aba';
      snd.onclick = (e) => { e.stopPropagation(); invoke('toggle_mute', { id: t.id }); };
      li.append(snd);
    }
    if (state.splits.some((p) => p.includes(t.id))) {
      const mark = icon('split');
      mark.classList.add('split-mark');
      li.append(mark);
    }
    li.append(close);
    tabEvents(li, t);
    if (t.folder != null) li.dataset.folder = t.folder;
    dragTarget(li, t);
    return li;
  };

  const folders = $('folders');
  folders.replaceChildren();
  for (const f of state.folders.filter((x) => x.space === state.active_space)) {
    const head = el('li', 'folder' + (f.open ? ' open' : ''));
    head.dataset.key = `f${f.id}`;
    const chev = icon('chev');
    chev.classList.add('chev');
    if (f.icon) head.append(el('span', 'emoji', f.icon));
    else {
      const fi = icon('folder');
      if (f.color) fi.style.color = f.color;
      head.append(fi);
    }
    head.append(el('span', 'title', f.name));
    if (window.folderCounts) head.append(el('span', 'count', String(tabs.filter((x) => x.folder === f.id).length)));
    head.append(chev);
    head.onclick = () => invoke('update_folder', { id: f.id, open: !f.open });
    head.tabIndex = -1;
    head.setAttribute('role', 'treeitem');
    head.setAttribute('aria-expanded', String(f.open));
    head.setAttribute('aria-label', `Pasta ${f.name}`);
    head.onkeydown = (e) => {
      if (e.key === 'Enter' || e.key === ' ' || (e.key === 'ArrowRight' && !f.open) || (e.key === 'ArrowLeft' && f.open)) {
        e.preventDefault();
        invoke('update_folder', { id: f.id, open: !f.open });
      }
    };
    head.ondblclick = () => { const name = askFolderName(f.name); if (name) invoke('update_folder', { id: f.id, name }); };
    head.oncontextmenu = (e) => { e.preventDefault(); e.stopPropagation(); folderMenu(e, f); };
    folders.append(head);
    if (!f.open) continue;
    for (const t of tabs.filter((x) => x.folder === f.id)) folders.append(tabRow(t, ' in-folder'));
  }

  for (const t of tabs) {
    const active = current && t.id === current.id;
    if (t.folder != null) continue;
    if (t.pinned) {
      const b = el('button', 'pin' + (active ? ' active' : '') + (t.awake ? '' : ' asleep'));
      b.dataset.key = `t${t.id}`;
      b.title = tabTitle(t);
      b.append(favicon(t));
      tabEvents(b, t);
      dragTarget(b, t);
      pinned.append(b);
      continue;
    }
    list.append(tabRow(t, ''));
  }

  renderDeck();

  const spaces = $('spaces');
  spaces.replaceChildren();
  state.spaces.forEach((s, i) => {
    const b = el('button', 'dot' + (s.id === state.active_space ? ' active' : ''));
    b.dataset.key = `s${s.id}`;
    b.title = `${s.name} (Ctrl+Shift+${i + 1})`;
    b.append(emblem(s));
    if (s.locked && !(state.unlocked || []).includes(s.id)) {
      const lock = icon('lock');
      lock.classList.add('lock');
      b.append(lock);
    }
    b.onclick = () => invoke('switch_space', { id: s.id });
    b.ondblclick = () => openStudio(s);
    b.oncontextmenu = (ev) => {
      ev.preventDefault();
      showMenu(ev, [
        { label: 'Abrir em nova janela', icon: 'plus', run: () => invoke('new_window', { space: s.id }) },
        { label: 'Editar no Studio', icon: 'edit', run: () => openStudio(s) },
      ]);
    };
    // Drag to reorder Spaces.
    b.draggable = true;
    b.addEventListener('dragstart', (ev) => { ev.dataTransfer.setData('text/space', s.id); });
    b.addEventListener('dragover', (ev) => { if (ev.dataTransfer.types.includes('text/space')) { ev.preventDefault(); b.classList.add('drop-before'); } });
    b.addEventListener('dragleave', () => b.classList.remove('drop-before'));
    b.addEventListener('drop', (ev) => {
      b.classList.remove('drop-before');
      const id = ev.dataTransfer.getData('text/space');
      if (!id || id === s.id) return;
      ev.preventDefault();
      const from = state.spaces.findIndex((x) => x.id === id);
      invoke('move_space', { id, to: from < i ? i - 1 : i });
    });
    spaces.append(b);
  });

  // Motion: the new Space slides in from its side of the pager; otherwise rows
  // move to their new places and the selection glides to the new tab.
  const sidebar = $('sidebar');
  if (spaceChanged) {
    const from = state.spaces.findIndex((s) => s.id === shownSpace);
    const to = state.spaces.findIndex((s) => s.id === state.active_space);
    Motion.swipe([$('pinned'), document.querySelector('.space-head'), $('scroll')], to >= from ? 1 : -1);
    const dot = document.querySelector('#spaces .dot.active');
    if (dot) glideTo(fromDot, dot, sidebar);
  } else if (before) {
    Motion.play(before, listRoots, sidebar);
    const node = document.querySelector('#sidebar [aria-selected="true"]');
    if (fromActive && node) glideTo(fromActive, node, sidebar);
  }
  if (horizontal() && activeChanged) document.querySelector('#scroll .tab.active')?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  shownSpace = state.active_space;
  shownActive = current?.id ?? null;
}

// Hands the selected look from one row to another (Crest's matched geometry).
function glideTo(from, node, layer) {
  if (!from || Motion.reduced.matches) return;
  const to = node.getBoundingClientRect();
  const look = getComputedStyle(node);
  if (look.backgroundColor === 'rgba(0, 0, 0, 0)' && look.boxShadow === 'none') return;
  const duration = Motion.ms('--feedback') * 1.2;
  node.animate([{ backgroundColor: 'transparent', boxShadow: 'none' }, { backgroundColor: 'transparent', boxShadow: 'none', offset: 0.8 }, {}], { duration });
  Motion.glide(from, to, layer, { background: look.backgroundColor, boxShadow: look.boxShadow, borderRadius: look.borderRadius });
}

listen('state', (e) => { state = e.payload; syncKeymap(); if (!drag?.started) render(); });
listen('focus-address', () => { document.body.classList.remove('collapsed'); $('address').focus(); $('address').select(); });

// Window controls (borderless frame)
$('win-min').onclick = () => appWindow.minimize();
$('win-max').onclick = () => appWindow.toggleMaximize();
// Maximize turns into restore while the window fills the screen.
let maxShown = null;
async function syncMaxButton() {
  const max = !!(await Promise.resolve(appWindow.isMaximized?.()).catch(() => false));
  if (max === maxShown) return;
  maxShown = max;
  setIcon($('win-max'), max ? 'restore' : 'max');
  $('win-max').title = max ? 'Restaurar' : 'Maximizar';
}
addEventListener('resize', syncMaxButton);
syncMaxButton();
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
  if (!suggestions.length) ul.hidden = true;
  else if (ul.hidden) Motion.show(ul);
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
$('address').addEventListener('focus', () => {
  // Tabs on top: suggestions need room, so the address opens in the palette.
  if (horizontal()) { $('address').blur(); invoke('open_overlay', { mode: 'address', url: null }); return; }
  $('address').select();
});
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
  b.onclick = () => {
    if (b.dataset.nav === 'reload') Motion.pulse(b, 'm-reload');
    invoke('nav', { action: b.dataset.nav });
  };
});
$('new-tab').onclick = () => invoke('new_tab', {});
$('shield').onclick = () => invoke('toggle_site_blocking');

// Archive and history dialog
const DAY = new Intl.DateTimeFormat(I18N.locale(), { weekday: 'long', day: 'numeric', month: 'long' });
let historySeq = 0;
let historyTab = 'archive';

function dayList(ul, items, onPick) {
  let lastDay = '';
  for (const v of items) {
    const day = DAY.format(new Date(v.time * 1000));
    if (day !== lastDay) { ul.append(el('li', 'day', day)); lastDay = day; }
    const li = el('li');
    li.title = v.url;
    li.append(el('span', 'title', v.title || host(v.url)), el('span', 'url', host(v.url)));
    li.onclick = () => { $('history').close(); onPick(v); };
    ul.append(li);
  }
  if (!items.length) ul.append(el('li', 'day', 'Nada por aqui.'));
}

async function loadHistory() {
  const seq = ++historySeq;
  const q = $('history-q').value.toLowerCase();
  $('ht-archive').classList.toggle('sel', historyTab === 'archive');
  $('ht-history').classList.toggle('sel', historyTab === 'history');
  $('history-clear').textContent = historyTab === 'archive' ? 'Esvaziar' : 'Limpar';
  const ul = $('history-list');
  if (historyTab === 'archive') {
    const items = state.archive
      .filter((a) => a.space === state.active_space && (!q || a.url.toLowerCase().includes(q) || a.title.toLowerCase().includes(q)))
      .reverse();
    ul.replaceChildren();
    dayList(ul, items, (a) => invoke('restore_archived', { url: a.url, space: a.space }));
    return;
  }
  const items = await invoke('history_list', { query: $('history-q').value });
  if (seq !== historySeq) return;
  ul.replaceChildren();
  dayList(ul, items, (v) => invoke('new_tab', { url: v.url }));
}
function openHistory(tab) {
  if (typeof tab === 'string') historyTab = tab;
  if ($('history').open) return loadHistory();
  $('history-q').value = '';
  loadHistory();
  $('history').showModal();
  $('history-q').focus();
}
$('history-btn').onclick = () => openHistory('archive');
$('ht-archive').onclick = () => { historyTab = 'archive'; loadHistory(); };
$('ht-history').onclick = () => { historyTab = 'history'; loadHistory(); };
$('history-q').addEventListener('input', loadHistory);
$('history-clear').onclick = async () => {
  const what = historyTab === 'archive' ? 'o arquivo' : 'todo o histórico';
  if (!confirm(`Limpar ${what} do Space "${activeSpace().name}"?`)) return;
  await invoke(historyTab === 'archive' ? 'clear_archive' : 'clear_history');
  if (historyTab === 'archive') state.archive = state.archive.filter((a) => a.space !== state.active_space);
  loadHistory();
};
$('reader').onclick = () => invoke('reader_mode');
listen('open-history', () => openHistory('history'));

// Spaces are created and styled in Crest Studio (settings panel).
async function newSpace() {
  const houses = Object.keys(HOUSES);
  const house = houses[state.spaces.length % houses.length];
  const id = await invoke('add_space', { name: 'Novo Space', color: HOUSES[house].colors[2], private: false, house, look: null });
  invoke('open_panel', { section: `spaces:${id}` });
}
const openStudio = (space) => invoke('open_panel', { section: `spaces:${space.id}` });
$('add-space').onclick = newSpace;
$('space-name').onclick = () => openStudio(activeSpace());
$('space-menu').onclick = () => openStudio(activeSpace());

// Settings panel (shown over the page card)
$('settings-btn').onclick = () => invoke('open_panel', { section: 'look' });

// Floating sidebar: hidden to an edge strip, revealed on hover.
let peekTimer = null;
let retreatTimer = null;
function peek(on) {
  clearTimeout(peekTimer);
  clearTimeout(retreatTimer);
  if (!state?.settings.floating_sidebar) return;
  document.body.classList.toggle('collapsed', !on);
  // The page is drawn above this view, so it reclaims the space only after
  // the sidebar has retreated (CrestMotion.sidebarRetreat).
  if (on) invoke('sidebar_peek', { on });
  else retreatTimer = setTimeout(() => invoke('sidebar_peek', { on }), Motion.ms('--sidebar-retreat'));
}
$('edge').addEventListener('mouseenter', () => peek(true));
$('sidebar').addEventListener('mouseenter', () => clearTimeout(peekTimer));
$('sidebar').addEventListener('mouseleave', () => {
  const busy = document.activeElement === $('address') || !$('ctx').hidden || document.querySelector('dialog[open]');
  if (!busy) peekTimer = setTimeout(() => peek(false), 350);
});

// Elastic scrolling, like the Mac's rubber band: at either end the tab list
// follows the wheel with growing resistance, then springs back.
(() => {
  const sc = $('scroll');
  const LIMIT = 72;
  let stretch = 0;
  let settle = null;
  sc.addEventListener('wheel', (e) => {
    if (horizontal()) {
      // Tab chips: the wheel scrolls the strip sideways.
      if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) sc.scrollLeft += e.deltaY;
      return;
    }
    if (Motion.reduced.matches || !state || lookOf(state.settings).elastic === false || Math.abs(e.deltaX) > Math.abs(e.deltaY)) return;
    const atTop = sc.scrollTop <= 0 && e.deltaY < 0;
    const atEnd = sc.scrollTop + sc.clientHeight >= sc.scrollHeight - 1 && e.deltaY > 0;
    if (!atTop && !atEnd && !stretch) return;
    sc.classList.remove('settling');
    const resistance = 1 - Math.min(1, Math.abs(stretch) / LIMIT);
    const next = stretch - e.deltaY * 0.28 * resistance;
    // Pulling back past the rest position ends the stretch instead of flipping it.
    stretch = Math.sign(next) !== Math.sign(stretch) && stretch !== 0 ? 0 : Math.max(-LIMIT, Math.min(LIMIT, next));
    sc.style.setProperty('--stretch', `${stretch}px`);
    clearTimeout(settle);
    settle = setTimeout(() => {
      stretch = 0;
      sc.classList.add('settling');
      sc.style.setProperty('--stretch', '0px');
    }, 90);
  }, { passive: true });
})();

// A horizontal touchpad swipe over the sidebar pages between Spaces, like
// Crest's Space pager; past the first or last Space the content shakes.
let swipeX = 0;
let swipeReset = null;
let swipeLocked = false;
$('sidebar').addEventListener('wheel', (e) => {
  if (Math.abs(e.deltaX) <= Math.abs(e.deltaY)) return;
  if (horizontal() && $('scroll').contains(e.target)) return;
  e.preventDefault();
  if (swipeLocked) return;
  swipeX += e.deltaX;
  clearTimeout(swipeReset);
  swipeReset = setTimeout(() => { swipeX = 0; }, 160);
  if (Math.abs(swipeX) < 90) return;
  const i = state.spaces.findIndex((s) => s.id === state.active_space) + Math.sign(swipeX);
  swipeX = 0;
  swipeLocked = true;
  setTimeout(() => { swipeLocked = false; }, 450);
  if (i < 0 || i >= state.spaces.length) {
    for (const node of [$('pinned'), document.querySelector('.space-head'), $('scroll')]) Motion.pulse(node, 'm-shake');
  } else invoke('switch_space', { id: state.spaces[i].id });
}, { passive: false });

// Accessibility: icon buttons get spoken names, arrows move between tabs,
// F6 cycles address bar → tab list → page.
function labelIconButtons(root) {
  root.querySelectorAll('button[title]').forEach((b) => {
    if (!b.textContent.trim()) b.setAttribute('aria-label', b.title.replace(/\s*\(.*\)$/, ''));
  });
}
new MutationObserver(() => labelIconButtons(document.body)).observe(document.body, { childList: true, subtree: true });
labelIconButtons(document.body);

function tabNodes() {
  return [...document.querySelectorAll('#pinned [data-tab], #folders .folder, #folders [data-tab], #tabs [data-tab]')];
}
function focusTabList() {
  const nodes = tabNodes();
  const target = nodes.find((n) => n.getAttribute('aria-selected') === 'true') || nodes[0];
  if (target) { nodes.forEach((n) => { n.tabIndex = -1; }); target.tabIndex = 0; target.focus(); }
}
document.addEventListener('keydown', (e) => {
  if (e.key === 'F6') {
    e.preventDefault();
    if (document.activeElement === $('address')) focusTabList();
    else if (document.activeElement && document.activeElement.closest('#pinned, #folders, #tabs')) invoke('run_command', { cmd: 'focuspage' });
    else { $('address').focus(); $('address').select(); }
    return;
  }
  if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(e.key)) return;
  const nodes = tabNodes();
  const i = nodes.indexOf(document.activeElement);
  if (i < 0) return;
  e.preventDefault();
  const next = e.key === 'Home' ? 0 : e.key === 'End' ? nodes.length - 1 : Math.max(0, Math.min(nodes.length - 1, i + (e.key === 'ArrowDown' ? 1 : -1)));
  nodes[i].tabIndex = -1;
  nodes[next].tabIndex = 0;
  nodes[next].focus();
});

// Shortcuts while the sidebar itself has focus: the same combo → command map
// site pages use (Settings › Shortcuts), refreshed when it changes.
let keymap = {};
let keymapOf = null;
function syncKeymap() {
  const k = JSON.stringify(state?.settings.shortcuts || {});
  if (k === keymapOf) return;
  keymapOf = k;
  invoke('shortcuts').then((list) => { keymap = Object.fromEntries(list.filter((s) => s.combo).map((s) => [s.combo, s.id])); });
}
function comboOf(e) {
  let k = e.key;
  if (/^Key[A-Z]$/.test(e.code)) k = e.code.slice(3);
  else if (/^Digit[0-9]$/.test(e.code)) k = e.code.slice(5);
  else if (k.length === 1) k = k.toUpperCase();
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(k)) return '';
  return (e.ctrlKey ? 'Ctrl+' : '') + (e.altKey ? 'Alt+' : '') + (e.shiftKey ? 'Shift+' : '') + k;
}
document.addEventListener('keydown', (e) => {
  if (!state) return;
  const digit = /^Digit[1-9]$/.test(e.code) ? e.code.slice(5) : null;
  if (digit && e.ctrlKey && !e.altKey) {
    invoke('run_command', { cmd: (e.shiftKey ? 'space' : 'tab') + digit });
  } else {
    const cmd = keymap[comboOf(e)];
    if (!cmd) return;
    if (cmd === 'address') { document.body.classList.remove('collapsed'); $('address').focus(); $('address').select(); }
    else if (cmd === 'history') openHistory('history');
    else invoke('run_command', { cmd });
  }
  e.preventDefault();
});

// "Now playing": tabs making sound in any Space, with mute and go-to.
// Widget deck (BrowserSidebarWidgetDeck): stacked cards above the footer,
// one per tab playing sound plus the update notice. Swipe sideways, use the
// dots or the arrow keys to flip through them.
const DECK = { scales: [1, 0.965, 0.93], opacities: [1, 0.9, 0.75], content: [1, 0.45, 0], peek: 6, depth: 3 };
let deckSelected = null;
let deckUpdate = null;
const deckCards = new Map();

function deckItems() {
  const items = state.tabs.filter((t) => t.audio || (t.muted && t.awake)).slice(0, 5).map((t) => ({ id: `np${t.id}`, tab: t }));
  if (deckUpdate) items.push({ id: 'update', update: deckUpdate });
  return items;
}

function deckCard(item) {
  const card = el('div', 'deck-card');
  const body = el('div', 'deck-body');
  if (item.tab) {
    const t = item.tab;
    const space = state.spaces.find((x) => x.id === t.space);
    const art = el('div', 'deck-art');
    art.append(favicon(t));
    const text = el('button', 'deck-text');
    text.append(el('span', 'deck-kicker', t.muted ? 'Sem som' : 'Tocando agora'), el('span', 'deck-title', tabTitle(t)),
      el('span', 'deck-sub', space ? space.name : host(t.url)));
    text.title = 'Ir para a aba';
    text.onclick = () => invoke('activate_tab', { id: t.id });
    const mute = el('button', 'deck-control focal');
    mute.append(icon(t.muted ? 'muted' : 'sound'));
    mute.title = t.muted ? 'Ativar som' : 'Silenciar aba';
    mute.onclick = () => invoke('toggle_mute', { id: t.id });
    body.append(art, text, mute);
  } else {
    const u = item.update;
    const art = el('div', 'deck-art update');
    art.append(icon('download'));
    const text = el('div', 'deck-text');
    text.append(el('span', 'deck-kicker', 'Atualização'), el('span', 'deck-title', `Blazon ${u.version}`),
      el('span', 'deck-sub', 'Pronta para instalar'));
    const acts = el('div', 'deck-actions');
    const go = el('button', 'deck-action primary', 'Atualizar');
    go.onclick = () => { go.disabled = true; go.textContent = 'Baixando…'; invoke('install_update').catch((err) => { go.disabled = false; go.textContent = 'Atualizar'; toast(String(err)); }); };
    const notes = el('button', 'deck-action', 'Novidades');
    notes.onclick = () => invoke('new_tab', { url: u.page });
    const hide = el('button', 'deck-control quiet');
    hide.append(icon('x'));
    hide.title = 'Agora não';
    hide.onclick = () => { deckUpdate = null; renderDeck(); };
    acts.append(go, notes);
    text.append(acts);
    body.append(art, text, hide);
  }
  card.append(body);
  return card;
}

function renderDeck() {
  const deck = $('deck');
  const items = deckItems();
  if (!items.length) {
    Motion.hide(deck, () => { deck.querySelector('.deck-stack')?.replaceChildren(); deckCards.clear(); });
    return;
  }
  if (deck.hidden || deck.classList.contains('m-leaving')) Motion.show(deck, 'rise');
  if (!items.some((i) => i.id === deckSelected)) deckSelected = items[0].id;
  const stack = deck.querySelector('.deck-stack');
  const start = items.findIndex((i) => i.id === deckSelected);
  const order = items.map((_, k) => items[(start + k) % items.length]);
  const seen = new Set();
  order.forEach((item, depth) => {
    seen.add(item.id);
    const fresh = deckCard(item);
    let card = deckCards.get(item.id);
    if (card) card.replaceChildren(...fresh.childNodes);
    else {
      card = fresh;
      deckCards.set(item.id, card);
      stack.append(card);
      card.animate([{ opacity: 0, transform: `scale(${Motion.reduced.matches ? 1 : 0.97})` }, {}], { duration: Motion.ms('--space-swipe'), easing: Motion.ease('--snappy') });
    }
    const d = Math.min(depth, DECK.depth - 1);
    const visible = depth < DECK.depth;
    card.style.zIndex = String(DECK.depth - depth);
    card.style.transform = `translateY(${d * DECK.peek}px) scale(${Motion.reduced.matches ? 1 : DECK.scales[d]})`;
    card.style.opacity = visible ? String(DECK.opacities[d]) : '0';
    card.style.setProperty('--content', String(DECK.content[d]));
    card.inert = depth !== 0;
    card.classList.toggle('front', depth === 0);
  });
  for (const [id, card] of deckCards) {
    if (seen.has(id)) continue;
    deckCards.delete(id);
    card.animate([{}, { opacity: 0, transform: `scale(${Motion.reduced.matches ? 1 : 0.97})` }], { duration: Motion.ms('--dismissal'), easing: 'ease-out' }).onfinish = () => card.remove();
  }
  stack.style.paddingBottom = `${(Math.min(items.length, DECK.depth) - 1) * DECK.peek}px`;
  const dots = deck.querySelector('.deck-dots');
  dots.replaceChildren();
  dots.hidden = items.length < 2;
  items.forEach((item) => {
    const b = el('button', 'deck-dot' + (item.id === deckSelected ? ' on' : ''));
    b.title = item.tab ? tabTitle(item.tab) : 'Atualização';
    b.onclick = () => { deckSelected = item.id; renderDeck(); };
    dots.append(b);
  });
  deck.setAttribute('aria-valuetext', `${start + 1} / ${items.length}`);
}

function cycleDeck(direction) {
  const items = deckItems();
  if (items.length < 2) return;
  const i = items.findIndex((x) => x.id === deckSelected);
  deckSelected = items[(i + direction + items.length) % items.length].id;
  renderDeck();
}

// Dragging the front card tracks the pointer at 35%, rubber-banded to 28px;
// past 40px it flips to the next card.
(() => {
  const deck = $('deck');
  let start = null;
  deck.addEventListener('pointerdown', (e) => { if (e.button === 0 && deckItems().length > 1) start = { x: e.clientX, moved: 0 }; });
  addEventListener('pointermove', (e) => {
    if (!start) return;
    const dx = e.clientX - start.x;
    start.moved = dx;
    const front = deck.querySelector('.deck-card.front');
    if (!front || Math.abs(dx) < 6) return;
    const tracked = Math.max(-28, Math.min(28, dx * 0.35));
    front.style.translate = `${tracked}px 0`;
    front.classList.add('dragging');
  });
  addEventListener('pointerup', () => {
    if (!start) return;
    const dx = start.moved;
    start = null;
    const front = deck.querySelector('.deck-card.front');
    if (front) { front.classList.remove('dragging'); front.style.translate = ''; }
    if (Math.abs(dx) > 40) {
      cycleDeck(dx < 0 ? 1 : -1);
      document.body.classList.add('just-dragged');
      setTimeout(() => document.body.classList.remove('just-dragged'), 0);
    }
  });
  deck.addEventListener('wheel', (e) => {
    if (Math.abs(e.deltaX) <= Math.abs(e.deltaY) || Math.abs(e.deltaX) < 20) return;
    e.preventDefault();
    e.stopPropagation();
    if (deck.dataset.wheel) return;
    deck.dataset.wheel = '1';
    setTimeout(() => delete deck.dataset.wheel, 350);
    cycleDeck(e.deltaX > 0 ? 1 : -1);
  }, { passive: false });
  deck.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); cycleDeck(e.key === 'ArrowRight' ? 1 : -1); }
  });
})();

// Extension side panel (Crest's ExtensionSidePanel): the footer button lists
// the Space's extensions that have a panel, popup or options page; the page
// opens in a column beside the site, under a header drawn here.
$('ext-btn').onclick = async () => {
  const r = $('ext-btn').getBoundingClientRect();
  const at = { clientX: r.left, clientY: r.top - 6, anchor: $('ext-btn'), above: true };
  let pages = [];
  try { pages = await invoke('extension_panels', { space: state.active_space }); } catch (err) {
    return showMenu(at, [{ sub: String(err) }]);
  }
  const KIND = { panel: 'Painel', popup: 'Janela da extensão', options: 'Opções' };
  showMenu(at, pages.length
    ? [{ sub: 'Abrir ao lado da página' }, ...pages.map((p) => ({ label: `${p.name} · ${KIND[p.kind]}`, icon: 'puzzle', run: () => invoke('open_side_panel', { id: p.id, page: p.page, name: p.name }).catch((e) => toast(String(e))) }))]
    : [{ sub: 'Nenhuma extensão com painel neste Space.' }, { label: 'Instalar extensões…', icon: 'plus', run: () => invoke('open_panel', { section: 'extensions' }) }]);
};
listen('side-panel', (e) => {
  const head = $('side-head');
  const p = e.payload;
  if (!p) { head.hidden = true; return; }
  Object.assign(head.style, { left: `${p.x}px`, top: `${p.y}px`, width: `${p.w}px` });
  $('side-name').textContent = p.name;
  if (head.hidden) Motion.show(head, 'rise');
});
$('side-close').onclick = () => invoke('close_side_panel');

// Toasts: crash notices and other one-off messages.
function toast(text, actions = []) {
  $('toast-text').textContent = text;
  const box = $('toast-actions');
  box.replaceChildren();
  for (const [label, run, primary] of [...actions, ['Fechar', () => {}]]) {
    const b = el('button', primary ? 'primary' : '', label);
    b.onclick = () => { Motion.hide($('toast')); run(); };
    box.append(b);
  }
  Motion.show($('toast'), 'rise');
}

listen('tab-crashed', (e) => {
  const t = state?.tabs.find((x) => x.id === e.payload);
  toast(`A aba "${t ? tabTitle(t) : 'atual'}" travou.`, [
    ['Recarregar', () => invoke('activate_tab', { id: e.payload }).then(() => invoke('nav', { action: 'reload' })), true],
    ['Relatar', () => invoke('open_panel', { section: 'report' })],
  ]);
});

// Downloads
let downloads = [];
const fmtBytes = (n) => (n > 1e9 ? `${(n / 1e9).toFixed(1)} GB` : n > 1e6 ? `${(n / 1e6).toFixed(1)} MB` : `${Math.max(0, Math.round(n / 1e3))} KB`);
function drawDownloads() {
  const active = downloads.filter((d) => d.state === 'progress').length;
  $('dl-badge').hidden = !active;
  $('dl-badge').textContent = String(active);
  const ul = $('dl-list');
  ul.replaceChildren();
  if (!downloads.length) ul.append(el('li', 'meta', 'Nenhum download nesta sessão.'));
  for (const d of [...downloads].reverse()) {
    const li = el('li');
    li.append(el('span', 'name', d.name));
    const pct = d.total > 0 ? Math.min(100, (d.received / d.total) * 100) : 0;
    const status = d.state === 'done' ? `Concluído · ${fmtBytes(d.received)}` : d.state === 'failed' ? 'Falhou ou foi cancelado' : `${fmtBytes(d.received)}${d.total > 0 ? ` de ${fmtBytes(d.total)}` : ''}`;
    li.append(el('span', 'meta', `${d.host} · ${status}`));
    if (d.state === 'progress') {
      const bar = el('div', 'bar');
      const fill = el('i');
      fill.style.width = `${pct}%`;
      bar.append(fill);
      li.append(bar);
    }
    const acts = el('div', 'acts');
    const act = (label, action) => { const b = el('button', '', label); b.type = 'button'; b.onclick = () => invoke('download_action', { id: d.id, action }); acts.append(b); };
    if (d.state === 'done') { act('Abrir', 'open'); act('Mostrar na pasta', 'folder'); }
    if (d.state === 'progress') act('Cancelar', 'cancel');
    else act('Remover', 'remove');
    li.append(acts);
    ul.append(li);
  }
}
listen('downloads', (e) => {
  const before = downloads.length;
  downloads = e.payload;
  drawDownloads();
  if (downloads.length > before) flyDownload();
});
// A new download flies from the page edge to the downloads button along an
// arc, shrinking as it goes (BrowserMacDownloadFeedbackLayer / FlightPath).
function flyDownload() {
  const button = $('downloads-btn');
  const land = () => Motion.pulse(button, 'm-bounce');
  if (Motion.reduced.matches) return land();
  const bar = $('sidebar').getBoundingClientRect();
  const to = button.getBoundingClientRect();
  const src = { x: state?.settings.sidebar_right ? bar.left + 24 : bar.right - 24, y: innerHeight * 0.4 };
  const dst = { x: to.left + to.width / 2, y: to.top + to.height / 2 };
  const lift = Math.min(140, Math.hypot(dst.x - src.x, dst.y - src.y) * 0.3);
  const ctl = { x: (src.x + dst.x) / 2, y: Math.max(20, Math.min(src.y, dst.y) - lift) };
  const at = (t) => ({
    x: (1 - t) ** 2 * src.x + 2 * (1 - t) * t * ctl.x + t * t * dst.x,
    y: (1 - t) ** 2 * src.y + 2 * (1 - t) * t * ctl.y + t * t * dst.y,
  });
  const ease = (t) => (t < 0.5 ? 2 * t * t : 1 - (-2 * t + 2) ** 2 / 2);
  const fly = el('div', 'dl-flight');
  fly.append(icon('download'));
  document.body.append(fly);
  // 0.14s appearing, 0.1s pause, 0.78s flight; fades over the last 10%.
  const total = 1.02;
  const frames = [{ offset: 0, opacity: 0, transform: `translate(${src.x}px, ${src.y}px) scale(.55)` },
    { offset: 0.14 / total, opacity: 1, transform: `translate(${src.x}px, ${src.y}px) scale(1)`, easing: 'linear' },
    { offset: 0.24 / total, opacity: 1, transform: `translate(${src.x}px, ${src.y}px) scale(1)` }];
  for (let i = 1; i <= 20; i++) {
    const p = i / 20;
    const t = ease(p);
    const pt = at(t);
    frames.push({ offset: (0.24 + 0.78 * p) / total, opacity: p > 0.9 ? 1 - (p - 0.9) / 0.1 : 1, transform: `translate(${pt.x}px, ${pt.y}px) scale(${1 - t * 0.68})` });
  }
  fly.animate(frames, { duration: total * 1000, fill: 'forwards' }).onfinish = () => { fly.remove(); land(); };
}

$('downloads-btn').onclick = () => { drawDownloads(); $('downloads').showModal(); };
$('dl-clear').onclick = () => invoke('clear_downloads');
invoke('downloads_list').then((l) => { downloads = l; drawDownloads(); });

invoke('get_state').then((s) => {
  state = s;
  syncKeymap();
  render();
  // First run: Crest-style welcome flow over the whole window.
  if (!s.settings.welcomed) setTimeout(() => invoke('open_overlay', { mode: 'welcome' }), 300);
});
// A new version shows up as a card in the widget deck, like Crest's.
listen('update-available', (e) => {
  deckUpdate = e.payload;
  deckSelected = 'update';
  if (state) renderDeck();
});

invoke('take_crash_report').then((crash) => {
  if (crash) toast('O Blazon fechou inesperadamente na última vez. O relatório foi salvo.', [['Relatar', () => invoke('open_panel', { section: 'report' }), true]]);
});
