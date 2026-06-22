use axum::response::Html;

pub async fn page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Conexoes - Song Request Linux</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #0f1115;
      --panel: #181b21;
      --panel-2: #20242c;
      --text: #f3f5f7;
      --muted: #a9b0bb;
      --line: #343945;
      --action: #62a8ff;
      --ok: #2bd180;
      --bad: #ff6b6b;
      font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    * { box-sizing: border-box; }
    body { margin: 0; background: var(--bg); color: var(--text); }
    header, main { max-width: 980px; margin: 0 auto; padding: 18px; }
    header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
    h1 { margin: 0; font-size: 20px; }
    section {
      display: grid;
      gap: 12px;
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 16px;
      margin-bottom: 14px;
    }
    button, a.button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 38px;
      border: 1px solid #3d7fc8;
      border-radius: 6px;
      background: var(--action);
      color: #07111f;
      padding: 8px 12px;
      font-weight: 700;
      text-decoration: none;
      cursor: pointer;
    }
    select {
      min-height: 38px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      color: var(--text);
      padding: 8px 10px;
      min-width: min(100%, 420px);
    }
    input {
      min-height: 38px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      color: var(--text);
      padding: 8px 10px;
      min-width: min(100%, 420px);
    }
    label {
      display: grid;
      gap: 6px;
      color: var(--muted);
      font-size: 13px;
    }
    a.secondary {
      border-color: var(--line);
      background: var(--panel-2);
      color: var(--text);
    }
    .row { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
    .muted { color: var(--muted); }
    .message { min-height: 20px; color: var(--muted); }
    .message.error { color: var(--bad); }
    code {
      display: block;
      padding: 10px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      overflow-wrap: anywhere;
      color: var(--ok);
    }
    .bad { color: var(--bad); }
  </style>
</head>
<body>
  <header>
    <h1>Conexoes</h1>
    <a class="button secondary" href="/">Dashboard</a>
  </header>
  <main>
    <section>
      <h2>Configuracao</h2>
      <label>
        Provider padrao
        <select id="default-provider">
          <option value="spotify">Spotify</option>
          <option value="youtube">YouTube</option>
        </select>
      </label>
      <label>
        Spotify Client ID
        <input id="spotify-client-id" autocomplete="off" placeholder="Client ID do app Spotify">
      </label>
      <label>
        Twitch Client ID
        <input id="twitch-client-id" autocomplete="off" placeholder="Client ID do app Twitch">
      </label>
      <label>
        Twitch Bot Username
        <input id="twitch-bot-username" autocomplete="off" placeholder="preenchido apos OAuth do bot">
      </label>
      <label>
        Twitch Channel
        <input id="twitch-channel" autocomplete="off" placeholder="canal_do_streamer">
      </label>
      <div class="row">
        <button id="save-config">Salvar configuracao</button>
      </div>
      <div class="message" id="config-message"></div>
    </section>

    <section>
      <h2>Spotify</h2>
      <div class="muted" id="spotify-status">Carregando...</div>
      <div class="row">
        <button id="spotify-start">Gerar link de login</button>
      </div>
      <code id="spotify-link">O link aparece aqui.</code>
      <div class="muted">
        Abra este link em uma janela privada se precisar garantir que a conta conectada seja a conta correta.
      </div>
    </section>

    <section>
      <h2>Playlist fallback</h2>
      <div class="muted">
        Esta playlist sera usada quando nao houver pedidos na fila. Nesta etapa ela fica salva; a reproducao automatica entra no proximo passo.
      </div>
      <div class="row">
        <button id="load-playlists">Carregar playlists</button>
        <select id="playlist-select">
          <option value="">Nenhuma playlist carregada</option>
        </select>
        <button id="save-playlist">Salvar fallback</button>
      </div>
      <div class="message" id="playlist-message"></div>
    </section>

    <section>
      <h2>Twitch Bot</h2>
      <div class="muted">
        Conecte a conta bot em uma janela privada para nao reaproveitar a sessao do streamer.
        No console da Twitch, cadastre o redirect: https://localhost:7443/auth/twitch/callback.
        Se o navegador avisar sobre certificado local, aceite para concluir o OAuth no seu proprio PC.
      </div>
      <div class="row">
        <button id="twitch-start">Gerar link do bot</button>
      </div>
      <code id="twitch-link">O link aparece aqui.</code>
      <div class="message" id="twitch-message"></div>
    </section>
  </main>
  <script>
    const statusEl = document.getElementById('spotify-status');
    const linkEl = document.getElementById('spotify-link');
    const playlistSelect = document.getElementById('playlist-select');
    const playlistMessage = document.getElementById('playlist-message');
    const configMessage = document.getElementById('config-message');
    const twitchLink = document.getElementById('twitch-link');
    const twitchMessage = document.getElementById('twitch-message');
    let playlists = [];
    async function api(path, options = {}) {
      const response = await fetch(path, {
        headers: { 'content-type': 'application/json', ...(options.headers || {}) },
        ...options
      });
      const data = await response.json();
      if (!response.ok) throw new Error(data.error || 'Falha na requisicao');
      return data;
    }
    async function refresh() {
      const [status, config] = await Promise.all([
        api('/api/connections/status'),
        api('/api/config')
      ]);
      document.getElementById('default-provider').value = config.default_provider;
      document.getElementById('spotify-client-id').value = config.spotify_client_id || '';
      document.getElementById('twitch-client-id').value = config.twitch_client_id || '';
      document.getElementById('twitch-bot-username').value = config.twitch_bot_username || '';
      document.getElementById('twitch-channel').value = config.twitch_channel || '';
      twitchMessage.textContent = config.twitch_bot_token_configured
        ? 'Token do bot salvo. Reinicie o app para conectar ao chat.'
        : 'Bot ainda nao conectado.';
      statusEl.textContent = status.spotify.token_configured
        ? 'Spotify conectado.'
        : status.spotify.client_id_configured
          ? 'Client ID configurado. Falta login.'
          : 'Configure SPOTIFY_CLIENT_ID antes de conectar.';
      statusEl.className = status.spotify.client_id_configured ? 'muted' : 'bad';
      if (status.spotify.fallback_playlist) {
        playlistMessage.textContent = `Fallback atual: ${status.spotify.fallback_playlist.name}`;
        playlistMessage.className = 'message';
      }
    }
    document.getElementById('spotify-start').addEventListener('click', async () => {
      try {
        const result = await api('/api/connections/spotify/start', { method: 'POST' });
        linkEl.textContent = result.auth_url;
        window.open(result.auth_url, '_blank', 'noopener,noreferrer');
      } catch (error) {
        linkEl.textContent = error.message;
      }
    });
    document.getElementById('save-config').addEventListener('click', async () => {
      try {
        await api('/api/config', {
          method: 'POST',
          body: JSON.stringify({
            default_provider: document.getElementById('default-provider').value,
            spotify_client_id: document.getElementById('spotify-client-id').value,
            twitch_client_id: document.getElementById('twitch-client-id').value,
            twitch_bot_username: document.getElementById('twitch-bot-username').value,
            twitch_channel: document.getElementById('twitch-channel').value,
            twitch_bot_oauth_token: null
          })
        });
        configMessage.textContent = 'Configuracao salva. Reinicie o app para aplicar provider/Twitch bot.';
        configMessage.className = 'message';
        await refresh();
      } catch (error) {
        configMessage.textContent = error.message;
        configMessage.className = 'message error';
      }
    });
    document.getElementById('twitch-start').addEventListener('click', async () => {
      try {
        const result = await api('/api/connections/twitch/start', { method: 'POST' });
        twitchLink.textContent = result.auth_url;
        window.open(result.auth_url, '_blank', 'noopener,noreferrer');
      } catch (error) {
        twitchLink.textContent = error.message;
      }
    });
    document.getElementById('load-playlists').addEventListener('click', async () => {
      try {
        playlists = await api('/api/spotify/playlists');
        playlistSelect.innerHTML = playlists.length
          ? playlists.map((playlist, index) => `<option value="${index}">${playlist.name} (${playlist.tracks.total})</option>`).join('')
          : '<option value="">Nenhuma playlist encontrada</option>';
        playlistMessage.textContent = playlists.length
          ? `${playlists.length} playlists carregadas.`
          : 'Nenhuma playlist encontrada.';
        playlistMessage.className = 'message';
      } catch (error) {
        playlistMessage.textContent = error.message;
        playlistMessage.className = 'message error';
      }
    });
    document.getElementById('save-playlist').addEventListener('click', async () => {
      try {
        const playlist = playlists[Number(playlistSelect.value)];
        if (!playlist) throw new Error('Carregue e selecione uma playlist primeiro.');
        await api('/api/spotify/fallback-playlist', {
          method: 'POST',
          body: JSON.stringify({ id: playlist.id, name: playlist.name, uri: playlist.uri })
        });
        playlistMessage.textContent = `Fallback salvo: ${playlist.name}`;
        playlistMessage.className = 'message';
        await refresh();
      } catch (error) {
        playlistMessage.textContent = error.message;
        playlistMessage.className = 'message error';
      }
    });
    refresh();
  </script>
</body>
</html>"#,
    )
}
