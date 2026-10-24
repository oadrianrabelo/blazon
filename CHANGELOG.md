# Changelog

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
