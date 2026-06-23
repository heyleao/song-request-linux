# Guia Rapido

Use este guia como uma receita. Faca um passo por vez.

## 1. Abrir o app

Abra o Song Request Linux pelo atalho do sistema ou rode:

```bash
./scripts/song-request-linux-open
```

Depois abra no navegador:

```text
http://127.0.0.1:7384/
```

## 2. Twitch

O Twitch serve para o bot ler o chat.

1. Abra: https://dev.twitch.tv/console/apps
2. Crie um app novo.
3. Tipo do app: `Public`.
4. Redirect URL:

```text
https://localhost:7443/auth/twitch/callback
```

5. Copie o `Client ID`.
6. No Song Request Linux, va em `Configuracao`.
7. Preencha:
   - `Twitch Client ID`
   - `Conta do bot`
   - `Canal`
8. Clique em `Conectar bot`.
9. Entre com a conta do bot, nao com a conta principal.

Pronto quando aparecer `Twitch configurado`.

## 3. Spotify

O Spotify toca as musicas quando o provider for Spotify.

1. Abra: https://developer.spotify.com/dashboard
2. Crie um app novo.
3. Redirect URI:

```text
http://127.0.0.1:7384/auth/spotify/callback
```

4. Copie o `Client ID`.
5. No Song Request Linux, cole em `Spotify Client ID`.
6. Clique em `Salvar`.
7. Clique em `Login Spotify`.
8. Entre com a sua conta Spotify.

Importante:

- Precisa ser Spotify Premium.
- Deixe o Spotify aberto no PC da live.
- Se der erro de device, aperte play em qualquer musica no Spotify e tente de novo.

## 4. Playlist fallback

A playlist fallback toca quando nao tem pedido na fila.

1. Marque `Ativar playlist fallback`.
2. Clique em `Carregar playlists`.
3. Escolha a playlist.
4. Clique em `Salvar fallback`.

Se desmarcar, a playlist nao toca sozinha.

## 5. Persistencia da fila

Isso decide se os pedidos continuam depois que fechar e abrir o app.

- Marcado: guarda a fila para a proxima live.
- Desmarcado: a proxima abertura comeca com fila vazia.

Use desmarcado se voce quer sempre comecar a live limpo.

## 6. YouTube

Use YouTube se quiser aceitar links ou pedidos do YouTube.

1. Abra: https://console.cloud.google.com/apis/credentials
2. Crie ou escolha um projeto.
3. Ative `YouTube Data API v3`.
4. Crie uma `API Key`.
5. Cole a key no Song Request Linux.

Para tocar YouTube, o modo mais simples e estavel e o Pear Desktop.

No Pear:

1. Abra o Pear Desktop / YouTube Music Desktop.
2. Ative o plugin `API Server`.
3. Porta: `26538`.
4. Authorization: `No Authorization`.
5. Reinicie o Pear.

No Song Request Linux, deixe:

```text
Pear API: http://127.0.0.1:26538/api/v1
```

## 7. OBS

Adicione estas fontes como `Browser Source` no OBS.

Overlay da musica:

```text
http://127.0.0.1:7384/overlay?max=48&width=520&size=24&lines=1
```

Tamanho da fonte no OBS:

```text
Width/Largura: 620
Height/Altura: 120
```

O `width=520` da URL controla a largura interna do texto. A Browser Source deve
ficar um pouco maior para sobrar area transparente e evitar corte.

Player YouTube OBS, use so se voce escolheu `Browser Source OBS` no player YouTube:

```text
http://127.0.0.1:7384/player
```

Se voce usa Pear Desktop, normalmente nao precisa do `Player YouTube OBS`.

## 8. Comandos do chat

Pedidos:

```text
!sr nome da musica
!sr link_do_youtube
```

Ver musica e fila:

```text
!song
!fila
!queue
```

Remover seu ultimo pedido:

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

Voce pode trocar os nomes dos comandos na aba `Configuracao`.

## 9. Fechar o app

Pelo dashboard, clique em `Encerrar`.

Ou rode:

```bash
./scripts/song-request-linux-stop
```

## 10. Se algo der errado

Veja a aba `Logs`.

Coisas comuns:

- Spotify nao toca: abra o Spotify no PC e aperte play.
- Twitch nao responde: confira se entrou com a conta do bot.
- YouTube nao toca no Pear: confira se o Pear esta aberto e o API Server esta ligado.
- OBS nao mostra overlay: confira se a URL da Browser Source esta correta.
