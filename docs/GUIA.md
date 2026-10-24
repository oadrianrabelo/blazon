# Guia de uso do Blazon

## A janela

- **Barra lateral** (esquerda por padrão), de cima para baixo:
  - botões de navegação, escudo do bloqueador, modo leitura e controles da janela;
  - barra de endereço;
  - abas fixadas;
  - cabeçalho do Space;
  - pastas e abas;
  - rodapé com configurações, seletor de Spaces e Arquivo.
- **Cartão da página**: o site aberto, recuado sobre o fundo heráldico do Space.
- A janela não tem bordas por padrão. Para usar a barra de título do Windows, vá em
  Configurações → Aparência → **Barra de título do Windows**.

## Boas-vindas

Na primeira vez que o Blazon abre, uma tela de boas-vindas (como a do Crest) ajuda a:
1. escolher a casa do primeiro Space;
2. dar nome a ele e criar outros Spaces (Trabalho, Estudos, Pesquisa);
3. importar favoritos e histórico de outros navegadores;
4. tornar o Blazon o navegador padrão.

Para ver de novo: **Configurações → Sobre → Rever boas-vindas**.

## Spaces

Um Space é um navegador dentro do navegador. Cada Space tem separados:
- cookies e logins (você pode estar logado em contas diferentes do mesmo site);
- cache, armazenamento e senhas salvas;
- histórico, Arquivo e sugestões da barra de endereço;
- abas, fixadas, pastas e pasta de downloads.

| Ação | Como |
| --- | --- |
| Criar | **＋** no seletor do rodapé; o Space novo abre direto no Studio |
| Trocar | Clique no brasão do rodapé ou use **Ctrl+Shift+1–9** |
| Editar | Clique no nome do Space ou em **…**, ou dê dois cliques no brasão do rodapé |
| Excluir | Studio → **Excluir Space** (apaga abas, cookies e dados dele) |

### Spaces privados

No Studio, use **Novo privado**. O Space usa o modo InPrivate do WebView2:
- nada vai para o histórico;
- cookies não ficam no disco;
- as abas somem quando você fecha o Blazon.

### Spaces bloqueados

No Studio, use **Bloquear com Windows Hello**. Depois disso:
- abrir o Space pede PIN, rosto ou digital;
- o seletor mostra um cadeado;
- as abas dele não aparecem na paleta.

Para bloquear de novo tudo o que foi aberto, use a paleta → **Bloquear Spaces agora**.
É preciso ter o Windows Hello configurado no PC.

### Várias janelas

- **Ctrl+N** (ou paleta → Nova janela) abre uma janela com um Space que ainda não está aberto
  em nenhuma janela. Se todos já estiverem abertos, cria um Space novo.
- Clique direito num brasão do rodapé → **Abrir em nova janela** leva aquele Space (e suas
  abas) para uma janela própria.
- Cada Space aparece em uma janela por vez. Escolher um Space que já está aberto em outra janela
  traz essa janela para a frente.
- As janelas extras reabrem na próxima vez. Fechar a janela principal fecha o Blazon.

### Ordem e busca por Space

- Arraste os brasões no rodapé para reordenar, ou use ↑/↓ no Studio.
- No Studio, cada Space pode ter seu próprio mecanismo de busca. Isso vale para a barra, a
  paleta e a página inicial.

## Studio de brasões

Abra em Configurações → **Spaces e Studio**. Ao lado de cada mudança, você vê a prévia
do brasão e da barra lateral.

| Etapa | O que muda |
| --- | --- |
| Modelo | As 9 casas clássicas (Winter, Lion, Storm, Dragon, Meadow, Iron, River, Sun, Vigil): brasão e barra juntos |
| Formato | Escudo, escudo francês, redondo, oval, losango, hexágono, octógono, quadrado, selo, flâmula, distintivo |
| Campo | Liso, vertical, horizontal, diagonal, chevron, quartelado, cruzado, gomos, barras, listras, xadrez (com quantidade) |
| Faixa | Pala, faixa, banda, asna, cruz, aspa, chefe, bordadura, pairle, pira, cantão, besante |
| Emblema | 34 figuras heráldicas, um emoji, um monograma de até 2 letras ou nenhum; tamanho ajustável |
| Borda | Linha, linha dupla, contorno, louros, raios, anel duplo, selo, pérolas; espessura e detalhes |
| Acabamento | Fosco, brilho ou relevo; sombra suave, elevada ou nenhuma |
| Cores | Campo, segundo campo, faixa, emblema, borda e contorno; fundo e faixas da barra lateral |
| Barra lateral | Padrão do fundo: diagonal, chevron, faixas, dividido, quartelado, listras, xadrez, losangos, degradê ou liso |

Os botões **Embaralhar** e **Desfazer** ficam no topo.

## Abas

- **Nova aba**: Ctrl+T ou "＋ Nova aba". A página inicial mostra o brasão do Space, uma
  busca e os sites mais visitados naquele Space.
- **Fixar**: clique direito → Fixar. As fixadas ficam em blocos no topo e não dormem enquanto o
  Space estiver aberto numa janela.
- **Pastas**: clique direito numa área vazia da lista → Nova pasta, ou clique direito numa
  aba → "Nova pasta com esta aba". Arraste abas para dentro e para fora. Clique direito na pasta
  → **Cor e ícone…** escolhe uma cor ou um emoji.
- **Voltar depois de reabrir**: cada aba guarda os últimos endereços. Depois de reiniciar o app, o
  botão Voltar (Alt+←) continua funcionando.
- **Reordenar**: arraste.
- **Fechar**: ✕, clique do meio ou Ctrl+W.
- **Reabrir**: Ctrl+Shift+T.
- **Lado a lado**: clique direito → "Lado a lado com nova aba" ou com uma aba existente. Para
  desfazer, clique direito → "Desfazer lado a lado".
- **Som**: o ícone de alto-falante aparece quando a aba toca áudio. Clique nele para silenciar.
- **Abas dormindo**: depois de um tempo sem uso (padrão: 30 min), a aba libera memória e
  recarrega quando você volta. O prazo fica em Configurações → Geral. Abas tocando som não
  dormem.
- **Memória**: Spaces que não estão abertos em nenhuma janela dormem por completo, fixadas
  incluídas, e liberam o processo deles. Configurações → Geral → **Memória** mostra quanto cada
  Space usa e tem o botão **Dormir agora**.
- **Traduzir página**: clique direito na aba → Traduzir página (ou paleta). A página abre pelo
  Google Tradutor no idioma da interface.
- **Arquivo**: abas fechadas vão para o Arquivo (ícone de caixa no rodapé), junto com abas
  paradas, se você ativar Configurações → Geral → "Arquivar abas paradas após". Clique num item
  para restaurá-lo. A aba **Histórico** do mesmo diálogo mostra tudo o que você visitou.

## Barra de endereço e paleta

- Digite um endereço ou uma pesquisa. As sugestões vêm das abas abertas e do histórico do Space.
- **Ctrl+K** abre a **paleta de comandos** sobre a página. Ela busca:
  - abas de todos os Spaces (exceto bloqueados);
  - comandos;
  - Spaces;
  - histórico.
- **Tab para buscar**: digite `yt`, `w`, `gh`, `g`, `m`, `a`, `r`, `ml` ou `tr` e aperte Tab
  para pesquisar direto em YouTube, Wikipédia, GitHub, Google, Maps, Amazon, Reddit, Mercado
  Livre ou no Tradutor.
- O mecanismo de busca padrão fica em Configurações → Busca. Você pode usar um endereço próprio.

## Peek e links

- **Peek**: Shift+clique num link abre o site num cartão flutuante. Ali você pode usar
  "Abrir como aba", "Copiar link" ou Esc para fechar. A tecla muda em Configurações → Links.
- **Links de outros apps**: com o Blazon como navegador padrão, eles abrem como abas na
  janela já aberta.
- **Janela rápida**: em Configurações → Links → Abrir em, escolha **Janela rápida**. O link abre
  numa janela pequena, sem trazer o navegador para frente. No alto, escolha o Space e clique em
  **Abrir no Blazon** para levar a página como aba, ou feche a janela.
- **Rotas**: em Configurações → Links você manda endereços com um trecho (ex.: `figma.com`)
  sempre para um Space, e escolhe o Space padrão para o resto. Em "Abrir em" também dá para
  escolher **Peek**: o link de outro app abre como prévia flutuante, sem virar aba.

## Privacidade

- **Bloqueador**: ligado por padrão. O escudo na barra mostra o estado e pausa ou retoma o
  bloqueio no site atual. A lista de sites pausados fica em Configurações → Privacidade.
- **Senhas**: o Blazon oferece salvar senhas e preencher formulários pelo armazenamento do
  WebView2, separado por Space. Dá para desligar em Configurações → Privacidade.
- **Downloads**: escolha uma pasta por Space em Configurações → Privacidade.
- **Permissões** (câmera, microfone, localização, notificações): cada site pede na primeira vez
  e a escolha fica salva no Space.

## Senhas

Em **Configurações → Senhas**:
- **Importar de um navegador**: Chrome, Edge, Brave, Vivaldi e Opera, todos os perfis. Versões
  novas do Chrome protegem as senhas com uma chave que só o próprio Chrome abre; nesse caso o
  Blazon avisa quantas ficaram de fora, e você exporta em CSV pelo gerenciador de senhas do
  Chrome.
- **Importar arquivo CSV**: exportado pelo Chrome, Edge, Firefox, Safari ou Bitwarden. Depois de
  importar, apague o CSV: ele guarda as senhas sem proteção.
- As senhas ficam num cofre criptografado pelo Windows (DPAPI), que só a sua conta do Windows
  abre. **Mostrar** pede o Windows Hello quando ele está configurado.
- **Preencher**: num site com senha salva, aparece uma **chave** na barra lateral. Clique nela
  (e escolha a conta, se houver mais de uma) para preencher usuário e senha. O Blazon só preenche
  se a aba ainda estiver no site da senha.
- Senhas novas continuam sendo salvas pelo WebView2 de cada Space, como antes.

## Downloads

O botão de seta no rodapé abre a lista de downloads da sessão. Ali você vê o progresso e pode
abrir o arquivo, mostrar na pasta, cancelar ou limpar a lista. Quando um download começa,
aparece um aviso na barra. A pasta de cada Space se escolhe em Configurações → Privacidade.

## Permissões de sites

Em Configurações → Privacidade → **Permissões de sites**, escolha o Space para ver as
permissões que os sites já pediram (câmera, microfone, localização, notificações, área de
transferência…). Mude para Permitir, Bloquear ou **Perguntar de novo**.

## Extensões

Em Configurações → **Extensões**:
- Instale extensões **descompactadas**, ou seja, a pasta que contém o `manifest.json`.
- A instalação é por Space; ative, desative ou remova quando quiser.
- O WebView2 não acessa a Chrome Web Store e suporta só parte das APIs de extensão: extensões
  simples (tema, leitura, produtividade) costumam funcionar, as mais complexas podem não funcionar.
- Para extensões que vêm como `.crx`: renomeie para `.zip`, extraia e escolha a pasta.
- **Painel lateral**: o botão de peça no rodapé lista as extensões do Space que têm painel,
  janela própria (popup) ou página de opções. Escolha uma e ela abre numa coluna ao lado da
  página, com um ✕ para fechar. Funciona com extensões instaladas a partir desta versão (o
  Blazon precisa lembrar a pasta para ler o `manifest.json`).

## Site fora do ar

Quando um site não abre (endereço errado, sem internet, servidor fora do ar), o Blazon mostra uma
página própria no estilo do Crest, com dicas, **Tentar de novo** e os detalhes técnicos. Páginas de
erro do próprio site (404, 500) e avisos de certificado continuam como vêm.

## Modo leitura

Clique no ícone de livro na barra (ou paleta → Modo leitura). A página vira um texto limpo, com
tipografia confortável que segue o tema claro/escuro do Windows. Clique de novo para voltar.

## Widgets e downloads

- Acima do rodapé fica um **baralho de cartões**: um para cada aba tocando som (com mudo e ir
  para a aba) e um quando há atualização. Deslize para os lados, use os pontos ou as setas.
- Ao começar um download, o ícone voa até o botão de downloads.

## Primeiros passos

Um guia com prática que não mexe nas suas abas: fixar, pastas, limpar abas soltas, lado a lado,
paleta e Peek. Abre depois das boas-vindas e em **Configurações → Sobre → Abrir guia**.

## Backup e sincronização

Em **Configurações → Backup**:
- **Exportar backup** salva um arquivo `.json` com Spaces, looks do Studio, abas, pastas,
  arquivo, histórico e configurações. Cookies e logins ficam de fora (eles moram no perfil do
  WebView2 de cada Space). Spaces privados não levam abas.
- **Restaurar** tem dois modos:
  - **Substituir tudo** troca os dados atuais pelos do backup e reinicia o Blazon;
  - **Mesclar** acrescenta os Spaces do backup (com abas, pastas e histórico) aos que você já tem.
- **Backup automático**: escolha uma pasta (por exemplo, dentro do OneDrive ou do Google Drive).
  O Blazon mantém `blazon-backup.json` atualizado, no máximo a cada 10 minutos e só quando algo
  mudou, e guarda as 5 últimas cópias datadas. Em outro computador, restaure esse arquivo para
  levar tudo junto.

## Acessibilidade

- Toda a barra lateral funciona pelo teclado: **F6** alterna entre a barra de endereço, a lista
  de abas e a página.
- Na lista de abas: setas ↑/↓, Home e End movem o foco; Enter ou Espaço abre a aba; Delete fecha;
  Shift+F10 ou a tecla de menu abre o menu da aba.
- Nas pastas: Enter, → e ← abrem e fecham.
- Botões de ícone têm nomes para leitores de tela (Narrador, NVDA), e o foco do teclado aparece
  com um contorno.

## Relatar um problema

Quando algo der errado, use **Configurações → Relatar problema** (ou a paleta):
1. Descreva o que você fez e o que aconteceu.
2. Clique em **Gerar relatório**. O relatório traz a versão, o Windows, o WebView2, as configurações
   e o log recente.
3. **Copiar** para colar na conversa, ou **Abrir issue no GitHub**, que já abre a issue preenchida.

O Blazon registra automaticamente:
- erros da interface;
- travamentos do app (na próxima abertura aparece um aviso com o botão **Relatar**);
- páginas que travam (aviso com **Recarregar**).

O log fica em `%APPDATA%\app.blazon.browser\logs` e só guarda domínios, nunca endereços
completos, títulos ou senhas.

## Idioma e atualizações

- **Idioma**: em Configurações → Geral, escolha Automático (segue o Windows), Português ou English.
- **Atualizações**: o Blazon procura uma versão nova uma vez por dia e avisa na barra. Ao clicar em
  Atualizar, ele baixa o instalador da Release no GitHub, confere o SHA-256, instala e reabre
  sozinho. Também dá para procurar manualmente em Configurações → Sobre.

## Aparência

Em Configurações → Aparência você ajusta:

| Grupo | Opções |
| --- | --- |
| Presets | Clássico, Compacto, Suave, Mínimo |
| Janela e barra | Lado, largura, barra **flutuante** (esconde e volta ao encostar o mouse na borda; Ctrl+Shift+S), barra de título do Windows, faixas heráldicas |
| Abas | Tamanho, formato (reto, arredondado, pílula), estilo da selecionada (preenchida, contorno, brilho), seguir a cor do Space, apagar abas dormindo |
| Fixadas | Quantas por linha (ou linhas equilibradas), esticar, fundo |
| Pastas | Mostrar quantidade, ícone na cor do Space |
| Cores | Contorno ao editar o endereço, cor de destaque própria |
| Páginas | Zoom padrão, de 50% a 300% |
| Restaurar tudo | Volta tudo ao padrão |

### Abas no topo ou na lateral

Em **Configurações → Aparência → Abas**, escolha **Na lateral** (a barra do Crest) ou **No topo**
(como no Safari), ou aperte **Ctrl+Shift+L**. No topo, a primeira linha tem navegação, o endereço
no meio, Spaces e botões; a segunda tem o Space, as fixadas e as abas como fichas, que rolam para
os lados com a roda do mouse. Clicar no endereço abre a busca sobre a página. Menus e janelas
da barra se abrem por cima da página (que fica congelada numa imagem enquanto estão abertos).

**Esconder a barra de abas** (Ctrl+Shift+S) deixa a página com a janela toda; encoste o mouse na
borda (lateral ou de cima) para a barra voltar.

### Texto

Em **Configurações → Aparência → Texto → Fontes**:
- **Mac** (padrão): a interface usa a Inter, a fonte livre mais parecida com a San Francisco do
  macOS, com tamanhos ópticos, espaçamento e peso ajustados; e o texto de tudo, páginas incluídas,
  é suavizado em tons de cinza, como no Mac, em vez do ClearType. A suavização muda depois de
  reiniciar o Blazon.
- **Windows**: Segoe UI e ClearType.

### Materiais e movimento

Em **Configurações → Aparência → Materiais e movimento**:
- **Liquid Glass**: menus, paleta, avisos e cartões de vidro que curva o que está atrás, como no
  macOS 26. Desligue se o computador for mais simples; também desliga sozinho com as
  transparências do Windows desligadas.
- **Barra translúcida (Mica)**: a barra mostra o papel de parede desfocado. Fica completa depois
  de reiniciar o Blazon.
- **Transição entre páginas** e **Rolagem elástica**.
- Em **Janela e barra lateral → Botões da janela**, escolha **Mac** para as três bolinhas.

### Animações

As animações seguem as do Crest, com os mesmos tempos e molas:
- as abas deslizam para o novo lugar ao reordenar, fixar, abrir ou fechar pastas;
- a seleção desliza da aba anterior para a nova;
- ao trocar de Space, o conteúdo entra pelo lado do Space escolhido e as cores mudam aos poucos;
- num touchpad, deslize dois dedos para os lados sobre a barra lateral para trocar de Space;
- a barra de endereço se preenche com a cor do Space enquanto a página carrega;
- ao arrastar, a aba levanta com sombra e as outras abrem espaço para ela; solte numa pasta
  fechada para guardá-la ali, ou entre as fixadas para fixá-la (Esc cancela);
- a paleta é de vidro: a página aparece desfocada atrás dela;
- o Peek cresce até o lugar antes de mostrar a página e encolhe ao fechar;
- botões encolhem de leve ao apertar;
- menus, paleta, avisos e a barra flutuante aparecem e somem com as transições do Crest.

Se **Configurações do Windows → Acessibilidade → Efeitos visuais → Efeitos de animação** estiver
desligado, o Blazon mantém só os esmaecimentos, sem movimento.

## Atalhos

Todos os atalhos da primeira tabela podem ser trocados em **Configurações → Atalhos**: clique em
**Alterar** e pressione a nova combinação (Esc cancela, Backspace desliga o atalho). Se a
combinação já for de outro comando, ela muda de lugar e o outro fica sem atalho; **Padrão**
devolve a original. Valem na barra lateral e dentro das páginas, sem recarregar.

| Atalho padrão | Ação |
| --- | --- |
| Ctrl+T | Nova aba |
| Ctrl+N | Nova janela |
| Ctrl+W | Fechar aba |
| Ctrl+Shift+T | Reabrir aba fechada |
| Ctrl+K | Paleta de comandos |
| Ctrl+L | Barra de endereço |
| Ctrl+H | Histórico |
| Ctrl+Tab / Ctrl+Shift+Tab | Próxima / anterior |
| Ctrl+Shift+S | Mostrar ou esconder a barra de abas |
| Ctrl+Shift+L | Abas na lateral ou no topo |
| Ctrl+Shift+R | Modo leitura |
| Ctrl+Shift+P | Fixar ou soltar a aba |
| Ctrl+Shift+D | Duplicar a aba |
| Ctrl+M | Silenciar a aba |
| Ctrl+, | Configurações |

Fixos:

| Atalho | Ação |
| --- | --- |
| F6 | Endereço → abas → página |
| Ctrl+1 … 8, Ctrl+9 | Aba pela posição, última aba |
| Ctrl+Shift+1 … 9 | Trocar de Space |
| Shift+clique num link | Peek |
| Alt+← / Alt+→, F5 | Voltar / avançar, recarregar |
| Ctrl + / Ctrl − / Ctrl+0 | Zoom |
| Ctrl+F | Buscar na página |
| F12 | Ferramentas do desenvolvedor |
