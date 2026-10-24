// Liquid Glass, after macOS 26 (Crest's .glassEffect): frosted surfaces that
// also bend what is behind them at their edges, like a lens, with a rim
// highlight and a light that follows the pointer.
//
// The refraction is an SVG displacement filter used in backdrop-filter (which
// Chromium, and so WebView2, supports). Each surface size gets its own
// displacement map: a rounded rectangle whose edge band pushes pixels toward
// the center, strongest at the rim.
const Glass = (() => {
  const SELECTOR = '.ctx, .suggest, dialog, .toast, .deck-card, .switcher, .drag-ghost, .side-head, .palette, .pbtn, .qmenu, .w-card, .lg-surface';
  const NS = 'http://www.w3.org/2000/svg';
  const filters = new Map();
  let defs = null;
  let enabled = true;

  function host() {
    if (defs) return defs;
    const svg = document.createElementNS(NS, 'svg');
    svg.setAttribute('width', '0');
    svg.setAttribute('height', '0');
    svg.setAttribute('aria-hidden', 'true');
    svg.style.cssText = 'position:absolute;width:0;height:0;overflow:hidden';
    defs = document.createElementNS(NS, 'defs');
    svg.append(defs);
    document.body.append(svg);
    return defs;
  }

  // Displacement map for a w×h rounded rectangle of radius r (at half
  // resolution; the filter stretches it). Red/green hold the x/y shift.
  function map(w, h, r) {
    const s = 0.5;
    const cw = Math.max(2, Math.round(w * s));
    const ch = Math.max(2, Math.round(h * s));
    const c = document.createElement('canvas');
    c.width = cw;
    c.height = ch;
    const ctx = c.getContext('2d');
    const img = ctx.createImageData(cw, ch);
    const band = Math.min(30, Math.min(w, h) / 3);
    const hw = w / 2;
    const hh = h / 2;
    const rr = Math.min(r, hw, hh);
    for (let y = 0; y < ch; y++) {
      for (let x = 0; x < cw; x++) {
        const px = (x + 0.5) / s - hw;
        const py = (y + 0.5) / s - hh;
        const qx = Math.abs(px) - (hw - rr);
        const qy = Math.abs(py) - (hh - rr);
        // Signed distance to the edge (negative inside) and the outward normal.
        const ox = Math.max(qx, 0);
        const oy = Math.max(qy, 0);
        const outside = Math.hypot(ox, oy);
        const dist = outside + Math.min(Math.max(qx, qy), 0) - rr;
        let nx;
        let ny;
        if (qx > 0 && qy > 0) { nx = (ox / (outside || 1)) * Math.sign(px); ny = (oy / (outside || 1)) * Math.sign(py); }
        else if (qx > qy) { nx = Math.sign(px); ny = 0; }
        else { nx = 0; ny = Math.sign(py); }
        const depth = -dist;
        const t = depth < band ? 1 - Math.max(0, depth) / band : 0;
        const k = t * t;
        const i = (y * cw + x) * 4;
        img.data[i] = 128 - nx * k * 127;
        img.data[i + 1] = 128 - ny * k * 127;
        img.data[i + 2] = 128;
        img.data[i + 3] = 255;
      }
    }
    ctx.putImageData(img, 0, 0);
    return c.toDataURL();
  }

  // Sizes are rounded to 4px so menus of nearly the same size share a map,
  // and at most MAX_FILTERS are kept: the least recently used one that no
  // surface on screen still uses goes first.
  const MAX_FILTERS = 32;
  let serial = 0;
  function evict() {
    if (filters.size <= MAX_FILTERS) return;
    const used = new Set([...document.querySelectorAll('.lg')].map((n) => n.style.getPropertyValue('--lg-filter')));
    for (const [key, f] of [...filters].sort((a, b) => a[1].last - b[1].last)) {
      if (filters.size <= MAX_FILTERS) break;
      if (used.has(`url(#${f.id})`)) continue;
      f.node.remove();
      filters.delete(key);
    }
  }
  function filterFor(w, h, r) {
    w = Math.max(8, Math.round(w / 4) * 4);
    h = Math.max(8, Math.round(h / 4) * 4);
    const key = `${w}x${h}r${Math.round(r)}`;
    const known = filters.get(key);
    if (known) { known.last = performance.now(); return known.id; }
    const id = `lg-${serial++}`;
    const f = document.createElementNS(NS, 'filter');
    f.id = id;
    for (const [k, v] of Object.entries({ x: 0, y: 0, width: w, height: h, filterUnits: 'userSpaceOnUse', primitiveUnits: 'userSpaceOnUse', 'color-interpolation-filters': 'sRGB' })) f.setAttribute(k, v);
    const im = document.createElementNS(NS, 'feImage');
    for (const [k, v] of Object.entries({ x: 0, y: 0, width: w, height: h, preserveAspectRatio: 'none', result: 'map' })) im.setAttribute(k, v);
    im.setAttribute('href', map(w, h, r));
    const d = document.createElementNS(NS, 'feDisplacementMap');
    for (const [k, v] of Object.entries({ in: 'SourceGraphic', in2: 'map', scale: Math.min(56, Math.min(w, h) / 3), xChannelSelector: 'R', yChannelSelector: 'G' })) d.setAttribute(k, v);
    f.append(im, d);
    host().append(f);
    filters.set(key, { id, node: f, last: performance.now() });
    evict();
    return id;
  }

  const sized = new WeakMap();
  const resize = new ResizeObserver((entries) => {
    for (const e of entries) apply(e.target);
  });

  function apply(el) {
    if (!enabled) { el.style.removeProperty('--lg-filter'); return; }
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    if (w < 8 || h < 8) return;
    const r = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 0;
    const key = `${w}x${h}x${r}`;
    if (sized.get(el) === key) return;
    sized.set(el, key);
    el.style.setProperty('--lg-filter', `url(#${filterFor(w, h, r)})`);
  }

  function adopt(root) {
    const nodes = root.matches?.(SELECTOR) ? [root] : [];
    nodes.push(...(root.querySelectorAll?.(SELECTOR) || []));
    for (const el of nodes) {
      if (el.classList.contains('lg')) continue;
      el.classList.add('lg');
      resize.observe(el);
      apply(el);
    }
  }

  // The light follows the pointer over a glass surface.
  addEventListener('pointermove', (e) => {
    const el = e.target.closest?.('.lg');
    if (!el) return;
    const r = el.getBoundingClientRect();
    el.style.setProperty('--lx', `${e.clientX - r.left}px`);
    el.style.setProperty('--ly', `${e.clientY - r.top}px`);
  }, { passive: true });

  function start() {
    adopt(document.body);
    new MutationObserver((list) => {
      for (const m of list) m.addedNodes.forEach((n) => n.nodeType === 1 && adopt(n));
    }).observe(document.body, { childList: true, subtree: true });
  }
  if (document.body) start(); else addEventListener('DOMContentLoaded', start);

  // Settings › Appearance › Liquid Glass, or Windows' "Transparency effects" off.
  function setEnabled(on) {
    if (on === enabled) return;
    enabled = on;
    document.documentElement.classList.toggle('no-lg', !on);
    document.querySelectorAll('.lg').forEach((el) => { sized.delete(el); apply(el); });
  }
  return { setEnabled, refresh: (el) => { sized.delete(el); apply(el); } };
})();
