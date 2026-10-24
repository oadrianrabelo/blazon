// Reader mode: Mozilla Readability extracts the article and the page is
// replaced by a clean, Crest-styled view. Running it again restores the page.
(() => {
  if (document.documentElement.dataset.crestReader) { location.reload(); return; }
  /*READABILITY*/
  const article = new Readability(document.cloneNode(true)).parse();
  if (!article || !article.content) { alert('Esta página não tem um texto para o modo leitura.'); return; }
  const site = location.hostname.replace(/^www\./, '');
  document.documentElement.dataset.crestReader = '1';
  document.head.innerHTML = '<meta charset="utf-8"><meta name="viewport" content="width=device-width">';
  const style = document.createElement('style');
  style.textContent = `
    :root { color-scheme: light dark; --bg: #f7f3ea; --fg: #2b2620; --muted: #7d7365; --link: #7a4b16; }
    @media (prefers-color-scheme: dark) { :root { --bg: #1c1b19; --fg: #e6e0d4; --muted: #9b9282; --link: #e0b56a; } }
    html { background: var(--bg); }
    body { margin: 0; color: var(--fg); font: 20px/1.65 Georgia, "Iowan Old Style", "Times New Roman", serif; }
    article { max-width: 680px; margin: 0 auto; padding: 56px 24px 96px; }
    .site { font: 600 13px "Segoe UI", system-ui, sans-serif; letter-spacing: .06em; text-transform: uppercase; color: var(--muted); }
    h1 { font-size: 38px; line-height: 1.2; margin: 8px 0 10px; }
    .byline { font: 14px "Segoe UI", system-ui, sans-serif; color: var(--muted); margin-bottom: 32px; }
    img, video, figure { max-width: 100%; height: auto; }
    a { color: var(--link); }
    pre, code { font-size: 15px; white-space: pre-wrap; }
    blockquote { margin-left: 0; padding-left: 18px; border-left: 3px solid var(--muted); color: var(--muted); }
  `;
  document.head.append(style);
  document.title = article.title || document.title;
  const wrap = document.createElement('article');
  const p = (cls, text) => { const e = document.createElement(cls === 'h1' ? 'h1' : 'p'); if (cls !== 'h1') e.className = cls; e.textContent = text; return e; };
  wrap.append(p('site', article.siteName || site), p('h1', article.title || ''));
  if (article.byline) wrap.append(p('byline', article.byline));
  const content = document.createElement('div');
  content.innerHTML = article.content;
  content.querySelectorAll('script, iframe, style').forEach((n) => n.remove());
  wrap.append(content);
  document.body.replaceChildren(wrap);
  document.body.removeAttribute('class');
  document.body.removeAttribute('style');
  scrollTo(0, 0);
})();
