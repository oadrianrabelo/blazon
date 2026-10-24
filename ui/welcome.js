// Welcome flow on first run, after Crest's setup window: welcome, appearance,
// Spaces, bringing data from other browsers, and done. Runs in the overlay,
// which covers the whole window. Uses overlay.js's invoke, $, el and state.

const WELCOME_EXTRA_SPACES = [['Trabalho', 'lion'], ['Estudos', 'river'], ['Pesquisa', 'meadow']];

function startWelcome() {
  document.body.classList.add('welcome-mode');
  const stage = $('welcome');
  stage.hidden = false;

  // Backdrop: the houses' fields take turns, slowly cross-fading
  // (CrestMotion.onboardingBackdrop 1.6s, onboardingWelcomeHold 7s).
  const fields = [el('div', 'w-field'), el('div', 'w-field')];
  stage.prepend(...fields);
  let front = 0;
  let cycling = true;
  const houseKeys = Object.keys(HOUSES);
  const paint = (node, house) => {
    const h = HOUSES[house];
    node.replaceChildren();
    const b = el('div', `banner ${h.pattern}` + (TILED.includes(h.pattern) ? ' tiles' : ''));
    b.append(el('div', 'p-mid'), el('div', 'p-low'));
    b.style.backgroundImage = tileImage(h.pattern, h.colors);
    h.colors.forEach((c, i) => node.style.setProperty(`--c${i}`, c));
    node.append(b);
  };
  const show = (house) => {
    front = 1 - front;
    paint(fields[front], house);
    fields[front].classList.add('on');
    fields[1 - front].classList.remove('on');
    HOUSES[house].colors.forEach((c, i) => document.documentElement.style.setProperty(`--c${i}`, c));
  };
  let turn = 0;
  const space = () => state.spaces.find((s) => s.id === state.active_space) || state.spaces[0];
  show(space()?.house || houseKeys[0]);
  setInterval(() => { if (cycling) show(houseKeys[++turn % houseKeys.length]); }, 7000);

  const card = el('div', 'w-card');
  const rail = el('div', 'w-rail');
  const body = el('div', 'w-body');
  card.append(rail, body);
  stage.append(card);

  const refresh = async () => { state = await invoke('get_state'); };
  const houseLook = (k) => ({ crest: crestOf({ house: k }), sidebar: { colors: [...HOUSES[k].colors], pattern: HOUSES[k].pattern } });
  const crestNode = (spaceLike) => {
    const t = document.createElement('template');
    t.innerHTML = renderCrest(crestOf(spaceLike));
    return t.content.firstChild;
  };
  const button = (label, cls, run) => { const b = el('button', `w-btn ${cls}`, label); b.onclick = run; return b; };
  const actions = (...buttons) => { const a = el('div', 'w-actions'); a.append(...buttons); return a; };

  const steps = [
    // Welcome
    () => {
      const emblem = el('div', 'w-emblem');
      emblem.append(crestNode(space() || { house: 'winter' }));
      return [emblem, el('h1', '', 'Bem-vindo ao Blazon'), el('p', 'w-lead', 'A internet nas suas cores.'),
        actions(button('Começar', 'primary', next))];
    },
    // 01 — Appearance
    () => {
      const grid = el('div', 'w-houses');
      for (const [k, h] of Object.entries(HOUSES)) {
        const b = el('button', 'w-house' + (space()?.house === k ? ' sel' : ''));
        b.append(crestNode({ house: k }), el('span', '', h.title));
        b.onclick = async () => {
          cycling = false;
          show(k);
          const s = space();
          await invoke('update_space', { id: s.id, name: s.name, color: HOUSES[k].colors[2], house: k, look: houseLook(k) });
          await refresh();
          grid.querySelectorAll('.sel').forEach((n) => n.classList.remove('sel'));
          b.classList.add('sel');
        };
        grid.append(b);
      }
      return [el('p', 'w-eyebrow', '01 — Aparência'), el('h2', '', 'Escolha a casa do seu Space'),
        el('p', 'w-lead', 'O brasão e as cores da barra lateral vêm juntos. Formato, campo, faixa, borda e acabamento ficam em Configurações › Spaces.'),
        grid, actions(button('Voltar', '', back), button('Continuar', 'primary', next))];
    },
    // 02 — Your Spaces
    () => {
      const s = space();
      const name = el('input', 'w-input');
      name.value = s.name;
      name.placeholder = 'Pessoal';
      name.onchange = async () => {
        const v = name.value.trim() || I18N.t('Pessoal');
        await invoke('update_space', { id: s.id, name: v, color: s.color, house: s.house, look: s.look });
        await refresh();
      };
      const list = el('div', 'w-spaces');
      const draw = () => {
        list.replaceChildren();
        for (const sp of state.spaces.filter((x) => !x.private)) {
          const item = el('div', 'w-space');
          item.append(crestNode(sp), el('span', '', sp.name));
          list.append(item);
        }
        for (const [pt, house] of WELCOME_EXTRA_SPACES) {
          // Spaces are created with a name in the interface's language.
          const label = I18N.t(pt);
          if (state.spaces.some((x) => x.name === label)) continue;
          const add = el('button', 'w-space add');
          add.append(el('span', 'plus', '+'), el('span', '', label));
          add.onclick = async () => {
            await invoke('add_space', { name: label, color: HOUSES[house].colors[2], private: false, house, look: houseLook(house) });
            await refresh();
            // Adding a Space switches to it; stay on the first one here.
            await invoke('switch_space', { id: s.id }).catch(() => {});
            await refresh();
            draw();
          };
          list.append(add);
        }
      };
      draw();
      const label = el('label', 'w-field-label', 'Nome do seu Space');
      label.append(name);
      return [el('p', 'w-eyebrow', '02 — Seus Spaces'), el('h2', '', 'Um Space para cada parte da sua vida'),
        el('p', 'w-lead', 'Cada Space tem as próprias abas, logins e histórico. Acrescente outros agora ou depois.'),
        label, list, actions(button('Voltar', '', back), button('Continuar', 'primary', next))];
    },
    // 03 — Bring your data
    () => {
      const list = el('div', 'w-imports');
      list.append(el('p', 'w-lead', 'Procurando outros navegadores…'));
      invoke('import_sources').then((sources) => {
        list.replaceChildren();
        if (!sources.length) list.append(el('p', 'w-lead', 'Nenhum outro navegador encontrado. Dá para importar um arquivo HTML de favoritos em Configurações › Importar.'));
        for (const src of sources) {
          const row = el('div', 'w-import');
          row.append(el('span', '', src.name));
          const go = button('Importar', '', async () => {
            go.disabled = true;
            go.textContent = 'Importando…';
            try {
              const msg = await invoke('import_browser', { id: src.id, bookmarks: true, history: src.kinds.includes('history'), newSpace: null });
              go.textContent = 'Importado';
              if (typeof msg === 'string') row.title = msg;
            } catch (err) {
              go.textContent = 'Falhou';
              row.title = String(err);
            }
            await refresh();
          });
          row.append(go);
          list.append(row);
        }
      });
      return [el('p', 'w-eyebrow', '03 — Seus dados'), el('h2', '', 'Traga o que você já tem'),
        el('p', 'w-lead', 'Favoritos e histórico vão para o Space atual. Do Arc, vêm os Spaces e as pastas.'),
        list, actions(button('Voltar', '', back), button('Continuar', 'primary', next))];
    },
    // Done
    () => {
      const row = el('div', 'w-spaces');
      for (const sp of state.spaces.filter((x) => !x.private)) {
        const item = el('div', 'w-space');
        item.append(crestNode(sp), el('span', '', sp.name));
        row.append(item);
      }
      return [el('h1', '', 'Seus Spaces estão prontos'), row,
        el('p', 'w-lead', 'Ctrl+K abre a paleta de comandos. Shift+clique num link abre o Peek. Tudo se ajusta em Configurações.'),
        actions(button('Tornar navegador padrão', '', () => invoke('open_default_apps')), button('Abrir o Blazon', 'primary', finish))];
    },
  ];

  let index = 0;
  function draw(direction = 0) {
    rail.replaceChildren(...steps.map((_, i) => el('i', i === index ? 'on' : i < index ? 'done' : '')));
    body.replaceChildren(...steps[index]());
    if (direction) {
      const dx = Motion.reduced.matches ? 0 : 28 * direction;
      body.animate([{ opacity: 0, transform: `translateX(${dx}px)` }, { opacity: 1, transform: 'none' }],
        { duration: Motion.ms('--step'), easing: Motion.ease('--snappy') });
    }
    body.querySelector('.primary')?.focus();
  }
  function next() { if (index < steps.length - 1) { index++; draw(1); } }
  function back() { if (index > 0) { index--; draw(-1); } }
  async function finish() {
    await refresh();
    await invoke('set_settings', { settings: { ...state.settings, welcomed: true } });
    stage.animate([{ opacity: 1 }, { opacity: 0, transform: `scale(${Motion.reduced.matches ? 1 : 1.02})` }],
      { duration: Motion.ms('--chrome'), easing: Motion.ease('--smooth'), fill: 'forwards' }).onfinish = shut;
  }
  draw();
  card.animate([{ opacity: 0, transform: `translateY(${Motion.reduced.matches ? 0 : 16}px)` }, { opacity: 1, transform: 'none' }],
    { duration: Motion.ms('--chrome'), easing: Motion.ease('--smooth') });
}
