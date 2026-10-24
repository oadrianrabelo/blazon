# Crest para Windows

Um navegador leve com **Spaces**, inspirado no [Crest](https://github.com/pauljoda/Crest)
(Mac/iOS) de Paul Joda. Esta é uma reescrita independente para Windows, feita em
Rust com [Tauri 2](https://tauri.app) e o **WebView2**: o Chromium (Edge) que já vem
instalado no Windows 10/11. O instalador tem poucos MB e não traz um navegador
inteiro junto.

## O que funciona

- **Visual do Crest**: cada Space tem uma "casa" do Crest Studio (Winter, Lion,
  Storm, Dragon, Meadow, Iron, River, Sun, Vigil) com brasão, paleta e faixas
  heráldicas no fundo, ou uma cor própria. A página fica num cartão recuado e a
  janela não tem bordas, com os controles na barra lateral.
- **Spaces** isolados: cada um tem seus próprios cookies, logins, cache e
  armazenamento (pastas de dados separadas do WebView2), além de nome e cor.
- Abas verticais na barra lateral, abas **fixadas** (clique direito fixa ou
  desafixa), reordenação arrastando, clique do meio fecha.
- Barra de endereço com busca (DuckDuckGo, Google, Bing, Brave ou Startpage).
- A sessão é restaurada ao reabrir. As abas só carregam quando você as abre,
  para economizar memória.
- Pop-ups de login (OAuth) abrem em janela própria; os links "abrir em nova
  janela" viram abas.
- A sidebar pode ficar à esquerda ou à direita, e o tema claro/escuro segue o Windows.
- DevTools (F12), zoom (Ctrl +/−/roda) e downloads nativos do Edge.
- **Histórico** por Space, com sugestões na barra de endereço (abas abertas e
  páginas visitadas) e uma janela de histórico pesquisável (Ctrl+H).
- **Spaces privados** (InPrivate): sem histórico, e as abas somem ao fechar.
- **Abas dormindo**: as abas em segundo plano liberam memória depois de 30 min
  (configurável). Abas fixadas nunca dormem.
- Reabrir aba fechada (Ctrl+Shift+T).
- **Pastas** dentro do Space (clique direito na lista → Nova pasta, ou arraste
  abas para a pasta), e menu de contexto nas abas: fixar, mover, duplicar, copiar link.
- **Navegador padrão**: o instalador `.exe` registra o Crest no Windows. Para
  usar, escolha-o em Configurações → Aplicativos → Aplicativos padrão. Links de
  outros apps abrem como abas na janela já aberta.

### Atalhos

| Atalho | Ação |
| --- | --- |
| Ctrl+T | Nova aba |
| Ctrl+W | Fechar aba |
| Ctrl+Shift+T | Reabrir aba fechada |
| Ctrl+H | Histórico |
| Ctrl+L | Focar o endereço |
| Ctrl+Tab / Ctrl+Shift+Tab | Próxima / anterior |
| Ctrl+1–9 | Ir para a aba (9 = última) |
| Ctrl+Shift+1–9 | Trocar de Space |
| Alt+← / Alt+→, F5 | Voltar / avançar, recarregar |

## Instalar

Baixe o `.exe` (NSIS, sem precisar de admin) ou o `.msi` em
**Actions → Build Windows → crest-windows** (o artefato de cada build) ou em
**Releases** quando houver uma tag `v*`. Se o WebView2 não estiver instalado,
o instalador baixa ele sozinho.

## Desenvolver

Requisitos: Rust estável, Node 22 e o WebView2 (já presente no Windows 10/11).

```sh
npm ci
npm run dev     # roda em modo desenvolvimento
npm run build   # gera os instaladores em src-tauri/target/release/bundle
```

Estrutura:

- `src-tauri/src/main.rs`: núcleo (Spaces, abas, webviews, persistência em
  `%APPDATA%\app.crest.windows\state.json`).
- `ui/`: barra lateral em HTML/CSS/JS puro, mais a página de nova aba.

## Ainda não tem

O motor WebKit e o que depende dele (Reader, tradução), extensões da Chrome Web Store,
sincronização e o Crest Studio completo (aqui só dá para escolher a casa ou uma cor).

## Licença

MPL-2.0, a mesma do Crest original. Os brasões em `ui/crests/` e as paletas
das casas vêm do Crest (MPL-2.0); as figuras heráldicas são arte de Lorc
(game-icons.net, CC BY 3.0). Veja `ui/crests/CREDITS.txt`.
