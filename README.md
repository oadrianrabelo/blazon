# Crest para Windows

Um navegador leve com **Spaces**, recriado a partir do [Crest](https://github.com/pauljoda/Crest)
(Mac/iOS) de Paul Joda. É uma reescrita para Windows em Rust com
[Tauri 2](https://tauri.app) e o **WebView2**, o Chromium do Edge que já vem no
Windows 10/11. O instalador tem poucos MB.

## Recursos

**Spaces e Crest Studio**
- Cada Space tem cookies, logins, cache, histórico, arquivo e pasta de downloads
  próprios. Também há Spaces **privados** (InPrivate) e Spaces **bloqueados com o Windows Hello**.
- **Crest Studio** em Configurações → Spaces e Studio. Você parte das 9 casas do Crest
  (Winter, Lion, Storm, Dragon, Meadow, Iron, River, Sun e Vigil) ou monta o brasão peça
  por peça:
  - formato, campo, faixa e emblema (34 figuras heráldicas, emoji ou monograma);
  - borda, acabamento e sombra;
  - cores nomeadas pelo que pintam, e padrão e cores do fundo da barra lateral;
  - botões Embaralhar e Desfazer.

**Abas**
- Abas verticais, fixadas, pastas, **lado a lado** e arrastar para reordenar.
- Menu de clique direito, silenciar a aba e reabrir a aba fechada (Ctrl+Shift+T).
- Abas paradas **dormem** para liberar memória. Abas soltas sem uso podem ir sozinhas
  para o **Arquivo** (botão no rodapé), de onde você as restaura.
- **Peek**: Shift+clique num link abre uma prévia flutuante.
- **Paleta de comandos** (Ctrl+K): busca em todas as abas, comandos, Spaces e
  histórico, além de "Tab para buscar" em sites (`yt`, `w`, `gh`, `g`, `m`, `a`, `r`, `ml`, `tr`).

**Aparência** (Configurações → Aparência)
- Presets prontos e barra lateral à esquerda ou à direita, com largura ajustável.
- **Barra flutuante** (Ctrl+Shift+S) e janela sem bordas ou com a moldura do Windows.
- Tamanho, formato e estilo da aba selecionada; fixadas por linha; pastas com contagem.
- Cor de destaque própria, faixas heráldicas ligadas ou desligadas e zoom padrão das páginas.

**Privacidade e links**
- **Bloqueador de anúncios e rastreadores** com o motor do Brave e as listas
  EasyList/EasyPrivacy, atualizadas toda semana. O escudo na barra pausa o bloqueador por site.
- Salvar senhas e preencher formulários pelo WebView2, com dados separados por Space.
- **Modo leitura** (ícone de livro), usando o Readability da Mozilla.
- **Navegador padrão**: links de outros apps abrem no Crest, com **rotas** que mandam cada
  site para um Space.
- **Importar** do Arc (Spaces, pastas e favoritos) e dos favoritos do Chrome, Edge, Brave e Vivaldi.

### Atalhos

| Atalho | Ação |
| --- | --- |
| Ctrl+T / Ctrl+W | Nova aba / fechar |
| Ctrl+Shift+T | Reabrir aba fechada |
| Ctrl+K | Paleta de comandos |
| Ctrl+L | Endereço |
| Ctrl+H | Histórico |
| Ctrl+Tab, Ctrl+1–9 | Trocar de aba |
| Ctrl+Shift+1–9 | Trocar de Space |
| Ctrl+Shift+S | Mostrar/esconder a barra |
| Shift+clique | Peek |
| F12 | DevTools |

## Instalar

Em **Actions → Build Windows**, abra o build mais recente e baixe o artefato
`crest-windows`. Ao criar uma tag `v*`, o build também publica uma **Release**. Prefira o `.exe`: ele
instala sem pedir administrador e registra o Crest como navegador. Depois, escolha o Crest em
Configurações do Windows → Aplicativos → Aplicativos padrão.

## Desenvolver

Você precisa de Rust estável, Node 22 e do WebView2.

```sh
npm ci
npm run dev     # modo desenvolvimento
npm run build   # instaladores em src-tauri/target/release/bundle
```

Para o bloqueador funcionar localmente, baixe `easylist.txt` e `easyprivacy.txt`
para `src-tauri/adblock/`. O CI já faz isso.

Organização do código:
- `src-tauri/src/main.rs`: Spaces, abas, layout e persistência
  (`%APPDATA%\app.crest.windows`).
- `overlay.rs`: paleta e Peek.
- `blocker.rs`: bloqueador de anúncios.
- `webview2.rs`: APIs COM do WebView2 (bloqueio, áudio, downloads, senhas).
- `hello.rs`: Windows Hello.
- `import.rs`: importação de outros navegadores.
- `reader.js`: modo leitura.
- `ui/`:
  - barra lateral (`index.html`, `app.js`);
  - configurações e Studio (`settings.*`);
  - paleta e Peek (`overlay.*`);
  - página inicial (`newtab.*`);
  - gerador de brasões (`crest.js`, `charges.js`) e casas (`houses.js`).

## Fica de fora

O motor WebKit e o que depende dele (tradução da página inteira e FairPlay), a
sincronização via iCloud e as extensões da Chrome Web Store.

## Licença

MPL-2.0, a mesma do Crest original.
- A geometria dos brasões, as casas e as paletas do Crest Studio vêm do Crest (MPL-2.0).
- As figuras heráldicas são arte de Lorc (game-icons.net, CC BY 3.0); veja `ui/CREDITS-heraldry.txt`.
- O bloqueador usa o [adblock-rust](https://github.com/brave/adblock-rust) (MPL-2.0) e as listas
  EasyList/EasyPrivacy.
- O modo leitura usa o
  [Readability](https://github.com/mozilla/readability) (Apache-2.0); veja `src-tauri/src/vendor/`.
