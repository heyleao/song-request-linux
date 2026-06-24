# Guia Rapido

[English](SETUP_EN.md)

Este guia e a receita curta para deixar o Song Request Linux pronto para live.
Faca um passo por vez.

## 1. Instalar e abrir

Pacote `.tar.gz`:

```bash
tar -xzf song-request-linux-0.1.6-linux-x86_64.tar.gz
cd song-request-linux-0.1.6-linux-x86_64
./scripts/check-runtime-prereqs
./scripts/install-desktop-entry
./scripts/song-request-linux-open
```

O pacote ja traz o app compilado. Rust, Cargo e Git so sao necessarios para instalacao via repositorio.

Opcionais por modo:

- Spotify: app Spotify aberto no PC e conta Premium.
- YouTube/Pear: Pear Desktop aberto com API Server ativo.
- YouTube/OBS: `yt-dlp` instalado para tocar audio no `/player`.

No CachyOS/Arch via Git:

```bash
git clone https://github.com/heyleao/song-request-linux.git
cd song-request-linux
./scripts/install-user-friendly --with-pear
```

Abrir depois de instalado:

```bash
./scripts/song-request-linux-open
```

Dashboard:

```text
http://127.0.0.1:7384/
```

## 2. Tela principal

![Dashboard principal](images/dashboard-overview.png)

A tela principal mostra:

- modo ativo;
- musica tocando agora;
- fila de pedidos;
- eventos recentes;
- botoes de play, pause, skip e volume.

## 3. Escolher o modo

Na tela principal `Operacao`, escolha um modo principal no card `Provider ativo`.

![Provider ativo na tela principal](images/setup-provider.png)

Use `Spotify` se voce quer:

- buscar musica por nome no Spotify;
- controlar play, pause, skip e volume no Spotify;
- usar playlist fallback do Spotify quando a fila estiver vazia.

Use `YouTube via Pear` se voce quer:

- buscar musica por nome no YouTube;
- tocar pelo Pear Desktop;
- aceitar links do YouTube direto.

Use `YouTube via OBS Browser` se voce quer:

- buscar musica por nome no YouTube;
- tocar pela fonte `http://127.0.0.1:7384/player` no OBS;
- controlar o audio pelo mixer do OBS.

Use um modo por vez: Spotify, YouTube via Pear ou YouTube via OBS Browser. Se trocar o modo, salve a configuracao.

## 4. Twitch

O bot da Twitch le o chat e responde aos comandos.

1. Abra https://dev.twitch.tv/console/apps
2. Crie um app.
3. Tipo do app: `Public`.
4. Redirect URL:

```text
https://localhost:7443/auth/twitch/callback
```

5. Copie o `Client ID`.
6. No Song Request Linux, preencha:
   - `Twitch Client ID`
   - `Conta do bot`
   - `Canal`
7. Clique em `Salvar`.
8. Clique em `Conectar bot`.
9. Entre com a conta do bot, nao com a conta principal.

Esta pronto quando o topo mostrar `Twitch configurado`.

## 5. Spotify

Pule este passo se voce escolheu YouTube via Pear ou YouTube via OBS Browser.

1. Abra https://developer.spotify.com/dashboard
2. Crie um app.
3. Adicione a Redirect URI:

```text
http://127.0.0.1:7384/auth/spotify/callback
```

4. Copie o `Client ID`.
5. Cole no campo `Spotify Client ID`.
6. Clique em `Salvar`.
7. Clique em `Login Spotify`.
8. Entre com sua conta Spotify.

Importante:

- precisa ser Spotify Premium;
- abra o Spotify no PC da live;
- deixe uma musica pronta ou tocando;
- o app nao deve mandar playback para celular.

Se aparecer erro de device, abra o Spotify no PC, aperte play em qualquer musica
e tente de novo.

## 6. Playlist fallback

Use fallback se quiser musica tocando quando nao houver pedidos.

1. Marque `Ativar playlist fallback quando a fila estiver vazia`.
2. Clique em `Carregar playlists`.
3. Escolha a playlist.
4. Salve.

Se desmarcar, o app nao volta para playlist sozinho.

## 7. Persistencia da fila

Essa opcao decide o que acontece ao fechar e abrir o app.

Marcado:

```text
A fila e salva e volta na proxima abertura.
```

Desmarcado:

```text
A proxima abertura comeca com fila vazia.
```

Use desmarcado se voce quer sempre comecar uma live limpa.

## 8. YouTube: Pear ou OBS Browser

Pule este passo se voce escolheu apenas Spotify.

### YouTube API

A API Key e usada pelo Song Request Linux para buscar por texto, validar duracao/categoria e escolher o video. Ela vale tanto para `YouTube via Pear` quanto para `YouTube via OBS Browser`.

O Pear nao usa essa API Key como login. O Pear toca usando a conta Google/YouTube logada dentro do proprio Pear.

1. Abra https://console.cloud.google.com/apis/credentials
2. Crie ou escolha um projeto.
3. Ative `YouTube Data API v3`.
4. Crie uma `API Key`.
5. Opcional: crie mais API Keys no mesmo projeto ou em projetos separados para ter mais margem de quota.
6. Cole as keys no campo de YouTube do Song Request Linux, uma por linha.
7. Salve.

A busca por texto usa YouTube Data API e pode bater limite. Quando ha varias
keys salvas, o app tenta a proxima se uma key falhar. Link direto do YouTube e o
caminho mais leve para pedidos especificos.

### Escolher onde o YouTube toca

No Setup, o campo `Modo de operacao` tem tres opcoes. Para YouTube, escolha uma:

- `Pear Desktop`: toca pelo app Pear e usa a API local do Pear.
- `Browser Source OBS`: toca direto em uma fonte de navegador do OBS.

Use apenas uma opcao por vez. Se escolher Pear, nao adicione o `/player` no OBS.
Se escolher OBS Browser, mantenha a fonte `/player` aberta no OBS durante a live.

### Pear Desktop

1. Abra o Pear Desktop.
2. Ative o plugin `API Server`.
3. Use a porta `26538`.
4. Deixe `Authorization` como `No Authorization`.
5. Reinicie o Pear.

No Song Request Linux, use:

```text
http://127.0.0.1:26538/api/v1
```

Se o Pear estiver fechado, a musica pode entrar na fila do app, mas nao vai tocar
ate o Pear abrir e a API estar ativa.

## Atualizar o app

Use `Atualizar pelo GitHub` no painel. O navegador chama o backend local, que
baixa o GitHub, recompila e reinicia o app. Depois do reinicio, o painel mostra
se atualizou ou se ja estava na ultima versao.

Se o painel nao abrir, use o comando manual:

```bash
./scripts/update-from-github --restart
```

## 9. OBS

Adicione uma `Browser Source` para o overlay.

![Guia e URLs do OBS](images/guide-tab.png)

URL:

```text
http://127.0.0.1:7384/overlay?max=48&width=520&size=24&lines=2
```

Tamanho da fonte:

```text
Largura: 620
Altura: 150
```

O `width=520` limita o texto dentro do overlay. Use `lines=2` para permitir
duas linhas no nome da musica. O texto do topo pode ser alterado no Setup ou
com `label=Texto` na URL.

Fonte de player YouTube:

```text
http://127.0.0.1:7384/player
```

Use essa fonte so se o player do YouTube estiver como `Browser Source OBS`.
Se voce usa Pear Desktop, nao adicione essa fonte.

Configuracao da fonte no OBS:

1. Adicione uma nova `Browser Source`.
2. Cole a URL `http://127.0.0.1:7384/player`.
3. Marque `Controlar audio via OBS`.
4. Mantenha a fonte ativa na cena da live.

Para o streamer ouvir a musica do Browser Source:

1. Abra as propriedades da fonte `http://127.0.0.1:7384/player`.
2. Marque `Controlar audio via OBS`.
3. No Mixer, abra `Propriedades avancadas de audio`.
4. Em `Monitoramento de audio`, selecione `Monitorar e enviar saida` para essa fonte.
5. Ajuste o volume no mixer do OBS. O comando `!vol` nao controla o OBS Browser.

## 10. Comandos do chat

Pedidos:

```text
!sr nome da musica
!sr link_do_youtube
```

Musica atual e fila:

```text
!song
!fila
!queue
!q
```

Remover o ultimo pedido do usuario:

```text
!rm
!remove
```

Volume:

```text
!vol
!vol 30
```

Controle para mod/streamer:

```text
!skip
!play
!pause
!next
```

Na configuracao voce pode trocar os comandos e escolher quem pode usar cada um.

![Configuracao avancada de comandos e limites](images/advanced-setup.png)

Cargos reconhecidos:

```text
Streamer
Moderador
VIP
Subscriber
Follower
```

Os cargos vem das tags/badges oficiais da Twitch. Nao precisa uma lista manual
de mods ou VIPs.

Nota: a Twitch nao envia uma tag confiavel de follower no IRC para todo chatter comum. O SRL trata chatter comum como Follower; para bloquear nao seguidores de verdade, ative o modo seguidores no chat da Twitch.

## 11. Limites por cargo

Na configuracao avancada, defina quantas musicas cada cargo pode ter pendente.

Exemplo:

```text
Follower: 1
Subscriber: 3
VIP: 3
Moderador: 10
Streamer: 0
```

`0` significa sem limite.

O limite conta a musica atual e as proximas musicas do mesmo usuario.

## 12. Testar antes da live

1. Veja se o topo mostra Twitch, modo ativo e API como configurados.
2. No dashboard, mande um pedido manual.
3. Veja se aparece em `Fila de pedidos`.
4. Veja se o player toca.
5. Teste no chat:

```text
!sr system of a down spiders
!song
!q
```

## 13. Fechar

Pelo dashboard, clique em `Encerrar`.

Ou rode:

```bash
./scripts/song-request-linux-stop
```

## 14. Se algo der errado

Veja a aba `Logs`.

Problemas comuns:

- Spotify nao toca: abra o Spotify no PC e aperte play.
- Spotify diz que nao tem device: o app nao encontrou um device valido no PC.
- Twitch nao responde: confira se conectou com a conta do bot.
- YouTube nao busca: confira a API Key e a YouTube Data API v3.
- Pear nao toca: confira se o Pear esta aberto e o API Server ligado.
- OBS nao mostra overlay: confira a URL e o tamanho da Browser Source.
- Mod nao consegue pedir mais musica: confira o limite de pedidos por cargo.
