// Getting Started (Crest's BrowserGettingStartedView): short lessons with a
// practice sidebar that only lives on this page, so the person's own tabs
// never change. Opened after the welcome flow and from Settings › About.

const params = new URLSearchParams(location.search);
const EN = (params.get('lang') || navigator.language).startsWith('en');
const T = (pt, en) => (EN ? en : pt);
document.documentElement.lang = EN ? 'en' : 'pt-BR';
document.title = T('Primeiros passos', 'Getting started');

// Space colors come in the address (?c=c0,c1,c2&p=pattern).
const colors = (params.get('c') || '').split(',').filter((c) => /^#[0-9a-f]{3,8}$/i.test(c));
const pattern = params.get('p') || 'diagonal';
if (colors.length === 3) {
  colors.forEach((c, i) => document.documentElement.style.setProperty(`--c${i}`, c));
  const banner = document.getElementById('banner');
  banner.className = `banner ${pattern}` + (TILED.includes(pattern) ? ' tiles' : '');
  banner.style.backgroundImage = tileImage(pattern, colors);
}

const $ = (id) => document.getElementById(id);
function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}
const CHECK = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="m5 12 5 5 9-10"/></svg>';
const FOLDER = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/></svg>';
const svg = (markup) => { const s = el('span'); s.innerHTML = markup; return s.firstChild; };

// The practice Space.
let practice;
function resetPractice() {
  practice = {
    tabs: [
      { id: 1, title: 'Gmail', letter: 'M', pinned: false },
      { id: 2, title: T('Guia da trilha de fim de semana', 'Weekend trail guide'), letter: 'G' },
      { id: 3, title: T('Lista de viagem', 'Packing list'), letter: 'L' },
      { id: 4, title: T('Mapa da trilha', 'Trail map'), letter: 'T' },
    ],
    folders: [],
    cleared: false,
    focus: 0,
    cards: [T('Guia da trilha', 'Trail guide'), T('Lista de viagem', 'Packing list')],
  };
}
resetPractice();

const done = new Set(JSON.parse(sessionStorageGet('blazon-guide') || '[]'));
function sessionStorageGet(k) { try { return sessionStorage.getItem(k); } catch { return null; } }
function markDone(i) {
  done.add(i);
  try { sessionStorage.setItem('blazon-guide', JSON.stringify([...done])); } catch { /* private page */ }
  drawChapters();
}

function mockSidebar(actions) {
  const side = el('div', 'mock');
  const pinned = el('div', 'pinned');
  const folders = el('ul', 'tabs');
  const loose = el('ul', 'tabs');
  const row = (t, extra = '') => {
    const li = el('li', 'tab' + extra);
    li.dataset.key = `t${t.id}`;
    li.append(el('span', 'fav', t.letter), el('span', 'title', t.title));
    if (actions.tab) actions.tab(li, t);
    return li;
  };
  for (const t of practice.tabs.filter((x) => x.pinned)) {
    const b = el('button', 'pin');
    b.dataset.key = `t${t.id}`;
    b.title = t.title;
    b.append(el('span', 'fav', t.letter));
    pinned.append(b);
  }
  for (const f of practice.folders) {
    const head = el('li', 'folder open');
    head.dataset.key = `f${f.id}`;
    head.append(svg(FOLDER), el('span', 'title', f.name));
    if (actions.folder) actions.folder(head, f);
    folders.append(head);
    for (const t of practice.tabs.filter((x) => x.folder === f.id)) folders.append(row(t, ' in-folder'));
  }
  const line = el('div', 'line');
  if (actions.clear) {
    const clear = el('button', '', T('Limpar', 'Clear'));
    clear.onclick = actions.clear;
    line.append(clear);
  }
  if (!practice.cleared) for (const t of practice.tabs.filter((x) => !x.pinned && x.folder == null)) loose.append(row(t));
  side.append(pinned, folders, line, loose);
  return side;
}

function preview() {
  const p = el('div', 'preview');
  p.append(el('small', '', T('PÁGINA DE EXEMPLO', 'EXAMPLE PAGE')), el('h3', '', T('Guia da trilha de fim de semana', 'Weekend trail guide')),
    el('pre', '', T('Distância: 6,8 km\nDuração: 2 horas\nPercurso: volta do lago', 'Distance: 4.2 miles\nDuration: 2 hours\nRoute: lakeside loop')));
  return p;
}

// Rebuilds the stage, animating rows to their new places.
function redraw(stage, build) {
  const before = Motion.snapshot(stage);
  stage.replaceChildren(...build());
  Motion.play(before, [stage], stage);
}

function task(text, ok) {
  const t = el('div', 'task' + (ok ? ' done' : ''));
  const c = el('span', 'check');
  c.append(svg(CHECK));
  t.append(c, el('span', 'text', text));
  return t;
}

const LESSONS = [
  {
    title: T('Fixe seus apps do dia a dia', 'Pin your everyday apps'),
    lead: T('Abas fixadas ficam no topo da barra e não dormem enquanto o Space está aberto. Clique com o botão direito em Gmail e escolha Fixar, ou use o botão abaixo.',
      'Pinned tabs live at the top of the sidebar and stay awake while their Space is open. Right-click Gmail and choose Pin, or use the button below.'),
    goal: T('Fixe o Gmail', 'Pin Gmail'),
    isDone: () => practice.tabs[0].pinned,
    build(stage, refresh) {
      const pin = () => { practice.tabs[0].pinned = true; refresh(); };
      return [mockSidebar({ tab: (li, t) => { if (t.id === 1) li.oncontextmenu = (e) => { e.preventDefault(); pin(); }; } }), preview()];
    },
    action: () => [T('Fixar Gmail', 'Pin Gmail'), () => { practice.tabs[0].pinned = true; }],
  },
  {
    title: T('Organize com pastas', 'Organize with folders'),
    lead: T('Pastas guardam abas por assunto e ficam acima da linha. Crie a pasta Fins de semana e coloque a Lista de viagem nela. No Blazon, é só arrastar a aba até a pasta.',
      'Folders keep tabs by topic and sit above the line. Create a Weekends folder and put the Packing list in it. In Blazon, just drag the tab onto the folder.'),
    goal: T('Lista de viagem dentro de Fins de semana', 'Packing list inside Weekends'),
    isDone: () => practice.tabs[2].folder === 1,
    build(stage, refresh) {
      return [mockSidebar({
        tab: (li, t) => {
          if (t.id !== 3 || !practice.folders.length) return;
          li.draggable = true;
          li.ondragstart = (e) => e.dataTransfer.setData('text/plain', 'packing');
        },
        folder: (head) => {
          head.ondragover = (e) => { e.preventDefault(); head.classList.add('drop-into'); };
          head.ondragleave = () => head.classList.remove('drop-into');
          head.ondrop = (e) => { e.preventDefault(); practice.tabs[2].folder = 1; refresh(); };
        },
      }), preview()];
    },
    action: () => (!practice.folders.length
      ? [T('Nova pasta', 'New folder'), () => { practice.folders.push({ id: 1, name: T('Fins de semana', 'Weekends') }); }]
      : [T('Mover a lista para a pasta', 'Move the list into the folder'), () => { practice.tabs[2].folder = 1; }]),
  },
  {
    title: T('Limpe as abas soltas', 'Clear loose tabs'),
    lead: T('Abaixo da linha ficam as abas soltas. Elas podem ir sozinhas para o Arquivo depois do prazo que você escolher em Configurações › Geral. Para limpar agora, use Limpar na linha. Fixadas e pastas ficam.',
      'Below the line are loose tabs. They can go to the Archive on their own after the time you choose in Settings › General. To clear them now, use Clear on the line. Pinned tabs and folders stay.'),
    goal: T('Use Limpar na barra de prática', 'Use Clear in the practice sidebar'),
    isDone: () => practice.cleared,
    build(stage, refresh) {
      return [mockSidebar({ clear: () => { practice.cleared = true; refresh(); } }), preview()];
    },
  },
  {
    title: T('Veja páginas lado a lado', 'View pages side by side'),
    lead: T('Clique com o botão direito numa aba e escolha Lado a lado. A borda mostra qual página recebe o teclado. Pratique: escolha um cartão e mude ele de lado.',
      'Right-click a tab and choose Side by side. The border shows which page receives the keyboard. Practice: pick a card and move it to the other side.'),
    goal: T('Mude um cartão de lado', 'Move a card to the other side'),
    isDone: () => practice.swapped,
    build(stage, refresh) {
      const cards = el('div', 'cards');
      cards.style.gridColumn = '1 / -1';
      practice.cards.forEach((name, i) => {
        const c = el('div', 'card' + (practice.focus === i ? ' focus' : ''));
        c.dataset.key = `c${name}`;
        c.onclick = () => { practice.focus = i; refresh(); };
        const btns = el('div', 'row-btns');
        const move = el('button', '', i === 0 ? T('Mover para a direita →', 'Move right →') : T('← Mover para a esquerda', '← Move left'));
        move.onclick = (e) => { e.stopPropagation(); practice.cards.reverse(); practice.focus = 1 - i; practice.swapped = true; refresh(); };
        btns.append(move);
        c.append(el('h3', '', name), el('p', '', practice.focus === i ? T('Este cartão recebe o teclado.', 'This card gets the keyboard.') : T('Clique para focar.', 'Click to focus.')), btns);
        cards.append(c);
      });
      return [cards];
    },
  },
  {
    title: T('Paleta, Peek e Spaces', 'Palette, Peek and Spaces'),
    lead: T('Três atalhos fazem quase tudo no Blazon:', 'Three shortcuts do almost everything in Blazon:'),
    goal: T('Abra um link no Peek', 'Open a link in Peek'),
    isDone: () => practice.peeked,
    build() {
      const box = el('div', 'preview');
      box.style.gridColumn = '1 / -1';
      const item = (keys, text) => { const p = el('p'); p.style.margin = '0 0 8px'; keys.forEach((k) => p.append(el('kbd', '', k), ' ')); p.append(text); return p; };
      const link = el('a', 'peek-link', T('esta página da Wikipédia', 'this Wikipedia page'));
      link.href = EN ? 'https://en.wikipedia.org/wiki/Heraldry' : 'https://pt.wikipedia.org/wiki/Her%C3%A1ldica';
      link.onclick = (e) => { if (e.shiftKey || e.altKey) practice.peeked = true; setTimeout(draw, 300); };
      const peek = el('p');
      peek.style.margin = '12px 0 0';
      peek.append(T('Experimente: Shift+clique em ', 'Try it: Shift+click '), link, '.');
      box.append(item(['Ctrl', 'K'], T('abre a paleta: abas, comandos, histórico e Spaces.', 'opens the palette: tabs, commands, history and Spaces.')),
        item(['Shift', T('clique', 'click')], T('abre um link no Peek, um cartão flutuante.', 'opens a link in Peek, a floating card.')),
        item(['Ctrl', 'Shift', '1…9'], T('troca de Space. Dois dedos para os lados no touchpad também.', 'switches Space. A two-finger sideways swipe on the touchpad does too.')),
        peek);
      return [box];
    },
  },
];

let index = Math.min(LESSONS.length - 1, Math.max(0, Number(params.get('lesson') || 0)));

function drawChapters() {
  const nav = $('chapters');
  nav.replaceChildren(el('h1', '', T('Primeiros passos', 'Getting started')),
    el('p', '', T('A prática usa um Space de exemplo. Suas abas não mudam.', 'Practice uses an example Space. Your tabs are unchanged.')));
  LESSONS.forEach((l, i) => {
    const b = el('button', (i === index ? 'on' : '') + (done.has(i) ? ' done' : ''));
    b.append(el('span', 'n', done.has(i) ? '✓' : String(i + 1)), el('span', '', l.title));
    b.onclick = () => go(i);
    nav.append(b);
  });
}

function draw(direction = 0) {
  const l = LESSONS[index];
  if (l.isDone()) markDone(index);
  const view = $('lesson');
  const stage = el('div', 'stage');
  const refresh = () => {
    redraw(stage, () => l.build(stage, refresh));
    if (l.isDone() && !done.has(index)) markDone(index);
    view.querySelector('.task')?.replaceWith(task(l.goal, l.isDone()));
    drawAction();
  };
  stage.append(...l.build(stage, refresh));
  const actionBox = el('div');
  const drawAction = () => {
    actionBox.replaceChildren();
    const a = l.action?.();
    if (!a || l.isDone()) return;
    const b = el('button', 'btn', a[0]);
    b.style.marginTop = '12px';
    b.onclick = () => { a[1](); refresh(); };
    actionBox.append(b);
  };
  drawAction();
  const nav = el('div', 'nav');
  const reset = el('button', 'btn', T('Recomeçar prática', 'Reset practice'));
  reset.onclick = () => { resetPractice(); draw(); };
  const prev = el('button', 'btn', T('Anterior', 'Previous'));
  prev.disabled = index === 0;
  prev.onclick = () => go(index - 1);
  const next = el('button', 'btn primary', index === LESSONS.length - 1 ? T('Pronto', 'Done') : T('Próximo', 'Next'));
  next.onclick = () => (index === LESSONS.length - 1 ? markDone(index) : go(index + 1));
  nav.append(el('span', 'count', `${index + 1} / ${LESSONS.length}`), reset, prev, next);
  view.replaceChildren(el('p', 'eyebrow', T(`Lição ${index + 1}`, `Lesson ${index + 1}`)), el('h2', '', l.title), el('p', 'lead', l.lead),
    stage, actionBox, task(l.goal, l.isDone()), nav,
    el('p', 'note', T('Esta página fica no seu histórico. Para abrir de novo: Configurações › Sobre › Primeiros passos.', 'This page stays in your history. To open it again: Settings › About › Getting started.')));
  if (direction) Motion.enter(view);
  drawChapters();
}

function go(i) {
  const dir = i > index ? 1 : -1;
  index = i;
  draw(dir);
}

addEventListener('keydown', (e) => {
  if (e.target.closest('input')) return;
  if (e.key === 'ArrowRight' && index < LESSONS.length - 1) go(index + 1);
  if (e.key === 'ArrowLeft' && index > 0) go(index - 1);
});
draw();
