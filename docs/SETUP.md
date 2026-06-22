# Guia de Configuracao

O app roda localmente em:

```text
http://127.0.0.1:7384/
```

## 1. Spotify

Abra o [Spotify Developer Dashboard](https://developer.spotify.com/dashboard).

Crie um app e copie o **Client ID**. Ele identifica o app local para o Spotify.

No app Spotify, cadastre este redirect:

```text
http://127.0.0.1:7384/auth/spotify/callback
```

Depois, no Song Request Linux:

1. Abra `Connections`.
2. Cole o `Spotify Client ID`.
3. Salve.
4. Clique em `Gerar link de login`.

Requisitos:

- A conta Spotify precisa ser Premium.
- O Spotify precisa estar aberto em algum device.
- Se aparecer `NO_ACTIVE_DEVICE`, abra o Spotify e aperte play/pause uma vez.

## 2. Twitch Bot

Abra o [Twitch Developer Console](https://dev.twitch.tv/console/apps).

Crie um app do tipo **Public**. Ele gera o **Client ID** usado para autorizar a
conta bot.

Cadastre este redirect:

```text
https://localhost:7443/auth/twitch/callback
```

No Song Request Linux, preencha:

- `Twitch Client ID`
- `Twitch Bot Username`
- `Twitch Channel`

Depois clique em `Conectar bot` em uma janela privada logada na conta bot.

## 3. YouTube

Abra [Google Cloud Credentials](https://console.cloud.google.com/apis/credentials).

No Google Cloud:

1. Crie ou selecione um projeto.
2. Ative a **YouTube Data API v3**.
3. Crie uma **API Key**.

No Song Request Linux, preencha:

- `YouTube API Key`
- `Maximo YouTube`, padrao `360` segundos
- `Aceitar YouTube nao marcado como musica`, se quiser liberar excecoes

O app usa metadados do YouTube para bloquear videos longos e, por padrao,
bloquear videos que nao estejam na categoria Musica.

## 4. Comandos

Todos podem usar:

```text
!sr nome da musica
!sr https://youtu.be/VIDEO_ID
!song
!fila
!queue
!q
!vol
!comandos
```

Moderador/broadcaster:

```text
!skip
!vol 30
!play
!pause
!next
```

## 5. Abrir E Fechar

Abrir:

```bash
./scripts/song-request-linux-open
```

Fechar:

```bash
./scripts/song-request-linux-stop
```

Tambem pode fechar pelo botao `Encerrar` no dashboard.
