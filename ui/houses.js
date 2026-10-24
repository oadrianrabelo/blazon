// Shared by the sidebar and the start page.

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

const SHIELD = 'M15.68 17.24L84.32 17.24L84.32 46.88Q84.32 78.08 50 87.44Q15.68 78.08 15.68 46.88Z';

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

const SIDEBAR_PATTERNS = {
  diagonal: 'Diagonal', chevron: 'Chevron', bands: 'Faixas', split: 'Dividido', quartered: 'Quartelado',
  stripes: 'Listras', checkered: 'Xadrez', lozenges: 'Losangos', gradient: 'Degradê', solid: 'Liso',
};

/// Sidebar colors (field, middle, lower) and banner pattern of a Space.
function look(space) {
  const sb = space.look && space.look.sidebar;
  if (sb && Array.isArray(sb.colors) && sb.colors.length === 3) return { colors: sb.colors, pattern: sb.pattern || 'diagonal' };
  const h = HOUSES[space.house];
  if (h) return h;
  const c = /^#[0-9a-f]{6}$/i.test(space.color) ? space.color : '#3b82f6';
  return { colors: [mix(c, '#0b0d10', 0.22), mix(c, '#0b0d10', 0.5), c], pattern: 'diagonal' };
}

// Crest's tiled banners: a 53px cell (sidebar width / 5), softened to keep the tabs readable.
function tileImage(pattern, [, c1, c2]) {
  const s = 53;
  const cell = (x, y, fill) => `<rect x="${x}" y="${y}" width="${s}" height="${s}" fill="${fill}" fill-opacity=".2"/>`;
  const lozenge = (x, y, fill) =>
    `<path d="M${x + s / 2} ${y}L${x + s} ${y + s / 2}L${x + s / 2} ${y + s}L${x} ${y + s / 2}Z" fill="${fill}" fill-opacity=".2"/>`;
  let body;
  if (pattern === 'stripes') body = cell(0, 0, c1) + cell(0, s, c1);
  else if (pattern === 'checkered') body = cell(0, 0, c1) + cell(s, s, c1);
  else if (pattern === 'lozenges') body = lozenge(0, 0, c1) + lozenge(s, 0, c2) + lozenge(0, s, c2) + lozenge(s, s, c1);
  else return '';
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${2 * s}" height="${2 * s}">${body}</svg>`;
  return `url("data:image/svg+xml,${encodeURIComponent(svg)}")`;
}


// Look & feel options stored in settings.appearance, with Crest's defaults.
const LOOK_DEFAULTS = {
  tabScale: 1, // 0.9 compact … 1.2 large
  tabShape: 'rounded', // square | rounded | pill
  selected: 'fill', // fill | outline | glow
  followAccent: false, // tint the selected tab with the Space accent
  dimUnloaded: true,
  pinsPerRow: 0, // 0 = balanced rows
  stretchPins: true,
  pinFill: true,
  folderCounts: false,
  folderAccent: true,
  addressOutline: true,
  banner: true, // heraldic banner behind the sidebar
  accent: '', // custom accent overriding the Space's
};

const LOOK_PRESETS = {
  crest: { title: 'Clássico', hint: 'O visual original do Crest', look: {} },
  compact: { title: 'Compacto', hint: 'Mais abas na tela', look: { tabScale: 0.9, tabShape: 'square', pinsPerRow: 6 } },
  soft: { title: 'Suave', hint: 'Abas em pílula com brilho', look: { tabShape: 'pill', selected: 'glow', followAccent: true } },
  minimal: { title: 'Mínimo', hint: 'Sem faixas nem fundos', look: { banner: false, pinFill: false, selected: 'outline', dimUnloaded: false } },
};

function lookOf(settings) {
  return { ...LOOK_DEFAULTS, ...(settings && settings.appearance) };
}
