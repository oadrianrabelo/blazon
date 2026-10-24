// Motion helpers for Crest-style transitions (see motion.css for the tokens).
// Lists are rebuilt on every state change, so movement is animated FLIP-style:
// remember where keyed nodes were, rebuild, then play the difference.
const Motion = (() => {
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  const css = getComputedStyle(document.documentElement);
  const token = (name) => css.getPropertyValue(name).trim();
  const ms = (name) => parseFloat(token(name)) * 1000;
  const ease = (name) => token(name) || 'ease-out';

  // Positions of every [data-key] node under the given roots.
  function snapshot(...roots) {
    const map = new Map();
    for (const root of roots) {
      for (const node of root.querySelectorAll('[data-key]')) {
        const r = node.getBoundingClientRect();
        if (r.width) map.set(node.dataset.key, { node, rect: r });
      }
    }
    return map;
  }

  // Moves surviving nodes from their old place, grows new ones in and fades
  // removed ones out from where they were (CrestMotion.collection).
  function play(before, roots, layer) {
    const seen = new Set();
    const duration = ms('--collection');
    for (const root of roots) {
      for (const node of root.querySelectorAll('[data-key]')) {
        const key = node.dataset.key;
        seen.add(key);
        const old = before.get(key);
        const now = node.getBoundingClientRect();
        if (!now.width) continue;
        if (!old) {
          node.animate([{ opacity: 0, transform: `scale(${reduced.matches ? 1 : 0.96})` }, { opacity: 1, transform: 'none' }],
            { duration, easing: ease('--snappy') });
          continue;
        }
        if (reduced.matches) continue;
        const dx = old.rect.left - now.left;
        const dy = old.rect.top - now.top;
        const sx = old.rect.width / now.width;
        const sy = old.rect.height / now.height;
        if (Math.abs(dx) < 1 && Math.abs(dy) < 1 && Math.abs(sx - 1) < 0.02 && Math.abs(sy - 1) < 0.02) continue;
        node.animate([{ transformOrigin: 'top left', transform: `translate(${dx}px, ${dy}px) scale(${sx}, ${sy})` }, { transformOrigin: 'top left', transform: 'none' }],
          { duration, easing: ease('--snappy') });
      }
    }
    if (!layer) return;
    const box = layer.getBoundingClientRect();
    for (const [key, { node, rect }] of before) {
      if (seen.has(key)) continue;
      node.style.cssText += `;position:absolute;margin:0;left:${rect.left - box.left}px;top:${rect.top - box.top}px;width:${rect.width}px;height:${rect.height}px;pointer-events:none;z-index:3`;
      node.removeAttribute('data-key');
      layer.append(node);
      node.animate([{ opacity: 1, transform: 'none' }, { opacity: 0, transform: `scale(${reduced.matches ? 1 : 0.96})` }],
        { duration: ms('--dismissal'), easing: 'ease-out', fill: 'forwards' }).onfinish = () => node.remove();
    }
  }

  // The selection highlight glides from the previous row to the new one.
  function glide(from, to, layer, look = {}) {
    if (reduced.matches || !from || !to.width) return;
    const box = layer.getBoundingClientRect();
    const g = document.createElement('div');
    g.className = 'm-glide';
    Object.assign(g.style, look);
    g.style.cssText += `;left:${to.left - box.left}px;top:${to.top - box.top}px;width:${to.width}px;height:${to.height}px`;
    layer.append(g);
    const dx = from.left - to.left;
    const dy = from.top - to.top;
    g.style.transformOrigin = 'top left';
    g.animate([{ transform: `translate(${dx}px, ${dy}px) scale(${from.width / to.width}, ${from.height / to.height})`, opacity: 1 },
      { transform: 'none', opacity: 1, offset: 0.85 }, { transform: 'none', opacity: 0 }],
    { duration: ms('--feedback') * 1.2, easing: ease('--snappy') }).onfinish = () => g.remove();
  }

  // Pages of the Space pager: the new Space slides in from the side it lives on.
  function swipe(nodes, direction) {
    const offset = reduced.matches ? 0 : 36 * direction;
    for (const node of nodes) {
      node.animate([{ opacity: 0, transform: `translateX(${offset}px)` }, { opacity: 1, transform: 'none' }],
        { duration: ms('--space-swipe'), easing: ease('--snappy') });
    }
  }

  // Shows a floating pane (menus, toasts) or hides it after its exit animation.
  function show(node, kind = 'pane') {
    node.classList.remove('m-leaving');
    node.getAnimations().forEach((a) => a.cancel());
    node.hidden = false;
    node.animate(kind === 'rise'
      ? [{ opacity: 0, transform: `translateY(${reduced.matches ? 0 : 12}px)` }, { opacity: 1, transform: 'none' }]
      : [{ opacity: 0, transform: `scale(${reduced.matches ? 1 : 0.985})` }, { opacity: 1, transform: 'none' }],
    { duration: ms('--floating-pane'), easing: ease('--smooth') });
  }
  // Opens a floating pane out of the control (or point) that summoned it: the
  // shape grows from there and the content fades in once it has room, like a
  // Liquid Glass button turning into its menu.
  function morphFrom(node, from) {
    node.classList.remove('m-leaving');
    node.getAnimations().forEach((a) => a.cancel());
    node.hidden = false;
    if (reduced.matches || !from) return show(node);
    const to = node.getBoundingClientRect();
    if (!to.width || !to.height) return show(node);
    const duration = ms('--floating-pane');
    node.animate([
      { transformOrigin: 'top left', transform: `translate(${from.left - to.left}px, ${from.top - to.top}px) scale(${from.width / to.width}, ${from.height / to.height})`, opacity: 0.4 },
      { transformOrigin: 'top left', transform: 'none', opacity: 1 },
    ], { duration, easing: ease('--smooth') });
    for (const child of node.children) {
      child.animate([{ opacity: 0 }, { opacity: 0, offset: 0.3 }, { opacity: 1 }], { duration });
    }
  }

  function hide(node, done) {
    if (node.hidden || node.classList.contains('m-leaving')) { if (done) done(); return; }
    node.classList.add('m-leaving');
    node.animate([{ opacity: 1 }, { opacity: 0, transform: `scale(${reduced.matches ? 1 : 0.985})` }],
      { duration: ms('--dismissal'), easing: 'ease-out' }).onfinish = () => {
      if (!node.classList.contains('m-leaving')) return;
      node.classList.remove('m-leaving');
      node.hidden = true;
      if (done) done();
    };
  }

  // Replays a one-shot CSS animation class (reload turn, bounce, shake).
  // Content of a newly chosen page or step settles in (CrestMotion.step).
  function enter(node) {
    node.animate([{ opacity: 0, transform: `translateY(${reduced.matches ? 0 : 8}px)` }, { opacity: 1, transform: 'none' }],
      { duration: ms('--step'), easing: ease('--snappy') });
  }

  function pulse(node, cls) {
    node.classList.remove(cls);
    void node.offsetWidth;
    node.classList.add(cls);
    node.addEventListener('animationend', () => node.classList.remove(cls), { once: true });
  }

  return { reduced, ms, ease, snapshot, play, glide, swipe, show, hide, enter, pulse, morphFrom };
})();
