// The start page has no IPC; the app calls window.crestStart(data) after load.
const $ = (id) => document.getElementById(id);

function greeting() {
  const h = new Date().getHours();
  const day = new Intl.DateTimeFormat('pt-BR', { weekday: 'long', day: 'numeric', month: 'long' }).format(new Date());
  return `${h < 5 ? 'Boa noite' : h < 12 ? 'Bom dia' : h < 18 ? 'Boa tarde' : 'Boa noite'} · ${day}`;
}

function host(url) {
  try { return new URL(url).hostname.replace(/^www\./, ''); } catch { return ''; }
}

function favicon(url, label) {
  const box = document.createElement('span');
  box.className = 'fav';
  box.textContent = (label[0] || '•').toUpperCase();
  const img = new Image();
  img.onload = () => { box.textContent = ''; box.style.background = 'none'; box.append(img); };
  img.src = `https://icons.duckduckgo.com/ip3/${encodeURIComponent(new URL(url).hostname)}.ico`;
  return box;
}

window.crestStart = ({ space, search, sites }) => {
  if (space) {
    const { colors, pattern } = look(space);
    colors.forEach((c, i) => document.documentElement.style.setProperty(`--c${i}`, c));
    const banner = $('banner');
    banner.className = `banner ${pattern}` + (TILED.includes(pattern) ? ' tiles' : '');
    banner.style.backgroundImage = tileImage(pattern, colors);
    $('name').textContent = space.name;
    const emblem = $('emblem');
    emblem.replaceChildren();
    if (HOUSES[space.house]) {
      const img = new Image();
      img.src = `crests/${space.house}.svg`;
      img.alt = '';
      emblem.append(img);
    } else {
      emblem.innerHTML = `<svg viewBox="10 12 80 80"><path d="${SHIELD}" fill="${colors[2]}" stroke="${colors[0]}" stroke-width="3"/></svg>`;
    }
  }
  // The search setting is a URL prefix ending in "q=" (or similar); split it into a GET form.
  try {
    const u = new URL(search);
    const param = [...u.searchParams.keys()].pop() || 'q';
    u.searchParams.delete(param);
    $('search').action = u.origin + u.pathname;
    $('q').name = param;
  } catch {}
  const nav = $('sites');
  nav.replaceChildren();
  for (const s of sites) {
    const a = document.createElement('a');
    a.className = 'site';
    a.href = s.url;
    a.title = s.url;
    const label = s.title || host(s.url);
    const name = document.createElement('span');
    name.textContent = host(s.url) || label;
    a.append(favicon(s.url, label), name);
    nav.append(a);
  }
};

$('greet').textContent = greeting();
$('q').focus();
