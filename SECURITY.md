# Segurança do Blazon

## O que é atualizado por quem

| Parte | Quem atualiza | Como |
| --- | --- | --- |
| Motor das páginas (WebView2, o Chromium do Edge) | Microsoft | Sozinho, junto com o Edge, sem depender de uma versão nova do Blazon |
| Proteção contra phishing e downloads perigosos | Microsoft (SmartScreen) | Ligada nas páginas; Configurações → Privacidade |
| Dependências Rust e npm | Dependabot | PRs semanais; correções de segurança abrem na hora |
| Avisos de vulnerabilidade | Workflow `Security` | `cargo audit` e `npm audit` a cada push e toda segunda-feira |
| O próprio Blazon | Você | Merge dos PRs; o CI testa no Windows antes |

## Atualizações assinadas

Cada instalador publicado vem com `<instalador>.sig`, a assinatura Ed25519 de
`blazon-update:<versão>:<sha256 do instalador>`. O Blazon só instala uma
atualização cuja assinatura confere com a chave pública embutida nele
(`UPDATE_KEY` em `src-tauri/src/updater.rs`). Uma versão publicada sem a chave
privada, mesmo por alguém com acesso ao repositório, é recusada.

- A chave privada fica só no secret `BLAZON_UPDATE_KEY` do repositório (PEM
  PKCS#8). O CI assina com `scripts/sign-update.mjs`; sem o secret, a versão não
  é publicada.
- **Trocar a chave**: gere um par novo, publique uma versão com a chave pública
  nova em `UPDATE_KEY` assinada ainda com a chave antiga, e só depois troque o
  secret.
- Se a chave privada vazar, troque-a assim que possível (como acima) e avise na
  página de versões.

## Relatar uma falha

Abra uma issue sem detalhes públicos pedindo um contato, ou use
"Report a vulnerability" na aba Security do repositório.
