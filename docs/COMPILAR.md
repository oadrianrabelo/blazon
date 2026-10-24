# Compilar e publicar

## Requisitos (Windows)

- [Rust](https://rustup.rs) estável (alvo `x86_64-pc-windows-msvc`) e o Visual Studio Build
  Tools com "Desenvolvimento para desktop com C++".
- [Node.js](https://nodejs.org) 22.
- WebView2 Runtime (já vem no Windows 10/11).

## Rodar e gerar o instalador

```sh
npm ci
npm run dev      # abre o app em modo desenvolvimento
npm run build    # gera src-tauri/target/release/bundle/{nsis,msi}
```

Para o bloqueador funcionar em builds locais, baixe as listas antes:

```sh
mkdir src-tauri\adblock
curl -L -o src-tauri\adblock\easylist.txt https://easylist.to/easylist/easylist.txt
curl -L -o src-tauri\adblock\easyprivacy.txt https://easylist.to/easylist/easyprivacy.txt
```

Sem elas, o app funciona normalmente, só sem bloqueio até baixar as listas sozinho.

### Verificar o código fora do Windows

Dá para checar a compilação para Windows a partir do Linux ou macOS, sem gerar o executável:

```sh
rustup target add x86_64-pc-windows-msvc
cd src-tauri && cargo check --target x86_64-pc-windows-msvc && cargo clippy --target x86_64-pc-windows-msvc
```

## Integração contínua

`.github/workflows/build.yml` roda em `windows-latest` a cada push:
1. instala Node e Rust (com cache);
2. baixa as listas de bloqueio;
3. roda `tauri build`;
4. publica o artefato **`blazon-windows`** (`.exe` NSIS e `.msi`).

Numa tag `v*`, cria também uma **Release** com os instaladores.

### Lançar uma versão

1. Atualize `version` em `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` e `package.json`,
   e acrescente a entrada no `CHANGELOG.md`.
2. `git tag v0.2.0 && git push origin v0.2.0`.

## Instalador

- **NSIS (`.exe`)**: instala por usuário, sem administrador. Os hooks em
  `src-tauri/windows/hooks.nsh` registram o Blazon como navegador (Clients\StartMenuInternet,
  RegisteredApplications, associações http/https/html/pdf) e removem tudo na desinstalação.
- **MSI**: alternativa para implantação corporativa; não registra o navegador.

## Renomear o repositório

O app já se chama Blazon. O repositório ainda se chama `crest-windows-vibe-coded`. Para renomear:

1. No GitHub: **Settings → General → Repository name** → por exemplo `blazon`. O GitHub
   redireciona os endereços antigos.
2. Em cada clone: `git remote set-url origin https://github.com/<usuário>/blazon.git`.

Nada no código depende do nome do repositório.
