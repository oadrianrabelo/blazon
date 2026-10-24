// Crest Studio arms rendered as SVG: a backplate (shape) holding a field,
// a division, a band (ordinary) and an emblem, framed by a trim, with a finish
// and a shadow. Geometry follows Crest's BrowserSpaceCrest* shapes.

const CREST_SHAPES = {
  shield: 'Escudo', frenchShield: 'Escudo francês', circle: 'Redondo', oval: 'Oval', diamond: 'Losango', hexagon: 'Hexágono',
  octagon: 'Octógono', roundedSquare: 'Quadrado', seal: 'Selo', banner: 'Flâmula', badge: 'Distintivo', none: 'Nenhum',
};
const CREST_DIVISIONS = {
  plain: 'Liso', perPale: 'Vertical', perFess: 'Horizontal', perBend: 'Diagonal', perChevron: 'Chevron', quarterly: 'Quartelado',
  perSaltire: 'Cruzado', gyronny: 'Gomos', barry: 'Barras', paly: 'Listras', checky: 'Xadrez',
};
const COUNTED_DIVISIONS = ['gyronny', 'barry', 'paly', 'checky'];
const CREST_ORDINARIES = {
  none: 'Nenhuma', pale: 'Pala', fess: 'Faixa', bend: 'Banda', chevron: 'Asna', cross: 'Cruz', saltire: 'Aspa', chief: 'Chefe',
  bordure: 'Bordadura', pall: 'Pairle', pile: 'Pira', canton: 'Cantão', roundel: 'Besante',
};
const CREST_TRIMS = {
  none: 'Nenhuma', line: 'Linha', doubleLine: 'Linha dupla', shield: 'Contorno', laurel: 'Louros', sunburst: 'Raios',
  doubleRing: 'Anel duplo', seal: 'Selo', beaded: 'Pérolas',
};
const CREST_FINISHES = { flat: 'Fosco', sheen: 'Brilho', embossed: 'Relevo' };
const CREST_DEPTHS = { none: 'Sem sombra', soft: 'Suave', lifted: 'Elevada' };
const CREST_FIGURES = {
  direwolf: 'Lobo', lion: 'Leão', stag: 'Cervo', dragon: 'Dragão', dragonHead: 'Cabeça de dragão', wyvern: 'Serpe', hydra: 'Hidra',
  griffin: 'Grifo', eagle: 'Águia', falcon: 'Falcão', raven: 'Corvo', bat: 'Morcego', bear: 'Urso', boar: 'Javali', fox: 'Raposa',
  horse: 'Cavalo', unicorn: 'Unicórnio', serpent: 'Serpente', kraken: 'Kraken', seahorse: 'Cavalo-marinho', scorpion: 'Escorpião',
  rose: 'Rosa', lily: 'Lírio', pine: 'Pinheiro', willow: 'Salgueiro', castle: 'Castelo', sword: 'Espada', swords: 'Espadas',
  axes: 'Machados', trident: 'Tridente', anchor: 'Âncora', scales: 'Balança', fish: 'Peixe', sun: 'Sol',
};
// Figures drawn here (Crest uses SF Symbols for these two).
const DRAWN_FIGURES = {
  sun: 'M256 136a120 120 0 1 1 0 240a120 120 0 1 1 0-240ZM241 16h30v80h-30ZM241 416h30v80h-30ZM16 241h80v30H16ZM416 241h80v30h-80ZM78 99l21-21 57 57-21 21ZM356 377l21-21 57 57-21 21ZM78 413l57-57 21 21-57 57ZM356 135l57-57 21 21-57 57Z',
  fish: 'M40 256c60-90 160-130 250-110 50 11 95 40 130 80l60-60v180l-60-60c-35 40-80 69-130 80-90 20-190-20-250-110Zm150-30a22 22 0 1 0 0 44a22 22 0 1 0 0-44Z',
};

// Arms of Crest's nine houses: shape, figure, trim, colors (field, second
// field, band, emblem, border, edge) and extras, from SpaceHouse.cs.
const HOUSE_ARMS = {
  winter: { shape: 'frenchShield', charge: 'direwolf', trim: 'line', edge: 0.35, colors: ['#dee4e8', '#86adc4', '#3e4e5e', '#57626d', '#3e4e5e', '#1e2833'] },
  lion: { shape: 'shield', charge: 'lion', trim: 'doubleLine', edge: 0.25, colors: ['#981d28', '#722030', '#d9ae51', '#d9ae51', '#d9ae51', '#3c0e1a'], chargeScale: 1.25, trimWeight: 0.85, finish: 'sheen', depth: 'lifted' },
  storm: { shape: 'hexagon', charge: 'stag', trim: 'doubleLine', edge: 0.3, colors: ['#d5a73b', '#b08f4a', '#17181b', '#17181b', '#17181b', '#17181b'] },
  dragon: { shape: 'shield', charge: 'dragon', trim: 'line', edge: 0.25, colors: ['#120f10', '#1a100d', '#bf2e23', '#bf2e23', '#bf2e23', '#7a1e12'], chargeScale: 1.25, depth: 'lifted' },
  meadow: { shape: 'circle', charge: 'rose', trim: 'laurel', edge: 0.25, colors: ['#306b3b', '#345738', '#deba51', '#deba51', '#bca766', '#152318'] },
  iron: { shape: 'octagon', charge: 'kraken', trim: 'beaded', edge: 0.5, colors: ['#0e1a1c', '#2c3a3c', '#cba64d', '#cba64d', '#cba64d', '#9c9781'], finish: 'embossed' },
  river: { shape: 'banner', charge: 'fish', trim: 'line', edge: 0.2, colors: ['#2d5e9d', '#a42830', '#d9dee4', '#d9dee4', '#d9dee4', '#0f1e2e'], division: 'barry', divisionCount: 6 },
  sun: { shape: 'circle', charge: 'sun', trim: 'sunburst', edge: 0.25, colors: ['#d9772d', '#8b3d1b', '#e4b755', '#b82a1e', '#e4b755', '#35160b'] },
  vigil: { shape: 'seal', charge: 'raven', trim: 'doubleRing', edge: 0.45, colors: ['#101112', '#272a2e', '#7b838c', '#cfd1d4', '#7b838c', '#7b838c'], trimWeight: 0.9, depth: 'none' },
};

const CREST_DEFAULTS = {
  shape: 'shield', division: 'plain', divisionCount: 6, ordinary: 'none', chargeKind: 'figure', charge: 'lion', text: '',
  chargeScale: 1.2, trim: 'line', trimWeight: 0.75, detail: 14, finish: 'flat', depth: 'soft', edge: 0.25,
  colors: ['#3b82f6', '#1e3a8a', '#f8fafc', '#f8fafc', '#f8fafc', '#0b1220'],
};

/// The full arms of a Space: its own Studio crest, its house's, or one built from its color.
function crestOf(space) {
  if (space && space.look && space.look.crest) return { ...CREST_DEFAULTS, ...space.look.crest };
  const arms = space && HOUSE_ARMS[space.house];
  if (arms) return { ...CREST_DEFAULTS, ...arms };
  const c = (space && /^#[0-9a-f]{6}$/i.test(space.color)) ? space.color : '#3b82f6';
  return { ...CREST_DEFAULTS, chargeKind: 'none', colors: [c, mix(c, '#000000', 0.6), '#f8fafc', '#f8fafc', mix(c, '#ffffff', 0.6), mix(c, '#000000', 0.35)] };
}

const P = (x, y) => `${(10 + x * 80).toFixed(2)} ${(10 + y * 80).toFixed(2)}`;

function polygon(sides, rotation) {
  const pts = [];
  for (let i = 0; i < sides; i++) {
    const a = rotation + (i / sides) * 2 * Math.PI;
    pts.push(`${(50 + Math.cos(a) * 40).toFixed(2)} ${(50 + Math.sin(a) * 40).toFixed(2)}`);
  }
  return `M${pts.join('L')}Z`;
}

function scalloped(n) {
  let d = '';
  for (let i = 0; i < n; i++) {
    const s = (i / n) * 2 * Math.PI - Math.PI / 2;
    const e = ((i + 1) / n) * 2 * Math.PI - Math.PI / 2;
    const m = (s + e) / 2;
    const v = (a, r) => `${(50 + Math.cos(a) * r).toFixed(2)} ${(50 + Math.sin(a) * r).toFixed(2)}`;
    if (i === 0) d += `M${v(s, 36)}`;
    d += `Q${v(m, 43.2)} ${v(e, 36)}`;
  }
  return d + 'Z';
}

function platePath(shape, detail) {
  switch (shape) {
    case 'circle': return 'M10 50a40 40 0 1 0 80 0a40 40 0 1 0-80 0Z';
    case 'oval': return 'M18.8 50a31.2 40 0 1 0 62.4 0a31.2 40 0 1 0-62.4 0Z';
    case 'frenchShield': return `M${P(0.08, 0.04)}L${P(0.92, 0.04)}L${P(0.92, 0.7)}Q${P(0.92, 0.97)} ${P(0.5, 0.97)}Q${P(0.08, 0.97)} ${P(0.08, 0.7)}Z`;
    case 'diamond': return `M${P(0.5, 0.02)}L${P(0.96, 0.5)}L${P(0.5, 0.98)}L${P(0.04, 0.5)}Z`;
    case 'hexagon': return polygon(6, -Math.PI / 2);
    case 'octagon': return polygon(8, Math.PI / 8);
    case 'roundedSquare': return 'M31.6 14h36.8a17.6 17.6 0 0 1 17.6 17.6v36.8a17.6 17.6 0 0 1-17.6 17.6H31.6A17.6 17.6 0 0 1 14 68.4V31.6A17.6 17.6 0 0 1 31.6 14Z';
    case 'seal': return scalloped(Math.max(6, detail || 14));
    case 'banner': return `M${P(0.12, 0.04)}L${P(0.88, 0.04)}L${P(0.88, 0.76)}L${P(0.5, 0.97)}L${P(0.12, 0.76)}Z`;
    case 'badge': return 'M16.93 56.52A35.2 35.2 0 1 1 83.07 56.52L50 88.4Z';
    case 'none': return 'M10 10h80v80H10Z';
    default: return `M${P(0.06, 0.08)}L${P(0.94, 0.08)}L${P(0.94, 0.46)}Q${P(0.94, 0.86)} ${P(0.5, 0.98)}Q${P(0.06, 0.86)} ${P(0.06, 0.46)}Z`;
  }
}

function divisionPath(div, count) {
  const n = Math.max(2, count || 6);
  const R = (x, y, w, h) => `M${x} ${y}h${w}v${h}h${-w}Z`;
  switch (div) {
    case 'perPale': return R(50, 0, 50, 100);
    case 'perFess': return R(0, 50, 100, 50);
    case 'perBend': return 'M100 0L100 100L0 100Z';
    case 'perChevron': return 'M0 48L50 68L100 48L100 100L0 100Z';
    case 'quarterly': return R(50, 0, 50, 50) + R(0, 50, 50, 50);
    case 'perSaltire': return 'M0 0L100 0L50 50ZM0 100L100 100L50 50Z';
    case 'gyronny': {
      let d = '';
      const w = n * 2;
      for (let i = 0; i < w; i += 2) {
        const a = (i / w) * 2 * Math.PI - Math.PI / 2;
        const b = ((i + 1) / w) * 2 * Math.PI - Math.PI / 2;
        d += `M50 50L${50 + Math.cos(a) * 100} ${50 + Math.sin(a) * 100}L${50 + Math.cos(b) * 100} ${50 + Math.sin(b) * 100}Z`;
      }
      return d;
    }
    case 'barry': { let d = ''; for (let i = 1; i < n; i += 2) d += R(0, (i * 100) / n, 100, 100 / n); return d; }
    case 'paly': { let d = ''; for (let i = 1; i < n; i += 2) d += R((i * 100) / n, 0, 100 / n, 100); return d; }
    case 'checky': {
      let d = '';
      for (let r = 0; r < n; r++) for (let c = 0; c < n; c++) if ((r + c) % 2) d += R((c * 100) / n, (r * 100) / n, 100 / n, 100 / n);
      return d;
    }
    default: return '';
  }
}

// Bands: filled paths, or [path, strokeWidth] for stroked ones.
function ordinaryShape(ord) {
  const b = 13.6;
  switch (ord) {
    case 'pale': return `M${50 - b / 2} 0h${b}v100h${-b}Z`;
    case 'fess': return `M0 ${50 - b / 2}h100v${b}H0Z`;
    case 'bend': return ['M-2 -2L102 102', b];
    case 'chevron': return [`M${P(0.04, 0.78)}L${P(0.5, 0.34)}L${P(0.96, 0.78)}`, b];
    case 'cross': return `M${50 - b / 2} 0h${b}v100h${-b}ZM0 ${50 - b / 2}h100v${b}H0Z`;
    case 'saltire': return ['M-2 -2L102 102M102 -2L-2 102', b];
    case 'chief': return `M0 0h100v${10 + b * 1.4}H0Z`;
    case 'pall': return [`M${P(0.02, 0.02)}L50 50M${P(0.98, 0.02)}L50 50M50 50L50 102`, b];
    case 'pile': return `M${P(0.22, -0.02)}L${P(0.78, -0.02)}L${P(0.5, 0.8)}Z`;
    case 'canton': return 'M0 0h38.8v38.8H0Z';
    case 'roundel': return 'M36.4 50a13.6 13.6 0 1 0 27.2 0a13.6 13.6 0 1 0-27.2 0Z';
    default: return null;
  }
}

function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
}

let crestSeq = 0;

/// SVG markup for arms. `size` only sets the width/height attributes.
function renderCrest(arms, size) {
  const a = { ...CREST_DEFAULTS, ...arms };
  const [field, field2, band, emblem, border, edge] = a.colors;
  const id = `cr${++crestSeq}`;
  const plate = platePath(a.shape, a.detail);
  const parts = [];
  const shadow = { none: '', soft: 'dy="2" stdDeviation="2.5" flood-opacity=".3"', lifted: 'dy="4" stdDeviation="4.5" flood-opacity=".4"' }[a.depth] || '';
  parts.push('<defs>');
  parts.push(`<clipPath id="${id}c"><path d="${plate}"/></clipPath>`);
  if (shadow) parts.push(`<filter id="${id}s" x="-20%" y="-20%" width="140%" height="150%"><feDropShadow dx="0" ${shadow} flood-color="#000"/></filter>`);
  parts.push(`<linearGradient id="${id}g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".34"/><stop offset=".5" stop-color="#fff" stop-opacity=".06"/><stop offset="1" stop-color="#000" stop-opacity=".12"/></linearGradient>`);
  parts.push('</defs>');
  parts.push(`<g${shadow ? ` filter="url(#${id}s)"` : ''}>`);
  // Trim behind/around the plate.
  parts.push(trimMarkup(a, plate, border));
  if (a.shape !== 'none') {
    parts.push(`<path d="${plate}" fill="${field}"/>`);
    parts.push(`<g clip-path="url(#${id}c)">`);
    const div = divisionPath(a.division, a.divisionCount);
    if (div) parts.push(`<path d="${div}" fill="${field2}"/>`);
    const ord = ordinaryShape(a.ordinary);
    if (Array.isArray(ord)) parts.push(`<path d="${ord[0]}" fill="none" stroke="${band}" stroke-width="${ord[1]}"/>`);
    else if (ord) parts.push(`<path d="${ord}" fill="${band}"/>`);
    if (a.ordinary === 'bordure') parts.push(`<path d="${plate}" fill="none" stroke="${band}" stroke-width="14.4"/>`);
    if (a.finish === 'sheen') parts.push(`<rect x="0" y="0" width="100" height="100" fill="url(#${id}g)"/>`);
    if (a.finish === 'embossed') parts.push(`<path d="${plate}" fill="none" stroke="#fff" stroke-opacity=".28" stroke-width="5" transform="translate(-1 -1)"/><path d="${plate}" fill="none" stroke="#000" stroke-opacity=".3" stroke-width="5" transform="translate(1.2 1.6)"/>`);
    parts.push('</g>');
    if (a.edge > 0) parts.push(`<path d="${plate}" fill="none" stroke="${edge}" stroke-width="${(a.edge * 4).toFixed(2)}" stroke-linejoin="round"/>`);
  }
  parts.push(chargeMarkup(a, emblem));
  parts.push('</g>');
  const dim = size ? ` width="${size}" height="${size}"` : '';
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="-6 -6 112 112"${dim}>${parts.join('')}</svg>`;
}

function chargeMarkup(a, color) {
  const s = 40 * a.chargeScale;
  const x = 50 - s / 2;
  const y = (a.shape === 'shield' || a.shape === 'frenchShield' || a.shape === 'banner' || a.shape === 'badge' ? 48 : 50) - s / 2;
  if (a.chargeKind === 'emoji' && a.text) {
    return `<text x="50" y="${y + s * 0.82}" font-size="${s * 0.86}" text-anchor="middle" font-family="Segoe UI Emoji, Apple Color Emoji, Noto Color Emoji, sans-serif">${esc(a.text)}</text>`;
  }
  if (a.chargeKind === 'monogram' && a.text) {
    const t = esc(a.text.slice(0, 2).toUpperCase());
    return `<text x="50" y="${y + s * 0.78}" font-size="${s * (t.length > 1 ? 0.62 : 0.8)}" font-weight="700" text-anchor="middle" fill="${color}" font-family="Georgia, 'Times New Roman', serif">${t}</text>`;
  }
  if (a.chargeKind !== 'figure') return '';
  const d = DRAWN_FIGURES[a.charge] || CHARGE_PATHS[a.charge];
  if (!d) return '';
  return `<svg x="${x}" y="${y}" width="${s}" height="${s}" viewBox="0 0 512 512"><path fill="${color}" d="${d}"/></svg>`;
}

function trimMarkup(a, plate, color) {
  const w = Math.max(0.8, 3.2 * a.trimWeight);
  const around = (scale, width) => `<path d="${plate}" fill="none" stroke="${color}" stroke-width="${(width / scale).toFixed(2)}" stroke-linejoin="round" transform="translate(50 50) scale(${scale}) translate(-50 -50)"/>`;
  const n = Math.max(6, a.detail || 14);
  switch (a.trim) {
    case 'shield': return around(1.12, w * 1.15);
    case 'line': return around(1.1, w * 0.8);
    case 'doubleLine':
    case 'doubleRing': return around(1.09, w * 0.8) + around(1.2, w * 0.6);
    case 'seal': return `<path d="${scalloped(Math.max(8, n))}" fill="none" stroke="${color}" stroke-width="${w * 0.8}" transform="translate(50 50) scale(1.27) translate(-50 -50)"/>`;
    case 'sunburst': {
      let d = '';
      const step = (2 * Math.PI) / n;
      const spread = (step * Math.min(0.45, 0.35 * a.trimWeight)) / 2;
      const pt = (ang, r) => `${(50 + Math.cos(ang) * r).toFixed(2)} ${(50 + Math.sin(ang) * r).toFixed(2)}`;
      for (let i = 0; i < n; i++) {
        const ang = i * step - Math.PI / 2;
        d += `M${pt(ang - spread, 41)}L${pt(ang, 50)}L${pt(ang + spread, 41)}Z`;
      }
      return `<path d="${d}" fill="${color}"/>`;
    }
    case 'beaded': {
      let out = '';
      for (let i = 0; i < n * 2; i++) {
        const ang = (i / (n * 2)) * 2 * Math.PI - Math.PI / 2;
        out += `<circle cx="${(50 + Math.cos(ang) * 45.5).toFixed(2)}" cy="${(50 + Math.sin(ang) * 45.5).toFixed(2)}" r="${(1.8 * a.trimWeight + 0.4).toFixed(2)}"/>`;
      }
      return `<g fill="${color}">${out}</g>`;
    }
    case 'laurel': {
      let out = '';
      for (const side of [-1, 1]) {
        for (let i = 0; i < 7; i++) {
          const ang = Math.PI / 2 + side * (0.35 + i * 0.36);
          const cx = 50 + Math.cos(ang) * 46;
          const cy = 52 + Math.sin(ang) * 44;
          const rot = (ang * 180) / Math.PI + (side > 0 ? 60 : -60);
          out += `<ellipse cx="${cx.toFixed(2)}" cy="${cy.toFixed(2)}" rx="5.2" ry="2.3" transform="rotate(${rot.toFixed(1)} ${cx.toFixed(2)} ${cy.toFixed(2)})"/>`;
        }
      }
      return `<g fill="${color}">${out}</g>`;
    }
    default: return '';
  }
}

/// A random but tasteful crest: random parts, colors from one hue family.
function shuffleCrest(current) {
  const pick = (o) => { const k = Object.keys(o); return k[Math.floor(Math.random() * k.length)]; };
  const hue = Math.floor(Math.random() * 360);
  const hsl = (h, s, l) => {
    const a = (s / 100) * Math.min(l / 100, 1 - l / 100);
    const f = (n) => { const k = (n + h / 30) % 12; const c = l / 100 - a * Math.max(-1, Math.min(k - 3, 9 - k, 1)); return Math.round(c * 255).toString(16).padStart(2, '0'); };
    return `#${f(0)}${f(8)}${f(4)}`;
  };
  const metal = Math.random() < 0.5 ? '#d9ae51' : '#dee4e8';
  return {
    ...current,
    shape: pick(Object.fromEntries(Object.entries(CREST_SHAPES).filter(([k]) => k !== 'none'))),
    division: Math.random() < 0.5 ? 'plain' : pick(CREST_DIVISIONS),
    ordinary: Math.random() < 0.55 ? 'none' : pick(CREST_ORDINARIES),
    chargeKind: 'figure',
    charge: pick(CREST_FIGURES),
    trim: pick(CREST_TRIMS),
    finish: pick(CREST_FINISHES),
    colors: [hsl(hue, 55, 34), hsl((hue + 25) % 360, 45, 24), metal, metal, metal, hsl(hue, 50, 12)],
  };
}
