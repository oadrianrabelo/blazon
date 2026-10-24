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

## Testes

```sh
npm test         # testes da interface (node --test) e do Rust (cargo test)
```

- **Rust**: endereços, rotas de links, reparo de dados, lado a lado, busca por Space, leitura de
  `state.json` antigos, importação, bloqueador, versões.
- **Interface**: todas as casas, figuras e opções do Studio geram SVG válido; tradução para inglês.

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
3. O CI publica a Release com `Blazon_<versão>_x64-setup.exe` e o `.msi`. Quem já tem o Blazon
   instalado recebe o aviso de atualização em até um dia.

### Como funciona a atualização

- O app consulta a API do GitHub pela última Release (`updater.rs`).
- Se ela for mais nova que a versão instalada, o app baixa o instalador NSIS com o `curl.exe` do
  Windows.
- O download é conferido com o SHA-256 que o GitHub publica para o arquivo.
- O instalador roda com `/P /R /UPDATE`: sem perguntas, e reabre o Blazon no fim.
- Não é preciso chave nem segredo. Se o repositório for renomeado, o GitHub redireciona a API.

## Assinatura de código

Sem assinatura, o Windows mostra "O Windows protegeu o computador" na primeira instalação
(clique em **Mais informações → Executar assim mesmo**). Para assinar:

1. Arranje um certificado de assinatura de código:
   - [SignPath](https://signpath.org), gratuito para projetos open source;
   - [Azure Trusted Signing](https://azure.microsoft.com/products/trusted-signing), barato e mensal;
   - um certificado OV/EV de uma autoridade (Sectigo, DigiCert…).
2. Com um `.pfx`, crie os segredos do repositório em **Settings → Secrets and variables →
   Actions**:
   - `WINDOWS_CERTIFICATE`: o `.pfx` em Base64 (`[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))`);
   - `WINDOWS_CERTIFICATE_PASSWORD`: a senha.
3. O passo **Sign installers** do CI passa a assinar o `.exe` e o `.msi` sozinho. Sem os segredos,
   o passo é pulado.

Certificados EV ou o Azure Trusted Signing eliminam o aviso do SmartScreen na hora. Certificados
OV ganham reputação aos poucos.

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
