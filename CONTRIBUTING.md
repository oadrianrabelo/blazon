# Contribuindo com o Blazon

Obrigado pelo interesse! O Blazon é um navegador para Windows em Rust, Tauri 2 e
WebView2, inspirado no [Crest](https://github.com/pauljoda/Crest).

## Antes de começar

- Para erros, use **Configurações → Relatar problema** no Blazon: ele abre uma issue já
  preenchida. Para ideias, abra uma issue "Ideia ou pedido".
- **Falhas de segurança** não vão em issues públicas: veja [SECURITY.md](SECURITY.md).
- Como compilar e a organização do código: [docs/COMPILAR.md](docs/COMPILAR.md) e
  [docs/ARQUITETURA.md](docs/ARQUITETURA.md).

## Pull requests

1. Crie uma branch a partir da `main`; a `main` só recebe mudanças por pull request.
2. Rode localmente:
   - `node --test tests/ui.test.mjs`
   - em `src-tauri`: `cargo fmt --check`, `cargo clippy --all-targets` e `cargo test`
3. Abra o pull request preenchendo o modelo. O CI compila, testa, instala o Blazon num
   Windows limpo e tira prints das telas (branch `ci-screenshots`); o workflow Security
   procura vulnerabilidades conhecidas.
4. Uma versão nova sai quando a `main` recebe uma versão diferente em
   `src-tauri/tauri.conf.json`: o CI assina o instalador e publica a Release.

## Estilo

- Textos da interface em português; a tradução para inglês fica em `ui/i18n.js`.
- Comentários explicam o porquê, não o quê.
- Mudanças que alguém vê ou sente entram no CHANGELOG.md.

Ao contribuir, você concorda que sua contribuição seja distribuída sob a
[MPL-2.0](LICENSE).
