// Interface language. The UI is written in Portuguese; in English, every text
// shown (including dialogs and generated strings) is translated through this
// dictionary as it appears, so features don't need per-string wiring.

const I18N = (() => {
  const EN = {
    // Durations
    '1 dia': '1 day', '1 hora': '1 hour', '1 semana': '1 week', '12 horas': '12 hours', '15 minutos': '15 minutes',
    '3 dias': '3 days', '30 minutos': '30 minutes', '4 horas': '4 hours', 'Nunca': 'Never',
    // Sidebar, menus and dialogs
    'Pesquisar ou digitar endereço': 'Search or enter address', 'Nova aba': 'New tab', 'Nova aba (Ctrl+T)': 'New tab (Ctrl+T)',
    'Voltar (Alt+←)': 'Back (Alt+←)', 'Avançar (Alt+→)': 'Forward (Alt+→)', 'Recarregar (F5)': 'Reload (F5)', 'Recarregar': 'Reload',
    'Parar': 'Stop', 'Minimizar': 'Minimize', 'Maximizar': 'Maximize', 'Fechar': 'Close', 'Fechar (Ctrl+W)': 'Close (Ctrl+W)',
    'Fechar (Esc)': 'Close (Esc)', 'Configurações': 'Settings', 'Novo Space': 'New Space', 'Editar Space': 'Edit Space',
    'Arquivo e histórico (Ctrl+H)': 'Archive and history (Ctrl+H)', 'Downloads': 'Downloads', 'Arquivo': 'Archive',
    'Histórico': 'History', 'Pesquisar': 'Search', 'Limpar': 'Clear', 'Esvaziar': 'Empty', 'Nada por aqui.': 'Nothing here.',
    'Bloqueador de anúncios': 'Ad blocker', 'Modo leitura': 'Reader mode', 'Fixar': 'Pin', 'Desafixar': 'Unpin',
    'Nova pasta com esta aba': 'New folder with this tab', 'Mover para': 'Move to', 'Fora da pasta': 'Out of the folder',
    'Desfazer lado a lado': 'Undo side by side', 'Lado a lado com nova aba': 'Side by side with a new tab', 'Lado a lado com': 'Side by side with',
    'Duplicar': 'Duplicate', 'Copiar link': 'Copy link', 'Nova pasta': 'New folder', 'Renomear': 'Rename',
    'Excluir pasta (mantém as abas)': 'Delete folder (keeps the tabs)', 'Cor e ícone…': 'Color and icon…', 'Cor e ícone da pasta': 'Folder color and icon',
    'Ícone (um emoji)': 'Icon (an emoji)', 'Padrão': 'Default', 'Salvar': 'Save', 'Cancelar': 'Cancel', 'Nome da pasta:': 'Folder name:',
    'Abrir em nova janela': 'Open in new window', 'Editar no Studio': 'Edit in Studio', 'Silenciar aba': 'Mute tab', 'Ativar som': 'Unmute',
    'aba aberta': 'open tab', 'Limpar concluídos': 'Clear finished', 'Nenhum download nesta sessão.': 'No downloads in this session.',
    'Abrir': 'Open', 'Mostrar na pasta': 'Show in folder', 'Remover': 'Remove', 'Falhou ou foi cancelado': 'Failed or canceled',
    'Concluído': 'Done', 'Ver downloads': 'View downloads', 'Relatar': 'Report', 'privado': 'private', 'Privado': 'Private',
    'Personalizado': 'Custom', 'bloqueado': 'locked',
    'O Blazon fechou inesperadamente na última vez. O relatório foi salvo.': 'Blazon closed unexpectedly last time. The report was saved.',
    // Palette and Peek
    'Pesquisar, abrir um site ou digitar um comando': 'Search, open a site or type a command',
    '↑↓ navegar · Enter abrir · Tab buscar em um site (yt, w, gh…) · Esc fechar': '↑↓ navigate · Enter open · Tab search a site (yt, w, gh…) · Esc close',
    'Abas abertas': 'Open tabs', 'Comandos': 'Commands', 'Nova janela': 'New window', 'Fechar aba': 'Close tab',
    'Reabrir aba fechada': 'Reopen closed tab', 'Duplicar aba': 'Duplicate tab', 'Fixar ou desafixar aba': 'Pin or unpin tab',
    'Copiar link da aba': 'Copy tab link', 'Pausar ou retomar o bloqueador neste site': 'Pause or resume the blocker on this site',
    'Silenciar ou ativar som da aba': 'Mute or unmute tab', 'Privacidade e bloqueador': 'Privacy and blocker', 'Arquivo de abas': 'Tab archive',
    'Bloquear Spaces agora': 'Lock Spaces now', 'Relatar problema': 'Report a problem', 'Mostrar ou esconder a barra lateral': 'Show or hide the sidebar',
    'Aparência': 'Appearance', 'Rotas de links e Peek': 'Link routes and Peek', 'Importar de outro navegador': 'Import from another browser',
    'Abrir como aba': 'Open as tab', 'Wikipédia': 'Wikipedia', 'Tradutor': 'Translate', 'Mercado Livre': 'Mercado Livre',
    // Settings: navigation and headers
    'Spaces e Studio': 'Spaces and Studio', 'Geral': 'General', 'Busca': 'Search', 'Privacidade': 'Privacy', 'Extensões': 'Extensions',
    'Links': 'Links', 'Importar': 'Import', 'Atalhos': 'Shortcuts', 'Sobre': 'About',
    'Ajuste a barra lateral, as abas e as cores. As mudanças aparecem na hora.': 'Adjust the sidebar, tabs and colors. Changes show right away.',
    'Começar de um preset': 'Start from a preset', 'Restaurar tudo': 'Reset everything',
    'Restaurar todas as opções de aparência?': 'Reset all appearance options?',
    'Clássico': 'Classic', 'O visual original do Crest': "Crest's original look", 'Compacto': 'Compact', 'Mais abas na tela': 'More tabs on screen',
    'Suave': 'Soft', 'Abas em pílula com brilho': 'Pill tabs with glow', 'Mínimo': 'Minimal', 'Sem faixas nem fundos': 'No banners or fills',
    'Janela e barra lateral': 'Window and sidebar', 'Lado da barra': 'Sidebar side', 'Esquerda': 'Left', 'Direita': 'Right',
    'Largura da barra': 'Sidebar width', 'Barra lateral flutuante': 'Floating sidebar',
    'Esconde a barra; ela volta ao encostar o mouse na borda (Ctrl+Shift+S).': 'Hides the sidebar; it slides back when the pointer reaches the edge (Ctrl+Shift+S).',
    'Barra de título do Windows': 'Windows title bar', 'Usa a moldura padrão em vez da janela sem bordas.': 'Uses the standard frame instead of the borderless window.',
    'Faixas heráldicas': 'Heraldic banners', 'Mostra o padrão do brasão no fundo da barra.': "Shows the crest's pattern behind the sidebar.",
    'Abas': 'Tabs', 'Tamanho': 'Size', 'Normal': 'Normal', 'Grande': 'Large', 'Enorme': 'Huge', 'Formato': 'Shape', 'Reto': 'Square',
    'Arredondado': 'Rounded', 'Pílula': 'Pill', 'Aba selecionada': 'Selected tab', 'Preenchida': 'Filled', 'Contorno': 'Outline', 'Brilho': 'Glow',
    'Seguir a cor do Space': "Follow the Space's color", 'Tinge a aba selecionada com a cor de destaque.': 'Tints the selected tab with the accent color.',
    'Apagar abas não carregadas': 'Dim unloaded tabs', 'Abas dormindo aparecem mais claras.': 'Sleeping tabs look lighter.',
    'Fixadas': 'Pinned', 'Por linha': 'Per row', 'Linhas equilibradas': 'Balanced rows', 'Esticar para preencher a linha': 'Stretch to fill the row',
    'Fundo nos fixados': 'Fill behind pins', 'Pastas': 'Folders', 'Mostrar quantidade de abas': 'Show tab counts', 'Ícone na cor do Space': "Icon in the Space's color",
    'Cores': 'Colors', 'Contorno colorido ao editar o endereço': 'Colored outline while editing the address', 'Cor de destaque própria': 'Custom accent color',
    'Substitui a cor do Space nas abas, botões e contornos.': "Replaces the Space's color on tabs, buttons and outlines.",
    'Páginas': 'Pages', 'Zoom padrão': 'Default zoom', 'Vale para abas novas e para as abertas.': 'Applies to new and open tabs.',
    'Memória, inicialização e navegador padrão.': 'Memory, startup and default browser.', 'Dormir abas em segundo plano após': 'Sleep background tabs after',
    'Libera memória; a aba recarrega quando você volta. Fixadas nunca dormem.': 'Frees memory; the tab reloads when you come back. Pinned tabs never sleep.',
    'Arquivar abas paradas após': 'Archive idle tabs after',
    'Abas soltas sem uso vão para o Arquivo (botão no rodapé da barra). Fixadas e abas em pastas ficam.': 'Unused loose tabs go to the Archive (button at the bottom of the sidebar). Pinned tabs and tabs in folders stay.',
    'Navegador padrão': 'Default browser', 'Usar o Blazon para links': 'Use Blazon for links', 'Abrir Aplicativos padrão': 'Open Default apps',
    'Escolha "Blazon" em Navegador da Web nos Aplicativos padrão do Windows.': 'Pick "Blazon" as the Web browser in Windows Default apps.',
    'Idioma': 'Language', 'Automático': 'Automatic', 'Idioma da interface': 'Interface language',
    'O que a barra de endereço usa quando você digita algo que não é um site.': "What the address bar uses when you type something that isn't a site.",
    'Mecanismo': 'Engine', 'Busca padrão': 'Default search', 'Personalizada': 'Custom', 'Endereço personalizado': 'Custom address',
    'A pesquisa é colocada no fim do endereço.': 'The search is appended to the address.',
    'Bloqueio de anúncios e rastreadores, senhas e downloads.': 'Ad and tracker blocking, passwords and downloads.', 'Bloqueador': 'Blocker',
    'Bloquear anúncios e rastreadores': 'Block ads and trackers', 'Usa as listas EasyList e EasyPrivacy, atualizadas toda semana.': 'Uses the EasyList and EasyPrivacy lists, updated weekly.',
    'Sites com o bloqueador pausado': 'Sites with the blocker paused', 'Retomar': 'Resume',
    'Nenhum site pausado. Use o escudo na barra lateral para pausar num site.': 'No paused sites. Use the shield in the sidebar to pause on a site.',
    'Senhas e formulários': 'Passwords and forms', 'Oferecer para salvar senhas e preencher formulários': 'Offer to save passwords and fill forms',
    'Guardado pelo próprio WebView2, separado por Space. Vale para abas novas.': 'Stored by WebView2 itself, separate per Space. Applies to new tabs.',
    'Downloads por Space': 'Downloads per Space', 'Escolher pasta…': 'Choose folder…', 'Pasta Downloads do Windows': 'Windows Downloads folder',
    'Permissões de sites': 'Site permissions', 'Câmera, microfone, localização, notificações e outras.': 'Camera, microphone, location, notifications and more.',
    'Carregando…': 'Loading…', 'Permitir': 'Allow', 'Bloquear': 'Block', 'Perguntar de novo': 'Ask again',
    'Nenhuma permissão salva neste Space. Os sites pedem na primeira vez que precisam.': 'No saved permissions in this Space. Sites ask the first time they need one.',
    'Microfone': 'Microphone', 'Câmera': 'Camera', 'Localização': 'Location', 'Notificações': 'Notifications', 'Outros sensores': 'Other sensors',
    'Leitura da área de transferência': 'Clipboard read', 'Downloads automáticos': 'Automatic downloads', 'Acesso a arquivos': 'File access',
    'Reprodução automática': 'Autoplay', 'Fontes locais': 'Local fonts', 'Mensagens MIDI': 'MIDI messages', 'Janelas': 'Windows',
    'Extensões descompactadas (uma pasta com manifest.json), instaladas por Space. O WebView2 não acessa a Chrome Web Store e suporta só parte das APIs de extensão.': "Unpacked extensions (a folder with manifest.json), installed per Space. WebView2 can't reach the Chrome Web Store and supports only part of the extension APIs.",
    'Instalar e gerenciar em': 'Install and manage in', 'Instaladas': 'Installed', 'Nenhuma extensão neste Space.': 'No extensions in this Space.',
    'Adicionar extensão (pasta)…': 'Add extension (folder)…', 'Como conseguir uma extensão descompactada': 'How to get an unpacked extension',
    'Do código-fonte': 'From source', 'De um .crx': 'From a .crx',
    'Muitas extensões publicam a pasta pronta no GitHub (procure por "unpacked" ou baixe o .zip da release e extraia).': 'Many extensions publish the ready folder on GitHub (look for "unpacked", or download the release .zip and extract it).',
    'Um .crx é um .zip com cabeçalho: renomeie para .zip, extraia e escolha a pasta.': 'A .crx is a .zip with a header: rename it to .zip, extract it and choose the folder.',
    'Como o Blazon abre links de outros apps e prévias rápidas.': 'How Blazon opens links from other apps and quick previews.',
    'Links de outros apps': 'Links from other apps', 'Abrir em': 'Open in', 'Quando nenhuma rota combina.': 'When no route matches.', 'Space atual': 'Current Space',
    'Rotas': 'Routes', 'Nova rota': 'New route', 'Nenhuma rota ainda.': 'No routes yet.', 'ex.: figma.com': 'e.g. figma.com',
    'Abrir prévia com': 'Open preview with', 'Shift + clique': 'Shift + click', 'Alt + clique': 'Alt + click', 'Desligado': 'Off',
    'Clique num link segurando a tecla para ver numa janela flutuante. Esc fecha.': 'Click a link while holding the key to see it in a floating window. Esc closes.',
    'Traga favoritos, histórico e Spaces de outro navegador. Abas importadas começam dormindo; o histórico alimenta as sugestões da barra de endereço.': 'Bring bookmarks, history and Spaces from another browser. Imported tabs start asleep; history feeds the address bar suggestions.',
    'Navegadores encontrados neste PC': 'Browsers found on this PC', 'Nenhum navegador encontrado': 'No browser found',
    'Procuro Arc, Chrome, Edge, Brave, Vivaldi, Opera, Chromium, Firefox, Zen, LibreWolf, Waterfox e Floorp.': 'Looks for Arc, Chrome, Edge, Brave, Vivaldi, Opera, Chromium, Firefox, Zen, LibreWolf, Waterfox and Floorp.',
    'Spaces, pastas e favoritos': 'Spaces, folders and favorites', 'Favoritos': 'Bookmarks', 'No Space atual': 'In the current Space',
    'Importando…': 'Importing…', 'Arquivo de favoritos': 'Bookmarks file', 'Favoritos em HTML': 'Bookmarks HTML',
    'Qualquer navegador exporta favoritos neste formato (Exportar favoritos).': 'Every browser exports bookmarks in this format (Export bookmarks).',
    'Escolher arquivo…': 'Choose file…', 'O que não dá para importar': "What can't be imported", 'Senhas': 'Passwords',
    'O WebView2 não permite adicionar senhas salvas. Use um gerenciador de senhas ou deixe o Blazon salvar ao entrar nos sites.': "WebView2 can't add saved passwords. Use a password manager, or let Blazon save them as you sign in to sites.",
    'Abas abertas e extensões': 'Open tabs and extensions', 'Ainda não suportado.': 'Not supported yet.', 'Favoritos importados': 'Imported bookmarks',
    'Funcionam na barra lateral e dentro das páginas.': 'They work in the sidebar and inside pages.', 'Focar o endereço': 'Focus the address bar',
    'Próxima / anterior': 'Next / previous', 'Ir para a aba': 'Go to tab', 'Trocar de Space': 'Switch Space', 'Paleta de comandos': 'Command palette',
    'Peek (prévia do link)': 'Peek (link preview)', 'Mostrar ou esconder a barra': 'Show or hide the sidebar', 'Voltar / avançar': 'Back / forward',
    'Zoom': 'Zoom', 'Buscar na página': 'Find in page', 'Ferramentas do desenvolvedor': 'Developer tools',
    'Gera um relatório com o que aconteceu, a versão, o sistema e o log recente. Para mandar ao desenvolvimento, copie e cole na conversa ou abra uma issue no GitHub.': 'Builds a report with what happened, the version, the system and the recent log. To send it, copy and paste it into the conversation or open a GitHub issue.',
    'Descrição': 'Description', 'Gerar relatório': 'Generate report', 'Copiar': 'Copy', 'Copiado!': 'Copied!', 'Abrir issue no GitHub': 'Open GitHub issue',
    'Abrir pasta de logs': 'Open logs folder', 'O que vai no relatório': "What's in the report",
    'O que você estava fazendo e o que aconteceu? (ex.: cliquei em Importar do Chrome e nada aconteceu)': 'What were you doing and what happened? (e.g. I clicked Import from Chrome and nothing happened)',
    'Versão, Windows, WebView2, quantidades (Spaces, abas) e configurações, e o log recente. O log só registra domínios, nunca endereços completos, títulos ou senhas.': 'Version, Windows, WebView2, counts (Spaces, tabs), settings and the recent log. The log only records domains, never full addresses, titles or passwords.',
    'Um navegador leve para Windows, com Spaces e brasões. Feito em Rust com Tauri e o WebView2 do Windows.': 'A lightweight Windows browser with Spaces and crests. Built in Rust with Tauri and Windows WebView2.',
    // Studio
    'Spaces e Crest Studio': 'Spaces and Studio',
    'Cada Space tem seu brasão, cores e fundo. Escolha um modelo ou monte peça por peça.': 'Each Space has its own crest, colors and background. Pick a template or build it piece by piece.',
    'Novo privado': 'New private', 'Embaralhar': 'Shuffle', 'Desfazer': 'Undo', 'Excluir Space': 'Delete Space', 'Bloquear com Windows Hello': 'Lock with Windows Hello',
    'Remover bloqueio': 'Remove lock', 'Pede PIN, rosto ou digital antes de mostrar as abas deste Space.': "Asks for PIN, face or fingerprint before showing this Space's tabs.",
    '↑ Subir': '↑ Up', '↓ Descer': '↓ Down', 'Busca padrão do Blazon': "Blazon's default search", 'Mecanismo de busca usado neste Space': 'Search engine used in this Space',
    'Modelo': 'Template', 'Campo': 'Field', 'Faixa': 'Band', 'Emblema': 'Emblem', 'Borda': 'Border', 'Acabamento': 'Finish', 'Barra lateral': 'Sidebar',
    'Comece de uma das nove casas clássicas: brasão e barra lateral juntos.': 'Start from one of the nine classic houses: crest and sidebar together.',
    'Quantidade': 'Count', 'Emoji 🦊': 'Emoji 🦊', 'Monograma AB': 'Monogram AB', 'Espessura': 'Thickness', 'Detalhes (raios, pérolas, selo)': 'Detail (rays, beads, seal)',
    'Brasão': 'Crest', 'Segundo campo': 'Second field', 'Fundo da barra': 'Sidebar background', 'Faixa do meio': 'Middle band', 'Faixa de baixo': 'Lower band',
    'O padrão das faixas no fundo da barra lateral.': 'The banner pattern behind the sidebar.',
    // Crest vocabulary
    'Escudo': 'Shield', 'Escudo francês': 'French shield', 'Redondo': 'Round', 'Oval': 'Oval', 'Losango': 'Lozenge', 'Hexágono': 'Hexagon',
    'Octógono': 'Octagon', 'Quadrado': 'Rounded square', 'Selo': 'Seal', 'Flâmula': 'Banner', 'Distintivo': 'Badge', 'Nenhum': 'None', 'Nenhuma': 'None',
    'Liso': 'Plain', 'Vertical': 'Vertical', 'Horizontal': 'Horizontal', 'Diagonal': 'Diagonal', 'Quartelado': 'Quartered', 'Cruzado': 'Crossed',
    'Gomos': 'Wedges', 'Barras': 'Bars', 'Listras': 'Stripes', 'Xadrez': 'Checks', 'Pala': 'Pale', 'Banda': 'Bend', 'Asna': 'Chevron', 'Cruz': 'Cross',
    'Aspa': 'Saltire', 'Chefe': 'Chief', 'Bordadura': 'Bordure', 'Pairle': 'Pall', 'Pira': 'Pile', 'Cantão': 'Canton', 'Besante': 'Roundel',
    'Linha': 'Line', 'Linha dupla': 'Double line', 'Louros': 'Laurel', 'Raios': 'Sunburst', 'Anel duplo': 'Double ring', 'Pérolas': 'Beaded',
    'Fosco': 'Flat', 'Relevo': 'Embossed', 'Sem sombra': 'No shadow', 'Elevada': 'Lifted', 'Faixas': 'Bands', 'Dividido': 'Split', 'Losangos': 'Lozenges',
    'Degradê': 'Gradient', 'Lobo': 'Wolf', 'Leão': 'Lion', 'Cervo': 'Stag', 'Dragão': 'Dragon', 'Cabeça de dragão': 'Dragon head', 'Serpe': 'Wyvern',
    'Hidra': 'Hydra', 'Grifo': 'Griffin', 'Águia': 'Eagle', 'Falcão': 'Falcon', 'Corvo': 'Raven', 'Morcego': 'Bat', 'Urso': 'Bear', 'Javali': 'Boar',
    'Raposa': 'Fox', 'Cavalo': 'Horse', 'Unicórnio': 'Unicorn', 'Serpente': 'Serpent', 'Cavalo-marinho': 'Seahorse', 'Escorpião': 'Scorpion',
    'Rosa': 'Rose', 'Lírio': 'Lily', 'Pinheiro': 'Pine', 'Salgueiro': 'Willow', 'Castelo': 'Castle', 'Espada': 'Sword', 'Espadas': 'Swords',
    'Machados': 'Axes', 'Tridente': 'Trident', 'Âncora': 'Anchor', 'Balança': 'Scales', 'Peixe': 'Fish', 'Sol': 'Sun',
    // Start page and reader
    'Nova aba ': 'New tab ', 'Pesquisar na web': 'Search the web', 'Digite um endereço ou pesquisa na barra lateral (Ctrl+L)': 'Type an address or search in the sidebar (Ctrl+L)',
    'Esta página não tem um texto para o modo leitura.': "This page has no article for reader mode.",
    // Messages from the app core
    'Nada para importar.': 'Nothing to import.', 'Navegador não encontrado.': 'Browser not found.', 'Nenhum arquivo escolhido.': 'No file chosen.',
    'Nenhuma pasta escolhida.': 'No folder chosen.', 'Nenhum favorito encontrado nesse arquivo.': 'No bookmarks found in that file.',
    'Não deu para ler o arquivo do navegador. Feche o navegador e tente de novo.': "Couldn't read the browser's file. Close that browser and try again.",
    'Abra uma aba neste Space primeiro (as configurações ficam no perfil do Space).': "Open a tab in this Space first (these settings live in the Space's profile).",
    'O Windows Hello (PIN, rosto ou digital) não está configurado neste PC.': 'Windows Hello (PIN, face or fingerprint) is not set up on this PC.',
    'Verificação cancelada.': 'Verification canceled.', 'Space bloqueado.': 'Space locked.', 'Só no Windows.': 'Windows only.',
    'Esta versão do WebView2 não suporta extensões.': "This WebView2 version doesn't support extensions.", 'Tempo esgotado.': 'Timed out.',
    'Barra de favoritos': 'Bookmarks bar', 'Outros favoritos': 'Other bookmarks', 'Favoritos do celular': 'Mobile bookmarks', 'Menu de favoritos': 'Bookmarks menu',
  };

  // Generated texts with names or numbers.
  const PATTERNS = [
    [/^Abrir (.+)$/, 'Open $1'],
    [/^Pesquisar "(.+)"$/, 'Search "$1"'],
    [/^Buscar "(.+)" no (.+)$/, 'Search "$1" on $2'],
    [/^Tab para buscar no (.+)$/, 'Tab to search $1'],
    [/^Ir para (.+)$/, 'Go to $1'],
    [/^Num Space novo "(.+)"$/, 'In a new Space "$1"'],
    [/^Excluir o Space "(.+)"\? As abas, cookies e logins dele serão apagados\.$/, 'Delete the Space "$1"? Its tabs, cookies and logins will be erased.'],
    [/^Limpar o arquivo do Space "(.+)"\?$/, 'Clear the archive of the Space "$1"?'],
    [/^Limpar todo o histórico do Space "(.+)"\?$/, 'Clear all history of the Space "$1"?'],
    [/^Baixando "(.+)"$/, 'Downloading "$1"'],
    [/^A aba "(.+)" travou\.$/, 'The tab "$1" crashed.'],
    [/^(.+) de (.+ [KMG]B)$/, '$1 of $2'],
    [/^Bloqueador pausado em (.+)\. Clique para retomar\.$/, 'Blocker paused on $1. Click to resume.'],
    [/^Bloqueando anúncios em (.+)\. Clique para pausar neste site\.$/, 'Blocking ads on $1. Click to pause on this site.'],
    [/^Bloqueando anúncios\. Clique para pausar neste site\.$/, 'Blocking ads. Click to pause on this site.'],
    [/^Importar de (.+)\? Cada Space do Arc vira um Space novo\.$/, 'Import from $1? Each Arc Space becomes a new Space.'],
    [/^Importar de (.+)\? Os favoritos entram como pastas no Space atual\.$/, 'Import from $1? Bookmarks come in as folders in the current Space.'],
    [/^Não deu para importar: (.+)$/, (m, e) => `Couldn't import: ${t(e)}`],
    [/^Extensão instalada: (.+)\. Recarregue as abas para usar\.$/, 'Extension installed: $1. Reload your tabs to use it.'],
    [/^Remover "(.+)"\?$/, 'Remove "$1"?'],
    [/^Importado: (.+)\.$/, (m, list) => `Imported: ${list
      .replace(/(\d+) Spaces/, '$1 Spaces').replace(/(\d+) pastas de favoritos/, '$1 bookmark folders')
      .replace(/(\d+) abas/, '$1 tabs').replace(/(\d+) páginas do histórico/, '$1 history pages')}.`],
    [/^(Bom dia|Boa tarde|Boa noite) · (.+)$/, (m, g, day) => `${{ 'Bom dia': 'Good morning', 'Boa tarde': 'Good afternoon', 'Boa noite': 'Good evening' }[g]} · ${day}`],
    [/^Permissão (\d+)$/, 'Permission $1'],
    [/^Bug: (.+)$/, 'Bug: $1'],
  ];

  const stored = () => { try { return localStorage.getItem('blazon.lang'); } catch { return null; } };
  const resolve = (pref) => (pref === 'pt' || pref === 'en' ? pref : (navigator.language || 'pt').toLowerCase().startsWith('pt') ? 'pt' : 'en');
  let lang = ['pt', 'en'].includes(stored()) ? stored() : resolve('auto');
  document.documentElement.lang = lang === 'pt' ? 'pt-BR' : 'en';

  function t(s) {
    if (lang === 'pt' || typeof s !== 'string' || !s.trim()) return s;
    const k = s.trim();
    if (EN[k] !== undefined) return s.replace(k, EN[k]);
    for (const [re, rep] of PATTERNS) {
      if (re.test(k)) return s.replace(k, k.replace(re, rep));
    }
    // Lists like "Favoritos · Histórico" translate piece by piece.
    if (k.includes(' · ')) return s.replace(k, k.split(' · ').map((p) => t(p)).join(' · '));
    return s;
  }

  const SKIP = new Set(['SCRIPT', 'STYLE', 'TEXTAREA', 'PRE', 'CODE']);
  const ATTRS = ['title', 'placeholder', 'aria-label'];

  function translate(root) {
    if (lang === 'pt') return;
    if (root.nodeType === Node.TEXT_NODE) {
      if (root.parentNode && !SKIP.has(root.parentNode.nodeName)) {
        const v = t(root.data);
        if (v !== root.data) root.data = v;
      }
      return;
    }
    if (root.nodeType !== Node.ELEMENT_NODE) return;
    for (const a of ATTRS) {
      const v = root.getAttribute(a);
      if (v) { const tv = t(v); if (tv !== v) root.setAttribute(a, tv); }
    }
    // Typed text and code stay as they are; only their attributes translate.
    if (SKIP.has(root.nodeName)) return;
    for (const c of root.childNodes) translate(c);
  }

  if (lang !== 'pt') {
    const start = () => {
      translate(document.body);
      document.title = t(document.title);
      new MutationObserver((list) => {
        for (const m of list) {
          if (m.type === 'characterData') translate(m.target);
          else if (m.type === 'attributes') translate(m.target);
          else m.addedNodes.forEach(translate);
        }
      }).observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: ATTRS });
    };
    if (document.body) start();
    else addEventListener('DOMContentLoaded', start);
    const [a, c, p] = [window.alert, window.confirm, window.prompt];
    window.alert = (m) => a.call(window, t(String(m)));
    window.confirm = (m) => c.call(window, t(String(m)));
    window.prompt = (m, d) => p.call(window, t(String(m)), d);
  }

  /// Applies the language setting; reloads the page when it changes.
  function sync(pref) {
    const want = resolve(pref);
    try { localStorage.setItem('blazon.lang', want); } catch {}
    if (want !== lang) location.reload();
  }

  return { t, sync, lang: () => lang, locale: () => (lang === 'pt' ? 'pt-BR' : 'en-US'), setForPage: (l) => { lang = l; } };
})();
