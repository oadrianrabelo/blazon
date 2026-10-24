<div align="center">
  <img src="src-tauri/icons/128x128.png" width="96" alt="Ícone do Blazon">
  <h1>Blazon</h1>
  <p><strong>Um navegador leve para Windows, com Spaces e brasões.</strong></p>
</div>

Em heráldica, *blazon* é a descrição formal de um brasão, e *to blazon* é exibir algo
com orgulho. No **Blazon**, cada parte da sua vida digital (trabalho, casa, estudos, um
projeto) ganha um **Space** próprio, com brasão, cores, logins e histórico separados.

O Blazon nasceu como uma recriação para Windows do [Crest](https://github.com/pauljoda/Crest),
de Paul Joda (Mac/iOS). Herda dele o conceito de Spaces, o visual e o Crest Studio, mas foi
reescrito do zero em Rust com [Tauri 2](https://tauri.app). As páginas rodam no **WebView2**,
o Chromium do Edge que já vem no Windows 10/11, então o instalador tem poucos MB e não carrega
outro navegador junto.

## Destaques

- **Spaces isolados**: cada um tem seus cookies, logins, cache, histórico, arquivo e pasta
  de downloads. Também há Spaces privados (InPrivate) e Spaces bloqueados com o Windows Hello.
- **Studio de brasões**: comece de uma das 9 casas clássicas ou monte o seu escolhendo
  formato, campo, faixa, emblema (34 figuras heráldicas, emoji ou monograma), borda,
  acabamento e cores. O brasão também pinta a barra lateral.
- **Abas verticais** com fixadas, pastas, lado a lado, abas que dormem para poupar memória e um
  arquivo automático de abas paradas.
- **Paleta de comandos** (Ctrl+K), **Peek** (Shift+clique abre uma prévia do link) e **modo leitura**.
- **Bloqueador de anúncios e rastreadores** embutido, com o motor do Brave e as listas
  EasyList/EasyPrivacy.
- **Importação** de favoritos e histórico do Chrome, Edge, Brave, Vivaldi, Opera, Firefox,
  Zen, LibreWolf e outros (todos os perfis); Spaces do Arc; e arquivo HTML de favoritos.
- **Personalização completa** da aparência: presets, barra flutuante, formato e tamanho das
  abas, cores, zoom padrão e mais.
- **Várias janelas**, downloads, permissões por site, extensões descompactadas, português e inglês.
- **Atualiza sozinho** pelas Releases do GitHub e tem um botão **Relatar problema** que monta um
  relatório pronto para enviar.

## Documentação

| Documento | Conteúdo |
| --- | --- |
| [Guia de uso](docs/GUIA.md) | Todos os recursos, passo a passo, e os atalhos |
| [Importar de outros navegadores](docs/IMPORTAR.md) | O que vem de cada navegador, onde fica e o que não dá para importar |
| [Arquitetura](docs/ARQUITETURA.md) | Como o código é organizado, onde ficam os dados e o modelo de segurança |
| [Compilar e publicar](docs/COMPILAR.md) | Ambiente de desenvolvimento, CI, releases e como renomear o repositório |
| [Changelog](CHANGELOG.md) | O que entrou em cada etapa |
| [Créditos e licenças](NOTICE.md) | Projetos e artes de terceiros |

## Instalar

1. No GitHub, abra **Actions → Build Windows**, entre no build mais recente e baixe o
   artefato **`blazon-windows`**. Quando houver uma tag `v*`, os instaladores também ficam em
   **Releases**.
2. Rode o **`.exe`**. Ele instala sem pedir administrador e registra o Blazon como navegador.
3. Para usar como navegador padrão, abra **Configurações do Windows → Aplicativos →
   Aplicativos padrão → Blazon** e defina-o para HTTP e HTTPS.

Se o WebView2 não estiver instalado (raro no Windows 10/11), o instalador baixa ele sozinho.

## Licença

[MPL-2.0](LICENSE), a mesma do Crest. Veja [NOTICE.md](NOTICE.md) para os créditos de terceiros.
