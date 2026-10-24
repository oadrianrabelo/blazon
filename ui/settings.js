const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

const ICONS = {
  look: '<circle cx="13.5" cy="6.5" r="1.5"/><circle cx="17.5" cy="10.5" r="1.5"/><circle cx="8.5" cy="7.5" r="1.5"/><circle cx="6.5" cy="12.5" r="1.5"/><path d="M12 2a10 10 0 1 0 0 20c1.1 0 2-.9 2-2 0-.5-.2-1-.5-1.3-.3-.4-.5-.8-.5-1.3 0-1.1.9-2 2-2h2.4A5.6 5.6 0 0 0 22 9.8C22 5.5 17.5 2 12 2Z"/>',
  general: '<circle cx="12" cy="12" r="3"/><path d="M12 2.5v3M12 18.5v3M4.6 4.6l2.1 2.1M17.3 17.3l2.1 2.1M2.5 12h3M18.5 12h3M4.6 19.4l2.1-2.1M17.3 6.7l2.1-2.1"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  spaces: '<path d="M4 4h16v6c0 6-4 9-8 10-4-1-8-4-8-10Z"/>',
  privacy: '<path d="M12 3 5 6v5c0 4.5 3 8.3 7 10 4-1.7 7-5.5 7-10V6Z"/>',
  link: '<path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7"/><path d="M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7"/>',
  import: '<path d="M12 3v12M7 10l5 5 5-5M5 21h14"/>',
  keys: '<rect x="2" y="6" width="20" height="12" rx="2"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10"/>',
  puzzle: '<path d="M10 3a2 2 0 0 1 4 0v2h4v4h2a2 2 0 0 1 0 4h-2v6h-6v-2a2 2 0 0 0-4 0v2H4v-6h2a2 2 0 0 0 0-4H4V5h6Z"/>',
  bug: '<path d="M8 6a4 4 0 0 1 8 0v1H8Z"/><rect x="6" y="7" width="12" height="13" rx="6"/><path d="M12 11v9M3 13h3M18 13h3M4 7l2.5 2M20 7l-2.5 2M4 20l2.5-2.5M20 20l-2.5-2.5"/>',
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

// Repository for problem reports (GitHub redirects it if the repository is renamed).
const REPO_URL = 'https://github.com/oadrianrabelo/crest-windows-vibe-coded';
let reportDraft = '';
const SEARCH_ENGINES = [
  ['https://duckduckgo.com/?q=', 'DuckDuckGo'],
  ['https://www.google.com/search?q=', 'Google'],
  ['https://www.bing.com/search?q=', 'Bing'],
  ['https://search.brave.com/search?q=', 'Brave Search'],
  ['https://www.startpage.com/do/search?q=', 'Startpage'],
  ['https://www.ecosia.org/search?q=', 'Ecosia'],
  ['https://kagi.com/search?q=', 'Kagi'],
];
let state = null;
let section = decodeURIComponent(location.hash.slice(1)) || 'look';

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
  spaces: {
    title: 'Spaces e Studio',
    icon: 'spaces',
    render: () => renderStudio(),
  },
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
        group('Idioma',
          select('Idioma da interface', null, [['auto', 'Automático'], ['pt', 'Português'], ['en', 'English']], () => state.settings.language || 'auto', setS('language'))),
        group('Abas',
          select('Dormir abas em segundo plano após', 'Libera memória; a aba recarrega quando você volta. Fixadas nunca dormem.',
            [[0, 'Nunca'], [15, '15 minutos'], [30, '30 minutos'], [60, '1 hora'], [240, '4 horas']], S('sleep_minutes'), setS('sleep_minutes'))),
        group('Arquivo',
          select('Arquivar abas paradas após', 'Abas soltas sem uso vão para o Arquivo (botão no rodapé da barra). Fixadas e abas em pastas ficam.',
            [[0, 'Nunca'], [12, '12 horas'], [24, '1 dia'], [72, '3 dias'], [168, '1 semana']], S('archive_hours'), setS('archive_hours'))),
        group('Navegador padrão',
          row('Usar o Blazon para links', 'Escolha "Blazon" em Navegador da Web nos Aplicativos padrão do Windows.', open)),
      ];
    },
  },
  search: {
    title: 'Busca',
    icon: 'search',
    render() {
      const engines = SEARCH_ENGINES;
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
  privacy: {
    title: 'Privacidade',
    icon: 'privacy',
    render() {
      const allow = el('div');
      const drawAllow = () => {
        allow.replaceChildren();
        const list = state.settings.adblock_allow || [];
        if (!list.length) allow.append(el('p', 'lead', 'Nenhum site pausado. Use o escudo na barra lateral para pausar num site.'));
        list.forEach((h) => {
          const b = el('button', 'btn danger', 'Retomar');
          b.onclick = () => { saveSettings({ adblock_allow: state.settings.adblock_allow.filter((x) => x !== h) }); drawAllow(); };
          allow.append(row(h, null, b));
        });
      };
      drawAllow();
      const downloads = group('Downloads por Space');
      for (const s of state.spaces) {
        const wrap = el('div', 'seg');
        wrap.style.background = 'none';
        const pick = el('button', 'btn', 'Escolher pasta…');
        pick.onclick = async () => {
          const path = await invoke('pick_folder');
          if (path) { await invoke('set_space_downloads', { id: s.id, path }); s.downloads = path; render(); }
        };
        wrap.append(pick);
        if (s.downloads) {
          const clear = el('button', 'btn danger', 'Padrão');
          clear.onclick = async () => { await invoke('set_space_downloads', { id: s.id, path: '' }); s.downloads = ''; render(); };
          wrap.append(clear);
        }
        downloads.append(row(s.name, s.downloads || 'Pasta Downloads do Windows', wrap));
      }
      return [
        ...header('Privacidade', 'Bloqueio de anúncios e rastreadores, senhas e downloads.'),
        group('Bloqueador',
          toggle('Bloquear anúncios e rastreadores', 'Usa as listas EasyList e EasyPrivacy, atualizadas toda semana.', S('adblock'), setS('adblock'))),
        group('Sites com o bloqueador pausado', allow),
        group('Senhas e formulários',
          toggle('Oferecer para salvar senhas e preencher formulários', 'Guardado pelo próprio WebView2, separado por Space. Vale para abas novas.', S('passwords'), setS('passwords'))),
        downloads,
        permissionsGroup(),
      ];
    },
  },
  extensions: {
    title: 'Extensões',
    icon: 'puzzle',
    render() {
      let space = state.active_space;
      const list = el('div');
      const load = async () => {
        list.replaceChildren(el('p', 'lead', 'Carregando…'));
        try {
          const exts = await invoke('list_extensions', { space });
          list.replaceChildren();
          if (!exts.length) list.append(el('p', 'lead', 'Nenhuma extensão neste Space.'));
          for (const x of exts) {
            const wrap = el('div', 'seg');
            wrap.style.background = 'none';
            const on = el('input', 'switch');
            on.type = 'checkbox';
            on.checked = x.enabled;
            on.onchange = () => invoke('change_extension', { space, id: x.id, action: on.checked ? 'enable' : 'disable' });
            const del = el('button', 'btn danger', 'Remover');
            del.onclick = async () => { if (confirm(`Remover "${x.name}"?`)) { await invoke('change_extension', { space, id: x.id, action: 'remove' }); load(); } };
            wrap.append(on, del);
            list.append(row(x.name, x.id, wrap));
          }
        } catch (err) { list.replaceChildren(el('p', 'lead', String(err))); }
      };
      const add = el('button', 'btn primary', 'Adicionar extensão (pasta)…');
      add.onclick = async () => {
        try { alert(`Extensão instalada: ${await invoke('add_extension', { space })}. Recarregue as abas para usar.`); load(); } catch (err) { alert(String(err)); }
      };
      load();
      return [
        ...header('Extensões', 'Extensões descompactadas (uma pasta com manifest.json), instaladas por Space. O WebView2 não acessa a Chrome Web Store e suporta só parte das APIs de extensão.'),
        group('Space', row('Instalar e gerenciar em', null, spacePicker((v) => { space = v; load(); }))),
        group('Instaladas', list, (() => { const r = el('div', 'row-btns'); r.append(add); return r; })()),
        group('Como conseguir uma extensão descompactada',
          row('Do código-fonte', 'Muitas extensões publicam a pasta pronta no GitHub (procure por "unpacked" ou baixe o .zip da release e extraia).', el('span')),
          row('De um .crx', 'Um .crx é um .zip com cabeçalho: renomeie para .zip, extraia e escolha a pasta.', el('span'))),
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
        ...header('Links', 'Como o Blazon abre links de outros apps e prévias rápidas.'),
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
      const KIND = { spaces: 'Spaces, pastas e favoritos', bookmarks: 'Favoritos', history: 'Histórico' };
      const found = group('Navegadores encontrados neste PC');
      const run = async (button, call) => {
        button.disabled = true;
        const old = button.textContent;
        button.textContent = 'Importando…';
        try { alert(await call()); } catch (err) { alert(`Não deu para importar: ${err}`); }
        button.disabled = false;
        button.textContent = old;
      };
      const target = (name) => {
        const sel = el('select');
        for (const [v, l] of [['current', 'No Space atual'], ['new', `Num Space novo "${name}"`]]) {
          const o = el('option', '', l);
          o.value = v;
          sel.append(o);
        }
        return sel;
      };
      invoke('import_sources').then((sources) => {
        if (!sources.length) found.append(row('Nenhum navegador encontrado', 'Procuro Arc, Chrome, Edge, Brave, Vivaldi, Opera, Chromium, Firefox, Zen, LibreWolf, Waterfox e Floorp.', el('span')));
        for (const s of sources) {
          const box = el('div', 'opt');
          box.style.flexWrap = 'wrap';
          const txt = el('div', 'txt', s.name);
          txt.append(el('small', '', s.kinds.map((k) => KIND[k]).join(' · ')));
          const checks = el('div', 'seg');
          checks.style.background = 'none';
          const pick = {};
          for (const k of s.kinds) {
            const l = el('label', 'check-inline');
            const c = el('input');
            c.type = 'checkbox';
            c.checked = true;
            pick[k] = c;
            l.append(c, document.createTextNode(k === 'spaces' ? 'Spaces' : KIND[k]));
            checks.append(l);
          }
          const dest = s.id === 'arc' ? null : target(s.name);
          const go = el('button', 'btn primary', 'Importar');
          go.onclick = () => run(go, () => invoke('import_browser', {
            id: s.id,
            bookmarks: !!(pick.bookmarks?.checked || pick.spaces?.checked),
            history: !!pick.history?.checked,
            newSpace: dest && dest.value === 'new' ? s.name : null,
          }));
          checks.append(...(dest ? [dest] : []), go);
          box.append(txt, checks);
          found.append(box);
        }
      });
      const htmlDest = target('Favoritos importados');
      const htmlBtn = el('button', 'btn', 'Escolher arquivo…');
      htmlBtn.onclick = () => run(htmlBtn, () => invoke('import_html', { newSpace: htmlDest.value === 'new' ? 'Favoritos importados' : null }));
      const htmlWrap = el('div', 'seg');
      htmlWrap.style.background = 'none';
      htmlWrap.append(htmlDest, htmlBtn);
      return [
        ...header('Importar', 'Traga favoritos, histórico e Spaces de outro navegador. Abas importadas começam dormindo; o histórico alimenta as sugestões da barra de endereço.'),
        found,
        group('Arquivo de favoritos',
          row('Favoritos em HTML', 'Qualquer navegador exporta favoritos neste formato (Exportar favoritos).', htmlWrap)),
        group('O que não dá para importar',
          row('Senhas', 'O WebView2 não permite adicionar senhas salvas. Use um gerenciador de senhas ou deixe o Blazon salvar ao entrar nos sites.', el('span')),
          row('Abas abertas e extensões', 'Ainda não suportado.', el('span'))),
      ];
    },
  },
  keys: {
    title: 'Atalhos',
    icon: 'keys',
    render() {
      const keys = [
        ['Nova aba', 'Ctrl+T'], ['Nova janela', 'Ctrl+N'], ['Fechar aba', 'Ctrl+W'], ['Reabrir aba fechada', 'Ctrl+Shift+T'], ['Focar o endereço', 'Ctrl+L'],
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
  report: {
    title: 'Relatar problema',
    icon: 'bug',
    render() {
      const desc = el('textarea', 'report-desc');
      desc.placeholder = 'O que você estava fazendo e o que aconteceu? (ex.: cliquei em Importar do Chrome e nada aconteceu)';
      desc.value = reportDraft;
      desc.oninput = () => { reportDraft = desc.value; };
      const out = el('pre', 'report-out');
      out.hidden = true;
      const actions = el('div', 'row-btns');
      const gen = el('button', 'btn primary', 'Gerar relatório');
      const copy = el('button', 'btn', 'Copiar');
      const issue = el('button', 'btn', 'Abrir issue no GitHub');
      const logs = el('button', 'btn', 'Abrir pasta de logs');
      copy.disabled = issue.disabled = true;
      gen.onclick = async () => {
        out.textContent = await invoke('bug_report', { description: desc.value });
        out.hidden = false;
        copy.disabled = issue.disabled = false;
      };
      copy.onclick = async () => {
        await navigator.clipboard.writeText(out.textContent);
        copy.textContent = 'Copiado!';
        setTimeout(() => { copy.textContent = 'Copiar'; }, 1500);
      };
      issue.onclick = () => {
        const title = `Bug: ${(desc.value.trim().split('\n')[0] || 'problema no Blazon').slice(0, 80)}`;
        const body = out.textContent.length > 6000 ? `${out.textContent.slice(0, 6000)}\n\n(cortado; anexe o arquivo da pasta de logs)` : out.textContent;
        const url = `${REPO_URL}/issues/new?labels=bug&title=${encodeURIComponent(title)}&body=${encodeURIComponent(body)}`;
        invoke('close_panel').then(() => invoke('new_tab', { url }));
      };
      logs.onclick = () => invoke('open_logs_folder');
      actions.append(gen, copy, issue, logs);
      return [
        ...header('Relatar problema', 'Gera um relatório com o que aconteceu, a versão, o sistema e o log recente. Para mandar ao desenvolvimento, copie e cole na conversa ou abra uma issue no GitHub.'),
        group('Descrição', desc, actions, out),
        group('Privacidade',
          row('O que vai no relatório', 'Versão, Windows, WebView2, quantidades (Spaces, abas) e configurações, e o log recente. O log só registra domínios, nunca endereços completos, títulos ou senhas.', el('span'))),
      ];
    },
  },
  about: {
    title: 'Sobre',
    icon: 'about',
    render() {
      const p = (text) => el('p', 'lead', text);
      return [
        ...header('Blazon', 'Um navegador leve para Windows, com Spaces e brasões. Feito em Rust com Tauri e o WebView2 do Windows.'),
        p('Inspirado no Crest de Paul Joda (github.com/pauljoda/Crest), licença MPL-2.0. Brasões e paletas do Crest Studio vêm do projeto original; figuras heráldicas de Lorc (game-icons.net), CC BY 3.0.'),
      ];
    },
  },
};

// Crest Studio: each Space's arms and sidebar, step by step, with previews.

let studioSpace = null;
let studioStep = 'house';
const undoStack = [];

const STEPS = [
  ['house', 'Modelo'], ['shape', 'Formato'], ['field', 'Campo'], ['band', 'Faixa'], ['emblem', 'Emblema'],
  ['trim', 'Borda'], ['finish', 'Acabamento'], ['colors', 'Cores'], ['sidebar', 'Barra lateral'],
];
const CREST_COLOR_ROLES = ['Campo', 'Segundo campo', 'Faixa', 'Emblema', 'Borda', 'Contorno'];
const SIDEBAR_COLOR_ROLES = ['Fundo da barra', 'Faixa do meio', 'Faixa de baixo'];

function svgNode(markup) {
  const t = document.createElement('template');
  t.innerHTML = markup;
  return t.content.firstChild;
}

function draftOf(space) {
  return { crest: crestOf(space), sidebar: { ...look(space) } };
}

function saveStudio(space, draft, extra = {}) {
  undoStack.push(JSON.stringify({ id: space.id, look: space.look, house: space.house }));
  if (undoStack.length > 50) undoStack.shift();
  space.look = { crest: draft.crest, sidebar: draft.sidebar };
  Object.assign(space, extra);
  invoke('update_space', { id: space.id, name: space.name, color: draft.sidebar.colors[2], house: space.house, look: space.look });
  render();
}

function bannerNode(sidebar, tileSize) {
  const wrap = el('div');
  const b = el('div', `banner ${sidebar.pattern}` + (TILED.includes(sidebar.pattern) ? ' tiles' : ''));
  b.append(el('div', 'p-mid'), el('div', 'p-low'));
  sidebar.colors.forEach((c, i) => wrap.style.setProperty(`--c${i}`, c));
  b.style.backgroundImage = tileImage(sidebar.pattern, sidebar.colors);
  if (tileSize) b.style.backgroundSize = `${tileSize}px ${tileSize}px`;
  wrap.append(b);
  return wrap;
}

function tile(label, preview, selected, onPick) {
  const b = el('button', 'tile' + (selected ? ' sel' : ''));
  b.append(preview, el('span', '', label));
  b.onclick = onPick;
  return b;
}

function renderStudio() {
  if (section.startsWith('spaces:')) { studioSpace = section.slice(7); section = 'spaces'; }
  const space = state.spaces.find((s) => s.id === studioSpace) || state.spaces.find((s) => s.id === state.active_space);
  studioSpace = space.id;
  const draft = draftOf(space);

  const list = el('div', 'space-list');
  for (const s of state.spaces) {
    const b = el('button', 'space-card' + (s.id === space.id ? ' sel' : ''));
    const txt = el('div', '', s.name);
    txt.append(el('small', '', [s.private ? 'Privado' : (HOUSES[s.house]?.title || 'Personalizado'), s.locked ? 'bloqueado' : ''].filter(Boolean).join(' · ')));
    b.append(svgNode(renderCrest(crestOf(s))), txt);
    b.onclick = () => { studioSpace = s.id; render(); };
    list.append(b);
  }
  const addRow = el('div', 'row-btns');
  for (const priv of [false, true]) {
    const add = el('button', 'btn', priv ? 'Novo privado' : 'Novo Space');
    add.onclick = async () => {
      const houses = Object.keys(HOUSES);
      const house = houses[state.spaces.length % houses.length];
      studioSpace = await invoke('add_space', { name: priv ? 'Privado' : 'Novo Space', color: HOUSES[house].colors[2], private: priv, house, look: null });
      state = await invoke('get_state');
      render();
    };
    addRow.append(add);
  }
  list.append(addRow);

  // Header: live preview of the crest and the sidebar, name and actions.
  const head = el('div', 'studio-head');
  const pc = el('div', 'preview-crest');
  pc.append(svgNode(renderCrest(draft.crest)));
  const mini = el('div', 'mini-sidebar');
  mini.append(bannerNode(draft.sidebar, 18));
  const hint = el('div', 'tabs-hint');
  hint.append(el('i'), el('i'), el('i'));
  mini.style.background = draft.sidebar.colors[0];
  mini.append(hint);
  const meta = el('div', 'studio-meta');
  const name = el('input');
  name.type = 'text';
  name.value = space.name;
  name.onchange = () => {
    space.name = name.value.trim() || space.name;
    invoke('update_space', { id: space.id, name: space.name, color: draft.sidebar.colors[2], house: space.house, look: null });
  };
  const actions = el('div', 'row-btns');
  const shuffle = el('button', 'btn primary', 'Embaralhar');
  shuffle.onclick = () => saveStudio(space, { ...draft, crest: shuffleCrest(draft.crest) });
  const undo = el('button', 'btn', 'Desfazer');
  undo.disabled = !undoStack.some((u) => JSON.parse(u).id === space.id);
  undo.onclick = () => {
    for (let i = undoStack.length - 1; i >= 0; i--) {
      const u = JSON.parse(undoStack[i]);
      if (u.id !== space.id) continue;
      undoStack.splice(i, 1);
      space.look = u.look;
      space.house = u.house;
      const d = draftOf(space);
      invoke('update_space', { id: space.id, name: space.name, color: d.sidebar.colors[2], house: space.house, look: space.look || {} });
      break;
    }
    render();
  };
  actions.append(shuffle, undo);
  if (state.spaces.length > 1) {
    const del = el('button', 'btn danger', 'Excluir Space');
    del.onclick = () => {
      if (!confirm(`Excluir o Space "${space.name}"? As abas, cookies e logins dele serão apagados.`)) return;
      invoke('delete_space', { id: space.id });
      state.spaces = state.spaces.filter((s) => s.id !== space.id);
      studioSpace = null;
      render();
    };
    actions.append(del);
  }
  const lockBtn = el('button', 'btn', space.locked ? 'Remover bloqueio' : 'Bloquear com Windows Hello');
  lockBtn.title = 'Pede PIN, rosto ou digital antes de mostrar as abas deste Space.';
  lockBtn.onclick = async () => {
    try {
      await invoke('set_space_lock', { id: space.id, locked: !space.locked });
      space.locked = !space.locked;
      render();
    } catch (err) { alert(err); }
  };
  if (!space.private) actions.append(lockBtn);
  const engines = [['', 'Busca padrão do Blazon'], ...SEARCH_ENGINES];
  const search = el('select');
  for (const [v, l] of engines) { const o = el('option', '', l); o.value = v; search.append(o); }
  search.value = engines.some(([v]) => v === (space.search || '')) ? (space.search || '') : '';
  search.title = 'Mecanismo de busca usado neste Space';
  search.onchange = () => { space.search = search.value; invoke('set_space_search', { id: space.id, search: search.value }); };
  const order = el('div', 'row-btns');
  const idx = state.spaces.indexOf(space);
  for (const [label, to] of [['↑ Subir', idx - 1], ['↓ Descer', idx + 1]]) {
    const b = el('button', 'btn', label);
    b.disabled = to < 0 || to >= state.spaces.length;
    b.onclick = async () => { await invoke('move_space', { id: space.id, to }); state = await invoke('get_state'); render(); };
    order.append(b);
  }
  order.append(search);
  meta.append(name, actions, order);
  head.append(pc, mini, meta);

  const steps = el('div', 'steps');
  for (const [key, label] of STEPS) {
    const b = el('button', key === studioStep ? 'sel' : '', label);
    b.onclick = () => { studioStep = key; render(); };
    steps.append(b);
  }

  const body = el('div');
  const crestTile = (patch, label, selected) =>
    tile(label, svgNode(renderCrest({ ...draft.crest, ...patch })), selected, () => saveStudio(space, { ...draft, crest: { ...draft.crest, ...patch } }));
  const grid = (entries) => { const g = el('div', 'tiles'); g.append(...entries); return g; };
  const c = draft.crest;
  switch (studioStep) {
    case 'house':
      body.append(el('p', 'lead', 'Comece de uma das nove casas clássicas: brasão e barra lateral juntos.'), grid(Object.entries(HOUSES).map(([k, h]) =>
        tile(h.title, svgNode(renderCrest(crestOf({ house: k }))), space.house === k, () =>
          saveStudio(space, { crest: crestOf({ house: k }), sidebar: { colors: [...h.colors], pattern: h.pattern } }, { house: k })))));
      break;
    case 'shape':
      body.append(grid(Object.entries(CREST_SHAPES).map(([k, l]) => crestTile({ shape: k }, l, c.shape === k))));
      break;
    case 'field': {
      body.append(grid(Object.entries(CREST_DIVISIONS).map(([k, l]) => crestTile({ division: k }, l, c.division === k))));
      if (COUNTED_DIVISIONS.includes(c.division)) {
        const r = el('div', 'sub-row');
        const n = el('input');
        Object.assign(n, { type: 'range', min: 2, max: 10, step: 1, value: c.divisionCount });
        n.onchange = () => saveStudio(space, { ...draft, crest: { ...c, divisionCount: Number(n.value) } });
        r.append(el('span', '', 'Quantidade'), n);
        body.append(r);
      }
      break;
    }
    case 'band':
      body.append(grid(Object.entries(CREST_ORDINARIES).map(([k, l]) => crestTile({ ordinary: k }, l, c.ordinary === k))));
      break;
    case 'emblem': {
      const r = el('div', 'sub-row');
      const emoji = el('input');
      emoji.type = 'text';
      emoji.placeholder = 'Emoji 🦊';
      emoji.value = c.chargeKind === 'emoji' ? c.text : '';
      emoji.onchange = () => emoji.value.trim() && saveStudio(space, { ...draft, crest: { ...c, chargeKind: 'emoji', text: emoji.value.trim() } });
      const mono = el('input');
      mono.type = 'text';
      mono.maxLength = 2;
      mono.placeholder = 'Monograma AB';
      mono.value = c.chargeKind === 'monogram' ? c.text : '';
      mono.onchange = () => mono.value.trim() && saveStudio(space, { ...draft, crest: { ...c, chargeKind: 'monogram', text: mono.value.trim() } });
      const scale = el('input');
      Object.assign(scale, { type: 'range', min: 0.6, max: 1.6, step: 0.05, value: c.chargeScale });
      scale.onchange = () => saveStudio(space, { ...draft, crest: { ...c, chargeScale: Number(scale.value) } });
      r.append(emoji, mono, el('span', '', 'Tamanho'), scale);
      body.append(r, grid([
        crestTile({ chargeKind: 'none' }, 'Nenhum', c.chargeKind === 'none'),
        ...Object.entries(CREST_FIGURES).map(([k, l]) => crestTile({ chargeKind: 'figure', charge: k }, l, c.chargeKind === 'figure' && c.charge === k)),
      ]));
      break;
    }
    case 'trim': {
      body.append(grid(Object.entries(CREST_TRIMS).map(([k, l]) => crestTile({ trim: k }, l, c.trim === k))));
      const r = el('div', 'sub-row');
      const w = el('input');
      Object.assign(w, { type: 'range', min: 0.3, max: 1.6, step: 0.05, value: c.trimWeight });
      w.onchange = () => saveStudio(space, { ...draft, crest: { ...c, trimWeight: Number(w.value) } });
      const det = el('input');
      Object.assign(det, { type: 'range', min: 6, max: 24, step: 1, value: c.detail });
      det.onchange = () => saveStudio(space, { ...draft, crest: { ...c, detail: Number(det.value) } });
      r.append(el('span', '', 'Espessura'), w, el('span', '', 'Detalhes (raios, pérolas, selo)'), det);
      body.append(r);
      break;
    }
    case 'finish':
      body.append(grid([
        ...Object.entries(CREST_FINISHES).map(([k, l]) => crestTile({ finish: k }, l, c.finish === k)),
        ...Object.entries(CREST_DEPTHS).map(([k, l]) => crestTile({ depth: k }, l, c.depth === k)),
      ]));
      break;
    case 'colors': {
      const colorInput = (label, value, onSave) => {
        const box = el('label', 'color');
        const i = el('input');
        i.type = 'color';
        i.value = value;
        i.onchange = () => onSave(i.value);
        box.append(i, document.createTextNode(label));
        return box;
      };
      const g1 = el('div', 'colors');
      CREST_COLOR_ROLES.forEach((label, idx) => g1.append(colorInput(label, c.colors[idx], (v) => {
        const colors = [...c.colors];
        colors[idx] = v;
        saveStudio(space, { ...draft, crest: { ...c, colors } });
      })));
      const g2 = el('div', 'colors');
      SIDEBAR_COLOR_ROLES.forEach((label, idx) => g2.append(colorInput(label, draft.sidebar.colors[idx], (v) => {
        const colors = [...draft.sidebar.colors];
        colors[idx] = v;
        saveStudio(space, { ...draft, sidebar: { ...draft.sidebar, colors } });
      })));
      body.append(el('h3', 'lead', 'Brasão'), g1, el('h3', 'lead', 'Barra lateral'), g2);
      break;
    }
    case 'sidebar':
      body.append(el('p', 'lead', 'O padrão das faixas no fundo da barra lateral.'), grid(Object.entries(SIDEBAR_PATTERNS).map(([k, l]) => {
        const sw = el('div', 'swatch');
        sw.style.background = draft.sidebar.colors[0];
        sw.append(bannerNode({ ...draft.sidebar, pattern: k }, 12));
        return tile(l, sw, draft.sidebar.pattern === k, () => saveStudio(space, { ...draft, sidebar: { ...draft.sidebar, pattern: k } }));
      })));
      break;
  }

  const editor = el('div');
  editor.append(head, steps, body);
  const studio = el('div', 'studio');
  studio.append(list, editor);
  return [...header('Spaces e Studio', 'Cada Space tem seu brasão, cores e fundo. Escolha um modelo ou monte peça por peça.'), studio];
}

// Site permissions and extensions live in each Space's WebView2 profile.
const PERMISSION_KINDS = {
  1: 'Microfone', 2: 'Câmera', 3: 'Localização', 4: 'Notificações', 5: 'Outros sensores', 6: 'Leitura da área de transferência',
  7: 'Downloads automáticos', 8: 'Acesso a arquivos', 9: 'Reprodução automática', 10: 'Fontes locais', 11: 'Mensagens MIDI', 12: 'Janelas',
};

function spacePicker(onChange) {
  const sel = el('select');
  for (const s of state.spaces.filter((x) => !x.private)) {
    const o = el('option', '', s.name);
    o.value = s.id;
    sel.append(o);
  }
  sel.value = state.active_space;
  sel.onchange = () => onChange(sel.value);
  return sel;
}

function permissionsGroup() {
  const list = el('div');
  const load = async (space) => {
    list.replaceChildren(el('p', 'lead', 'Carregando…'));
    try {
      const perms = await invoke('list_permissions', { space });
      list.replaceChildren();
      if (!perms.length) list.append(el('p', 'lead', 'Nenhuma permissão salva neste Space. Os sites pedem na primeira vez que precisam.'));
      for (const p of perms) {
        const sel = el('select');
        for (const [v, l] of [['allow', 'Permitir'], ['deny', 'Bloquear'], ['default', 'Perguntar de novo']]) {
          const o = el('option', '', l);
          o.value = v;
          sel.append(o);
        }
        sel.value = p.state;
        sel.onchange = async () => { await invoke('set_permission', { space, kind: p.kind, origin: p.origin, value: sel.value }); if (sel.value === 'default') load(space); };
        list.append(row(p.origin.replace(/^https?:\/\//, ''), PERMISSION_KINDS[p.kind] || `Permissão ${p.kind}`, sel));
      }
    } catch (err) { list.replaceChildren(el('p', 'lead', String(err))); }
  };
  const g = group('Permissões de sites', row('Space', 'Câmera, microfone, localização, notificações e outras.', spacePicker(load)), list);
  load(state.active_space);
  return g;
}

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
    const b = el('button', id === section || section.startsWith(`${id}:`) ? 'sel' : '');
    b.append(icon(s.icon), document.createTextNode(s.title));
    b.onclick = () => { section = id; render(); };
    nav.append(b);
  }
  const main = $('main');
  const top = main.scrollTop;
  const key = section.startsWith('spaces:') ? 'spaces' : section;
  main.replaceChildren(...(SECTIONS[key] || SECTIONS.look).render());
  main.scrollTop = top;
}

$('close').append(icon('x'));
$('close').onclick = () => invoke('close_panel');
addEventListener('keydown', (e) => { if (e.key === 'Escape') invoke('close_panel'); });
listen('state', (e) => { const first = !state; state = e.payload; I18N.sync(state.settings.language); if (first) render(); else applySpaceLook(); });
listen('section', (e) => { section = e.payload; render(); });
invoke('get_state').then((s) => { state = s; I18N.sync(state.settings.language); render(); });
