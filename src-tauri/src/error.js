// Crest-style page for a site that could not be reached (BrowserNavigationFailureView).
// Run in the failed tab with BLAZON_ERROR = { url, code, lang } prepended.
(() => {
  const { url, code, lang } = BLAZON_ERROR;
  const en = lang === 'en' || (lang !== 'pt' && !/^pt\b/i.test(navigator.language));
  const T = (pt, eng) => (en ? eng : pt);
  let host = url;
  try { host = new URL(url).hostname.replace(/^www\./, ''); } catch {}
  const KINDS = {
    HOST_NAME_NOT_RESOLVED: ['search', T('Não foi possível encontrar este site', 'Can’t find this site'),
      T(`Não achamos o endereço de ${host}.`, `${host}’s address couldn’t be found.`),
      [T('Confira se o endereço está escrito certo.', 'Check that the address is spelled correctly.'), T('Pesquise o nome do site.', 'Search for the site by name.')]],
    TIMEOUT: ['clock', T('O site demorou demais para responder', 'The site took too long to respond'),
      T(`${host} não respondeu a tempo.`, `${host} didn’t respond in time.`),
      [T('Tente de novo em alguns instantes.', 'Try again in a moment.'), T('Confira sua conexão com a internet.', 'Check your internet connection.')]],
    DISCONNECTED: ['wifi', T('Você está sem internet', 'You’re offline'),
      T('A conexão com a internet caiu.', 'The internet connection dropped.'),
      [T('Confira o Wi-Fi ou o cabo de rede.', 'Check your Wi-Fi or network cable.'), T('Desligue e ligue o roteador.', 'Restart your router.')]],
  };
  const NET = ['wifi', T('Não foi possível conectar', 'Can’t connect'),
    T(`${host} recusou ou interrompeu a conexão.`, `${host} refused or reset the connection.`),
    [T('Confira sua conexão com a internet.', 'Check your internet connection.'), T('O site pode estar fora do ar. Tente mais tarde.', 'The site may be down. Try again later.'),
      T('Se você usa VPN ou proxy, confira se estão funcionando.', 'If you use a VPN or proxy, check that it works.')]];
  const [glyph, title, message, tips] = KINDS[code] || NET;
  const ICONS = {
    search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/><path d="M8.5 11h5"/>',
    clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
    wifi: '<path d="M2 8.8a15 15 0 0 1 20 0M5 12.5a10 10 0 0 1 14 0M8.5 16.1a5 5 0 0 1 7 0"/><path d="M12 20h.01"/><path d="m3 3 18 18"/>',
  };
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  document.title = title;
  document.documentElement.innerHTML = `<head><meta charset="utf-8"><meta name="color-scheme" content="light dark"><style>
    :root { color-scheme: light dark; --bg: #f4f1ea; --fg: #1d2127; --muted: #5d636b; --card: #fff; --line: rgba(0,0,0,.08); --accent: #3e4e5e; }
    @media (prefers-color-scheme: dark) { :root { --bg: #15181c; --fg: #eceae4; --muted: #9aa0a8; --card: #1e2227; --line: rgba(255,255,255,.08); --accent: #86adc4; } }
    * { box-sizing: border-box; }
    body { margin: 0; min-height: 100vh; display: grid; place-items: center; background: var(--bg); color: var(--fg);
      font: 14px/1.5 "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif; }
    main { width: min(520px, 90vw); padding: 32px 0; animation: in .39s linear(0, .156, .426, .661, .825, .922, .974, .997, 1.005, 1.006, 1.004, 1) both; }
    @keyframes in { from { opacity: 0; transform: translateY(10px); } }
    @media (prefers-reduced-motion: reduce) { main { animation-name: fade; } @keyframes fade { from { opacity: 0; } } }
    .icon { width: 56px; height: 56px; border-radius: 16px; display: grid; place-items: center; background: var(--card); box-shadow: 0 0 0 1px var(--line), 0 8px 24px rgba(0,0,0,.08); color: var(--accent); }
    .icon svg { width: 27px; height: 27px; }
    h1 { font: 600 24px/1.25 "Segoe UI Variable Display", "Segoe UI", system-ui, sans-serif; margin: 20px 0 6px; }
    p { margin: 0; color: var(--muted); }
    ul { margin: 18px 0 0; padding: 14px 18px 14px 34px; border-radius: 14px; background: var(--card); box-shadow: 0 0 0 1px var(--line); color: var(--muted); }
    li + li { margin-top: 4px; }
    .actions { display: flex; gap: 8px; margin-top: 22px; }
    button { font: inherit; font-weight: 600; border: 0; border-radius: 10px; padding: 9px 16px; background: var(--card); color: var(--fg); box-shadow: 0 0 0 1px var(--line);
      transition: transform .14s ease-out, filter .14s ease-in-out; }
    button:hover { filter: brightness(.97); }
    button:active { transform: scale(.975); }
    button.primary { background: var(--accent); color: var(--bg); box-shadow: none; }
    details { margin-top: 22px; color: var(--muted); font-size: 12px; }
    summary { cursor: default; }
    code { font: 12px Consolas, monospace; word-break: break-all; }
  </style></head><body><main>
    <div class="icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${ICONS[glyph]}</svg></div>
    <h1>${esc(title)}</h1><p>${esc(message)}</p>
    <ul>${tips.map((t) => `<li>${esc(t)}</li>`).join('')}</ul>
    <div class="actions"><button class="primary" id="retry">${T('Tentar de novo', 'Try again')}</button>${history.length > 1 ? `<button id="back">${T('Voltar', 'Go back')}</button>` : ''}</div>
    <details><summary>${T('Detalhes técnicos', 'Technical details')}</summary><p><code>${esc(code)}</code></p><p><code>${esc(url)}</code></p></details>
  </main></body>`;
  document.getElementById('retry').onclick = () => location.replace(url);
  const back = document.getElementById('back');
  if (back) back.onclick = () => history.back();
})();
