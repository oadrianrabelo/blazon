# Changelog

## 0.1.4

Mais Mac:
- **Liquid Glass**: menus, sugestões, avisos, cartões de widget, seletor de Spaces, paleta, Peek e
  boas-vindas em vidro que curva o que está atrás nas bordas, com brilho na borda e uma luz que
  segue o mouse. Os menus nascem do botão (ou do ponto) que os abriu.
- **Barra translúcida (Mica)**: a barra lateral mostra o papel de parede desfocado (Acrylic no
  Windows 10).
- **Transição entre páginas**: a página anterior esmaece ao trocar de aba e desliza ao trocar de
  Space.
- **Rolagem elástica** na lista de abas.
- **Botões da janela estilo Mac** (as três bolinhas), opcionais.
- **Ícones** redesenhados no estilo do SF Symbols.
- Tudo em Configurações → Aparência → Materiais e movimento.
- **Abas no topo**, como no Safari, ou na lateral (Configurações → Aparência → Abas, ou
  Ctrl+Shift+L). Menus e janelas da barra se abrem sobre a página.
- **Esconder a barra de abas** com nome claro (Ctrl+Shift+S); ela volta ao encostar na borda.
- **Fontes como no Mac**: Inter (parecida com a San Francisco) na interface e suavização em tons
  de cinza em todas as páginas. Em Aparência → Texto dá para trocar por Geist, Figtree, Segoe UI
  ou qualquer fonte instalada no Windows, com prévia, e voltar ao ClearType.
- **Mais personalização** em Configurações → Aparência:
  - botões da janela em quatro estilos: Windows, **Blazon** (os botões do Windows redesenhados,
    em teclas arredondadas de vidro), Mac e Grafite; na esquerda ou na direita; maximizar vira
    restaurar com a janela maximizada;
  - barra de ferramentas: escolha quais botões aparecem (voltar/avançar, recarregar, bloqueador,
    modo leitura, nova aba, configurações, extensões, downloads, arquivo);
  - cantos da página, de retos a bem redondos;
  - intensidade do vidro (leve, normal, forte) e velocidade das animações (calma, normal,
    rápida ou sem animação).
- **Atalhos personalizáveis** em Configurações → Atalhos, com novos comandos: modo leitura,
  fixar, duplicar, silenciar, configurações e abas no topo.

## 0.1.3

- **Senhas**: importa do Chrome, Edge, Brave, Vivaldi e Opera e de CSV (Chrome, Edge, Firefox,
  Safari, Bitwarden) para um cofre criptografado pelo Windows. Configurações → Senhas lista,
  mostra (com Windows Hello) e apaga; a chave na barra lateral preenche o login no site certo.
- **Painel lateral de extensões**: o botão de peça no rodapé abre o painel, a janela ou as opções
  de uma extensão numa coluna ao lado da página.
- **Teste no Windows no CI**: cada build é instalado numa máquina Windows limpa, que abre o
  Blazon, percorre as telas e guarda prints.
- **Landing page** de download em `site/`, publicada no GitHub Pages.
- Correções vistas nos prints: Studio em janelas menores e Spaces sugeridos com casas repetidas.

## 0.1.2

Mais perto do Crest do Mac:
- **Baralho de widgets** na barra lateral: "Tocando agora" e o aviso de atualização viram cartões
  de vidro empilhados; deslize para os lados, use os pontos ou as setas.
- **Download que voa** da página até o botão de downloads.
- **Primeiros passos**: guia com cinco lições e uma barra de prática (fixar, pastas, limpar,
  lado a lado, paleta e Peek), aberto depois das boas-vindas e em Configurações → Sobre.
- **Janela rápida** para links de outros apps (Configurações → Links → Abrir em): uma janela
  pequena onde você escolhe o Space e abre a página como aba.
- **Cantos arredondados** no cartão da página, no painel, no Peek e na janela rápida.

## 0.1.1

- **Backup e sincronização**: exportar e restaurar (substituir tudo ou mesclar), e backup
  automático numa pasta como a do OneDrive, com 5 cópias datadas.
- **Memória**: Spaces fora de qualquer janela dormem por completo; painel com o uso de cada Space
  e "Dormir agora". Abas tocando som não dormem mais.
- **Traduzir página** pelo menu da aba ou pela paleta.
- **Tocando agora**: cartão na barra lateral com as abas que tocam som.
- Rotas de links podem abrir no **Peek**.
- **Acessibilidade**: navegação completa pelo teclado (F6, setas, Enter, Delete, Shift+F10),
  nomes para leitores de tela e contorno de foco.
- **Animações do Crest**: os mesmos tempos e molas do Crest em abas, seleção, troca de Space
  (com gesto de dois dedos no touchpad), pastas, carregamento na barra de endereço, menus,
  paleta, Peek, configurações e página inicial; vidro nos menus. Respeita "Efeitos de animação"
  do Windows.
- **Arrastar como no Mac**: a aba levanta, as outras abrem espaço e ela assenta no lugar.
- **Paleta de vidro** sobre a página e **Peek** que cresce e encolhe.
- **Boas-vindas** na primeira abertura, como o setup do Crest: casa, Spaces, importação e navegador
  padrão.
- **Página de site fora do ar** no estilo do Crest, no lugar da página do Edge.
- Código do app dividido em módulos.

## 0.1.0

### Uso diário e suporte
- **Relatar problema**: log em arquivo, relatório de travamento, erros da interface e páginas
  travadas registrados; relatório pronto para copiar ou abrir como issue no GitHub.
- `state.json` ilegível vira cópia de backup em vez de ser sobrescrito.
- **Atualização automática** pelas Releases do GitHub, com verificação SHA-256.
- Assinatura de código opcional no CI.
- **Testes automáticos** (Rust e interface) rodando no CI.
- Ícone próprio do Blazon.

### Recursos
- **Várias janelas** (Ctrl+N, abrir Space em nova janela), reabertas ao iniciar.
- **Downloads** com lista própria; **permissões de sites** por Space; **extensões**
  descompactadas por Space.
- Reordenar Spaces, busca por Space, cor e ícone de pastas, Voltar depois de reabrir o app.
- **Interface em inglês** (automático, português ou inglês).

### Nome
- O projeto passa a se chamar **Blazon**. Dados de instalações com o nome anterior são migrados
  automaticamente.

### Importação
- Favoritos e histórico de Chrome, Edge, Brave, Vivaldi, Chromium, Opera e Opera GX, com todos
  os perfis.
- Favoritos e histórico de Firefox, Zen, LibreWolf, Waterfox e Floorp, com todos os perfis.
- Arc: Spaces, pastas, favoritos e histórico.
- Arquivo HTML de favoritos de qualquer navegador.
- Escolha do destino: o Space atual ou um Space novo.

### Arquivo, leitura e Windows Hello
- Arquivo de abas fechadas e de abas paradas (prazo configurável), com restauração.
- Modo leitura com o Readability da Mozilla.
- Spaces bloqueados com o Windows Hello.

### Studio de brasões
- Gerador de brasões em SVG: 12 formatos, 11 divisões, 13 faixas, 34 figuras, emoji ou
  monograma, 9 bordas, acabamentos e sombras.
- As 9 casas clássicas do Crest.
- Etapas com prévia, cores por papel, padrão da barra lateral, Embaralhar e Desfazer.

### Privacidade
- Bloqueador de anúncios e rastreadores (adblock-rust, EasyList/EasyPrivacy), pausável por site.
- Áudio e mudo por aba, pasta de downloads por Space, senhas e preenchimento automático.

### Paleta, Peek e links
- Paleta de comandos (Ctrl+K) com "Tab para buscar".
- Peek (Shift+clique).
- Rotas de links de outros apps por Space.

### Aparência
- Painel de configurações.
- Presets, barra flutuante, largura e lado da barra.
- Formato, tamanho e estilo das abas; fixadas por linha; pastas com contagem.
- Cor de destaque, faixas heráldicas e zoom padrão.

### Base
- Spaces isolados (dados separados do WebView2) e Spaces privados.
- Abas verticais, fixadas, pastas, lado a lado e arrastar para reordenar.
- Histórico com sugestões, abas que dormem, reabrir aba fechada.
- Visual do Crest: casas, faixas heráldicas, cartão da página e janela sem bordas.
- Página inicial com os sites mais visitados.
- Navegador padrão do Windows e instância única.
- Build e instaladores via GitHub Actions.
