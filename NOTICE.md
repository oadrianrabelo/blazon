# Créditos e licenças

O Blazon é distribuído sob a [Mozilla Public License 2.0](LICENSE).

| Componente | Origem | Licença | Onde |
| --- | --- | --- | --- |
| Conceito de Spaces, visual, casas, paletas e geometria dos brasões | [Crest](https://github.com/pauljoda/Crest), de Paul Joda | MPL-2.0 | `ui/houses.js`, `ui/crest.js`, importador do Arc em `src-tauri/src/import.rs` |
| Figuras heráldicas | Lorc, [game-icons.net](https://game-icons.net) (via Crest) | CC BY 3.0 | `ui/charges.js`, lista completa em `ui/CREDITS-heraldry.txt` |
| Motor do bloqueador | [adblock-rust](https://github.com/brave/adblock-rust), da Brave | MPL-2.0 | dependência `adblock` |
| Listas de bloqueio | [EasyList e EasyPrivacy](https://easylist.to) | GPLv3 / CC BY-SA 3.0 | baixadas no build e em tempo de execução |
| Modo leitura | [Readability](https://github.com/mozilla/readability), da Mozilla | Apache-2.0 | `src-tauri/src/vendor/` |
| Fontes da interface | [Inter](https://rsms.me/inter/), de Rasmus Andersson; [Geist](https://vercel.com/font), da Vercel; [Figtree](https://github.com/erikdkennedy/figtree), de Erik Kennedy (via Fontsource) | SIL OFL 1.1 | `ui/fonts/`, licenças em `ui/fonts/OFL-*.txt` |
| Framework | [Tauri](https://tauri.app) e [wry](https://github.com/tauri-apps/wry) | MIT / Apache-2.0 | dependências Rust |
| Motor das páginas | Microsoft Edge WebView2 Runtime | Licença da Microsoft | componente do Windows |

As figuras "Sol" e "Peixe" foram desenhadas para o Blazon como formas simples (no Crest elas
vêm dos SF Symbols da Apple, que não podem ser redistribuídos).
