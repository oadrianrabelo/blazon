# Arquitetura

O Blazon é um app [Tauri 2](https://tauri.app) com várias webviews numa mesma janela. O núcleo
em Rust guarda o estado (Spaces, abas, pastas, configurações) e controla as webviews. A
interface é HTML/CSS/JS puro, sem framework nem etapa de build.

## Webviews de cada janela

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

### Várias janelas

- A janela principal se chama `main`; as extras, `win-<n>`.
- As webviews da interface de cada janela levam o sufixo dela: `ui-win-3`, `panel-win-3`,
  `overlay-win-3`, `peek-win-3`.
- `AppState.windows` diz que Space cada janela mostra. `Data.active_space` é sempre o Space da
  janela focada, então os comandos continuam agindo "no Space atual".
- `sync_windows` reconcilia os dois: se o Space ativo já está em outra janela, essa janela vem
  para a frente.
- `layout` posiciona cada janela. Abas de um Space que mudou de janela são movidas com
  `reparent`.
- `commit` junta as mudanças: em até um quadro (12 ms) envia a cada barra lateral o estado com o Space da janela dela, e uma thread grava o `state.json` no máximo a cada meio segundo (e na hora ao fechar o navegador).

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
| `main.rs` | Estado do app (`AppState`), `commit`, comandos gerais, script de atalhos das páginas e a inicialização |
| `model.rs` | Modelo de dados (`Data`, `Space`, `Tab`, `Folder`, `Settings`…) e regras de reparo |
| `tabs.rs` | Webviews das abas, navegação, fixar, pastas, lado a lado, sono, arquivo, modo leitura |
| `spaces.rs` | Criar, trocar, ordenar e bloquear Spaces; rotas de links |
| `windowing.rs` | Janelas, layout das webviews, painel de configurações, barra flutuante |
| `history.rs` | Histórico, sugestões, página inicial e mesclagem de histórico importado |
| `importer.rs` | Aplica importações no estado (o parsing fica em `import.rs`) |
| `downloads.rs` | Lista de downloads |
| `profile.rs` | Permissões de sites, extensões e uso de memória por Space |
| `passwords.rs` | Cofre de senhas (DPAPI), importação do Chromium (AES-GCM pelo CNG do Windows) e de CSV, preenchimento |
| `sidepanel.rs` | Painel lateral de extensões: página da extensão numa coluna ao lado do site |
| `transition.rs` | Transição entre páginas: foto da página antiga numa camada (`fx.html`) acima do cartão, que esmaece ou desliza |
| `keys.rs` | Atalhos personalizáveis: tabela de comandos e combinações padrão, trocas salvas em `settings.shortcuts`, mapa enviado às páginas abertas |
| `fonts.rs` | Lista as famílias de fontes instaladas (DirectWrite) para Aparência → Texto |
| `quick.rs` | Janela rápida: janela pequena para links de outros apps, com escolha de Space |
| `backup.rs` | Exportar, restaurar (substituir ou mesclar) e backup automático numa pasta |
| `support.rs` | Relatório de problema, travamentos, atualizações, Aplicativos padrão |
| `overlay.rs` | Paleta e Peek; fotos da página (`CapturePreview`) para o vidro da paleta e a animação do Peek |
| `blocker.rs` | Bloqueador: motor `adblock` (Brave), listas EasyList/EasyPrivacy, atualização semanal via `curl.exe`, CSS para esconder anúncios |
| `webview2.rs` | APIs COM do WebView2: filtro de requisições (bloqueio), áudio e mudo, pasta de downloads, senhas e preenchimento, memória dos processos |
| `hello.rs` | Windows Hello (`UserConsentVerifier`) para Spaces bloqueados |
| `import.rs` | Importação: Arc, família Chromium, família Firefox e HTML de favoritos |
| `sqlite.rs` | Leitura mínima de SQLite via `winsqlite3.dll` do Windows (histórico e favoritos do Firefox) |
| `error.js` | Página de site fora do ar, injetada quando a navegação falha por rede |
| `reader.js` + `vendor/Readability.js` | Modo leitura, injetado na aba |
| `diag.rs` | Log em arquivo, relatório de travamento (panic hook), versões do Windows e do WebView2 |
| `updater.rs` | Atualizações: última Release no GitHub, download via `curl.exe`, verificação SHA-256, instalador em modo passivo |

## Interface (`ui/`)

| Arquivo | Responsabilidade |
| --- | --- |
| `index.html`, `app.js`, `style.css` | Barra lateral, menus, Arquivo/Histórico, aparência |
| `settings.*` | Painel de configurações e Studio |
| `overlay.*` | Paleta de comandos e barra do Peek |
| `newtab.*` | Página inicial |
| `motion.css`, `motion.js` | Animações do Crest: os tempos do `CrestMotion`, molas do SwiftUI convertidas em curvas `linear()`, reordenação estilo FLIP, seleção que desliza, troca de Space, painéis flutuantes e o modo de movimento reduzido do Windows |
| `guide.html`, `guide.js` | Primeiros passos: lições com uma barra de prática própria |
| `quick.html`, `quick.js` | Barra da janela rápida |
| `glass.js`, `glass.css` | Liquid Glass: filtro SVG de refração por tamanho de superfície, brilho e luz que segue o mouse |
| `welcome.js` | Boas-vindas da primeira abertura (no overlay, sobre a janela inteira) |
| `i18n.js` | Tradução para inglês: dicionário e padrões aplicados a todo texto exibido (observador de DOM) |
| `diag.js` | Envia erros de JavaScript da interface para o log |
| `houses.js` | Paletas e padrões da barra das 9 casas, opções de aparência e presets |
| `crest.js` | Gerador de brasões em SVG (formatos, campos, faixas, bordas, acabamentos, armas das casas) |
| `charges.js` | As figuras heráldicas (caminhos SVG) |

## Dados no disco

Tudo fica em `%APPDATA%\app.blazon.browser\`:

| Arquivo/pasta | Conteúdo |
| --- | --- |
| `state.json` | Spaces, abas, pastas, lado a lado, Arquivo e configurações; salvo no máximo a cada meio segundo depois de uma mudança, com escrita atômica |
| `history.json` | Histórico por Space (até 5.000 entradas), salvo a cada 30 s e ao fechar |
| `spaces\<id>\` | Pasta de dados do WebView2 de cada Space: cookies, cache, senhas |
| `adblock\` | Listas de bloqueio baixadas (as do instalador ficam nos recursos do app) |
| `logs\` | `blazon.log` (rotação em 1 MB), `crash.txt` do último travamento |
| `state.broken-*.json` | Cópia de um `state.json` ilegível, guardada em vez de ser sobrescrita |

- Instalações feitas com o nome anterior (`app.crest.windows`) são movidas automaticamente na
  primeira execução.
- Spaces privados não gravam histórico, e as abas deles são descartadas ao abrir o app.

## Fluxo de estado

1. A interface chama um comando (`invoke('activate_tab', …)`).
2. O Rust muda `Data` dentro de `with_data`, que trava o mutex só durante a mudança.
3. `apply_layout` cria, mostra, esconde e posiciona as webviews.
4. `commit` agenda o evento `state` para `ui` e `panel` e a gravação do `state.json`; a barra lateral redesenha no próximo quadro, só se algo que ela mostra mudou.

Os campos `look` dos Spaces e `appearance` das configurações são JSON livres, controlados pela
interface. Assim, novas opções visuais não exigem mudanças no Rust.
