// Quick Window toolbar (Crest's BrowserQuickWindow): page controls, the Space
// the page goes to, and "Open in Blazon", which moves it there as a tab.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const ICONS = {
  back: '<path d="m15 18-6-6 6-6"/>',
  forward: '<path d="m9 18 6-6-6-6"/>',
  reload: '<path d="M20 12a8 8 0 1 1-2.3-5.7L20 8.5"/><path d="M20 3.5v5h-5"/>',
  x: '<path d="M18 6 6 18M6 6l12 12"/>',
  down: '<path d="m6 9 6 6 6-6"/>',
};
function icon(name) {
  const t = document.createElement('template');
  t.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${ICONS[name]}</svg>`;
  return t.content.firstChild;
}
function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}
function crest(space) {
  const t = document.createElement('template');
  t.innerHTML = renderCrest(crestOf(space));
  const node = t.content.firstChild;
  node.classList?.add('crest');
  return node;
}
document.querySelectorAll('[data-i]').forEach((n) => n.append(icon(n.dataset.i)));

let state = null;
let target = null;

function paint(space) {
  const { colors, pattern } = look(space);
  colors.forEach((c, i) => document.documentElement.style.setProperty(`--c${i}`, c));
  const banner = $('banner');
  banner.className = `banner ${pattern}` + (TILED.includes(pattern) ? ' tiles' : '');
  banner.style.backgroundImage = tileImage(pattern, colors);
}

function drawPicker() {
  const space = state.spaces.find((s) => s.id === target) || state.spaces[0];
  paint(space);
  const pick = $('pick');
  pick.replaceChildren(crest(space), el('span', '', space.name), icon('down'));
  const menu = $('menu');
  menu.replaceChildren(el('div', 'sub', 'Abrir no Space'));
  for (const s of state.spaces.filter((x) => !x.locked || (state.unlocked || []).includes(x.id))) {
    const b = el('button', s.id === space.id ? 'sel' : '');
    b.append(crest(s), el('span', '', s.name));
    b.onclick = () => { target = s.id; Motion.hide(menu); drawPicker(); };
    menu.append(b);
  }
}

function showHost(url) {
  let host = url;
  try { host = new URL(url).hostname.replace(/^www\./, ''); } catch { /* not a URL yet */ }
  const box = $('host');
  box.replaceChildren(el('b', '', host));
  box.title = url;
}

$('pick').onclick = () => { const m = $('menu'); if (m.hidden) Motion.show(m); else Motion.hide(m); };
addEventListener('mousedown', (e) => { if (!e.target.closest('.qpick')) Motion.hide($('menu')); });
$('open').onclick = () => invoke('quick_to_tab', { space: target });
$('close').onclick = () => invoke('quick_close');
addEventListener('keydown', (e) => { if (e.key === 'Escape') invoke('quick_close'); });
document.querySelectorAll('[data-nav]').forEach((b) => {
  b.onclick = () => {
    if (b.dataset.nav === 'reload') Motion.pulse(b, 'm-reload');
    invoke('quick_nav', { action: b.dataset.nav });
  };
});
listen('quick-page', (e) => showHost(e.payload.url));

Promise.all([invoke('get_state'), invoke('quick_state')]).then(([s, q]) => {
  state = s;
  I18N.sync(state.settings.language);
  target = q.space || state.active_space;
  drawPicker();
  showHost(q.url);
});
