// Tests for the UI's pure logic: the crest generator, house looks and the
// English translation. Run with `node --test tests/`.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const ui = (f) => readFileSync(new URL(`../ui/${f}`, import.meta.url), 'utf8');

function crestContext() {
  const ctx = vm.createContext({});
  // Top-level const/function declarations stay in the script scope; expose what tests use.
  vm.runInContext(`${ui('houses.js')}\n${ui('charges.js')}\n${ui('crest.js')}
    globalThis.api = { HOUSES, HOUSE_ARMS, CREST_SHAPES, CREST_DIVISIONS, CREST_ORDINARIES, CREST_TRIMS, CREST_FIGURES,
      CHARGE_PATHS, DRAWN_FIGURES, renderCrest, crestOf, look, shuffleCrest, tileImage, lookOf, LOOK_DEFAULTS };`, ctx);
  return ctx.api;
}

test('every house has arms, a sidebar look and renders', () => {
  const c = crestContext();
  for (const house of Object.keys(c.HOUSES)) {
    assert.ok(c.HOUSE_ARMS[house], `arms for ${house}`);
    const svg = c.renderCrest(c.crestOf({ house }));
    assert.match(svg, /^<svg[\s\S]*<\/svg>$/);
    assert.equal(c.look({ house }).colors.length, 3);
  }
});

test('every figure has artwork', () => {
  const c = crestContext();
  for (const fig of Object.keys(c.CREST_FIGURES)) {
    assert.ok(c.DRAWN_FIGURES[fig] || c.CHARGE_PATHS[fig], `artwork for ${fig}`);
    assert.match(c.renderCrest({ chargeKind: 'figure', charge: fig }), /viewBox="0 0 512 512"/);
  }
});

test('every Studio option renders without breaking the SVG', () => {
  const c = crestContext();
  const groups = { shape: c.CREST_SHAPES, division: c.CREST_DIVISIONS, ordinary: c.CREST_ORDINARIES, trim: c.CREST_TRIMS };
  for (const [key, options] of Object.entries(groups)) {
    for (const value of Object.keys(options)) {
      const svg = c.renderCrest({ [key]: value, divisionCount: 5 });
      assert.ok(!svg.includes('NaN') && !svg.includes('undefined'), `${key}=${value}`);
    }
  }
});

test('emoji and monogram text is escaped', () => {
  const c = crestContext();
  const svg = c.renderCrest({ chargeKind: 'monogram', text: '<b' });
  assert.ok(svg.includes('&lt;B') && !svg.includes('<B'));
});

test('custom colors and Studio looks win over the house', () => {
  const c = crestContext();
  assert.equal(c.crestOf({ house: '', color: '#16a34a' }).colors[0], '#16a34a');
  const studio = { house: 'lion', look: { crest: { shape: 'seal' }, sidebar: { colors: ['#000000', '#111111', '#222222'], pattern: 'solid' } } };
  assert.equal(c.crestOf(studio).shape, 'seal');
  assert.equal(c.look(studio).pattern, 'solid');
});

test('shuffle keeps a valid crest', () => {
  const c = crestContext();
  for (let i = 0; i < 50; i++) {
    const s = c.shuffleCrest(c.crestOf({ house: 'winter' }));
    assert.ok(c.CREST_SHAPES[s.shape] && s.shape !== 'none');
    assert.ok(!c.renderCrest(s).includes('NaN'));
  }
});

function i18n(lang) {
  const ctx = vm.createContext({
    navigator: { language: lang },
    localStorage: { getItem: () => null, setItem() {} },
    document: { documentElement: {}, body: null, addEventListener() {}, title: '' },
    addEventListener() {},
    window: {},
    Node: { TEXT_NODE: 3, ELEMENT_NODE: 1 },
  });
  vm.runInContext(`${ui('i18n.js')}\nglobalThis.api = I18N;`, ctx);
  return ctx.api;
}

test('English translates fixed and generated texts', () => {
  const { t } = i18n('en');
  assert.equal(t('Nova aba'), 'New tab');
  assert.equal(t('  Configurações '), '  Settings ');
  assert.equal(t('Pesquisar "gatos"'), 'Search "gatos"');
  assert.equal(t('Baixando "a.pdf"'), 'Downloading "a.pdf"');
  assert.equal(t('Favoritos · Histórico'), 'Bookmarks · History');
  assert.equal(t('Importado: 2 Spaces, 10 abas.'), 'Imported: 2 Spaces, 10 tabs.');
  assert.equal(t('texto desconhecido'), 'texto desconhecido');
});

test('Portuguese leaves texts untouched', () => {
  const { t } = i18n('pt-BR');
  assert.equal(t('Nova aba'), 'Nova aba');
});
