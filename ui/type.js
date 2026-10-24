// Appearance › Text: the interface font and how its text is smoothed. Runs
// before the page draws, from the choice app.js last stored, so text never
// switches font; every interface page loads it.
const BLAZON_FONTS = {
  inter: { family: '"Blazon Inter"', tracking: -0.011, display: -0.022 },
  geist: { family: '"Blazon Geist"', tracking: -0.006, display: -0.018 },
  figtree: { family: '"Blazon Figtree"', tracking: 0, display: -0.012 },
  segoe: { family: '"Segoe UI Variable Text", "Segoe UI"', displayFamily: '"Segoe UI Variable Display", "Segoe UI"', tracking: 0, display: 0 },
};

// { font, installed, mac } from the appearance options. Before Geist and
// Figtree, "Windows" text meant Segoe UI as well.
function typeOf(a) {
  return { font: a.uiFont || (a.fontRendering === 'windows' ? 'segoe' : 'inter'), installed: a.installedFont || '', mac: a.fontRendering !== 'windows' };
}

function applyType(t) {
  const name = (t.installed || '').replace(/["\\]/g, '');
  const f = t.font === 'installed' && name ? { family: `"${name}"`, tracking: 0, display: 0 } : BLAZON_FONTS[t.font] || BLAZON_FONTS.inter;
  const fallback = ', "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif';
  const root = document.documentElement.style;
  root.setProperty('--font-text', f.family + fallback);
  root.setProperty('--font-display', (f.displayFamily || f.family) + fallback);
  root.setProperty('--tracking', f.tracking ? `${f.tracking}em` : 'normal');
  root.setProperty('--display-tracking', f.display ? `${f.display}em` : 'normal');
  root.setProperty('--text-weight', t.mac ? '430' : '400');
}

function storeType(a) {
  const t = typeOf(a);
  applyType(t);
  try { localStorage.setItem('blazon.type', JSON.stringify(t)); } catch {}
}

try {
  const saved = localStorage.getItem('blazon.type');
  // 0.1.4 betas stored just "mac" or "windows".
  if (saved === 'windows') applyType({ font: 'segoe', mac: false });
  else if (saved && saved !== 'mac') applyType(JSON.parse(saved));
} catch {}
