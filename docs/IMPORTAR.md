# Importar de outros navegadores

Abra **Configurações → Importar**. O Blazon lista os navegadores e **perfis** encontrados no PC.
Para cada um você escolhe:
- **o quê**: favoritos e/ou histórico (no Arc: Spaces e/ou histórico);
- **para onde**: o Space atual ou um Space novo com o nome do navegador.

## O que vem de cada navegador

| Navegador | Favoritos | Histórico | Outros |
| --- | --- | --- | --- |
| Arc | Abas fixadas e pastas de cada Space | ✓ | Cada Space do Arc vira um Space no Blazon; os favoritos do topo viram abas fixadas |
| Google Chrome | ✓ | ✓ | Todos os perfis |
| Microsoft Edge | ✓ | ✓ | Todos os perfis |
| Brave | ✓ | ✓ | Todos os perfis |
| Vivaldi | ✓ | ✓ | Todos os perfis |
| Chromium | ✓ | ✓ | Todos os perfis |
| Opera / Opera GX | ✓ | ✓ | |
| Firefox | ✓ | ✓ | Todos os perfis |
| Zen, LibreWolf, Waterfox, Floorp | ✓ | ✓ | Todos os perfis |
| Qualquer navegador | Arquivo HTML exportado | — | Use "Exportar favoritos" no outro navegador e **Escolher arquivo…** no Blazon |

## Como os dados chegam

- **Favoritos** viram **pastas** no Space. Pastas aninhadas viram nomes com caminho, por
  exemplo "Barra de favoritos / Trabalho / Clientes". As abas importadas começam
  **dormindo**: só carregam quando você clica.
- **Histórico**: as últimas 5.000 páginas de cada perfil entram no histórico do Space e
  alimentam as sugestões, a paleta e os sites mais visitados da página inicial. Endereços
  repetidos são juntados (somam as visitas e ficam com a data mais recente).
- Só endereços `http` e `https` são importados; bookmarklets (`javascript:`) e páginas internas
  ficam de fora.
- Cada importação aceita no máximo 3.000 favoritos, para não lotar a barra.

## Onde o Blazon procura

| Navegador | Pasta |
| --- | --- |
| Chrome | `%LOCALAPPDATA%\Google\Chrome\User Data\<perfil>` |
| Edge | `%LOCALAPPDATA%\Microsoft\Edge\User Data\<perfil>` |
| Brave | `%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<perfil>` |
| Vivaldi | `%LOCALAPPDATA%\Vivaldi\User Data\<perfil>` |
| Chromium | `%LOCALAPPDATA%\Chromium\User Data\<perfil>` |
| Opera / Opera GX | `%APPDATA%\Opera Software\Opera Stable` / `Opera GX Stable` |
| Firefox | `%APPDATA%\Mozilla\Firefox\Profiles\<perfil>` |
| Zen / LibreWolf / Waterfox / Floorp | `%APPDATA%\zen`, `librewolf`, `Waterfox`, `Floorp` + `\Profiles\<perfil>` |
| Arc | `%LOCALAPPDATA%\Packages\TheBrowserCompany.Arc_*\LocalCache\Local\Arc` |

- Nos navegadores Chromium, os perfis são as pastas `Default` e `Profile N`. Os nomes vêm do
  arquivo `Local State`.
- O histórico fica num banco SQLite que o navegador mantém travado enquanto está aberto. O
  Blazon **copia** o arquivo para uma pasta temporária, lê a cópia e apaga em seguida. Se der
  erro de leitura, feche o outro navegador e tente de novo.
- A leitura usa o `winsqlite3.dll` que já vem no Windows.

## O que não dá para importar

- **Senhas**: o WebView2 (o motor do Blazon) não tem nenhuma forma de adicionar senhas salvas
  de fora, e os navegadores atuais protegem as próprias senhas com criptografia ligada a eles.
  Use um gerenciador de senhas ou deixe o Blazon salvar conforme você entra nos sites.
- **Abas abertas, extensões, cookies e logins**: os logins precisam ser refeitos em cada Space,
  porque esse isolamento é justamente a proposta dos Spaces.
