# Arquitetura

O Blazon é um app [Tauri 2](https://tauri.app) com várias webviews numa mesma janela. O núcleo
em Rust guarda o estado (Spaces, abas, pastas, configurações) e controla as webviews. A
interface é HTML/CSS/JS puro, sem framework nem etapa de build.

## Webviews de uma janela

```
┌──────────────────────────── janela "main" ─────────────────────────────┐
│ "ui": barra lateral + fundo heráldico (cobre a janela inteira)          │
│ ┌───────────┐ ┌───────────────────── "tab-<id>" ────────────────────┐   │
│ │  abas,    │ │  a página (uma webview por aba acordada; só as      │   │
│ │  Spaces…  │ │  visíveis aparecem, duas no lado a lado)            │   │
│ │           │ │      "panel": configurações e Studio (sobre o cartão)│  │
│ └───────────┘ └─────────────────────────────────────────────────────┘   │
│ "overlay": paleta / barra do Peek, fundo transparente (janela inteira)  │
│ "peek": a página prévia do Peek, sobre o overlay                        │
└─────────────────────────────────────────────────────────────────────────┘
```

| Webview | Arquivo | IPC (comandos Rust) |
| --- | --- | --- |
| `ui` | `ui/index.html`, `app.js` | sim |
| `panel` | `ui/settings.html`, `settings.js` | sim |
| `overlay` | `ui/overlay.html`, `overlay.js` | sim |
| `tab-<id>`, `peek` | sites da internet, ou `ui/newtab.html` | **não** |

As permissões estão em `src-tauri/capabilities/default.json`: só as três webviews da interface
podem chamar comandos. Os sites nunca têm acesso ao IPC.

### Como as páginas falam com o app sem IPC

- **Atalhos dentro das páginas** (Ctrl+T, Ctrl+K, Shift+clique…): um script injetado nas abas
  (`shortcuts_js` em `main.rs`) chama `window.open("https://blazon-cmd.invalid/<comando>")`. O
  `on_new_window` do Rust intercepta esse endereço reservado, nega a janela e roda o comando.
  Uma página maliciosa conseguiria no máximo abrir ou fechar abas.
- **Página inicial**: depois de carregar, o Rust chama `window.crestStart({...})` com o Space e
  os sites mais visitados.

### Regra importante de threads

Criar uma webview na thread principal trava o Windows. Por isso:
- todos os comandos são `async`;
- callbacks do WebView2 usam `spawn` antes de criar algo;
- o redimensionamento da janela só reposiciona (`layout(app, false)`).

## Módulos Rust (`src-tauri/src`)

| Arquivo | Responsabilidade |
| --- | --- |
| `main.rs` | Modelo de dados (`Data`, `Space`, `Tab`, `Folder`, `Settings`), comandos, layout das webviews, persistência, histórico, sono e arquivo de abas, rotas de links, instância única |
| `overlay.rs` | Paleta e Peek |
| `blocker.rs` | Bloqueador: motor `adblock` (Brave), listas EasyList/EasyPrivacy, atualização semanal via `curl.exe`, CSS para esconder anúncios |
| `webview2.rs` | APIs COM do WebView2: filtro de requisições (bloqueio), áudio e mudo, pasta de downloads, senhas e preenchimento |
| `hello.rs` | Windows Hello (`UserConsentVerifier`) para Spaces bloqueados |
| `import.rs` | Importação: Arc, família Chromium, família Firefox e HTML de favoritos |
| `sqlite.rs` | Leitura mínima de SQLite via `winsqlite3.dll` do Windows (histórico e favoritos do Firefox) |
| `reader.js` + `vendor/Readability.js` | Modo leitura, injetado na aba |

## Interface (`ui/`)

| Arquivo | Responsabilidade |
| --- | --- |
| `index.html`, `app.js`, `style.css` | Barra lateral, menus, Arquivo/Histórico, aparência |
| `settings.*` | Painel de configurações e Studio |
| `overlay.*` | Paleta de comandos e barra do Peek |
| `newtab.*` | Página inicial |
| `houses.js` | Paletas e padrões da barra das 9 casas, opções de aparência e presets |
| `crest.js` | Gerador de brasões em SVG (formatos, campos, faixas, bordas, acabamentos, armas das casas) |
| `charges.js` | As figuras heráldicas (caminhos SVG) |

## Dados no disco

Tudo fica em `%APPDATA%\app.blazon.browser\`:

| Arquivo/pasta | Conteúdo |
| --- | --- |
| `state.json` | Spaces, abas, pastas, lado a lado, Arquivo e configurações; salvo a cada mudança, com escrita atômica |
| `history.json` | Histórico por Space (até 5.000 entradas), salvo a cada 30 s e ao fechar |
| `spaces\<id>\` | Pasta de dados do WebView2 de cada Space: cookies, cache, senhas |
| `adblock\` | Listas de bloqueio baixadas (as do instalador ficam nos recursos do app) |

- Instalações feitas com o nome anterior (`app.crest.windows`) são movidas automaticamente na
  primeira execução.
- Spaces privados não gravam histórico, e as abas deles são descartadas ao abrir o app.

## Fluxo de estado

1. A interface chama um comando (`invoke('activate_tab', …)`).
2. O Rust muda `Data` dentro de `with_data`, que trava o mutex só durante a mudança.
3. `apply_layout` cria, mostra, esconde e posiciona as webviews.
4. `commit` salva `state.json` e emite o evento `state` para `ui` e `panel`, que redesenham.

Os campos `look` dos Spaces e `appearance` das configurações são JSON livres, controlados pela
interface. Assim, novas opções visuais não exigem mudanças no Rust.
