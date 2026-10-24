// Smoke test on a real Windows machine (CI): opens the installed Blazon,
// walks through its main screens through WebView2's DevTools protocol and
// saves a screenshot of the whole screen at each step.
//
//   node tests/smoke/run.mjs <path to Blazon.exe> <screenshots folder>

import { spawn, execFileSync } from 'node:child_process';
import { mkdirSync, readdirSync, readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from 'playwright-core';

const [exe, out = 'screenshots'] = process.argv.slice(2);
mkdirSync(out, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
// Blazon opens WebView2's DevTools protocol on its interface when this is set.
const env = { ...process.env, BLAZON_DEVTOOLS_PORT: '9222' };
const problems = [];
let step = 0;

function shot(name) {
  step += 1;
  const file = join(out, `${String(step).padStart(2, '0')}-${name}.png`);
  execFileSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'tests/smoke/screenshot.ps1', file]);
  console.log(`📸 ${file}`);
}

async function waitFor(fn, what, timeout = 30000) {
  const end = Date.now() + timeout;
  for (;;) {
    try { const v = await fn(); if (v) return v; } catch { /* not yet */ }
    if (Date.now() > end) throw new Error(`timed out waiting for ${what}`);
    await sleep(250);
  }
}

const app = spawn(exe, [], { env, stdio: 'inherit' });
app.on('exit', (code) => { if (code) problems.push(`Blazon saiu com código ${code}`); });

let failed = null;
let browser = null;
await sleep(8000);
shot('abertura');
try {
  await waitFor(() => fetch('http://127.0.0.1:9222/json/version').then((r) => r.ok), 'DevTools', 60000);
  browser = await chromium.connectOverCDP('http://127.0.0.1:9222');
  const context = browser.contexts()[0];
  const page = (part) => waitFor(() => context.pages().find((p) => p.url().includes(part)), part);
  const watch = (p) => {
    p.on('pageerror', (e) => problems.push(`${p.url()}: ${e.message}`));
    p.on('console', (m) => { if (m.type() === 'error' && !/Failed to load resource/.test(m.text())) problems.push(`${p.url()}: ${m.text()}`); });
  };
  context.on('page', watch);
  context.pages().forEach(watch);

  const ui = await page('index.html');
  const invoke = (cmd, args = {}) => ui.evaluate(([c, a]) => window.__TAURI__.core.invoke(c, a), [cmd, args]);
  await ui.evaluate(() => window.__TAURI__.window.getCurrentWindow().toggleMaximize()).catch(() => {});
  await sleep(3000);

  // First run: the welcome flow.
  const welcome = await page('overlay.html#welcome');
  await welcome.waitForSelector('.w-btn.primary');
  shot('boas-vindas');
  const next = async (wait = 900) => { await welcome.click('.w-btn.primary'); await sleep(wait); };
  await next();
  await welcome.click('.w-house:nth-child(2)');
  await sleep(2200);
  shot('escolher-casa');
  await next();
  await welcome.click('.w-space.add');
  await sleep(1200);
  shot('spaces');
  await next(2500);
  shot('importar');
  await next();
  shot('pronto');
  await welcome.click('.w-btn.primary');
  await sleep(4000);
  shot('primeiros-passos');

  // Browsing.
  await invoke('new_tab', { url: 'https://example.com/' });
  await sleep(4000);
  shot('pagina');

  await invoke('open_overlay', { mode: 'palette' });
  await sleep(1800);
  shot('paleta');
  await invoke('close_overlay');
  await sleep(800);

  await invoke('open_panel', { section: 'look' });
  await sleep(2000);
  shot('configuracoes');
  await invoke('open_panel', { section: 'spaces' });
  await sleep(2000);
  shot('studio');
  await invoke('close_panel');
  await sleep(800);

  await invoke('new_tab', { url: 'https://blazon-teste.invalid/' });
  await sleep(5000);
  shot('site-fora-do-ar');

  await invoke('open_overlay', { mode: 'peek', url: 'https://en.wikipedia.org/wiki/Heraldry' });
  await sleep(5000);
  shot('peek');
  await invoke('close_overlay');
  await sleep(1000);

  // A link from another app, opened in the Quick Window.
  const s = await invoke('get_state');
  await invoke('set_settings', { settings: { ...s.settings, external_space: 'quick' } });
  spawn(exe, ['https://example.org/'], { env, stdio: 'ignore' });
  await sleep(5000);
  shot('janela-rapida');

} catch (e) {
  failed = e;
  console.error(`\n❌ ${e.message}`);
  try { shot('falha'); } catch { /* no screen */ }
}

// Blazon's own log.
const logs = join(process.env.APPDATA || '', 'app.blazon.browser', 'logs');
if (existsSync(logs)) {
  for (const f of readdirSync(logs)) {
    const text = readFileSync(join(logs, f), 'utf8');
    console.log(`\n--- ${f} ---\n${text.split('\n').slice(-60).join('\n')}`);
  }
}

await browser?.close().catch(() => {});
app.kill();
console.log(problems.length ? `\n⚠️ Problemas:\n- ${problems.join('\n- ')}` : '\n✅ Nenhum erro de interface.');
process.exit(failed ? 1 : 0);
