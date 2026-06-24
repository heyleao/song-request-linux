<p align="center">
  <img src="assets/logo-srl.png" alt="Song Request Linux" width="220">
</p>

<h1 align="center">Song Request Linux</h1>

<p align="center">
  Pedidos de musica para lives na Twitch, feito para Linux, com Spotify, YouTube/Pear, OBS overlay e dashboard local.
</p>

<p align="center">
  <a href="README_EN.md">English</a> ·
  <a href="docs/SETUP.md">Guia rapido</a> ·
  <a href="#instalar">Instalar</a> ·
  <a href="#como-funciona">Como funciona</a> ·
  <a href="#comandos">Comandos</a> ·
  <a href="#obs">OBS</a>
</p>

---

Song Request Linux e um app local para streamers que querem aceitar pedidos de
musica pelo chat da Twitch no Linux. Ele roda em `127.0.0.1`, abre um dashboard
no navegador, controla a fila e entrega overlays para OBS.

O app foi pensado para evitar dependencias Windows-only, WebView2 e Wine. O
fluxo atual e simples: escolha um provider principal, conecte o bot da Twitch,
configure o player e adicione o overlay no OBS.

## Estado Atual

- Dashboard local em `http://127.0.0.1:7384/`.
- Setup guiado para Twitch, Spotify, YouTube/Pear, fila e OBS.
- Twitch bot com comandos configuraveis.
- Cargos reconhecidos por tags oficiais da Twitch: streamer, moderator, VIP,
  subscriber e follower.
- Limite de pedidos por cargo, com `0` significando sem limite.

Nota sobre follower: o IRC da Twitch nao informa com seguranca se um chatter comum segue o canal. No SRL, chatter comum entra como `Follower`; para garantir que so seguidores possam pedir musica, ative o modo seguidores no chat da Twitch.
- Um provider ativo por vez: Spotify ou YouTube/Pear.
- Links do YouTube entram direto como YouTube.
- Busca de texto usa o provider ativo.
- Spotify com OAuth, busca, fila, controle de player, volume e playlist fallback.
- YouTube com YouTube Data API v3 e playback recomendado via Pear Desktop.
- Fila persistente opcional: o streamer decide se continua a fila ao reabrir.
- Overlay de musica atual para OBS.
- Logs e diagnostico no dashboard.
- Launcher com instancia unica e botao `Encerrar`.

## Instalar

No CachyOS/Arch, use o instalador amigavel:

```bash
git clone https://github.com/heyleao/song-request-linux.git
cd song-request-linux
./scripts/install-user-friendly --with-pear
```

Abrir:

```bash
./scripts/song-request-linux-open
```

Parar:

```bash
./scripts/song-request-linux-stop
```

Atualizar pelo GitHub:

```bash
./scripts/update-from-github --restart
```

Remover o app mantendo configs, tokens, logs e fila:

```bash
./scripts/uninstall-user
```

Remover tudo, incluindo dados locais:

```bash
./scripts/uninstall-user --remove-data
```

## Distribuicao

Caminho recomendado agora:

1. `tar.gz` portatil para qualquer Linux compativel com o binario.
2. Pacote Arch/CachyOS/AUR.
3. `.deb` e `.rpm` usando o mesmo binario e os mesmos scripts.
4. Flatpak depois, quando as permissoes de localhost, OBS, Pear e Spotify
   estiverem bem testadas no sandbox.

Gerar pacote portatil local:

```bash
./scripts/package-portable
```

O pacote gerado fica em `dist/` e inclui binario, scripts, logo, README, guia e
prints. Ele nao inclui config, tokens, logs, fila, `.env`, `.secrets` ou dados do
usuario.

## Como Funciona

![Dashboard do Song Request Linux](docs/images/dashboard-overview.png)

1. Abra o dashboard.
2. Na tela `Operacao`, escolha o modo no card `Provider ativo`:
   - `Spotify`: pedidos por texto buscam no Spotify.
   - `YouTube/Pear`: pedidos por texto buscam no YouTube e tocam no Pear.
3. Va em `Configuracao`.
4. Configure apenas o bloco do provider escolhido.
5. Conecte o bot da Twitch.
6. Salve a configuracao.
7. Teste um pedido no dashboard ou no chat.
8. Adicione o overlay no OBS.

Importante: hoje o app trabalha melhor com um provider ativo por vez. Se o
provider for Spotify, texto vai para Spotify. Se for YouTube/Pear, texto vai
para YouTube. Links do YouTube continuam sendo detectados como YouTube.

## Spotify

Use Spotify quando quiser pedidos por busca no Spotify e playlist fallback.

Requisitos:

- Spotify Premium.
- App Spotify aberto no PC da live.
- Um device ativo no PC antes de aceitar pedidos.
- `Client ID` criado no dashboard de desenvolvedor do Spotify.

Redirect URI do Spotify:

```text
http://127.0.0.1:7384/auth/spotify/callback
```

O app evita transferir playback para celular. Se nao houver device valido no PC
da live, o dashboard mostra erro e pede para abrir/tocar algo no Spotify local.

## YouTube/Pear

Use YouTube/Pear quando quiser tocar pedidos do YouTube.

Requisitos:

- YouTube Data API v3 ativa.
- API Key salva no dashboard.
- Pear Desktop / YouTube Music Desktop aberto.
- Plugin `API Server` do Pear ativo.

Config recomendada do Pear:

```text
Porta: 26538
Authorization: No Authorization
API: http://127.0.0.1:26538/api/v1
```

O app envia pedidos para a fila do Pear. O Pear e um app externo; se ele estiver
fechado ou com API desligada, o pedido pode entrar na fila do SRL, mas nao vai
tocar ate o Pear voltar.

## Fila e Fallback

- Pedido aceito entra na fila do app.
- Quando a fila tem pedidos, eles devem tocar antes da playlist fallback.
- Quando a fila fica vazia, o fallback pode voltar.
- A playlist fallback e opcional.
- A persistencia da fila tambem e opcional.

Persistencia marcada:

```text
O app salva a fila e continua depois de fechar/abrir.
```

Persistencia desmarcada:

```text
A proxima abertura comeca com fila limpa.
```

## OBS

Dashboard:

```text
http://127.0.0.1:7384/
```

Overlay recomendado:

```text
http://127.0.0.1:7384/overlay?max=48&width=520&size=24&lines=2
```

Tamanho recomendado da Browser Source:

```text
Largura: 620
Altura: 150
```

O parametro `width=520` controla a largura interna do texto. Use `lines=2` para
permitir duas linhas no nome da musica. O texto do topo pode ser alterado no
Setup ou com `label=Texto` na URL.

Player OBS para YouTube:

```text
http://127.0.0.1:7384/player
```

Use o player OBS apenas se voce escolheu `Browser Source OBS` como player do
YouTube. Se usa Pear, normalmente nao precisa dessa fonte.

## Comandos

![Configuracao avancada de comandos e limites](docs/images/advanced-setup.png)


Padrao para todos:

```text
!sr nome da musica
!sr https://youtu.be/VIDEO_ID
!song
!fila
!queue
!q
!rm
!remove
!vol
!comandos
```

Padrao para moderador/streamer:

```text
!skip
!vol 30
!play
!pause
!next
```

Na aba `Configuracao`, cada comando tem sua propria caixa de aliases e permissao.
Exemplo: voce pode trocar `!sr` por `!ssr`.

Permissoes disponiveis:

```text
Todos
Subscriber
VIP
Moderador
Streamer
```

Limites de pedidos por cargo:

```text
0 = sem limite
1..100 = maximo de pedidos pendentes daquele cargo
```

O limite conta a musica atual e as proximas musicas do mesmo solicitante.

## Dados Locais

Config publica:

```text
~/.config/song-request-linux/config.json
```

Estado, tokens, logs e fila:

```text
~/.local/state/song-request-linux/
```

Nao suba tokens, API keys, OAuth codes, `.env`, configs reais exportadas, logs
privados ou notas internas para o GitHub.

## Desenvolvimento

Rodar direto:

```bash
cargo run
```

Checar:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Simular comando de chat:

```bash
curl -X POST http://127.0.0.1:7384/api/chat-command \
  -H 'content-type: application/json' \
  -d '{"requester":"follower","message":"!song","role":"follower"}'
```

## Licenca

GPL-3.0-or-later.
