const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

const ICONS = {
  look: '<circle cx="13.5" cy="6.5" r="1.5"/><circle cx="17.5" cy="10.5" r="1.5"/><circle cx="8.5" cy="7.5" r="1.5"/><circle cx="6.5" cy="12.5" r="1.5"/><path d="M12 2a10 10 0 1 0 0 20c1.1 0 2-.9 2-2 0-.5-.2-1-.5-1.3-.3-.4-.5-.8-.5-1.3 0-1.1.9-2 2-2h2.4A5.6 5.6 0 0 0 22 9.8C22 5.5 17.5 2 12 2Z"/>',
  general: '<circle cx="12" cy="12" r="3"/><path d="M12 2.5v3M12 18.5v3M4.6 4.6l2.1 2.1M17.3 17.3l2.1 2.1M2.5 12h3M18.5 12h3M4.6 19.4l2.1-2.1M17.3 6.7l2.1-2.1"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  link: '<path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7"/><path d="M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7"/>',
  import: '<path d="M12 3v12M7 10l5 5 5-5M5 21h14"/>',
  keys: '<rect x="2" y="6" width="20" height="12" rx="2"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10"/>',
  about: '<circle cx="12" cy="12" r="9"/><path d="M12 16v-4M12 8h.01"/>',
  x: '<path d="M18 6 6 18M6 6l12 12"/>',
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

let state = null;
let section = location.hash.slice(1) || 'look';

// Saving

function saveSettings(patch) {
  state.settings = { ...state.settings, ...patch };
  invoke('set_settings', { settings: state.settings });
}

function saveLook(patch) {
  saveSettings({ appearance: { ...appearance(), ...patch } });
}

// Form builders. Each returns a row; `get` reads the current value, `set` stores it.

function row(title, hint, control) {
  const r = el('div', 'opt');
  const t = el('div', 'txt', title);
  if (hint) t.append(el('small', '', hint));
  r.append(t, control);
  return r;
}

function toggle(title, hint, get, set) {
  const c = el('input', 'switch');
  c.type = 'checkbox';
  c.checked = !!get();
  c.onchange = () => set(c.checked);
  return row(title, hint, c);
}

function select(title, hint, options, get, set) {
  const c = el('select');
  for (const [value, label] of options) {
    const o = el('option', '', label);
    o.value = String(value);
    c.append(o);
  }
  c.value = String(get());
  c.onchange = () => set(typeof get() === 'number' ? Number(c.value) : c.value);
  return row(title, hint, c);
}

function segmented(title, hint, options, get, set) {
  const c = el('div', 'seg');
  for (const [value, label] of options) {
    const b = el('button', get() === value ? 'sel' : '', label);
    b.onclick = () => { set(value); [...c.children].forEach((x) => x.classList.toggle('sel', x === b)); };
    c.append(b);
  }
  return row(title, hint, c);
}

function range(title, hint, min, max, step, fmt, get, set) {
  const wrap = el('div', 'seg');
  wrap.style.background = 'none';
  const c = el('input');
  c.type = 'range';
  Object.assign(c, { min, max, step });
  c.value = get();
  const out = el('output', '', fmt(get()));
  c.oninput = () => { out.textContent = fmt(Number(c.value)); };
  c.onchange = () => set(Number(c.value));
  wrap.append(c, out);
  return row(title, hint, wrap);
}

function group(title, ...rows) {
  const g = el('section', 'group');
  g.append(el('h3', '', title), ...rows);
  return g;
}

function header(title, lead) {
  return [el('h1', '', title), el('p', 'lead', lead)];
}

// Sections

const appearance = () => lookOf(state.settings);
const S = (k) => () => state.settings[k];
const L = (k) => () => appearance()[k];
const setS = (k) => (v) => saveSettings({ [k]: v });
const setL = (k) => (v) => saveLook({ [k]: v });

const SECTIONS = {
  look: {
    title: 'Aparência',
    icon: 'look',
    render() {
      const presets = el('div', 'presets');
      for (const [key, p] of Object.entries(LOOK_PRESETS)) {
        const b = el('button', 'preset');
        b.append(el('b', '', p.title), el('span', '', p.hint));
        b.onclick = () => { saveSettings({ appearance: { ...LOOK_DEFAULTS, ...p.look } }); render(); };
        presets.append(b);
      }
      const reset = el('div', 'row-btns');
      const rb = el('button', 'btn danger', 'Restaurar tudo');
      rb.onclick = () => {
        if (!confirm('Restaurar todas as opções de aparência?')) return;
        saveSettings({ appearance: {}, sidebar_width: 264, default_zoom: 1, sidebar_right: false, floating_sidebar: false });
        render();
      };
      reset.append(rb);
      return [
        ...header('Aparência', 'Ajuste a barra lateral, as abas e as cores. As mudanças aparecem na hora.'),
        el('h3', 'lead', 'Começar de um preset'),
        presets,
        group('Janela e barra lateral',
          segmented('Lado da barra', null, [[false, 'Esquerda'], [true, 'Direita']], S('sidebar_right'), setS('sidebar_right')),
          range('Largura da barra', null, 220, 400, 4, (v) => `${v}px`, S('sidebar_width'), setS('sidebar_width')),
          toggle('Barra lateral flutuante', 'Esconde a barra; ela volta ao encostar o mouse na borda (Ctrl+Shift+S).', S('floating_sidebar'), setS('floating_sidebar')),
          toggle('Barra de título do Windows', 'Usa a moldura padrão em vez da janela sem bordas.', S('native_frame'), setS('native_frame')),
          toggle('Faixas heráldicas', 'Mostra o padrão do brasão no fundo da barra.', L('banner'), setL('banner'))),
        group('Abas',
          segmented('Tamanho', null, [[0.9, 'Compacto'], [1, 'Normal'], [1.1, 'Grande'], [1.2, 'Enorme']], L('tabScale'), setL('tabScale')),
          segmented('Formato', null, [['square', 'Reto'], ['rounded', 'Arredondado'], ['pill', 'Pílula']], L('tabShape'), setL('tabShape')),
          segmented('Aba selecionada', null, [['fill', 'Preenchida'], ['outline', 'Contorno'], ['glow', 'Brilho']], L('selected'), setL('selected')),
          toggle('Seguir a cor do Space', 'Tinge a aba selecionada com a cor de destaque.', L('followAccent'), setL('followAccent')),
          toggle('Apagar abas não carregadas', 'Abas dormindo aparecem mais claras.', L('dimUnloaded'), setL('dimUnloaded'))),
        group('Fixadas',
          select('Por linha', null, [[0, 'Linhas equilibradas'], [1, '1'], [2, '2'], [3, '3'], [4, '4'], [5, '5'], [6, '6']], L('pinsPerRow'), setL('pinsPerRow')),
          toggle('Esticar para preencher a linha', null, L('stretchPins'), setL('stretchPins')),
          toggle('Fundo nos fixados', null, L('pinFill'), setL('pinFill'))),
        group('Pastas',
          toggle('Mostrar quantidade de abas', null, L('folderCounts'), setL('folderCounts')),
          toggle('Ícone na cor do Space', null, L('folderAccent'), setL('folderAccent'))),
        group('Cores',
          toggle('Contorno colorido ao editar o endereço', null, L('addressOutline'), setL('addressOutline')),
          accentRow()),
        group('Páginas',
          select('Zoom padrão', 'Vale para abas novas e para as abertas.', [50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300].map((z) => [z / 100, `${z}%`]), S('default_zoom'), setS('default_zoom'))),
        reset,
      ];
    },
  },
  general: {
    title: 'Geral',
    icon: 'general',
    render() {
      const open = el('button', 'btn', 'Abrir Aplicativos padrão');
      open.onclick = () => invoke('open_default_apps');
      return [
        ...header('Geral', 'Memória, inicialização e navegador padrão.'),
        group('Abas',
          select('Dormir abas em segundo plano após', 'Libera memória; a aba recarrega quando você volta. Fixadas nunca dormem.',
            [[0, 'Nunca'], [15, '15 minutos'], [30, '30 minutos'], [60, '1 hora'], [240, '4 horas']], S('sleep_minutes'), setS('sleep_minutes'))),
        group('Navegador padrão',
          row('Usar o Crest para links', 'Escolha "Crest" em Navegador da Web nos Aplicativos padrão do Windows.', open)),
      ];
    },
  },
  search: {
    title: 'Busca',
    icon: 'search',
    render() {
      const engines = [
        ['https://duckduckgo.com/?q=', 'DuckDuckGo'],
        ['https://www.google.com/search?q=', 'Google'],
        ['https://www.bing.com/search?q=', 'Bing'],
        ['https://search.brave.com/search?q=', 'Brave Search'],
        ['https://www.startpage.com/do/search?q=', 'Startpage'],
        ['https://www.ecosia.org/search?q=', 'Ecosia'],
        ['https://kagi.com/search?q=', 'Kagi'],
      ];
      const known = engines.some(([u]) => u === state.settings.search);
      const custom = el('input');
      custom.type = 'url';
      custom.placeholder = 'https://exemplo.com/busca?q=';
      custom.value = known ? '' : state.settings.search;
      custom.onchange = () => { if (/^https?:\/\//.test(custom.value)) saveSettings({ search: custom.value }); };
      return [
        ...header('Busca', 'O que a barra de endereço usa quando você digita algo que não é um site.'),
        group('Mecanismo',
          select('Busca padrão', null, [...engines, ...(known ? [] : [[state.settings.search, 'Personalizada']])], S('search'), (v) => { saveSettings({ search: v }); render(); }),
          row('Endereço personalizado', 'A pesquisa é colocada no fim do endereço.', custom)),
      ];
    },
  },
  links: {
    title: 'Links',
    icon: 'link',
    render() {
      const spaces = state.spaces.map((s) => [s.id, s.name]);
      const routes = el('div');
      const drawRoutes = () => {
        routes.replaceChildren();
        (state.settings.routes || []).forEach((r, i) => {
          const pattern = el('input');
          pattern.type = 'text';
          pattern.placeholder = 'ex.: figma.com';
          pattern.value = r.pattern;
          const space = el('select');
          for (const [id, name] of spaces) {
            const o = el('option', '', name);
            o.value = id;
            space.append(o);
          }
          space.value = r.space;
          const del = el('button', 'btn danger', 'Remover');
          const save = () => {
            const list = [...state.settings.routes];
            list[i] = { pattern: pattern.value.trim(), space: space.value };
            saveSettings({ routes: list });
          };
          pattern.onchange = save;
          space.onchange = save;
          del.onclick = () => { saveSettings({ routes: state.settings.routes.filter((_, j) => j !== i) }); drawRoutes(); };
          const wrap = el('div', 'seg');
          wrap.style.background = 'none';
          wrap.append(space, del);
          const r2 = el('div', 'opt');
          r2.append(pattern, wrap);
          pattern.style.flex = '1';
          routes.append(r2);
        });
        if (!(state.settings.routes || []).length) routes.append(el('p', 'lead', 'Nenhuma rota ainda.'));
      };
      drawRoutes();
      const add = el('button', 'btn', 'Nova rota');
      add.onclick = () => { saveSettings({ routes: [...(state.settings.routes || []), { pattern: '', space: state.active_space }] }); drawRoutes(); };
      const addRow = el('div', 'row-btns');
      addRow.append(add);
      const routesGroup = group('Rotas', routes, addRow);
      return [
        ...header('Links', 'Como o Crest abre links de outros apps e prévias rápidas.'),
        group('Links de outros apps',
          select('Abrir em', 'Quando nenhuma rota combina.', [['', 'Space atual'], ...spaces], S('external_space'), setS('external_space'))),
        routesGroup,
        group('Peek',
          select('Abrir prévia com', 'Clique num link segurando a tecla para ver numa janela flutuante. Esc fecha.', [['shift', 'Shift + clique'], ['alt', 'Alt + clique'], ['off', 'Desligado']], S('peek_modifier'), setS('peek_modifier'))),
      ];
    },
  },
  import: {
    title: 'Importar',
    icon: 'import',
    render() {
      const g = group('Navegadores encontrados');
      invoke('import_sources').then((sources) => {
        if (!sources.length) g.append(row('Nenhum navegador compatível encontrado', 'Arc, Chrome, Edge, Brave e Vivaldi são suportados.', el('span')));
        for (const s of sources) {
          const b = el('button', 'btn', 'Importar');
          b.onclick = async () => {
            const what = s.id === 'arc' ? 'Cada Space do Arc vira um Space novo.' : 'Os favoritos entram como pastas no Space atual.';
            if (!confirm(`Importar de ${s.name}? ${what}`)) return;
            b.disabled = true;
            try { alert(await invoke('import_browser', { id: s.id })); } catch (err) { alert(`Não deu para importar: ${err}`); }
            b.disabled = false;
          };
          g.append(row(s.name, null, b));
        }
      });
      return [...header('Importar', 'Traga Spaces, pastas e favoritos de outro navegador. As abas importadas começam dormindo.'), g];
    },
  },
  keys: {
    title: 'Atalhos',
    icon: 'keys',
    render() {
      const keys = [
        ['Nova aba', 'Ctrl+T'], ['Fechar aba', 'Ctrl+W'], ['Reabrir aba fechada', 'Ctrl+Shift+T'], ['Focar o endereço', 'Ctrl+L'],
        ['Próxima / anterior', 'Ctrl+Tab / Ctrl+Shift+Tab'], ['Ir para a aba', 'Ctrl+1 … 9'], ['Trocar de Space', 'Ctrl+Shift+1 … 9'],
        ['Histórico', 'Ctrl+H'], ['Paleta de comandos', 'Ctrl+K'], ['Peek (prévia do link)', 'Shift+clique'], ['Mostrar ou esconder a barra', 'Ctrl+Shift+S'], ['Voltar / avançar', 'Alt+← / Alt+→'],
        ['Recarregar', 'F5'], ['Zoom', 'Ctrl + / Ctrl − / Ctrl+0'], ['Buscar na página', 'Ctrl+F'], ['Ferramentas do desenvolvedor', 'F12'],
      ];
      const t = el('table', 'keys');
      for (const [what, k] of keys) {
        const tr = el('tr');
        const kd = el('td');
        kd.append(el('kbd', '', k));
        tr.append(el('td', '', what), kd);
        t.append(tr);
      }
      const g = el('section', 'group');
      g.append(t);
      return [...header('Atalhos', 'Funcionam na barra lateral e dentro das páginas.'), g];
    },
  },
  about: {
    title: 'Sobre',
    icon: 'about',
    render() {
      const p = (text) => el('p', 'lead', text);
      return [
        ...header('Crest para Windows', 'Um navegador leve com Spaces, feito em Rust com Tauri e o WebView2 do Windows.'),
        p('Inspirado no Crest de Paul Joda (github.com/pauljoda/Crest), licença MPL-2.0. Brasões e paletas do Crest Studio vêm do projeto original; figuras heráldicas de Lorc (game-icons.net), CC BY 3.0.'),
      ];
    },
  },
};

function accentRow() {
  const wrap = el('div', 'seg');
  wrap.style.background = 'none';
  const on = el('input', 'switch');
  on.type = 'checkbox';
  on.checked = !!appearance().accent;
  const color = el('input');
  color.type = 'color';
  color.value = appearance().accent || '#86adc4';
  color.disabled = !on.checked;
  on.onchange = () => { color.disabled = !on.checked; saveLook({ accent: on.checked ? color.value : '' }); };
  color.onchange = () => saveLook({ accent: color.value });
  wrap.append(color, on);
  return row('Cor de destaque própria', 'Substitui a cor do Space nas abas, botões e contornos.', wrap);
}

// Rendering

function applySpaceLook() {
  const space = state.spaces.find((s) => s.id === state.active_space);
  if (!space) return;
  const { colors } = look(space);
  colors.forEach((c, i) => document.documentElement.style.setProperty(`--c${i}`, c));
}

function render() {
  if (!state) return;
  applySpaceLook();
  const nav = $('nav');
  nav.querySelectorAll('button').forEach((b) => b.remove());
  for (const [id, s] of Object.entries(SECTIONS)) {
    const b = el('button', id === section ? 'sel' : '');
    b.append(icon(s.icon), document.createTextNode(s.title));
    b.onclick = () => { section = id; render(); };
    nav.append(b);
  }
  const main = $('main');
  const top = main.scrollTop;
  main.replaceChildren(...(SECTIONS[section] || SECTIONS.look).render());
  main.scrollTop = top;
}

$('close').append(icon('x'));
$('close').onclick = () => invoke('close_panel');
addEventListener('keydown', (e) => { if (e.key === 'Escape') invoke('close_panel'); });
listen('state', (e) => { const first = !state; state = e.payload; if (first) render(); else applySpaceLook(); });
listen('section', (e) => { section = e.payload; render(); });
invoke('get_state').then((s) => { state = s; render(); });
