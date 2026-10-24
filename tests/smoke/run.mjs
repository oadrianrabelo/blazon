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
  // Pages are found by what they contain: Tauri may serve index.html as "/".
  const has = async (p, sel) => p.evaluate((s) => !!document.querySelector(s), sel).catch(() => false);
  const page = (sel) => waitFor(async () => {
    for (const p of context.pages()) if (await has(p, sel)) return p;
    return null;
  }, sel).catch((e) => {
    console.log('Páginas abertas:', context.pages().map((p) => p.url()));
    throw e;
  });
  const watch = (p) => {
    p.on('pageerror', (e) => problems.push(`${p.url()}: ${e.message}`));
    p.on('console', (m) => { if (m.type() === 'error' && !/Failed to load resource/.test(m.text())) problems.push(`${p.url()}: ${m.text()}`); });
  };
  context.on('page', watch);
  context.pages().forEach(watch);

  const ui = await page('#sidebar');
  const invoke = (cmd, args = {}) => ui.evaluate(([c, a]) => window.__TAURI__.core.invoke(c, a), [cmd, args]);
  await ui.evaluate(() => window.__TAURI__.window.getCurrentWindow().toggleMaximize()).catch(() => {});
  await sleep(3000);

  // First run: the welcome flow.
  const welcome = await page('#welcome:not([hidden])');
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

  // Switching back to the first page: the old page fades over the new one.
  const tabs = (await invoke('get_state')).tabs;
  const first = tabs.find((t) => t.url.startsWith('https://example.com'));
  if (first) {
    await invoke('activate_tab', { id: tabs[tabs.length - 1].id });
    await sleep(1500);
    invoke('activate_tab', { id: first.id });
    await sleep(120);
    shot('transicao-de-aba');
    await sleep(1500);
  }

  // A context menu in Liquid Glass.
  const tab = await ui.$('#tabs [data-tab], #folders [data-tab]');
  if (tab) {
    await tab.click({ button: 'right' });
    await sleep(900);
    shot('menu-de-vidro');
    await ui.mouse.click(5, 300);
    await sleep(400);
  }

  // Mac-style window buttons and the Mica material.
  const before = await invoke('get_state');
  await invoke('set_settings', { settings: { ...before.settings, appearance: { ...before.settings.appearance, windowControls: 'mac', mica: true } } });
  await sleep(1500);
  shot('botoes-mac-e-mica');
  await invoke('set_settings', { settings: before.settings });
  await sleep(800);

  // Settings › Shortcuts: change one, see the other lose it, restore.
  await invoke('open_panel', { section: 'keys' });
  await sleep(1500);
  const taken = await invoke('set_shortcut', { id: 'palette', combo: 'Ctrl+T' });
  if (taken !== 'newtab') throw new Error(`Ctrl+T deveria sair de newtab, saiu de "${taken}"`);
  await invoke('open_panel', { section: 'keys' });
  await sleep(1200);
  shot('atalhos');
  await invoke('reset_shortcuts');
  await invoke('close_panel');
  await sleep(800);

  // Tabs along the top, a menu spreading over the page, then the hidden bar.
  await invoke('run_command', { cmd: 'tablayout' });
  await sleep(2000);
  shot('abas-no-topo');
  const chip = await ui.$('#tabs .tab');
  if (chip) {
    await chip.click({ button: 'right' });
    await sleep(1200);
    shot('abas-no-topo-menu');
    await ui.keyboard.press('Escape');
    await sleep(800);
  }
  await invoke('run_command', { cmd: 'sidebar' });
  await sleep(1500);
  shot('barra-escondida');
  await invoke('run_command', { cmd: 'sidebar' });
  await invoke('run_command', { cmd: 'tablayout' });
  await sleep(1500);

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
