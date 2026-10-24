const { invoke } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);
const mode = location.hash.slice(1) || 'palette';
let state = null;

const close = () => invoke('close_overlay');
addEventListener('keydown', (e) => { if (e.key === 'Escape') { e.preventDefault(); close(); } });
$('backdrop').onclick = close;

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

const GLYPHS = {
  cmd: '<path d="m9 18 6-6-6-6"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/>',
  space: '<path d="M15.7 17.2h68.6v29.7q0 31.2-34.3 40.5-34.3-9.3-34.3-40.5Z" transform="scale(.24) translate(0 -8)"/>',
};
function glyph(name) {
  const g = el('span', 'glyph');
  g.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${GLYPHS[name]}</svg>`;
  return g;
}
function favGlyph(url) {
  const g = el('span', 'glyph');
  try {
    const img = new Image();
    img.src = `https://icons.duckduckgo.com/ip3/${encodeURIComponent(new URL(url).hostname)}.ico`;
    img.onerror = () => g.replaceWith(glyph('globe'));
    g.append(img);
  } catch { return glyph('globe'); }
  return g;
}
const host = (u) => { try { return new URL(u).hostname.replace(/^www\./, ''); } catch { return ''; } };

function applySpaceLook() {
  const space = state.spaces.find((s) => s.id === state.active_space);
  const { colors } = look(space || {});
  colors.forEach((c, i) => document.documentElement.style.setProperty(`--c${i}`, c));
  const accent = lookOf(state.settings).accent;
  if (accent) document.documentElement.style.setProperty('--c2', accent);
}

// Command palette

// "Tab to search": a keyword then Tab searches that site.
const PROVIDERS = {
  g: ['Google', 'https://www.google.com/search?q='],
  yt: ['YouTube', 'https://www.youtube.com/results?search_query='],
  w: ['Wikipédia', 'https://pt.wikipedia.org/w/index.php?search='],
  gh: ['GitHub', 'https://github.com/search?q='],
  m: ['Maps', 'https://www.google.com/maps/search/'],
  a: ['Amazon', 'https://www.amazon.com.br/s?k='],
  r: ['Reddit', 'https://www.reddit.com/search/?q='],
  ml: ['Mercado Livre', 'https://lista.mercadolivre.com.br/'],
  tr: ['Tradutor', 'https://translate.google.com/?sl=auto&tl=pt&text='],
};
let provider = null;

const COMMANDS = [
  ['Nova aba', 'Ctrl+T', () => invoke('run_command', { cmd: 'newtab' })],
  ['Nova janela', 'Ctrl+N', () => invoke('new_window', { space: null })],
  ['Fechar aba', 'Ctrl+W', () => invoke('run_command', { cmd: 'close' })],
  ['Reabrir aba fechada', 'Ctrl+Shift+T', () => invoke('run_command', { cmd: 'reopen' })],
  ['Duplicar aba', '', () => tab() && invoke('duplicate_tab', { id: tab().id })],
  ['Fixar ou desafixar aba', '', () => tab() && invoke('toggle_pin', { id: tab().id })],
  ['Lado a lado com nova aba', '', () => tab() && invoke('split_tab', { id: tab().id, with: null })],
  ['Copiar link da aba', '', () => tab() && navigator.clipboard.writeText(tab().url)],
  ['Pausar ou retomar o bloqueador neste site', '', () => invoke('toggle_site_blocking')],
  ['Silenciar ou ativar som da aba', '', () => tab() && invoke('toggle_mute', { id: tab().id })],
  ['Privacidade e bloqueador', '', () => invoke('open_panel', { section: 'privacy' })],
  ['Modo leitura', '', () => invoke('reader_mode')],
  ['Traduzir página', '', () => invoke('translate_page', { target: I18N.lang() })],
  ['Arquivo de abas', '', () => invoke('run_command', { cmd: 'history' })],
  ['Bloquear Spaces agora', '', () => invoke('lock_now')],
  ['Relatar problema', '', () => invoke('open_panel', { section: 'report' })],
  ['Recarregar', 'F5', () => invoke('nav', { action: 'reload' })],
  ['Histórico', 'Ctrl+H', () => invoke('run_command', { cmd: 'history' })],
  ['Mostrar ou esconder a barra lateral', 'Ctrl+Shift+S', () => invoke('run_command', { cmd: 'sidebar' })],
  ['Configurações', '', () => invoke('open_panel', { section: 'look' })],
  ['Aparência', '', () => invoke('open_panel', { section: 'look' })],
  ['Rotas de links e Peek', '', () => invoke('open_panel', { section: 'links' })],
  ['Importar de outro navegador', '', () => invoke('open_panel', { section: 'import' })],
];
const tab = () => state.tabs.find((t) => t.id === state.active_tab[state.active_space]);

let items = [];
let sel = 0;
let seq = 0;

function looksLikeUrl(q) {
  return /^[a-z]+:\/\//i.test(q) || (!/\s/.test(q) && /\.[a-z]{2,}(\/|$|:)/i.test(q)) || /^localhost(:\d+)?/.test(q);
}

async function refresh() {
  const q = $('pal-q').value.trim();
  const ql = q.toLowerCase();
  const out = [];
  const add = (section, entry) => out.push({ section, ...entry });
  if (provider) {
    if (q) add(provider[0], { icon: 'search', title: `Buscar "${q}" no ${provider[0]}`, run: () => invoke('new_tab', { url: provider[1] + encodeURIComponent(q) }) });
  } else if (q) {
    if (looksLikeUrl(q)) add('', { icon: 'globe', title: `Abrir ${q}`, run: () => invoke('new_tab', { url: q }) });
    add('', { icon: 'search', title: `Pesquisar "${q}"`, run: () => invoke('new_tab', { url: state.settings.search + encodeURIComponent(q) }) });
    const kw = PROVIDERS[ql.split(' ')[0]];
    if (kw && !q.includes(' ')) add('', { icon: 'search', title: `Tab para buscar no ${kw[0]}`, hint: 'Tab', run: () => setProvider(ql) });
  }
  if (!provider) {
    const words = ql.split(/\s+/).filter(Boolean);
    const hit = (...xs) => words.every((w) => xs.some((x) => (x || '').toLowerCase().includes(w)));
    const spaceName = (id) => state.spaces.find((s) => s.id === id)?.name || '';
    const hidden = (id) => state.spaces.some((s) => s.id === id && s.locked) && !(state.unlocked || []).includes(id);
    const tabs = state.tabs.filter((t) => t.url && !hidden(t.space) && hit(t.title, t.url, spaceName(t.space))).slice(0, q ? 6 : 8);
    for (const t of tabs) {
      add('Abas abertas', { url: t.url, title: t.title || host(t.url), sub: `${spaceName(t.space)} · ${host(t.url)}`, run: () => invoke('activate_tab', { id: t.id }) });
    }
    for (const [title, hint, run] of COMMANDS.filter(([t]) => hit(t)).slice(0, q ? 5 : 6)) add('Comandos', { icon: 'cmd', title, hint, run });
    for (const s of state.spaces.filter((s) => s.id !== state.active_space && hit(s.name, 'space'))) {
      add('Spaces', { icon: 'space', title: `Ir para ${s.name}`, run: () => invoke('switch_space', { id: s.id }) });
    }
    if (q) {
      const my = ++seq;
      const hist = await invoke('suggest', { query: q });
      if (my !== seq) return;
      for (const h of hist.filter((h) => h.kind === 'history').slice(0, 5)) {
        add('Histórico', { url: h.url, title: h.title || host(h.url), sub: h.url, run: () => invoke('new_tab', { url: h.url }) });
      }
    }
  }
  items = out;
  sel = Math.min(sel, Math.max(0, items.length - 1));
  draw();
}

function draw() {
  const ul = $('pal-list');
  ul.replaceChildren();
  let last = null;
  items.forEach((it, i) => {
    if (it.section && it.section !== last) ul.append(el('li', 'sec', it.section));
    last = it.section;
    const li = el('li', 'item' + (i === sel ? ' sel' : ''));
    const text = el('span', 'text');
    text.append(el('span', 'title', it.title));
    if (it.sub) text.append(el('span', 'sub', it.sub));
    li.append(it.url ? favGlyph(it.url) : glyph(it.icon || 'cmd'), text);
    if (it.hint) li.append(el('span', 'hint', it.hint));
    li.onmousemove = () => { if (sel !== i) { sel = i; draw(); } };
    li.onclick = () => runItem(it);
    ul.append(li);
  });
  ul.querySelector('.sel')?.scrollIntoView({ block: 'nearest' });
}

async function runItem(it) {
  await close();
  it.run();
}

function setProvider(key) {
  provider = PROVIDERS[key];
  $('pal-chip').textContent = provider[0];
  $('pal-chip').hidden = false;
  $('pal-q').value = '';
  refresh();
}

function startPalette() {
  $('palette').hidden = false;
  const q = $('pal-q');
  // The app focuses this webview right after creating it.
  requestAnimationFrame(() => q.focus());
  addEventListener('focus', () => q.focus());
  q.addEventListener('input', () => { sel = 0; refresh(); });
  q.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (!items.length) return;
      sel = (sel + (e.key === 'ArrowDown' ? 1 : items.length - 1)) % items.length;
      draw();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (items[sel]) runItem(items[sel]);
    } else if (e.key === 'Tab') {
      e.preventDefault();
      const key = q.value.trim().toLowerCase();
      if (!provider && PROVIDERS[key]) setProvider(key);
    } else if (e.key === 'Backspace' && provider && !q.value) {
      provider = null;
      $('pal-chip').hidden = true;
      refresh();
    }
  });
  refresh();
}

// Peek toolbar, above the floating card (same geometry as overlay::peek_rect).

async function startPeek() {
  const bar = $('peekbar');
  const s = state.settings;
  const sw = s.floating_sidebar ? 8 : Math.min(420, Math.max(200, s.sidebar_width || 264));
  const m = 8;
  const cw = innerWidth - sw - m;
  const cx = s.sidebar_right ? m : sw;
  const side = Math.round(cw * 0.07);
  Object.assign(bar.style, { left: `${cx + side}px`, width: `${cw - 2 * side}px`, top: `${m + 7}px` });
  bar.hidden = false;
  $('peek-close').onclick = close;
  $('peek-tab').onclick = () => invoke('peek_to_tab');
  $('peek-copy').onclick = async () => navigator.clipboard.writeText((await invoke('peek_url')) || '');
  const update = async () => { $('peek-url').textContent = host((await invoke('peek_url')) || ''); };
  update();
  setInterval(update, 1500);
}

invoke('get_state').then((s) => {
  state = s;
  I18N.sync(state.settings.language);
  applySpaceLook();
  if (mode === 'peek') startPeek();
  else startPalette();
});
